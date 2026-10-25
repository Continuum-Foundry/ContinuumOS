//! Single-CPU guest adapter. Called during boot with IF clear, before scheduling.
//! IRQ/NMI handlers never mutate these tables or frames. No Rust reference to
//! mapped user bytes escapes. P3-04 must disable preemption around pins/copies.
use crate::{Local, record};
use continuum_kernel::{
    PhysAddr, Range, Result, error,
    memory::PAGE,
    owned_memory::{Access, Runtime, STACK_BASE, STACK_BYTES, Store},
};
use core::{arch::asm, cell::UnsafeCell};
type Memory = Runtime<16, 256, 128, 64>;
static MEMORY: Local<Memory> = Local(UnsafeCell::new(Memory::EMPTY));
#[repr(C, align(4096))]
struct Stacks([[u8; 73728]; 16]);
static STACKS: Local<Stacks> = Local(UnsafeCell::new(Stacks([[0; 73728]; 16])));
pub fn guard(v: u64) -> bool {
    let start = STACKS.0.get() as u64;
    v >= start
        && v < start + 16 * 73728
        && ((v - start) % 73728 == 0 || (v - start) % 73728 == 69632)
}
fn cr3() -> u64 {
    let v;
    unsafe {
        asm!("mov {}, cr3",out(reg) v,options(nomem,nostack));
    }
    v
}
fn switch(root: u64) {
    unsafe {
        asm!("mov cr3, {}",in(reg) root,options(nostack));
    }
}
struct GuestStore {
    kernel: PhysAddr,
}
impl Store for GuestStore {
    fn allocate(&mut self, pages: usize, alignment: u64) -> Result<PhysAddr> {
        crate::paging::runtime_allocate(pages, alignment)
    }
    fn release(&mut self, base: PhysAddr, pages: usize) {
        crate::paging::runtime_release(base, pages);
    }
    fn read(&self, base: PhysAddr, offset: u64) -> u8 {
        crate::paging::runtime_read(base, offset)
    }
    fn write(&mut self, base: PhysAddr, offset: u64, byte: u8) {
        crate::paging::runtime_write(base, offset, byte);
    }
    fn publish(&mut self, old: PhysAddr, new: PhysAddr) {
        if cr3() == old.0 {
            switch(new.0);
        }
    }
    fn retire(&mut self, root: PhysAddr) {
        if cr3() == root.0 {
            switch(self.kernel.0);
        }
    }
    fn prepare_kernel_stack(&mut self, slot: usize) -> Result<Range> {
        let base = STACKS.0.get() as u64 + slot as u64 * 73728 + PAGE;
        // No reference covers the unmapped guard pages.
        unsafe {
            core::ptr::write_bytes(base as *mut u8, 0, STACK_BYTES as usize);
        }
        Ok(Range {
            base,
            len: STACK_BYTES,
        })
    }
    fn release_kernel_stack(&mut self, _: usize) {
        // Caller is on the surviving M1 runtime stack. P3-04/P3-05 must
        // preserve this precondition before threads can use these stacks.
    }
}
fn need(v: bool) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err(error("runtime_memory", "self_check"))
    }
}
pub fn establish(mode: u64) -> Result<()> {
    let flags: u64;
    unsafe {
        asm!("pushfq","pop {}",out(reg) flags,options(nomem));
    }
    need(flags & (1 << 9) == 0)?;
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    let image = crate::paging::image_bounds();
    let r = unsafe { &mut *MEMORY.0.get() };
    let stack_storage = 16 * 73728;
    need(core::mem::size_of::<Memory>() + stack_storage <= 16 * 1024 * 1024)?;
    r.initialize(crate::paging::kernel_entries(), |p| {
        let v = p as u64;
        need(v >= image.0 && v < image.1)?;
        Ok(PhysAddr(info.kernel.base + v - image.0))
    })?;
    let inventory = r.inventory();
    record(
        "m1.runtime_memory",
        "runtime-memory",
        "ok",
        format_args!(
            "metadata_bytes={} table_storage_bytes={} kernel_stack_storage={} kernel_guard_bytes=131072 dynamic_ceiling=16777216 user_ceiling=67108864 spaces=16 allocation_slots=256 mappings_per_space=128 tables_per_bank=64 banks=2 initialized=true",
            inventory.metadata_bytes, inventory.table_storage_bytes, stack_storage
        ),
    );
    if mode != 30 {
        return Ok(());
    }
    let mut b = GuestStore {
        kernel: PhysAddr(cr3()),
    };
    let empty = r.inventory();
    let frames = crate::paging::runtime_frames();
    let s = r.create_with_stack(&mut b)?;
    let other = r.create_with_stack(&mut b)?;
    for (space, label) in [(s, 0), (other, 1)] {
        let stack = r.kernel_stack(space)?;
        let guard_before = crate::paging::kernel_leaf(stack.base - PAGE)?;
        let guard_after = crate::paging::kernel_leaf(stack.end())?;
        need(guard_before == 0 && guard_after == 0)?;
        for offset in (0..STACK_BYTES).step_by(PAGE as usize) {
            need(
                crate::paging::kernel_leaf(stack.base + offset)? & (7 | (1 << 63)) == 3 | (1 << 63),
            )?;
            need(unsafe { ((stack.base + offset) as *const u8).read_volatile() } == 0)?;
        }
        record(
            "m1.runtime_stack",
            "runtime-memory",
            "ok",
            format_args!(
                "space={label} base={:#x} bytes={} guard_before={guard_before:#x} guard_after={guard_after:#x} leaf={:#x} pages=16 zeroed=true root_kernel={:#x}",
                stack.base,
                stack.len,
                crate::paging::kernel_leaf(stack.base)?,
                r.tables(space)?.entry(0, 511)?
            ),
        );
    }
    let a = r.allocate(&mut b, s, 4097, 8192)?.unwrap();
    let (p, n) = r.backing(s, a)?;
    need(p.0 % 8192 == 0 && n == 8192)?;
    for offset in 0..n {
        need(b.read(p, offset) == 0)?;
    }
    r.write_staging(&mut b, s, a, 0, b"P3-02")?;
    let a = r.realloc(&mut b, s, a, 12289, 8192)?.unwrap();
    let (p, n) = r.backing(s, a)?;
    for (offset, &byte) in b"P3-02".iter().enumerate() {
        need(b.read(p, offset as u64) == byte)?;
    }
    for offset in 4097..n {
        need(b.read(p, offset) == 0)?;
    }
    r.map(&mut b, s, a, 0x400000, 0, n, Access::Write)?;
    need(
        r.map(&mut b, other, a, 0x400000, 0, n, Access::Execute)
            .is_err(),
    )?;
    let pin = r.pin(s, 0x400fff, 2, true)?;
    let before = r.inventory();
    need(
        r.unmap(&mut b, s, 0x400000).is_err()
            && r.destroy(&mut b, s).is_err()
            && r.protect(&mut b, s, 0x400000, Access::Read).is_err()
            && before == r.inventory(),
    )?;
    r.unpin(pin)?;
    // Kernel runs under both real roots; selected low pages are accessed only
    // after independent hardware-entry walks and supervisor alias inspection.
    switch(r.root(s)?.0);
    unsafe {
        (0x400000 as *mut u8).write_volatile(0x6b);
    }
    need(b.read(p, 0) == 0x6b)?;
    r.protect(&mut b, s, 0x400000, Access::Execute)?;
    need(cr3() == r.root(s)?.0)?;
    for (id, label) in [(s, 0), (other, 1)] {
        let t = r.tables(id)?;
        let root = t.root().0;
        record(
            "m1.runtime_space",
            "runtime-memory",
            "ok",
            format_args!(
                "space={label} root={root:#x} used={} user_start=0x10000 user_end=0x80000000 stack={STACK_BASE:#x} stack_bytes={STACK_BYTES}",
                t.used
            ),
        );
        for table in 0..t.used {
            for index in 0..512 {
                let entry = t.entry(table, index)?;
                if entry != 0 {
                    record(
                        "m1.runtime_entry",
                        "runtime-memory",
                        "ok",
                        format_args!("space={label} table={table} index={index} entry={entry:#x}"),
                    );
                }
            }
        }
        for v in [
            0x400000,
            STACK_BASE,
            STACK_BASE - PAGE,
            STACK_BASE + STACK_BYTES,
        ] {
            // Raw entries are recorded; Python independently follows indices,
            // permissions, physical ownership and guard absence from these.
            let mut table = 0;
            for (level, shift) in [39, 30, 21, 12].iter().enumerate() {
                let index = ((v >> shift) & 511) as usize;
                let e = t.entry(table, index)?;
                record(
                    "m1.runtime_walk",
                    "runtime-memory",
                    "ok",
                    format_args!(
                        "space={label} root={root:#x} virtual={v:#x} level={level} table={table} index={index} entry={e:#x}"
                    ),
                );
                if e & 1 == 0 || level == 3 {
                    break;
                }
                table = (((e & 0x000f_ffff_ffff_f000) - root) / PAGE) as usize;
            }
        }
        need(t.entry(0, 511)? & 4 == 0)?;
    }
    need(crate::paging::runtime_alias(p)? & (2 | (1 << 63)) == 1 << 63)?;
    for offset in (0..n).step_by(PAGE as usize) {
        let physical = PhysAddr(p.0 + offset);
        record(
            "m1.runtime_alias",
            "runtime-memory",
            "ok",
            format_args!(
                "physical={:#x} bytes=4096 entry={:#x} user_rx=true supervisor_ro_nx=true",
                physical.0,
                crate::paging::runtime_alias(physical)?
            ),
        );
    }
    switch(r.root(other)?.0);
    need(unsafe { (STACK_BASE as *const u8).read_volatile() } == 0)?;
    r.map(&mut b, other, a, 0x600000, 0, n, Access::Read)?;
    need(r.destroy(&mut b, s).is_err())?;
    r.unmap(&mut b, other, 0x600000)?;
    r.destroy(&mut b, s)?;
    r.destroy(&mut b, other)?;
    need(
        cr3() == b.kernel.0 && r.inventory() == empty && crate::paging::runtime_frames() == frames,
    )?;
    // Inject physical failures at every create stage; page-table exhaustion is
    // tested through the exact portable backend on the host.
    for quota in 0..16 {
        crate::paging::runtime_quota(quota)?;
        need(
            r.create_with_stack(&mut b).is_err()
                && r.inventory() == empty
                && crate::paging::runtime_frames() == frames,
        )?;
    }
    crate::paging::runtime_quota(usize::MAX)?;
    for cycle in 0..1000 {
        let s = r.create_with_stack(&mut b)?;
        let a = r.allocate(&mut b, s, 4096, 1)?.unwrap();
        let (p, _) = r.backing(s, a)?;
        for offset in 0..PAGE {
            need(b.read(p, offset) == 0)?;
        }
        r.write_staging(&mut b, s, a, 0, &[0xa5])?;
        r.map(&mut b, s, a, 0x400000, 0, PAGE, Access::Write)?;
        switch(r.root(s)?.0);
        need(unsafe { (0x400000 as *const u8).read_volatile() } == 0xa5)?;
        r.unmap(&mut b, s, 0x400000)?;
        r.destroy(&mut b, s)?;
        need(
            r.inventory() == empty
                && crate::paging::runtime_frames() == frames
                && cr3() == b.kernel.0,
        )?;
        if cycle == 0 || cycle == 999 {
            record(
                "m1.runtime_settlement",
                "runtime-memory",
                "ok",
                format_args!(
                    "cycle={cycle} allocated={} spaces=0 allocations=0 mappings=0 requested=0 committed=0 resident=0 page_tables=0 pins=0 zero_reuse=true",
                    crate::paging::runtime_frames()
                ),
            );
        }
    }
    record(
        "m1.runtime_memory_ready",
        "runtime-memory",
        "ok",
        format_args!(
            "cycles=1000 real_cr3=true active_protect=true pins=true shared_busy=true zero_reuse=true rollback=true settled=true frames={} user_execution=false",
            crate::paging::runtime_frames()
        ),
    );
    Ok(())
}

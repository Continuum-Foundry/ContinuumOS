//! Offline P3-03 experiment: inspect staged bytes/tables, never jump to user code.
use crate::{
    record,
    runtime_memory::{GuestStore, Memory},
};
use continuum_kernel::{
    Result, VirtAddr, error,
    memory::PAGE,
    owned_memory::{STACK_BASE, STACK_BYTES},
    user_image::*,
};
include!(env!("CONTINUUM_TRUSTED_IMAGES"));
static MUTATION: crate::Local<[u8; MAX_IMAGE]> =
    crate::Local(core::cell::UnsafeCell::new([0; MAX_IMAGE]));
fn need(v: bool) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err(error("loader", "self_check"))
    }
}
fn hex(b: &[u8; 32]) -> [u8; 64] {
    let mut o = [0; 64];
    for (i, x) in b.iter().enumerate() {
        o[i * 2] = b"0123456789abcdef"[(x >> 4) as usize];
        o[i * 2 + 1] = b"0123456789abcdef"[(x & 15) as usize];
    }
    o
}
fn text(b: &[u8]) -> &str {
    core::str::from_utf8(b).unwrap()
}
pub fn check(r: &mut Memory, store: &mut GuestStore) -> Result<()> {
    need(
        core::mem::size_of::<Memory>()
            + 16 * 73728
            + MAX_IMAGE
            + core::mem::size_of::<Launcher<16>>()
            <= 16 * 1024 * 1024,
    )?;
    let empty = r.inventory();
    let frames = crate::paging::runtime_frames();
    let mut launcher = Launcher::<16>::EMPTY;
    let mut ids = [None; 3];
    for (i, binding) in [Binding::UserA, Binding::UserB, Binding::Service]
        .into_iter()
        .enumerate()
    {
        let bytes = crate::paging::module(i)?;
        let (digest, size) = IMAGES[i];
        let id = launcher.bootstrap(
            r,
            store,
            TrustedImage {
                bytes,
                digest,
                size,
            },
            binding,
        )?;
        ids[i] = Some(id);
        let p = launcher.get(id)?;
        let plan = parse(bytes)?;
        let root = r.root(p.space)?;
        record(
            "m1.loader_image",
            "static-loader",
            "ok",
            format_args!(
                "image={i} bytes={size} digest={} root={:#x} entry={:#x} rsp={:#x} startup={STARTUP:#x} commitment={} state=staged",
                text(&hex(&digest)),
                root.0,
                p.entry,
                p.stack,
                plan.commitment
            ),
        );
        let c = p.context;
        record(
            "m1.loader_binding",
            "static-loader",
            "ok",
            format_args!(
                "image={i} slot={} generation={} principal={} authority_session={} service={} grants={} ceiling={} source=bootstrap",
                id.slot,
                id.generation,
                c.principal,
                c.session,
                c.service,
                c.grant_ceiling,
                c.commitment_ceiling
            ),
        );
        let tables = r.tables(p.space)?;
        // Dump complete nonzero low-half trees, not only asserted permissions.
        for t in 0..tables.used {
            for n in 0..512 {
                if t == 0 && n >= 256 {
                    continue;
                }
                let raw = tables.entry(t, n)?;
                if raw != 0 {
                    record(
                        "m1.loader_table",
                        "static-loader",
                        "ok",
                        format_args!("image={i} table={t} index={n} raw={raw:#x}"),
                    );
                }
            }
        }
        let mut page = [0u8; 4096];
        // Read actual mapped virtual bytes under each private CR3. No execution.
        unsafe {
            core::arch::asm!("mov cr3, {}",in(reg) root.0,options(nostack));
        }
        for (base, count) in plan.segments[..plan.count]
            .iter()
            .map(|s| (s.base, s.pages))
            .chain([(STARTUP, 1), (RETURN, 1), (STACK_BASE, STACK_BYTES / PAGE)])
        {
            for n in 0..count {
                let address = base + n * PAGE;
                for (o, x) in page.iter_mut().enumerate() {
                    *x = unsafe { ((address + o as u64) as *const u8).read_volatile() };
                }
                let raw = tables.inspect(VirtAddr(address))?;
                let alias = crate::paging::runtime_alias(continuum_kernel::PhysAddr(
                    raw & 0x000f_ffff_ffff_f000,
                ))?;
                need(alias & ((1 << 63) | 7) == (1 << 63) | 1)?;
                record(
                    "m1.loader_page",
                    "static-loader",
                    "ok",
                    format_args!(
                        "image={i} address={address:#x} leaf={raw:#x} alias={alias:#x} sha256={}",
                        text(&hex(&sha256(&page)))
                    ),
                );
            }
        }
        need(
            tables.inspect(VirtAddr(STACK_BASE - PAGE))? == 0
                && tables.inspect(VirtAddr(STACK_BASE + STACK_BYTES))? == 0,
        )?;
        let stack = r.kernel_stack(p.space)?;
        need(
            crate::paging::kernel_leaf(stack.base - PAGE)? == 0
                && crate::paging::kernel_leaf(stack.end())? == 0,
        )?;
        need(
            launcher
                .child_context(
                    id,
                    c.principal + 1,
                    c.session,
                    c.service,
                    c.grant_ceiling,
                    c.commitment_ceiling,
                )
                .is_err(),
        )?;
        need(
            launcher
                .child_context(
                    id,
                    c.principal,
                    c.session + 1,
                    c.service,
                    c.grant_ceiling,
                    c.commitment_ceiling,
                )
                .is_err(),
        )?;
        need(
            launcher
                .child_context(
                    id,
                    c.principal,
                    c.session,
                    !c.service,
                    c.grant_ceiling,
                    c.commitment_ceiling,
                )
                .is_err(),
        )?;
        need(
            launcher
                .child_context(
                    id,
                    c.principal,
                    c.session,
                    c.service,
                    4,
                    c.commitment_ceiling,
                )
                .is_err(),
        )?;
        need(
            launcher
                .child_context(
                    id,
                    c.principal,
                    c.session,
                    c.service,
                    0,
                    c.commitment_ceiling + 1,
                )
                .is_err(),
        )?;
        need(launcher.get(id)?.context == c)?;
    }
    for id in ids.into_iter().flatten() {
        launcher.destroy(r, store, id)?;
        need(launcher.get(id).is_err())?;
    }
    need(
        r.inventory() == empty && launcher.live() == 0 && crate::paging::runtime_frames() == frames,
    )?;
    let bytes = crate::paging::module(0)?;
    let (digest, size) = IMAGES[0];
    let ph = u64::from_le_bytes(bytes[32..40].try_into().unwrap()) as usize;
    let sh = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
    let mutations = [
        (4, 1u64, 1, "identity"),
        (16, 3, 2, "target"),
        (18, 3, 2, "target"),
        (52, 0, 2, "header_size"),
        (32, u64::MAX, 8, "overflow"),
        (ph, 2, 4, "program_kind"),
        (ph, 3, 4, "program_kind"),
        (ph, 7, 4, "program_kind"),
        (ph + 4, 7, 4, "permission"),
        (ph + 8, u64::MAX, 8, "overflow"),
        (ph + 32, u64::MAX, 8, "size"),
        (ph + 40, u64::MAX, 8, "overflow"),
        (ph + 40, 67108864, 8, "commitment"),
        (ph + 48, 3, 8, "alignment"),
        (ph + 16, 0, 8, "alignment"),
        (24, STARTUP, 8, "entry"),
        (sh + 4, 9, 4, "section_obligation"),
        (sh + 8, 0x400, 8, "section_obligation"),
    ];
    let scratch = unsafe { &mut *MUTATION.0.get() };
    for (case, (offset, value, width, reason)) in mutations.into_iter().enumerate() {
        scratch[..size].copy_from_slice(bytes);
        scratch[offset..offset + width].copy_from_slice(&value.to_le_bytes()[..width]);
        let altered = &scratch[..size];
        let e = launcher
            .bootstrap(
                r,
                store,
                TrustedImage {
                    bytes: altered,
                    digest: sha256(altered),
                    size,
                },
                Binding::UserA,
            )
            .err();
        need(
            e == Some(error("elf", reason))
                && r.inventory() == empty
                && launcher.live() == 0
                && crate::paging::runtime_frames() == frames,
        )?;
        record(
            "m1.loader_rejected",
            "static-loader",
            "ok",
            format_args!("case=malformed mutation={case} field=elf reason={reason} settled=true"),
        );
    }
    need(
        launcher
            .bootstrap(
                r,
                store,
                TrustedImage {
                    bytes: &bytes[..63],
                    digest: sha256(&bytes[..63]),
                    size: 63,
                },
                Binding::UserA,
            )
            .err()
            == Some(error("elf", "truncated")),
    )?;
    let pages = parse(bytes)?.commitment / PAGE;
    let mut failures = 0;
    for quota in 0..pages {
        crate::paging::runtime_quota(quota as usize)?;
        let e = launcher
            .bootstrap(
                r,
                store,
                TrustedImage {
                    bytes,
                    digest,
                    size,
                },
                Binding::UserA,
            )
            .err()
            .ok_or(error("loader", "expected_failure"))?;
        need(
            r.inventory() == empty
                && launcher.live() == 0
                && crate::paging::runtime_frames() == frames,
        )?;
        record(
            "m1.loader_rejected",
            "static-loader",
            "ok",
            format_args!(
                "case=allocation quota={quota} field={} reason={} settled=true",
                e.field, e.reason
            ),
        );
        failures += 1;
    }
    crate::paging::runtime_quota(usize::MAX)?;
    let mut wrong = digest;
    wrong[0] ^= 1;
    need(
        launcher
            .bootstrap(
                r,
                store,
                TrustedImage {
                    bytes,
                    digest: wrong,
                    size,
                },
                Binding::UserA,
            )
            .err()
            == Some(error("image", "digest")),
    )?;
    let fresh = launcher.bootstrap(
        r,
        store,
        TrustedImage {
            bytes,
            digest,
            size,
        },
        Binding::UserB,
    )?;
    need(
        fresh.slot == ids[0].unwrap().slot
            && fresh.generation > ids[0].unwrap().generation
            && launcher.get(ids[0].unwrap()).is_err(),
    )?;
    let c = launcher.get(fresh)?.context;
    record(
        "m1.loader_binding",
        "static-loader",
        "ok",
        format_args!(
            "image=3 slot={} generation={} principal={} authority_session={} service={} grants={} ceiling={} source=reuse stale_denied=true payload_forgery_denied=true",
            fresh.slot,
            fresh.generation,
            c.principal,
            c.session,
            c.service,
            c.grant_ceiling,
            c.commitment_ceiling
        ),
    );
    launcher.destroy(r, store, fresh)?;
    need(
        r.inventory() == empty && launcher.live() == 0 && crate::paging::runtime_frames() == frames,
    )?;
    record(
        "m1.loader_ready",
        "static-loader",
        "ok",
        format_args!(
            "images=3 allocation_failures={failures} malformed=19 digest_rejected=true authority_edits=15 stale_denied=true settled=true frames={frames} user_execution=false"
        ),
    );
    Ok(())
}

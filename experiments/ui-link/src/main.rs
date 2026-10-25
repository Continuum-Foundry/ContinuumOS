//! Compile/link only. No boot protocol, loader, stack setup, device or kernel dependency.
#![no_std]
#![no_main]
extern crate alloc;
#[cfg(feature = "host-feature")]
extern crate std;
mod arena;
mod fixture;
use core::panic::PanicInfo;
use custom_ui::{
    input::{Controller, Event, EventKind},
    paint::Canvas,
    surface::Surface,
};
use editor_model::{Model, Selection};

#[global_allocator]
static ARENA: arena::Arena<{ 64 * 1024 * 1024 }> = arena::Arena::new();

#[cfg(feature = "missing-hook")]
unsafe extern "C" {
    fn p2_missing_present_hook(checksum: u64) -> u64;
}

/// Standalone one-shot link fixture. Caller would need mapped zeroed ELF BSS,
/// valid SysV stack, user FP/SIMD context and single-thread process ownership.
/// No caller exists yet. Seed/time are data inputs, never fabricated OS services.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.ui_probe_entry")]
pub extern "C" fn ui_probe_entry(seed: u64, milliseconds: u64) -> u64 {
    let mut scene = fixture::scene(seed).unwrap();
    let mut controller =
        Controller::new(Model::new(1, 1, "ffi العربية हिन्दी", "", Selection::new(0, 0)).unwrap())
            .unwrap();
    controller
        .event(
            &scene.tree,
            Event {
                sequence: 1,
                time_us: milliseconds.saturating_mul(1000),
                revision: scene.tree.revision,
                target: "editor".into(),
                kind: EventKind::Text("probe ".into()),
            },
        )
        .unwrap();
    let fonts = fixture::fonts();
    let mut engine = text_core::Engine::new(&fonts).unwrap();
    let mut surface = Surface::new(640, 360, 640 * 4, 1).unwrap();
    let lease = surface.acquire(1).unwrap();
    let canvas = Canvas::new(surface.write(lease).unwrap(), 640, 360, 640 * 4, 100).unwrap();
    #[cfg(feature = "slint-aot")]
    let mut canvas = canvas;
    #[cfg(feature = "slint-aot")]
    let counters = {
        let runtime = slint_ui::Runtime::new().unwrap();
        runtime.advance_to(milliseconds).unwrap();
        slint_ui::paint(&runtime, &mut scene, &controller, &mut engine, &mut canvas).unwrap()
    };
    #[cfg(not(feature = "slint-aot"))]
    let counters = {
        // Keep the final selected retained/damage/layout-cache path live in the
        // linked image. This remains a one-shot, non-executed closure fixture.
        scene.selected = true;
        let mut frame = custom_ui::retained::Frame::new(640, 360).unwrap();
        let (damage, counters) = frame
            .draw(
                &mut scene,
                controller.provider.model(),
                None,
                &mut engine,
                100,
            )
            .unwrap();
        assert!(!damage.regions.is_empty());
        canvas.bytes.copy_from_slice(&frame.pixels);
        counters
    };
    let mut checksum = seed ^ counters.shape_work ^ counters.nodes;
    for b in canvas.bytes.iter() {
        checksum = checksum.rotate_left(5) ^ u64::from(*b);
    }
    let semantics = scene.semantics(controller.provider.model());
    checksum ^= semantics.len() as u64;
    surface.commit(lease, &[[0, 0, 640, 360]]).unwrap();
    surface.release(lease).unwrap();
    // A final observable return keeps the real layout/action/text/raster/surface path live.
    checksum ^= ARENA.used() as u64;
    #[cfg(feature = "missing-hook")]
    {
        checksum = unsafe { p2_missing_present_hook(checksum) };
    }
    checksum
}
#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

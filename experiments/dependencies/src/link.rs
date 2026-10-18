#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    #[cfg(feature = "architecture")]
    core::hint::black_box(dependency_probes::architecture(core::hint::black_box(4096)));
    #[cfg(feature = "raster")]
    core::hint::black_box(dependency_probes::raster());
    #[cfg(feature = "shaping")]
    core::hint::black_box(dependency_probes::shaping(
        core::hint::black_box(&[]),
        "hello",
    ));
    #[cfg(feature = "font")]
    core::hint::black_box(dependency_probes::font(core::hint::black_box(&[])));
    #[cfg(feature = "toolkit")]
    {
        let mut pixels = [slint::Rgb8Pixel::default(); 64];
        dependency_probes::toolkit(&mut pixels, 64);
        core::hint::black_box(pixels);
    }
    loop {
        core::hint::spin_loop();
    }
}

// Deliberately no allocator or fake platform functions. A failed link is evidence.
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

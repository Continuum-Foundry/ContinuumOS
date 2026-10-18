#![no_std]

// These functions force API type-checking/code generation; none is a runtime test.
// Font bytes come from the future caller; no unlicensed host fonts are embedded.
#[cfg(feature = "architecture")]
pub fn architecture(address: u64) -> u64 {
    x86_64::PhysAddr::new(address).as_u64()
        + core::mem::size_of::<x86_64::structures::paging::PageTable>() as u64
}

#[cfg(feature = "raster")]
pub fn raster() -> u32 {
    let mut image = tiny_skia::Pixmap::new(16, 16).expect("bounded dimensions");
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(30, 60, 90, 255);
    let rectangle = tiny_skia::Rect::from_xywh(0.0, 0.0, 8.0, 8.0).unwrap();
    image.fill_rect(rectangle, &paint, tiny_skia::Transform::identity(), None);
    image.data().iter().map(|value| u32::from(*value)).sum()
}

#[cfg(feature = "shaping")]
pub fn shaping(font: &[u8], text: &str) -> usize {
    let Some(face) = rustybuzz::Face::from_slice(font, 0) else {
        return 0;
    };
    let mut input = rustybuzz::UnicodeBuffer::new();
    input.push_str(text);
    input.guess_segment_properties();
    rustybuzz::shape(&face, &[], input).glyph_infos().len()
}

#[cfg(feature = "font")]
pub fn font(bytes: &[u8]) -> usize {
    let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) else {
        return 0;
    };
    let (metrics, bitmap) = font.rasterize('A', 16.0);
    metrics.width + bitmap.len()
}

#[cfg(feature = "toolkit")]
pub fn toolkit(buffer: &mut [slint::Rgb8Pixel], width: usize) {
    use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    window.set_size(slint::PhysicalSize::new(width as u32, 1));
    window.draw_if_needed(|renderer| {
        renderer.render(buffer, width);
    });
}

#![allow(dead_code)]
pub use continuum_rt::*;
pub fn log(b: &[u8]) {
    let _ = continuum_rt::log(b);
}
// Freestanding compiler intrinsics; volatile scalar accesses prevent recursion.
#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(
    d: *mut core::ffi::c_void,
    s: *const core::ffi::c_void,
    n: usize,
) -> *mut core::ffi::c_void {
    for i in 0..n {
        unsafe {
            d.cast::<u8>()
                .add(i)
                .write_volatile(s.cast::<u8>().add(i).read_volatile());
        }
    }
    d
}
#[unsafe(no_mangle)]
unsafe extern "C" fn memset(d: *mut core::ffi::c_void, b: i32, n: usize) -> *mut core::ffi::c_void {
    for i in 0..n {
        unsafe {
            d.cast::<u8>().add(i).write_volatile(b as u8);
        }
    }
    d
}

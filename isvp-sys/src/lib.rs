#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::useless_transmute)]
mod wrapper;
pub use wrapper::*;

extern "C" {
    pub fn snap_pic(file_name: *const i8, fmt: IMPPixelFormat) -> ::std::os::raw::c_int;
    pub fn IMP_OSD_SetPoolSize(new_pool_size: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}

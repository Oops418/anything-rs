use std::ffi::CString;
use std::os::raw::c_char;

#[link(name = "quicklook_shim")]
unsafe extern "C" {
    unsafe fn ql_preview(path: *const c_char);
    unsafe fn ql_close();
}

pub fn preview(path: String) {
    let c = CString::new(path).expect("nul in path");
    unsafe { ql_preview(c.as_ptr()) }
}

pub fn close() {
    unsafe { ql_close() }
}

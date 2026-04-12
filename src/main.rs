use std::ffi::{CString, c_char, c_int};

unsafe extern "C" {
    unsafe fn cen64_main(argc: c_int, argv: *const *const c_char);
}

fn main() {
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).expect("Failed to convert argument"))
        .collect();

    // 2. Create a vector of raw pointers to those strings
    let mut arg_ptrs: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
    arg_ptrs.push(std::ptr::null());

    unsafe { cen64_main(arg_ptrs.len() as c_int, arg_ptrs.as_ptr()) }
}

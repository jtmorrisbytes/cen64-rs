use std::ffi::{c_char, c_void, CStr};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_fopen_hook(
    filename: *const c_char,
    mode: *const c_char,
) -> *mut c_void {
    let file = CStr::from_ptr(filename).to_string_lossy();
    let m = CStr::from_ptr(mode).to_string_lossy();

    println!(
        "[Rust Hijack] Attempting to open file: {} with mode: {}",
        file, m
    );

    // Trigger the VS Code debugger immediately
    // This is the "Panic Button" you wanted
    std::arch::asm!("int3");

    // After you hit "Continue" in the debugger, we need to call the REAL fopen.
    // On Windows MSVC, the real symbol is usually in ucrt.lib
    // We can use the 'libc' crate to find the original system call.
    panic!("does this work");
    libc::fopen(filename, mode) as *mut c_void
}

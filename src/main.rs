mod c_compat;
use std::ffi::{c_char, c_int, CString};

// add a exception handler that catches any 'weird windows things'
#[cfg(windows)]
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS,
};

#[cfg(windows)]
// This function runs the moment any thread hits an error (C or Rust)
unsafe extern "system" fn global_exception_handler(info: *mut EXCEPTION_POINTERS) -> i32 {
    let record = &*(*info).ExceptionRecord;
    let code = record.ExceptionCode;

    // 0xE06D7363 is a standard C++ exception (often noise)
    // We want to catch 0xC0000090 (FPU) or 0xC0000005 (Access Violation)
    println!(
        "Caught Exception: 0x{:X} at address: {:?}",
        code.0, record.ExceptionAddress
    );
    std::arch::asm!("int3");
    // if code.0 != 0xE06D7363_u32 as i32 {

    // This triggers the "INT3" trap, forcing VS Code to pause right here
    // }

    EXCEPTION_CONTINUE_SEARCH
}

// unsafe extern "C" {
//     unsafe fn cen64_main(argc: c_int, argv: *const *const c_char);
// }

fn main() {
    env_logger::init();
    // on windows, add a exception handler to catch FPU exceptions and such
    #[cfg(windows)]
    unsafe {
        // Registering as '1' puts us at the front of the line
        AddVectoredExceptionHandler(1, Some(global_exception_handler));
    }

    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).expect("Failed to convert argument"))
        .collect();

    // 2. Create a vector of raw pointers to those strings
    let mut arg_ptrs: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
    arg_ptrs.push(std::ptr::null());
    log::debug!("starting cen64");
    unsafe { cen64_rs::cen64_main_rs(arg_ptrs.len() as c_int, arg_ptrs.as_ptr()) };
}

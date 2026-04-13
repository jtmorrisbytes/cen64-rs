use std::os::raw::{c_void};
use core::ffi::{c_int,c_char};

#[repr(C)]

pub struct rom_file {
    pub ptr: *mut c_void,     // The actual data buffer
    pub size: usize,          // Size of the ROM
    pub mapping: *mut c_void, // Windows HANDLE to file mapping
    pub file: *mut c_void,    // Windows HANDLE to the file itself
}
unsafe extern "C" {
    /// Closes a ROM file and unmaps it from memory.
    pub fn close_rom_file(file: *const rom_file) -> c_int;

    /// Opens a ROM file and maps it into the process memory space.
    /// Returns 0 on success, non-zero on failure.
    pub fn open_rom_file(path: *const c_char, file: *mut rom_file) -> c_int;
}

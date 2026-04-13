use std::os::raw::{c_char, c_int, c_void};
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct save_file {
    pub ptr: *mut c_void,      // Pointer to the save data in memory
    pub size: usize,           // Size of the save (e.g., 32768 for SRAM)
    pub mapping: *mut c_void,  // Windows HANDLE (File Mapping)
    pub file: *mut c_void,     // Windows HANDLE (File)
}


extern "C" {
    // Closes a save file (SRAM, EEPROM, FlashRAM)
    pub fn close_save_file(file: *const save_file) -> c_int;

    // Opens/Creates a save file. Returns 0 on success.
    // 'created' is updated to 1 if a new file was made.
    pub fn open_save_file(
        path: *const c_char,
        size: usize,
        file: *mut save_file,
        created: *mut c_int,
    ) -> c_int;

    // Opens a Game Boy (Transfer Pak) save file
    pub fn open_gb_save(
        path: *const c_char,
        file: *mut save_file,
    ) -> c_int;
}

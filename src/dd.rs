use crate::common::rom::rom_file;
use core::ffi::c_void;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct dd_variant {
    pub description: *const core::ffi::c_char,
    pub seed: u8,
    pub sha1: [u8; 20], // SHA1_SIZE is 20
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct dd_controller {
    pub bus: *mut c_void,
    pub ipl_rom: *const u8,
    pub rom: *const u8,
    pub rom_size: usize,
    pub retail: bool,

    pub write: bool,
    pub track_offset: u32,
    pub zone: u32,
    pub start_block: u8,
    pub bm_reset_held: bool,

    // DD_REGS_ADDRESS_LEN is 0x100 (64 registers)
    pub regs: [u32; 64],
    // Buffer lengths: C2S (0x400), DS (0x100), MS_RAM (0x100)
    pub c2s_buffer: [u8; 1024],
    pub ds_buffer: [u8; 256],
    pub ms_ram: [u8; 256],

    pub rtc_offset_seconds: i32,
}
extern "C" {
    pub fn dd_identify_variant(ipl: *mut rom_file) -> *const dd_variant;

    pub fn dd_init(
        dd: *mut dd_controller,
        bus: *mut c_void,
        ddipl: *const u8,
        ddrom: *const u8,
        ddrom_size: usize,
    ) -> core::ffi::c_int;

    pub fn dd_pi_write(opaque: *mut core::ffi::c_void, address: u32);

    pub fn dd_dma_read(opaque: *mut core::ffi::c_void, source: u32, dest: u32, length: u32) -> core::ffi::c_int;
    pub fn dd_dma_write(opaque: *mut core::ffi::c_void, source: u32, dest: u32, length: u32) -> core::ffi::c_int;

    pub fn read_dd_ipl_rom(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_dd_ipl_rom(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;

    pub fn read_dd_controller(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_dd_controller(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
}

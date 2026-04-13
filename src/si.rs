use core::ffi::*;

use crate::{bus::bus_controller, common::{rom::rom_file, save::save_file}, dd::dd_variant};

#[repr(C)]
pub struct controller {
    pub mempak_path: *const c_char,
    pub mempak_save: save_file,

    pub pak: c_void, // Assuming this is an i32/u32 enum
    pub pak_enabled: c_int,
    pub present: c_int,

    pub tpak_rom_path: *const c_char,
    pub tpak_rom: rom_file,
    pub tpak_save_path: *const c_char,
    pub tpak_save: save_file,
    pub tpak_mode: c_int,
    pub tpak_mode_changed: c_int,
    pub tpak_bank: c_int,

    // The GB jump tables: 256 function pointers each
    pub gb_readmem: [Option<unsafe extern "C" fn(*mut controller, u16) -> u8>; 256],
    pub gb_writemem: [Option<unsafe extern "C" fn(*mut controller, u16, u8)>; 256],
    
    pub cart: c_void,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum si_register {
    SI_DRAM_ADDR_REG = 0,
    SI_PIF_ADDR_RD64B_REG = 1,
    SI_RESERVED_1_REG = 2,
    SI_RESERVED_2_REG = 3,
    SI_PIF_ADDR_WR64B_REG = 4,
    SI_RESERVED_3_REG = 5,
    SI_STATUS_REG = 6,
    NUM_SI_REGISTERS = 7,
}

pub const NUM_SI_REGISTERS: usize = 7;




#[repr(C)]
pub struct si_controller {
    bus: *mut bus_controller,
    rom: *mut u8,
    command: [u8;64],
    ram: [u8;64],
    regs: [u32;NUM_SI_REGISTERS],
    pif_status: u32,
    input: [u8;4],
    eeprom: eeprom,
    rtc: rtc,
    controller: [controller;4]
}
#[repr(C)]
struct rtc {
    control: u16,
    now: u32,
    offset_seconds: u32
}
#[repr(C)]
struct eeprom {
    data: *mut u8,
    size: usize
}
#[cfg(feature="debug_mmio_register_access")]
unsafe extern "C" {
    pub static si_register_mnemonics: [* const c_char;NUM_SI_REGISTERS];
}

unsafe extern "C" {
    pub fn si_init(
        si: *mut si_controller,
        bus: *mut bus_controller,
        pif_rom: *const u8,
        cart_rom: *const u8,
        dd_variant: *const dd_variant,
        eeprom: *mut u8,
        eeprom_size: usize,
        controller: *const controller,
    ) -> core::ffi::c_int;

    // MMIO Read/Write Hooks
    pub fn read_pif_rom_and_ram(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_pif_rom_and_ram(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;

    pub fn read_si_regs(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_si_regs(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
}
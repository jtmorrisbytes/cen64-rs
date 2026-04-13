pub mod cart_db;

use core::ffi::*;

use crate::{ai::ai_controller, bus::bus_controller, common::{rom::rom_file, save::save_file}, dd::dd_controller, pi::{is_viewer, pi_controller}, rdp::rdp, ri::ri_controller, rsp::rsp, si::{controller, si_controller}, vi::vi_controller, vr4300::vr4300};

// extern them for now


unsafe extern "C" {
    pub unsafe fn cart_db_is_well_formed() -> bool;
    pub unsafe fn cen64_alloc_init() -> c_int;
    pub unsafe fn check_extensions() -> c_int;
    pub unsafe fn check_command_line_usage() -> c_int;
    pub unsafe fn cen64_alloc_cleanup() -> c_int;
    pub unsafe fn parse_options() -> c_int;
    pub unsafe fn print_command_line_usage(argv: *const *const c_char);
    pub unsafe fn cen64_free(ptr: *mut c_void);
    pub unsafe fn close_rom_file(ptr: *mut c_void);
    pub unsafe fn check_start_from_explorer();
    // Rust sees these as 'static' variables managed by C


    // pub unsafe fn cen64_alloc_init()

    pub unsafe fn load_roms(
        ddipl_path: *const c_char,
        ddrom_path: *const c_char,
        pifrom_path: *const c_char,
        cart_path: *const c_char,
        ddipl: *mut rom_file,
        // dd_variant
        dd_variant: *mut *const c_void, // const struct dd_variant **
        ddrom: *mut rom_file,
        pifrom: *mut rom_file,
        cart: *mut rom_file,
    ) -> c_int;
    pub unsafe fn device_destroy(device: *mut cen64_device, cart_path: *const c_char);
        pub fn device_create(
        device: *mut cen64_device,
        ddipl: *const rom_file,
        dd_variant: *const c_void,
        ddrom: *const rom_file,
        pifrom: *const rom_file,
        cart: *const rom_file,
        eeprom: *const save_file,
        sram: *const save_file,
        flashram: *const save_file,
        is: *mut is_viewer,
        controller: *const controller,
        no_audio: bool,
        no_video: bool,
        profiling: bool,
    ) -> *mut cen64_device;

}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cen64_device {
    pub bus: bus_controller,
    pub vr4300: *mut vr4300,

    pub ai: ai_controller,
    pub dd: dd_controller,
    pub pi: pi_controller,
    pub ri: ri_controller,
    pub si: si_controller,
    pub vi: vi_controller,

    pub rdp: rdp,
    pub rsp: rsp,

    pub debug_sfd: core::ffi::c_int,

    pub multithread: bool,
    pub other_thread_is_waiting: bool,
    
    // These must match the platform-specific definitions for Windows
    pub sync_mutex: c_void, 
    pub sync_cv: c_void,

    pub running: bool,
}


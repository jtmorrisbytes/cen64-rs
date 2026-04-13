use crate::{ai::ai_controller, dd::dd_controller, pi::pi_controller, rdp::rdp, ri::ri_controller, rsp::rsp, si::si_controller, vi::vi_controller};

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct bus_controller {
    pub ai: *mut ai_controller,
    pub dd: *mut dd_controller,
    pub pi: *mut pi_controller,
    pub ri: *mut ri_controller,
    pub si: *mut si_controller,
    pub vi: *mut vi_controller,

    pub rdp: *mut rdp,
    pub rsp: *mut rsp,
    pub vr4300: *mut (),

    // The address decoder
    pub map: (),

    // THE LANDMINE: Windows MSVC jmp_buf size
    // On x64 Windows, jmp_buf is typically 16-byte aligned 
    // and usually 256 bytes (32 * 8-byte registers).
    pub unwind_data: [u64; 32], 
}
unsafe extern "C" {
    /// Initializes the bus and sets up the address map.
    /// 'dd_present' is a boolean/int (1 if 64DD is attached).
    pub fn bus_init(bus: *mut bus_controller, dd_present: core::ffi::c_int) -> core::ffi::c_int;

    /// The "Heartbeat" of memory access.
    pub fn bus_read_word(
        bus: *const bus_controller,
        address: u32,
        word: *mut u32,
    ) -> core::ffi::c_int;

    /// The "Hammer" that writes to hardware.
    pub fn bus_write_word(
        bus: *mut bus_controller,
        address: u32,
        word: u32,
        dqm: u32,
    ) -> core::ffi::c_int;
}



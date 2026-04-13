use crate::bus::bus_controller;

#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum vi_register {
    VI_STATUS_REG = 0,
    VI_ORIGIN_REG = 1,   // THE HOLY GRAIL: Pointer to pixels in RDRAM
    VI_WIDTH_REG = 2,    // Horizontal resolution
    VI_INTR_REG = 3,     // V-Sync interrupt trigger
    VI_CURRENT_REG = 4,  // Current scanline
    VI_BURST_REG = 5,
    VI_V_SYNC_REG = 6,
    VI_H_SYNC_REG = 7,
    VI_LEAP_REG = 8,
    VI_H_START_REG = 9,
    VI_V_START_REG = 10,
    VI_V_BURST_REG = 11,
    VI_X_SCALE_REG = 12,
    VI_Y_SCALE_REG = 13,
    NUM_VI_REGISTERS = 14,
}

pub const NUM_VI_REGISTERS: usize = 14;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct render_area_bounds {
    pub start: core::ffi::c_uint,
    pub end: core::ffi::c_uint,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct render_area {
    pub x: render_area_bounds,
    pub y: render_area_bounds,
    pub height: core::ffi::c_uint,
    pub width: core::ffi::c_uint,
    pub hskip: core::ffi::c_int,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct vi_controller {
    pub bus: *mut bus_controller,
    pub regs: [u32; NUM_VI_REGISTERS],

    pub counter: u32,
    // Note: C compilers usually add 4 bytes of padding here 
    // to align the 8-byte 'display' pointer.
    pub _pad: u32, 

    pub display: *mut core::ffi::c_void, // cen64_gl_display
    pub screen: core::ffi::c_int,        // cen64_gl_screen
    pub window: *mut core::ffi::c_void,  // cen64_gl_window
    pub context: *mut core::ffi::c_void, // cen64_gl_context

    pub render_area: render_area,
    pub viuv: [f32; 8],
    pub quad: [f32; 8],

    pub last_update_time: u64, // cen64_time
    pub intr_counter: core::ffi::c_uint,
    pub frame_count: core::ffi::c_uint,
    pub field: core::ffi::c_uint,
}
unsafe extern "C" {
    /// Initializes the VI. 
    /// Pass 'true' to 'no_interface' to bypass the legacy C window creation.
    pub fn vi_init(
        vi: *mut vi_controller, 
        bus: *mut bus_controller, 
        no_interface: bool
    ) -> core::ffi::c_int;

    /// Advances the VI state by one cycle (V-Sync/Scanline logic).
    pub fn vi_cycle(vi: *mut vi_controller);

    // MMIO Register Hooks
    pub fn read_vi_regs(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_vi_regs(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
}

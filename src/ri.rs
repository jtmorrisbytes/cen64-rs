use crate::bus::bus_controller;

pub const MAX_RDRAM_SIZE: usize = 0x800000;
pub const MAX_RDRAM_SIZE_MASK: usize = MAX_RDRAM_SIZE - 1;
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum rdram_register {
    RDRAM_CONFIG_REG = 0,
    RDRAM_DEVICE_ID_REG = 1,
    RDRAM_DELAY_REG = 2,
    RDRAM_MODE_REG = 3,
    RDRAM_REF_INTERVAL_REG = 4,
    RDRAM_REF_ROW_REG = 5,
    RDRAM_RAS_INTERVAL_REG = 6,
    RDRAM_MIN_INTERVAL_REG = 7,
    RDRAM_ADDR_SELECT_REG = 8,
    NUM_RDRAM_REGISTERS = 9,
}

pub const NUM_RDRAM_REGISTERS: usize = 9;
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum ri_register {
    RI_MODE_REG = 0,
    RI_CONFIG_REG = 1,
    RI_CURRENT_LOAD_REG = 2,
    RI_SELECT_REG = 3,
    RI_REFRESH_REG = 4,
    RI_LATENCY_REG = 5,
    RI_RERROR_REG = 6,
    RI_WERROR_REG = 7,
    NUM_RI_REGISTERS = 8,
}

pub const NUM_RI_REGISTERS: usize = 8;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct ri_controller {
    pub bus: *mut bus_controller,

    pub rdram_regs: [u32; NUM_RDRAM_REGISTERS],
    pub regs: [u32; NUM_RI_REGISTERS],

    pub force_ram_alignment: u64,
    
    // THE 8MB SLAB: Direct hardware memory
    pub ram: [u8; MAX_RDRAM_SIZE],
}
unsafe extern "C" {
    // Initialization
    pub fn ri_init(ri: *mut ri_controller, bus: *mut bus_controller) -> core::ffi::c_int;

    // THE HOT PATH: Main 8MB Memory Access
    pub fn read_rdram(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_rdram(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;

    // THE COLD PATH: Memory Controller Registers
    pub fn read_rdram_regs(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_rdram_regs(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;

    pub fn read_ri_regs(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_ri_regs(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
}

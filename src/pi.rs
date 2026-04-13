use crate::{bus::bus_controller, common::save::save_file};

pub const IS_VIEWER_BASE_ADDRESS: u32 = 0x13FF_0000;
pub const IS_VIEWER_ADDRESS_LEN: u32  = 0x0000_1000;
pub const FLASHRAM_SIZE: u32 =  0x20000;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct is_viewer {
    pub base_address: u32,
    pub len: u32,

    pub buffer: *mut u8,
    pub output_buffer: *mut u8,
    pub output_buffer_pos: usize,
    pub output_buffer_conv: *mut u8,
    pub show_output: core::ffi::c_int,
    pub output_warning: core::ffi::c_int,

    // The iconv handle (The Bottleneck)
    pub cd: *mut core::ffi::c_void,
}
unsafe extern "C" {
    /// Initializes the IS-Viewer state and sets up iconv.
    pub fn is_viewer_init(is: *mut is_viewer, show_output: core::ffi::c_int) -> core::ffi::c_int;

    /// Maps the viewer to a specific physical address (usually 0x13FF0000).
    pub fn is_viewer_map(is: *mut is_viewer, address: u32) -> core::ffi::c_int;

    /// Reads a 32-bit word from the viewer's internal buffer.
    pub fn read_is_viewer(is: *mut is_viewer, address: u32, word: *mut u32) -> core::ffi::c_int;

    /// Writes a 32-bit word to the viewer. This is where the game "logs" text.
    pub fn write_is_viewer(is: *mut is_viewer, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum pi_register {
    PI_DRAM_ADDR_REG = 0,
    PI_CART_ADDR_REG = 1,
    PI_RD_LEN_REG = 2,
    PI_WR_LEN_REG = 3,
    PI_STATUS_REG = 4,
    PI_BSD_DOM1_LAT_REG = 5,
    PI_BSD_DOM1_PWD_REG = 6,
    PI_BSD_DOM1_PGS_REG = 7,
    PI_BSD_DOM1_RLS_REG = 8,
    PI_BSD_DOM2_LAT_REG = 9,
    PI_BSD_DOM2_PWD_REG = 10,
    PI_BSD_DOM2_PGS_REG = 11,
    PI_BSD_DOM2_RLS_REG = 12,
    NUM_PI_REGISTERS = 13,
}

pub const NUM_PI_REGISTERS: usize = 13;

unsafe extern "C" {
    #[link_name = "debug_mmio_register_access"]
    pub static PI_MNEMONICS: [*const core::ffi::c_char; NUM_PI_REGISTERS];
}
#[repr(u32)]
pub enum pi_status {
    PI_STATUS_DMA_BUSY = 1 << 0,
    PI_STATUS_IO_BUSY = 1 << 1,
    PI_STATUS_ERROR = 1 << 2,
    PI_STATUS_INTERRUPT = 1 << 3,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct flashram {
    pub data: *mut u8,       // Points to your VirtualAlloc'd save buffer
    pub status: u64,
    pub mode: flashram_mode, // enum size matches c_int
    pub offset: usize,
    pub rdram_pointer: usize,
}

#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum flashram_mode {
    FLASHRAM_IDLE = 0,
    FLASHRAM_ERASE = 1,
    FLASHRAM_WRITE = 2,
    FLASHRAM_READ = 3,
    FLASHRAM_STATUS = 4,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct pi_controller {
    pub bus: *mut bus_controller,

    pub rom: *const u8,
    pub rom_size: usize,
    pub sram: *const save_file,
    pub flashram_file: *const save_file,
    
    // Nested struct (mapped previously)
    pub flashram: flashram,
    pub is_viewer: *mut is_viewer,

    pub counter: u64,
    pub bytes_to_copy: u32,
    pub is_dma_read: bool,
    
    // Padding often inserted by C here to align the u32 array 
    // to an 8-byte boundary if bool is 1 byte
    pub _pad: [u8; 3], 

    pub regs: [u32; NUM_PI_REGISTERS],
}
extern "C" {
    // Initialization
    pub fn pi_init(
        pi: *mut pi_controller,
        bus: *mut bus_controller,
        rom: *const u8,
        rom_size: usize,
        sram: *const save_file,
        flashram: *const save_file,
        is: *mut is_viewer,
    ) -> core::ffi::c_int;

    // The core hardware "tick"
    pub fn pi_cycle_(pi: *mut pi_controller);

    // Register & Memory Access
    pub fn read_pi_regs(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_pi_regs(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
    
    pub fn read_cart_rom(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_cart_rom(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;

    pub fn read_flashram(opaque: *mut core::ffi::c_void, address: u32, word: *mut u32) -> core::ffi::c_int;
    pub fn write_flashram(opaque: *mut core::ffi::c_void, address: u32, word: u32, dqm: u32) -> core::ffi::c_int;
}

use core::{ffi::{c_char, c_int,c_void,c_uint}};
pub mod vr4300;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct rsp {
    pub bus: *mut bus_controller,
    pub pipeline: (),
    pub cp2: (),

    pub regs: [u32; NUM_RSP_REGISTERS],
    pub mem: [u8; SP_MEM_SIZE],

    // The Software Instruction Cache
    pub opcode_cache: [rsp_opcode; RSP_CACHE_SIZE],

    // The JIT Slabs (SSE2 optimized)
    pub vload_dynarec: [u8;0],
    pub vstore_dynarec: [u8;0],
}
// pub const NUM_RSP_REGISTERS: usize = 8;
pub const SP_MEM_SIZE: usize = 0x2000; // 8KB (4KB IMEM + 4KB DMEM)
pub const RSP_CACHE_SIZE: usize = 0x1000 / 4; // 1024 entries
pub const NUM_RSP_REGISTERS: usize = 40;


// decoder

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct rsp_opcode {
    /// The decoded instruction ID (maps to the rsp_vector_opcode enum)
    pub id: u32,
    /// Hardware flags (e.g., is_vector, is_branch, is_load)
    pub flags: u32,
}


#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum dp_register {
    DPC_START_REG = 0,    // Start address of the RDP command list
    DPC_END_REG = 1,      // End address (triggers execution)
    DPC_CURRENT_REG = 2,  // Current execution pointer
    DPC_STATUS_REG = 3,   // RDP Status (Busy, Paused, etc.)
    DPC_CLOCK_REG = 4,    // RDP Clock counter
    DPC_BUFBUSY_REG = 5,  // Command buffer status
    DPC_PIPEBUSY_REG = 6, // Processing pipeline status
    DPC_TMEM_REG = 7,     // Texture Memory status
    NUM_DP_REGISTERS = 8,
}
pub mod dp_status {
    pub const XBUS_DMA: u32 = 0x001;
    pub const FREEZE: u32 = 0x002;
    pub const FLUSH: u32 = 0x004;
    pub const START_GCLK: u32 = 0x008;
    pub const TMEM_BUSY: u32 = 0x010;
    pub const PIPE_BUSY: u32 = 0x020;
    pub const CMD_BUSY: u32 = 0x040; // Set this when Vulkan is crunching
    pub const CBUF_READY: u32 = 0x080;
    pub const DMA_BUSY: u32 = 0x100;
    pub const END_VALID: u32 = 0x200;
    pub const START_VALID: u32 = 0x400;
}

#[repr(C)]

pub struct rom_file {
    pub ptr: *mut c_void,     // The actual data buffer
    pub size: usize,          // Size of the ROM
    pub mapping: *mut c_void, // Windows HANDLE to file mapping
    pub file: *mut c_void,    // Windows HANDLE to the file itself
}


#[repr(C)]
pub struct controller {
    pub mempak_path: *const c_char,
    pub mempak_save: save_file,

    pub pak: (), // Assuming this is an i32/u32 enum
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

    pub cart: (),
}




#[repr(C)]
struct rtc {
    control: u16,
    now: u32,
    offset_seconds: u32,
}
#[repr(C)]
struct eeprom {
    data: *mut u8,
    size: usize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct save_file {
    pub ptr: *mut c_void,     // Pointer to the save data in memory
    pub size: usize,          // Size of the save (e.g., 32768 for SRAM)
    pub mapping: *mut c_void, // Windows HANDLE (File Mapping)
    pub file: *mut c_void,    // Windows HANDLE (File)
}

pub const NUM_SI_REGISTERS: usize = 7;
#[repr(C)]
pub struct si_controller {
    bus: *mut bus_controller,
    rom: *mut u8,
    command: [u8; 64],
    ram: [u8; 64],
    regs: [u32; NUM_SI_REGISTERS],
    pif_status: u32,
    input: [u8; 4],
    eeprom: eeprom,
    rtc: rtc,
    controller: [controller; 4],
}
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
    pub vr4300: *mut c_void,

    // The address decoder
    pub map: (),

    // THE LANDMINE: Windows MSVC jmp_buf size
    // On x64 Windows, jmp_buf is typically 16-byte aligned
    // and usually 256 bytes (32 * 8-byte registers).
    pub unwind_data: [u64; 32],
}
pub const MAX_RDRAM_SIZE: usize = 0x800000;
pub const NUM_RDRAM_REGISTERS: usize = 9;
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
pub const NUM_VI_REGISTERS: usize = 14;


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
pub struct render_area_bounds {
    pub start: core::ffi::c_uint,
    pub end: core::ffi::c_uint,
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

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cen64_device {
    pub bus: bus_controller,
    pub vr4300: *mut c_void,

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
    pub sync_mutex: (),
    pub sync_cv: (),

    pub running: bool,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct flashram {
    pub data: *mut u8, // Points to your VirtualAlloc'd save buffer
    pub status: u64,
    pub mode: c_int, // enum size matches c_int
    pub offset: usize,
    pub rdram_pointer: usize,
}
pub const NUM_PI_REGISTERS: usize = 13;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct pi_controller {
    pub bus: *mut c_void,

    pub rom: *const u8,
    pub rom_size: usize,
    pub sram: *const c_void,
    pub flashram_file: *const c_void,

    // Nested struct (mapped previously)
    pub flashram: flashram,
    pub is_viewer: *mut c_void,

    pub counter: u64,
    pub bytes_to_copy: u32,
    pub is_dma_read: bool,

    // Padding often inserted by C here to align the u32 array
    // to an 8-byte boundary if bool is 1 byte
    pub _pad: [u8; 3],

    pub regs: [u32; NUM_PI_REGISTERS],
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

pub const NUM_AI_REGISTERS: usize = 6;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cen64_ai_context {
    pub buffers: [c_uint; 3], // Guaranteed 32-bit
    pub unqueued_buffers: c_uint,
    pub cur_frequency: c_uint,
    pub frequency: c_uint,
    pub source: c_uint,

    // openal-sys pointers already handle platform-specific sizing
    pub dev: *mut c_void,
    pub ctx: *mut c_void,
}

#[repr(C)]
pub struct ai_fifo_entry {
    pub address: u32,
    pub length: u32,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct ai_controller {
    pub bus: *mut bus_controller,
    pub regs: [u32; NUM_AI_REGISTERS],

    pub ctx: cen64_ai_context,
    pub counter: u64,

    pub fifo_count: core::ffi::c_uint,
    pub fifo_wi: core::ffi::c_uint,
    pub fifo_ri: core::ffi::c_uint,

    // The N64 AI has a 2-slot hardware FIFO
    pub fifo: [ai_fifo_entry; 2],
    pub no_output: bool,
}

pub const NUM_DP_REGISTERS: usize = 8;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct rdp {
    pub regs: [u32; NUM_DP_REGISTERS],
    pub bus: *mut bus_controller,
}
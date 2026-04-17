use crate::bus_controller;

// pub const NUM_VR4300_REGISTERS: usize = 34; // R0-R31 + HI + LO
pub const NUM_MI_REGISTERS: usize = 4;
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum vr4300_register {
    // 0-31: General Purpose (R0-RA)
    VR4300_REGISTER_R0 = 0,
    VR4300_REGISTER_AT,
    VR4300_REGISTER_V0,
    VR4300_REGISTER_V1,
    VR4300_REGISTER_A0,
    VR4300_REGISTER_A1,
    VR4300_REGISTER_A2,
    VR4300_REGISTER_A3,
    VR4300_REGISTER_T0,
    VR4300_REGISTER_T1,
    VR4300_REGISTER_T2,
    VR4300_REGISTER_T3,
    VR4300_REGISTER_T4,
    VR4300_REGISTER_T5,
    VR4300_REGISTER_T6,
    VR4300_REGISTER_T7,
    VR4300_REGISTER_S0,
    VR4300_REGISTER_S1,
    VR4300_REGISTER_S2,
    VR4300_REGISTER_S3,
    VR4300_REGISTER_S4,
    VR4300_REGISTER_S5,
    VR4300_REGISTER_S6,
    VR4300_REGISTER_S7,
    VR4300_REGISTER_T8,
    VR4300_REGISTER_T9,
    VR4300_REGISTER_K0,
    VR4300_REGISTER_K1,
    VR4300_REGISTER_GP,
    VR4300_REGISTER_SP,
    VR4300_REGISTER_FP,
    VR4300_REGISTER_RA,

    // 32-63: Coprocessor 0 (TLB, Exceptions, Status)
    VR4300_REGISTER_CP0_0,
    /* ... skip to ... */ VR4300_REGISTER_CP0_31 = 63,

    // 64-95: Coprocessor 1 (Floating Point)
    VR4300_REGISTER_CP1_0,
    /* ... skip to ... */ VR4300_REGISTER_CP1_31 = 95,

    // 96-98: Mul/Div and FPU Control
    VR4300_REGISTER_HI = 96,
    VR4300_REGISTER_LO = 97,
    VR4300_CP1_FCR0 = 98,
    VR4300_CP1_FCR31 = 99,

    // The "Cycle-Type" Hack
    PIPELINE_CYCLE_TYPE = 100,

    NUM_VR4300_REGISTERS = 101,
}

pub const NUM_VR4300_REGISTERS: usize = 101;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct vr4300 {
    pub bus: *mut bus_controller,
    pub pipeline: vr4300_pipeline,

    pub regs: [u64; NUM_VR4300_REGISTERS],
    pub mi_regs: [u32; NUM_MI_REGISTERS],

    pub signals: core::ffi::c_uint,
    pub cp0: [u8; 0],

    pub dcache: [u8; 0],
    pub icache: [u8; 0],

    pub profile_samples: *mut u64,

    pub debug: [u8; 0],
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_pipeline {
    // Stage latches
    pub dcwb_latch: vr4300_dcwb_latch,
    pub exdc_latch: vr4300_exdc_latch,
    pub rfex_latch: vr4300_rfex_latch,
    pub icrf_latch: vr4300_icrf_latch,

    // Instruction and status history
    pub exception_history: u32, // 'unsigned' in C is u32
    pub cycles_to_stall: u32,
    pub fault_present: bool,    // Rust's bool is 1 byte, matching C99's bool
}
#[repr(u32)] // Match the C enum size
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum vr4300_fault_id {
    // You'll need to fill these in from the original enum header
    None = 0,
    Interrupt = 1,
    // ...
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_latch {
    pub pc: u64,
    pub fault: vr4300_fault_id,
    pub cause_data: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_icrf_latch {
    pub common: vr4300_latch,
    pub segment: *const segment, // Replace with your Rust Segment type
    pub pc: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_rfex_latch {
    pub common: vr4300_latch,
    pub opcode: vr4300_opcode, // Ensure this is also #[repr(C)]
    pub iw: u32,
    pub iw_mask: u32,
    pub paddr: u32,
    pub cached: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_exdc_latch {
    pub common: vr4300_latch,
    pub segment: *const segment,
    pub result: i64,
    pub dest: u32,
    pub request: vr4300_bus_request, // Ensure this is also #[repr(C)]
    pub cached: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_dcwb_latch {
    pub common: vr4300_latch,
    pub result: i64,
    pub dest: u32,
    pub last_op_was_cache_store: bool,
}
pub type vr4300_cacheop_func_t = Option<
    unsafe extern "C" fn(
        vr4300: *mut vr4300, // Pointer to your main CPU struct
        vaddr: u64,
        paddr: u32,
    ) -> std::os::raw::c_int,
>;
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum vr4300_bus_request_type {
    None = 0,
    Read = 1,
    Write = 2,
    Cache = 4,    // Note: C uses 4 for both Cache and CacheIdx
    CacheWrite = 5,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum vr4300_access_type {
    Word = 1 << 5,
    DWord = 0,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum vr4300_fmt {
    S = 16, // Single precision float
    D = 17, // Double precision float
    W = 20, // 32-bit fixed point word
    L = 21, // 64-bit fixed point long
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct vr4300_opcode {
    pub id: u32,
    pub flags: u32,
}
#[repr(C)]
#[derive(Clone, Copy,Debug)]
pub struct vr4300_bus_request {
    pub vaddr: u64,
    pub data: u64,
    pub wdqm: u64, // Write Data Quadrant Mask

    pub cacheop: vr4300_cacheop_func_t,
    pub paddr: u32,

    pub access_type: vr4300_access_type,
    pub request_type: vr4300_bus_request_type,
    pub size: u32,      // 'unsigned' in C
    pub postshift: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct segment {
    pub start: u64,
    pub length: u64,
    pub offset: u64,

    pub xmode_mask: u8,
    pub mapped: bool,
    pub cached: bool,
}

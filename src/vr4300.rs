use crate::bus::bus_controller;

// pub const NUM_VR4300_REGISTERS: usize = 34; // R0-R31 + HI + LO
pub const NUM_MI_REGISTERS: usize = 4;
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum vr4300_register {
    // 0-31: General Purpose (R0-RA)
    VR4300_REGISTER_R0 = 0, VR4300_REGISTER_AT, VR4300_REGISTER_V0,
    VR4300_REGISTER_V1, VR4300_REGISTER_A0, VR4300_REGISTER_A1,
    VR4300_REGISTER_A2, VR4300_REGISTER_A3, VR4300_REGISTER_T0,
    VR4300_REGISTER_T1, VR4300_REGISTER_T2, VR4300_REGISTER_T3,
    VR4300_REGISTER_T4, VR4300_REGISTER_T5, VR4300_REGISTER_T6,
    VR4300_REGISTER_T7, VR4300_REGISTER_S0, VR4300_REGISTER_S1,
    VR4300_REGISTER_S2, VR4300_REGISTER_S3, VR4300_REGISTER_S4,
    VR4300_REGISTER_S5, VR4300_REGISTER_S6, VR4300_REGISTER_S7,
    VR4300_REGISTER_T8, VR4300_REGISTER_T9, VR4300_REGISTER_K0,
    VR4300_REGISTER_K1, VR4300_REGISTER_GP, VR4300_REGISTER_SP,
    VR4300_REGISTER_FP, VR4300_REGISTER_RA,

    // 32-63: Coprocessor 0 (TLB, Exceptions, Status)
    VR4300_REGISTER_CP0_0, /* ... skip to ... */ VR4300_REGISTER_CP0_31 = 63,

    // 64-95: Coprocessor 1 (Floating Point)
    VR4300_REGISTER_CP1_0, /* ... skip to ... */ VR4300_REGISTER_CP1_31 = 95,

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
    pub pipeline: (),

    pub regs: [u64; NUM_VR4300_REGISTERS],
    pub mi_regs: [u32; NUM_MI_REGISTERS],

    pub signals: core::ffi::c_uint,
    pub cp0: [u8;0],

    pub dcache: [u8;0],
    pub icache: [u8;0],

    pub profile_samples: *mut u64,

    pub debug: [u8;0],
}

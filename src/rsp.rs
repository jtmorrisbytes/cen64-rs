use crate::{bus::bus_controller, dynarec::dynarec_slab};

// pub const NUM_RSP_REGISTERS: usize = 8;
pub const SP_MEM_SIZE: usize = 0x2000; // 8KB (4KB IMEM + 4KB DMEM)
pub const RSP_CACHE_SIZE: usize = 0x1000 / 4; // 1024 entries

#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum rsp_vector_opcode {
    VINVALID = 0,
    VABS, VADD, VADDC, VAND, VCH, VCL, VCR, VEQ, VGE,
    VLT, VMACF, VMACQ, VMACU, VMADH, VMADL, VMADM, VMADN,
    VMOV, VMRG, VMUDH, VMUDL, VMUDM, VMUDN, VMULF, VMULQ,
    VMULU, VNAND, VNE, VNOP, VNOR, VNULL, VNXOR, VOR, VRCP,
    VRCPH, VRCPL, VRNDN, VRNDP, VRSQ, VRSQH, VRSQL, VSAR,
    VSUB, VSUBC, VXOR,
}
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum sp_register {
    // These typically include:
    // MEM_ADDR, DRAM_ADDR, RD_LEN, WR_LEN, STATUS, DMA_FULL, DMA_BUSY, SEMAPHORE
    // Generated via X-macro in C
    SP_MEM_ADDR_REG = 0,
    SP_DRAM_ADDR_REG = 1,
    SP_RD_LEN_REG = 2,
    SP_WR_LEN_REG = 3,
    SP_STATUS_REG = 4,
    SP_DMA_FULL_REG = 5,
    SP_DMA_BUSY_REG = 6,
    SP_SEMAPHORE_REG = 7,
    
    NUM_SP_REGISTERS = 8,
}

// This connects the RSP registers to the CPU's Coprocessor 0 
pub const SP_REGISTER_OFFSET: usize = 0x04040000; // Typical MIPS CP0 mapping

#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum rsp_register {
    // GPRs (R0 - RA)
    RSP_REGISTER_R0 = 0, RSP_REGISTER_AT, RSP_REGISTER_V0,
    RSP_REGISTER_V1, RSP_REGISTER_A0, RSP_REGISTER_A1,
    RSP_REGISTER_A2, RSP_REGISTER_A3, RSP_REGISTER_T0,
    RSP_REGISTER_T1, RSP_REGISTER_T2, RSP_REGISTER_T3,
    RSP_REGISTER_T4, RSP_REGISTER_R5, RSP_REGISTER_T6,
    RSP_REGISTER_T7, RSP_REGISTER_S0, RSP_REGISTER_S1,
    RSP_REGISTER_S2, RSP_REGISTER_S3, RSP_REGISTER_S4,
    RSP_REGISTER_S5, RSP_REGISTER_S6, RSP_REGISTER_S7,
    RSP_REGISTER_T8, RSP_REGISTER_T9, RSP_REGISTER_K0,
    RSP_REGISTER_K1, RSP_REGISTER_GP, RSP_REGISTER_SP,
    RSP_REGISTER_FP, RSP_REGISTER_RA,

    // CP0 registers (SP_MEM_ADDR, SP_DRAM_ADDR, etc.)
    RSP_REGISTER_CP0_0, RSP_REGISTER_CP0_1, RSP_REGISTER_CP0_2,
    RSP_REGISTER_CP0_3, RSP_REGISTER_CP0_4, RSP_REGISTER_CP0_5,
    RSP_REGISTER_CP0_6, RSP_REGISTER_CP0_7,

    NUM_RSP_REGISTERS = 40,
}

pub const NUM_RSP_REGISTERS: usize = 40;



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
    pub vload_dynarec: dynarec_slab,
    pub vstore_dynarec: dynarec_slab,
}


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

// Register Extraction
pub const fn get_rs(iw: u32) -> usize { (iw >> 21 & 0x1F) as usize }
pub const fn get_rt(iw: u32) -> usize { (iw >> 16 & 0x1F) as usize }
pub const fn get_rd(iw: u32) -> usize { (iw >> 11 & 0x1F) as usize }

// Vector Extraction (RSP Specific)
pub const fn get_vs(iw: u32) -> usize { (iw >> 11 & 0x1F) as usize }
pub const fn get_vt(iw: u32) -> usize { (iw >> 16 & 0x1F) as usize }
pub const fn get_vd(iw: u32) -> usize { (iw >> 6 & 0x1F) as usize }
pub const fn get_el(iw: u32) -> usize { (iw >> 7 & 0xF) as usize }

// Instruction Metadata Flags (Matches OPCODE_INFO_*)
pub const INFO_NONE: u32   = 0;
pub const INFO_VECTOR: u32 = 1 << 1;
pub const INFO_BRANCH: u32 = 1 << 31;
pub const INFO_LOAD: u32   = 1 << 5;
pub const INFO_STORE: u32  = 1 << 6;

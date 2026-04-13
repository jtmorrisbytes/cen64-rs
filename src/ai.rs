use crate::bus::bus_controller;
use openal_sys::{ALuint, ALCdevice, ALCcontext};
pub const NUM_AI_REGISTERS: usize = 6;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum ai_register {
    AI_DRAM_ADDR_REG = 0,
    AI_LEN_REG = 1,
    AI_CONTROL_REG = 2,
    AI_STATUS_REG = 3,
    AI_DACRATE_REG = 4,
    AI_BITRATE_REG = 5,
    NUM_AI_REGISTERS = 6,
}



#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cen64_ai_context {
    pub buffers: [ALuint; 3],          // Guaranteed 32-bit
    pub unqueued_buffers: ALuint,
    pub cur_frequency: ALuint,
    pub frequency: ALuint,
    pub source: ALuint,

    // openal-sys pointers already handle platform-specific sizing
    pub dev: *mut ALCdevice, 
    pub ctx: *mut ALCcontext,
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

#[repr(C)]
pub struct ai_fifo_entry {
    pub address: u32,
    pub length: u32,
}

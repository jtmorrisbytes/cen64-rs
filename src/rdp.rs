use crate::bus::bus_controller;

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

pub const NUM_DP_REGISTERS: usize = 8;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct rdp {
    pub regs: [u32; NUM_DP_REGISTERS],
    pub bus: *mut bus_controller,
}

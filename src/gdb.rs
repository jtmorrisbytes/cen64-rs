use std::os::raw::c_void;

use crate::device::cen64_device;

// Verify these in the C headers!
pub const MAX_GDB_PACKET_SIZE: usize = 0x4000;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct gdb {
    pub socket: core::ffi::c_int,
    pub client: core::ffi::c_int,
    pub device: *mut cen64_device,
    pub pending_data: core::ffi::c_int,
    
    // Fixed-size buffers must match the C array sizes exactly
    pub packet_buffer: [core::ffi::c_char; MAX_GDB_PACKET_SIZE * 2],
    pub output_buffer: [core::ffi::c_char; MAX_GDB_PACKET_SIZE],
    
    pub flags: core::ffi::c_int,
    
    // Platform-specific threading primitives
    pub thread: c_void,
    pub client_mutex: c_void,
    pub client_semaphore: c_void,
}

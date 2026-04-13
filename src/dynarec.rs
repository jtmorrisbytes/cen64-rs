#[repr(C)]
#[allow(non_camel_case_types)]
pub struct dynarec_slab {
    pub size: usize,
    pub ptr: *mut u8, // Points to R-X (Executable) memory
}

use core::ffi::*;

use crate::device::cen64_device;
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cen64_mem {
    pub ptr: *mut cen64_device, // Pointer to the allocated block
    pub size: usize,            // Total size of the allocation
}
unsafe extern "C" {
    /// Global allocator cleanup (usually called at process exit)
    pub fn cen64_alloc_cleanup();

    /// Global allocator setup (call this first in main!)
    pub fn cen64_alloc_init() -> c_int;

    /// Main allocation function.
    /// if exec is true, the memory is marked as executable (for JIT/RSP logic).
    // pub fn cen64_alloc(m: *mut cen64_mem, size: usize, exec: bool) -> *mut c_void;

    /// Frees memory associated with a cen64_mem slab.
    pub fn cen64_free(m: *mut cen64_mem);
}

#[no_mangle]
pub unsafe extern "C" fn cen64_alloc(m: *mut cen64_mem, size: usize, exec: bool) -> *mut c_void {
    #[cfg(unix)]
    {
        use libc::{
            mmap, open, MAP_ANON, MAP_FAILED, MAP_PRIVATE, O_RDONLY, PROT_EXEC, PROT_READ,
            PROT_WRITE,
        };
        // 1. Open /dev/zero to get the "Zero Page" file descriptor
        let mut fd = open("/dev/zero\0".as_ptr() as *const i8, O_RDONLY);
        if fd < 0 {
            return core::ptr::null_mut();
        }

        let mut flags: core::ffi::c_int = MAP_PRIVATE;
        let mut perm: core::ffi::c_int = PROT_READ | PROT_WRITE;
        if exec {
            perm |= PROT_EXEC
        }
        #[cfg(target_os = "macos")]
        {
            FLAGS |= MAP_PRIVATE;
            FD = -1;
        }

        // 3. Map it (This is what Tyler's C code is doing)
        let ptr = mmap(
            core::ptr::null_mut(),
            size,
            prot,
            flags,
            fd, // The /dev/zero file descriptor
            0,
        );
        // 4. Apple manual zeroing (Tyler's quirk)
        #[cfg(target_os = "macos")]
        libc::memset(ptr, 0, size);
    }
    #[cfg(all(not(unix), windows))]
    {
        use windows::Win32::System::Memory::MEM_COMMIT;
        use windows::Win32::System::Memory::MEM_RESERVE;
        use windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE;
        use windows::Win32::System::Memory::PAGE_READWRITE;
        use windows::Win32::System::Memory::VirtualAlloc;

        let access = {
            if exec {
                PAGE_EXECUTE_READWRITE
            } else {
                PAGE_READWRITE
            }
        };
        unsafe {

            let ptr = VirtualAlloc(None, size, MEM_COMMIT | MEM_RESERVE, access);
            debug_assert_eq!(ptr.is_null(),false);
            (*m).ptr = ptr.cast();
        }
        (*m).ptr.cast()
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        compile_error!("unsupported operating system")
    }
}

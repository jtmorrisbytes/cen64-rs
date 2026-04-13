#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// pub use device::cart_db::*;

mod common;
pub mod dd;
pub mod device;
pub mod gdb;
pub mod options;
pub mod pi;
pub mod si;
pub mod ai;
pub mod bus;
pub mod ri;
pub mod vi;
pub mod rdp;
pub mod rsp;
pub mod dynarec;
pub mod vr4300;
use common::rom::rom_file;

use clap::Parser;
use core::ffi::*;
use device::*;
use libc::memset;

use crate::{
    common::{
        alloc::{cen64_alloc, cen64_mem},
        save::{open_save_file, save_file},
    },
    dd::dd_variant,
    device::cart_db::{
        cart_db_entry, cart_db_get_entry, cart_db_save_type, CART_DB_SAVE_TYPE_EEPROM_4KBIT,
        CART_DB_SAVE_TYPE_SRAM_1MBIT, CART_DB_SAVE_TYPE_SRAM_256KBIT,
        CART_DB_SAVE_TYPE_SRAM_768KBIT,
    },
    options::cen64_options,
    pi::{is_viewer, is_viewer_init, FLASHRAM_SIZE},
    si::controller,
};

// #[unsafe(no_mangle)]
#[link_section = ".data"]
pub static mut device: *mut cen64_device = std::ptr::null_mut();
unsafe extern "C" {
    // #[unsafe(no_mangle)]
    pub unsafe static mut options: cen64_options;
    pub unsafe static mut ddipl: rom_file;
    pub unsafe static mut ddrom: rom_file;
    pub unsafe static mut pifrom: rom_file;
    pub unsafe static mut cart: rom_file;
    pub unsafe static mut controller: [controller; 4];
    // #[link_name = "device"]
    pub unsafe static mut is_in: *mut is_viewer;
    pub static mut eeprom: save_file;
    pub static mut sram: save_file;
    pub static mut flashram: save_file;
    pub static mut is: is_viewer;
    pub static mut cart_info: *const cart_db_entry;
    pub static mut dd_variant: *const dd_variant;
    pub static mut save_type: cart_db_save_type;
    pub static mut debugger: *mut gdb::gdb;
    // pub static mut device: *mut cen64_device;
    pub fn run_device(device: *mut cen64_device, no_video: bool) -> core::ffi::c_int;
    pub fn load_paks(controller: *mut controller) -> core::ffi::c_int;
    // This matches: struct cen64_mem cen64_device_mem;
    pub static mut cen64_device_mem: cen64_mem;
    // pub static mut is_in: *mut is_viewer;

}

const EXIT_FAILURE: c_int = -1;

// Called when another simulation instance is desired.
// struct controller controller[4] = { { 0, }, };
// struct rom_file ddipl, ddrom, pifrom, cart;
// struct cen64_options options = default_cen64_options;
// struct p_ddvariant(*const dd_variant);
// struct cen64_mem cen64_device_mem;
// struct cen64_device *device;
// const struct cart_db_entry *cart_info;
// struct save_file eeprom;
// struct save_file sram;
// struct save_file flashram;
// struct is_viewer is, *is_in = NULL;
pub fn cen64_main_rs(argc: core::ffi::c_int, argv: *const *const c_char) -> c_int {
    log::debug!("starting cen64");
    // let mut device: cen64_device = unsafe {std::mem::zeroed()};
    // let mut device_ptr: *mut cen64_device = &raw mut device;
    unsafe {
        //   options.controller = controller;
        let mut status: c_int = 0;
        unsafe {
            #[cfg(windows)]
            {
                check_start_from_explorer();
            }

            if (!cart_db_is_well_formed()) {
                println!("Internal cart detection database is not well-formed.\n");
                return EXIT_FAILURE;
            }

            if (cen64_alloc_init() != 0) {
                println!("Failed to initialize the low-level allocators.\n");
                return EXIT_FAILURE;
            }

            if (check_extensions() != 0) {
                return EXIT_FAILURE;
            }
        }

        //   if (argc < 3) {
        //     print_command_line_usage(argv[0]);
        //     cen64_alloc_cleanup();
        //     return EXIT_SUCCESS;
        //   }
        let ops = crate::options::Cen64Args::try_parse().inspect_err(|e| log::error!("{e}"));
        if ops.is_err() {
            unsafe { cen64_alloc_cleanup() };
            return EXIT_FAILURE;
        }
        let ops = ops.unwrap();
        
        
        
        options.pifrom_path = std::ffi::CString::new(ops.pif_rom.to_str().unwrap())
            .unwrap()
            .into_raw();
        options.cart_path = std::ffi::CString::new(ops.cart_rom.unwrap().to_str().unwrap())
            .unwrap()
            .into_raw();
        
        options.multithread = ops.multithread;
        options.no_audio = ops.noaudio;
        options.no_video = ops.novideo;
        // parse_options();
        
        unsafe {
            libc::memset(
                std::ptr::from_mut(&mut ddipl).cast(),
                0,
                std::mem::size_of_val(&ddipl),
            );
            libc::memset(
                (&raw mut ddrom).cast::<core::ffi::c_void>(),
                0,
                core::mem::size_of_val(&ddrom),
            );

            libc::memset(
                core::ptr::from_mut(&mut cart).cast::<core::ffi::c_void>(),
                0,
                core::mem::size_of_val(&cart),
            );

            libc::memset(
                core::ptr::from_mut(&mut eeprom).cast::<core::ffi::c_void>(),
                0,
                core::mem::size_of_val(&eeprom),
            );

            libc::memset(
                core::ptr::from_mut(&mut sram).cast::<core::ffi::c_void>(),
                0,
                core::mem::size_of_val(&sram),
            );

            libc::memset(
                core::ptr::from_mut(&mut flashram).cast::<core::ffi::c_void>(),
                0,
                core::mem::size_of_val(&flashram),
            );
            libc::memset(
                core::ptr::from_mut(&mut is).cast::<core::ffi::c_void>(),
                0,
                core::mem::size_of_val(&is),
            );
        }
        //   dd_variant = NULL;
        unsafe {
            if (load_roms(
                options.ddipl_path,
                options.ddrom_path,
                options.pifrom_path,
                options.cart_path,
                &raw mut ddipl,
                (&raw mut dd_variant).cast(),
                &raw mut ddrom,
                &raw mut pifrom,
                &raw mut cart,
            ) != 0)
            {
                cen64_alloc_cleanup();
                return EXIT_FAILURE;
            }
        }

        unsafe {
            cart_info = cart_db_get_entry(cart.ptr.cast());
        }

        if (unsafe { cart.size >= 0x40 } && unsafe { cart_info.is_null() } == false) {
            unsafe {
                libc::printf(
                    c"Detected cart: %s[%s] - %s\n".as_ptr(),
                    (*cart_info).rom_id,
                    (*cart_info).regions,
                    (*cart_info).description,
                );
            }
        }

        // enum cart_db_save_type save_type = cart_info.save_type;
        if (unsafe { libc::strcmp((*cart_info).rom_id, c"NK4".as_ptr()) == 0 }) {
            // Special case for Japanese Kirby 64, which has different save types for different revisions
            let rom = cart.ptr.cast::<u8>();
            if (rom.add(0x3e).read() as char == 'J' && rom.add(0x3f).read() < 2) {
                save_type = CART_DB_SAVE_TYPE_SRAM_256KBIT
            };
        }
        // load the save file
        unsafe {
            match save_type {
                // --- 4K EEPROM ---
                CART_DB_SAVE_TYPE_EEPROM_4KBIT if options.eeprom_path.is_null() => {
                    println!(
                        "Warning: cart saves to 4kbit EEPROM, but none specified (see -eep4k)"
                    );
                    open_save_file(
                        core::ptr::null(),
                        0x200,
                        core::ptr::from_mut(&mut eeprom),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_EEPROM_4KBIT if options.eeprom_size != 0x200 => {
                    println!("Warning: cart saves to 4kbit EEPROM, but different size specified (see -eep4k)");
                    open_save_file(
                        options.eeprom_path,
                        0x200,
                        core::ptr::from_mut(&mut eeprom),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_EEPROM_4KBIT => {
                    open_save_file(
                        options.eeprom_path,
                        0x200,
                        core::ptr::from_mut(&mut eeprom),
                        core::ptr::null_mut(),
                    );
                }

                // --- 16K EEPROM ---
                cart_db_save_type::CART_DB_SAVE_TYPE_EEPROM_16KBIT
                    if options.eeprom_path.is_null() =>
                {
                    println!(
                        "Warning: cart saves to 16kbit EEPROM, but none specified (see -eep16k)"
                    );
                    open_save_file(
                        core::ptr::null(),
                        0x800,
                        core::ptr::from_mut(&mut eeprom),
                        core::ptr::null_mut(),
                    );
                }
                cart_db_save_type::CART_DB_SAVE_TYPE_EEPROM_16KBIT
                    if options.eeprom_size != 0x800 =>
                {
                    println!("Warning: cart saves to 16kbit EEPROM, but different size specified (see -eep16k)");
                    open_save_file(
                        options.eeprom_path,
                        0x800,
                        core::ptr::from_mut(&mut eeprom),
                        core::ptr::null_mut(),
                    );
                }
                cart_db_save_type::CART_DB_SAVE_TYPE_EEPROM_16KBIT => {
                    open_save_file(
                        options.eeprom_path,
                        0x800,
                        core::ptr::from_mut(&mut eeprom),
                        core::ptr::null_mut(),
                    );
                }

                // --- FLASH ---
                cart_db_save_type::CART_DB_SAVE_TYPE_FLASH_1MBIT
                    if options.flashram_path.is_null() =>
                {
                    let mut created: core::ffi::c_int = 0;
                    println!("Warning: cart saves to Flash, but none specified (see -flash)");
                    open_save_file(
                        core::ptr::null(),
                        FLASHRAM_SIZE as usize,
                        core::ptr::from_mut(&mut flashram),
                        &mut created,
                    );
                    if created != 0 {
                        libc::memset(flashram.ptr, 0xFF, FLASHRAM_SIZE as usize);
                    }
                }
                cart_db_save_type::CART_DB_SAVE_TYPE_FLASH_1MBIT => {
                    let mut created: core::ffi::c_int = 0;
                    open_save_file(
                        options.flashram_path,
                        FLASHRAM_SIZE as usize,
                        core::ptr::from_mut(&mut flashram),
                        &mut created,
                    );
                    if created != 0 {
                        libc::memset(flashram.ptr, 0xFF, FLASHRAM_SIZE as usize);
                    }
                }

                // --- SRAM 256K ---
                CART_DB_SAVE_TYPE_SRAM_256KBIT if options.sram_path.is_null() => {
                    println!(
                        "Warning: cart saves to 256kbit SRAM, but none specified (see -sram256k)"
                    );
                    open_save_file(
                        core::ptr::null(),
                        0x8000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_SRAM_256KBIT if options.sram_size != 0x8000 => {
                    println!("Warning: cart saves to 256kbit SRAM, but different size specified (see -sram256k)");
                    open_save_file(
                        options.sram_path,
                        0x8000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_SRAM_256KBIT => {
                    open_save_file(
                        options.sram_path,
                        0x8000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }

                // --- SRAM 768K ---
                CART_DB_SAVE_TYPE_SRAM_768KBIT if options.sram_path.is_null() => {
                    println!(
                        "Warning: cart saves to 768kbit SRAM, but none specified (see -sram768k)"
                    );
                    open_save_file(
                        core::ptr::null(),
                        0x18000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_SRAM_768KBIT if options.sram_size != 0x18000 => {
                    println!("Warning: cart saves to 768kbit SRAM, but different size specified (see -sram768k)");
                    open_save_file(
                        options.sram_path,
                        0x18000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_SRAM_768KBIT => {
                    open_save_file(
                        options.sram_path,
                        0x18000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }

                // --- SRAM 1M ---
                CART_DB_SAVE_TYPE_SRAM_1MBIT if options.sram_path.is_null() => {
                    println!("Warning: cart saves to 1mbit SRAM, but none specified (see -sram1m)");
                    open_save_file(
                        core::ptr::null(),
                        0x20000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_SRAM_1MBIT if options.sram_size != 0x20000 => {
                    println!("Warning: cart saves to 1mbit SRAM, but different size specified (see -sram1m)");
                    open_save_file(
                        options.sram_path,
                        0x20000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }
                CART_DB_SAVE_TYPE_SRAM_1MBIT => {
                    open_save_file(
                        options.sram_path,
                        0x20000,
                        core::ptr::from_mut(&mut sram),
                        core::ptr::null_mut(),
                    );
                }

                _ => {}
            }
        }

        if (load_paks((&raw mut controller).cast()) != 0) {
            cen64_alloc_cleanup();
            return EXIT_FAILURE;
        }

        if (options.eeprom_path.is_null() == false
            && open_save_file(
                options.eeprom_path,
                options.eeprom_size,
                &raw mut eeprom,
                std::ptr::null_mut(),
            ) != 0)
        {
            cen64_alloc_cleanup();
            return EXIT_FAILURE;
        }

        if (options.sram_path.is_null() == false
            && open_save_file(
                options.sram_path,
                options.sram_size,
                &raw mut sram,
                std::ptr::null_mut(),
            ) != 0)
        {
            cen64_alloc_cleanup();
            return EXIT_FAILURE;
        }

        if (options.flashram_path.is_null() == false) {
            let mut created: c_int = 0;
            if (open_save_file(
                options.flashram_path,
                FLASHRAM_SIZE as usize,
                &raw mut flashram,
                &raw mut created,
            ) == 0)
            {
                cen64_alloc_cleanup();
                return EXIT_FAILURE;
            }
            if (created != 0) {
                memset(flashram.ptr, 0xFF, FLASHRAM_SIZE as usize);
            }
        }

        if (is_viewer_init(&raw mut is, options.is_viewer_output) == 0) {
            cen64_alloc_cleanup();
            return EXIT_FAILURE;
        } else {
            is_in = &raw mut is;
        }

        let alloc_ptr = cen64_alloc(
            &raw mut cen64_device_mem,
            std::mem::size_of::<cen64_device>() + 4096,
            false,
        );
        // Allocate memory for and create the device.
        if (alloc_ptr.is_null()) {
            println!("Failed to allocate enough memory for a device.\n");
            status = EXIT_FAILURE;
        } else {
            // std::ptr::copy_nonoverlapping(cen64_device_mem.ptr, device, 1);
            device = cen64_device_mem.ptr;
            // let a = core::ptr::addr_of_mut!(device);
            // let b = core::ptr::addr_of_mut!(cen64_device_mem.ptr);
            // core::ptr::copy_nonoverlapping(b, a, 1);
            // device = cen64_device_mem.ptr;

            if (device_create(
                device,
                &ddipl,
                dd_variant.cast(),
                &ddrom,
                &pifrom,
                &cart,
                &eeprom,
                &sram,
                &flashram,
                is_in,
                controller.as_ptr(),
                options.no_audio,
                options.no_video,
                options.enable_profiling,
            )
            .is_null())
            {
                println!("Failed to create a device.\n");
                status = EXIT_FAILURE;
            } else {
                if (options.debugger_addr.is_null() == false) {
                    todo!("gdb init");
                    // debugger = gdb_alloc();
                    // gdb_init(debugger, device, options.debugger_addr);
                }

                (*device).multithread = options.multithread;
                status = run_device(device, options.no_video);
                device_destroy(device, options.cart_path);

                if (debugger.is_null() == false) {
                    todo!("GDB DESTROY");
                    // gdb_destroy(debugger);
                }
            }

            cen64_free((&raw mut cen64_device_mem).cast());
        }

        // Release resources.
        if (options.ddipl_path.is_null() == false) {
            close_rom_file((&raw mut ddipl).cast());
        }

        if (options.ddrom_path.is_null() == false) {
            close_rom_file((&raw mut ddrom).cast());
        }

        if (options.cart_path.is_null() == false) {
            close_rom_file((&raw mut cart).cast());
        }

        close_rom_file((&raw mut pifrom).cast());
        cen64_alloc_cleanup();
        return status;
    }
}

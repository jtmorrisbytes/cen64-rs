use core::ffi::*;
use clap::{Parser,ArgGroup,Subcommand};
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cen64_options {
    pub ddipl_path: *const c_char,
    pub ddrom_path: *const c_char,
    pub pifrom_path: *const c_char,
    pub cart_path: *const c_char,
    pub debugger_addr: *const c_char,

    pub eeprom_path: *const c_char,
    pub eeprom_size: usize,
    pub sram_path: *const c_char,
    pub sram_size: usize,
    pub flashram_path: *const c_char,
    pub is_viewer_output: c_int,

    // Pointer to the first element of the controller[4] array
    pub controller: *mut controller,

    // Use bool directly; Rust's bool is compatible with C99 _Bool
    pub enable_debugger: bool,
    pub enable_profiling: bool,
    pub multithread: bool,
    pub no_audio: bool,
    pub no_video: bool,
}

const  DEFAULT_CEN64_OPTIONS: cen64_options = cen64_options{
  ddipl_path: core::ptr::null(), // ddipl_path
  ddrom_path: core::ptr::null(), // ddrom_path
  pifrom_path:core::ptr::null(), // pifrom_path
  cart_path: core::ptr::null(), // cart_path
  debugger_addr:core::ptr::null(), // debugger_addr
  eeprom_path:core::ptr::null(), // eeprom_path
  eeprom_size:0,    // eeprom_size
  sram_path:core::ptr::null(), // sram_path
  sram_size:0,    // sram_size
  flashram_path:core::ptr::null(), // flashram_path
  is_viewer_output:0,    // is_viewer_output
  controller:core::ptr::null_mut(), // controller
  enable_debugger:false, // enable_debugger
  multithread:false, // enable_profiling
  no_audio:false, // multithread
  no_video:false, // no_audio
  enable_profiling:false, // no_video
};

use std::{net::IpAddr, path::PathBuf};

use crate::si::controller;



#[derive(Subcommand, Debug)]
pub enum SaveType {
    /// 256 kbit SRAM
    Sram { path: PathBuf },
    /// 256 kbit SRAM (explicit)
    Sram256k { path: PathBuf },
    /// 768 kbit SRAM
    Sram768k { path: PathBuf },
    /// 1 mbit SRAM
    Sram1m { path: PathBuf },
    /// 1 mbit FlashRAM
    Flash { path: PathBuf },
}


#[derive(Parser, Debug)]
#[command(name = "cen64-rs", version = "0.0.1", about = "Cycle-accurate N64 Emulator")]
// This ensures we can't accidentally pick two SRAM sizes at once
// #[command(group(
//     ArgGroup::new("sram_group")
//         .args(["sram", "sram256k", "sram768k", "sram1m"]),
// ))]
pub struct Cen64Args {
    /// Starts the debugger on interface:port (defafult localhost:64646)
    #[arg(long)]
    pub debug: bool,

    #[arg(long,default_value="127.0.0.1")]
    pub debugger_address:IpAddr,

    /// Profile the ROM (cpu-side)
    #[arg(long)]
    pub profile: bool,

    /// Run in a threaded (but quasi-accurate) mode
    #[arg(long)]
    pub multithread: bool,

    /// Path to the 64DD IPL ROM
    #[arg(long)]
    pub ddipl: Option<PathBuf>,

    /// Path to the 64DD disk ROM
    #[arg(long)]
    pub ddrom: Option<PathBuf>,

    /// Run emulator without user-interface components
    #[arg(long)]
    pub headless: bool,

    /// Run emulator without audio
    #[arg(long)]
    pub noaudio: bool,

    /// Run emulator without video
    #[arg(long)]
    pub novideo: bool,

    /// Path to 4 kbit EEPROM save
    #[arg(long)]
    pub eep4k: Option<PathBuf>,

    /// Path to 16 kbit EEPROM save
    #[arg(long)]
    pub eep16k: Option<PathBuf>,

    /// Path to 256 kbit SRAM save (standard)
    // #[arg(long)
    #[command(subcommand)]
    pub sram: Option<SaveType>,



    /// Show IS Viewer 64 output
    #[arg(long = "is-viewer")]
    pub is_viewer: bool,

    /// Controller configuration (e.g., num=1,pak=rumble)
    #[arg(long, action = clap::ArgAction::Append)]
    pub controller: Vec<String>,

    // --- POSITIONAL ARGUMENTS ---
    
    /// Path to the PIF IPL ROM
    pub pif_rom: PathBuf,

    /// Path to the Cartridge ROM
    pub cart_rom: Option<PathBuf>,
}


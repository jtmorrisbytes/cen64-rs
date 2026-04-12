//! build.rs — CEN64 build script
//!
//! Translated from: https://github.com/n64dev/cen64/blob/master/CMakeLists.txt
//! Original license: BSD-3-Clause (Tyler J. Stachecki)
//!
//! This is a BUILD-ONLY crate (Ship of Theseus phase 1):
//!   - No src/lib.rs, no Rust targets
//!   - `cargo build` compiles all C sources and links the final binary
//!   - The C codebase is completely untouched
//!
//! Build dependencies (Cargo.toml [build-dependencies]):
//!   cc    = "1"        # MIT — C compiler driver
//!
//! System requirements (must be installed by the user):
//!   Linux:   libopenal-dev, libgl-dev, libx11-dev, libiconv (in glibc)
//!   macOS:   brew install openal-soft sdl2
//!   Windows: vcpkg install openal-soft opengl iconv

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // -----------------------------------------------------------------------
    // Rerun triggers
    // -----------------------------------------------------------------------
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=common.h.in");
    for dir in &[
        "ai", "arch", "bus", "common", "dd", "device", "gdb", "os", "pi", "rdp", "ri", "rsp", "si",
        "vi", "vr4300",
    ] {
        println!("cargo:rerun-if-changed={}", dir);
    }
    println!("cargo:rerun-if-env-changed=CEN64_ARCH_SUPPORT");
    println!("cargo:rerun-if-env-changed=VR4300_BUSY_WAIT_DETECTION");
    println!("cargo:rerun-if-env-changed=DEBUG_MMIO_REGISTER_ACCESS");

    // -----------------------------------------------------------------------
    // Platform detection
    // -----------------------------------------------------------------------
    let is_windows = target.contains("windows");
    let is_macos = target.contains("apple");
    let is_linux = target.contains("linux");
    let is_msvc = target.contains("msvc");
    let is_x86_64 = target.contains("x86_64") || target.contains("i686");
    let is_arm = target.contains("arm");
    let is_debug = profile == "debug";
    let _target_arch = std::env::var("CARGO_CFG_TARGET_ARCH")
        .unwrap_or_else(|_| "env var CARGO_CFG_TARGET_ARCH is not defined".into());

    // if on windows and using msvc toolchain, use vcvars to place vars in the right place
    if is_windows && is_msvc {
        let mut vcvars = vcvars::Vcvars::new();

        // Get the specific variables needed for the toolchain
        if let Ok(path) = vcvars.get("PATH") {
            // Prepend the VS paths to the current PATH so ml64.exe is found
            let current_path = std::env::var_os("PATH").unwrap_or_default();
            let new_path = std::env::join_paths(
                std::env::split_paths(&path).chain(std::env::split_paths(&current_path)),
            )
            .unwrap();
            std::env::set_var("PATH", new_path);
        }

        // CC crate also needs these to link correctly
        if let Ok(include) = vcvars.get("INCLUDE") {
            std::env::set_var("INCLUDE", include);
        }
        if let Ok(lib) = vcvars.get("LIB") {
            std::env::set_var("LIB", lib);
        }
    }
    // we need to build iconv on windows first
    if is_windows {
        cc::Build::new()
            .file("win-iconv-0.0.10/win-iconv-0.0.10/win_iconv.c")
            .include("deps/compat") // Folder containing iconv.h
            .compile("iconv");
    }

    // -----------------------------------------------------------------------
    // SIMD tier selection
    //
    // CMakeLists: CEN64_ARCH_SUPPORT CMake cache variable.
    // Here: CEN64_ARCH_SUPPORT env var (set in .cargo/config.toml or shell).
    // Default: "Native" — same as the CMakeLists default.
    //
    // The MSVC path is structurally different: MSVC has no -msse* flags.
    // Instead it injects preprocessor defines and the compiler auto-selects
    // the right instruction encoding. We replicate this exactly.
    // -----------------------------------------------------------------------
    let arch_support = env::var("CEN64_ARCH_SUPPORT").unwrap_or_else(|_| "Native".to_string());

    // -----------------------------------------------------------------------
    // configure_file(common.h.in -> common.h)
    //
    // This is the trickiest CMake-ism to translate. CMake's configure_file()
    // reads common.h.in, replaces @VAR@ tokens and #cmakedefine lines, and
    // writes the result to the binary directory so C code can #include it.
    //
    // We replicate this entirely in Rust: read the template, substitute,
    // write to OUT_DIR/common.h. The cc::Build then gets OUT_DIR on its
    // include search path so the generated header is found at compile time.
    //
    // The two options driven by this file mirror CMakeLists:
    //   VR4300_BUSY_WAIT_DETECTION  (default ON  = "1")
    //   DEBUG_MMIO_REGISTER_ACCESS  (default OFF = "0")
    // -----------------------------------------------------------------------
    let busy_wait = env::var("VR4300_BUSY_WAIT_DETECTION").unwrap_or_else(|_| "1".to_string());
    let debug_mmio = env::var("DEBUG_MMIO_REGISTER_ACCESS").unwrap_or_else(|_| "0".to_string());

    let common_h_in = src.join("common.h.in");
    if common_h_in.exists() {
        let template = fs::read_to_string(&common_h_in).expect("could not read common.h.in");
        let generated = template
            // @VAR@ substitution
            .replace("@VR4300_BUSY_WAIT_DETECTION@", &busy_wait)
            .replace("@DEBUG_MMIO_REGISTER_ACCESS@", &debug_mmio)
            .replace(
                "#cmakedefine CEN64_ARCH_DIR @CEN64_ARCH_DIR@",
                "#undef CEN64_ARCH_DIR",
            )
            .replace(
                "#cmakedefine CEN64_ARCH_SUPPORT @CEN64_ARCH_SUPPORT@",
                "#undef CEN64_ARCH_SUPPORT",
            )
            // #cmakedefine substitution
            .replace(
                "#cmakedefine VR4300_BUSY_WAIT_DETECTION",
                if busy_wait == "1" {
                    "#define VR4300_BUSY_WAIT_DETECTION"
                } else {
                    "/* #undef VR4300_BUSY_WAIT_DETECTION */"
                },
            )
            .replace(
                "#cmakedefine DEBUG_MMIO_REGISTER_ACCESS",
                if debug_mmio == "1" {
                    "#define DEBUG_MMIO_REGISTER_ACCESS"
                } else {
                    "/* #undef DEBUG_MMIO_REGISTER_ACCESS */"
                },
            );
        fs::write(out_dir.join("common.h"), generated).expect("could not write OUT_DIR/common.h");
    }

    // -----------------------------------------------------------------------
    // Source file lists
    //
    // Mirrors the named set() blocks in CMakeLists verbatim.
    // Explicit lists rather than globs — same philosophy as the original.
    //
    // NOTE: rdp/n64video.c is intentionally absent from core_sources.
    // It needs -fno-strict-aliasing (see CMakeLists set_source_files_properties),
    // which cc::Build cannot apply per-file. We compile it in a separate
    // cc::Build instance below.
    // -----------------------------------------------------------------------

    let core_sources: &[&str] = &[
        // AI — Audio Interface
        "ai/context.c",
        "ai/controller.c",
        // BUS
        "bus/controller.c",
        "bus/memorymap.c",
        // COMMON
        "common/debug.c",
        "common/hash_table.c",
        "common/one_hot.c",
        "common/reciprocal.c",
        // DD — 64DD
        "dd/controller.c",
        // DEVICE
        "cen64.c",
        "device/cart_db.c",
        "device/device.c",
        "device/netapi.c",
        "device/options.c",
        "device/sha1.c",
        // GDB stub
        "gdb/gdb.c",
        "gdb/protocol.c",
        // OS common
        "os/common/gl_hints.c",
        "os/common/input.c",
        "os/common/local_time.c",
        // PI — Peripheral Interface
        "pi/controller.c",
        "pi/is_viewer.c",
        // RDP — rdp/n64video.c is compiled separately (see below)
        "rdp/cpu.c",
        "rdp/interface.c",
        // RI — RDRAM Interface
        "ri/controller.c",
        // RSP — Reality Signal Processor
        "rsp/cp0.c",
        "rsp/cp2.c",
        "rsp/cpu.c",
        "rsp/decoder.c",
        "rsp/functions.c",
        "rsp/interface.c",
        "rsp/opcodes.c",
        "rsp/pipeline.c",
        "rsp/vfunctions.c",
        // SI — Serial Interface
        "si/cic.c",
        "si/controller.c",
        "si/pak.c",
        "si/pak_transfer.c",
        "si/gb.c",
        "si/rtc.c",
        // VI — Video Interface
        "vi/controller.c",
        "vi/render.c",
        "vi/window.c",
        // VR4300 — main MIPS CPU
        "vr4300/cp0.c",
        "vr4300/cp1.c",
        "vr4300/cpu.c",
        "vr4300/dcache.c",
        "vr4300/decoder.c",
        "vr4300/debug.c",
        "vr4300/fault.c",
        "vr4300/functions.c",
        "vr4300/icache.c",
        "vr4300/interface.c",
        "vr4300/opcodes.c",
        "vr4300/pipeline.c",
        "vr4300/segment.c",
    ];

    // CMakeLists: ARCH_X86_64_SOURCES — compiled only on x86/x86_64
    let arch_x86_64_sources: &[&str] = &[
        "arch/x86_64/tlb/tlb.c",
        "arch/x86_64/rsp/vrcpsq.c",
        "arch/x86_64/rsp/vmov.c",
        "arch/x86_64/rsp/vdivh.c",
        "arch/x86_64/rsp/rsp.c",
        "arch/x86_64/rsp/vrsq.c",
        "arch/x86_64/rsp/transpose.c",
    ];

    // OS platform source sets — exactly one selected at runtime
    let os_posix_sources: &[&str] = &[
        "os/posix/alloc.c",
        "os/posix/cpuid.c",
        "os/posix/main.c", // contains main() on POSIX
        "os/posix/rom_file.c",
        "os/posix/save_file.c",
        "os/posix/timer.c",
    ];
    let os_x11_sources: &[&str] = &["os/x11/gl_config.c", "os/x11/gl_window.c"];
    let os_sdl_sources: &[&str] = &["os/sdl/gl_config.c", "os/sdl/gl_window.c"];
    let os_winapi_sources: &[&str] = &[
        "os/winapi/alloc.c",
        "os/winapi/console.c",
        "os/winapi/cpuid.c",
        "os/winapi/gl_config.c",
        "os/winapi/gl_window.c",
        "os/winapi/main.c", // contains main() on Windows
        "os/winapi/rom_file.c",
        "os/winapi/save_file.c",
        "os/winapi/timer.c",
    ];

    // -----------------------------------------------------------------------
    // Shared include-path / flag builder helper
    //
    // We call this closure to configure both the main build and the
    // n64video sub-build with identical includes and defines.
    // -----------------------------------------------------------------------
    let configure_build = |b: &mut cc::Build| {
        // C standard flag.
        //
        // GCC/Clang: -std=c99  (what the original CMakeLists passes)
        // MSVC:      /std:c11  (MSVC has no C99 mode; C11 is the earliest
        //                       explicit standard it accepts, VS 2019+)
        //            We do NOT call b.std() on MSVC because the cc crate
        //            translates "c99" → "-std:c99" which cl.exe rejects with
        //            D9002. We emit the MSVC flag manually instead.

        // if build windows, use vcvars to find the msvc paths and everything

        if is_msvc {
            b.flag("/std:c11");
        } else {
            b.std("c99");
        }
        b.include(&out_dir); // generated common.h
        b.include(&src); // project root
        b.include(src.join("os/common"));

        // Arch-specific include paths
        if is_x86_64 {
            b.include(src.join("arch/x86_64"));
            if is_windows {
                b.include(src.join("os/windows/x86_64"));
            } else {
                b.include(src.join("os/unix/x86_64"));
            }
        }
        if is_arm {
            b.include(src.join("arch/arm"));
            b.include(src.join("os/unix/arm"));
        }

        // Platform OS include paths
        if is_windows {
            b.include(src.join("os/winapi"));
            // if windows, attempt to include openal sdk from creative if available
            // I may also include softal here
            b.include("C:\\Program Files (x86)\\OpenAL 1.1 SDK\\include");
            println!(
                r"cargo:rustc-link-search=native=C:\Program Files (x86)\OpenAL 1.1 SDK\libs\Win64"
            );
            println!("cargo:rustc-link-lib=OpenAL32");
            println!("cargo:rustc-link-lib=gdi32");
            println!("cargo:rustc-link-lib=opengl32");
            println!("cargo:rustc-link-lib=user32");
            b.include(src.join("iconv.h"));
        } else if is_macos {
            b.include(src.join("os/posix"));
            b.include(src.join("os/sdl"));
            // SDL2 headers — try pkg-config, fall back to a common brew path
            if let Ok(flags) = pkg_config_cflags("sdl2") {
                for f in flags {
                    b.flag(&f);
                }
            }
        } else {
            b.include(src.join("os/posix"));
            b.include(src.join("os/x11"));
        }

        // POSIX feature macros (mirrors `if (DEFINED UNIX)` block)
        if !is_windows {
            b.define("_POSIX_C_SOURCE", "200112L");
            b.define("_BSD_SOURCE", None);
            b.define("_DEFAULT_SOURCE", None);
        }
        // macOS: signal.h quirk
        if is_macos {
            b.define("_DARWIN_C_SOURCE", None);
        }
        // msvc related fixes
        if is_windows && is_msvc {
            b.define("PATH_MAX", "4096");
            // build.rs
            b.define("WIN32_LEAN_AND_MEAN", None);
            // build.rs
            b.define("__asm__(x)", "");
            b.define("CEN64_COMPILER", "\"msvc\"");
            if is_x86_64 {
                b.define(
                    "CEN64_ARCH_DIR",
                    format!("\"{}\"", src.join("arch").to_str().unwrap()).as_str(),
                );
            }
            b.define("CEN64_ARCH_SUPPORT", "\"sse2\"");
            
            b.define("__builtin_trap", "abort");
            b.define("__builtin_bswap64", "_byteswap_uint64");
            b.define("__builtin_unreachable", "__assume(0)");


        }

        // Warnings — mirrors the GCC/Clang flags in CMakeLists
        if !is_msvc {
            b.flag("-Wall");
            b.flag("-Wextra");
            b.flag("-Wno-unused-parameter");
            if b.is_flag_supported("-Werror=implicit-function-declaration")
                .unwrap_or(false)
            {
                b.flag("-Werror=implicit-function-declaration");
            }
            if b.is_flag_supported("-Werror=discarded-qualifiers")
                .unwrap_or(false)
            {
                b.flag("-Werror=discarded-qualifiers");
            }
        }

        // SIMD / arch flags
        if is_x86_64 {
            if is_msvc {
                // MSVC: define injection instead of -m flags
                match arch_support.as_str() {
                    "SSE2" => {
                        b.define("__SSE2__", None);
                    }
                    "SSE3" => {
                        b.define("__SSE2__", None);
                        b.define("__SSE3__", None);
                    }
                    "SSSE3" => {
                        b.define("__SSE2__", None);
                        b.define("__SSE3__", None);
                        b.define("__SSSE3__", None);
                    }
                    "SSE4.1" => {
                        b.define("__SSE2__", None);
                        b.define("__SSE3__", None);
                        b.define("__SSSE3__", None);
                        b.define("__SSE4_1__", None);
                    }
                    "AVX" => {
                        b.define("__SSE2__", None);
                        b.define("__SSE3__", None);
                        b.define("__SSSE3__", None);
                        b.define("__SSE4_1__", None);
                        b.flag("/arch:AVX");
                    }
                    _ => {}
                }
            } else {
                // GCC / Clang: -m flags
                let flag = match arch_support.as_str() {
                    "SSE2" => Some("-msse2"),
                    "SSE3" => Some("-msse3"),
                    "SSSE3" => Some("-mssse3"),
                    "SSE4.1" => Some("-msse4"),
                    "AVX" => Some("-mavx"),
                    "Native" => Some("-march=native"),
                    _ => None,
                };
                if let Some(f) = flag {
                    b.flag(f);
                }

                // GCC-specific: accumulate-outgoing-args
                if b.is_flag_supported("-maccumulate-outgoing-args")
                    .unwrap_or(false)
                {
                    b.flag("-maccumulate-outgoing-args");
                }
            }
        }

        // ARM hard-float + NEON
        if is_arm && !is_windows {
            b.flag("-mfloat-abi=hard");
            b.flag("-mfpu=neon");
        }

        // Profile-specific optimisation flags
        // CMakeLists sets these per CMAKE_BUILD_TYPE; we key on PROFILE.
        //
        // Always use cc's .opt_level() abstraction — never raw -O flags.
        // The crate translates: opt_level(0) → /Od on MSVC, -O0 on GCC/Clang
        //                       opt_level(3) → /O2 on MSVC, -O3 on GCC/Clang
        if is_debug {
            b.opt_level(0);
            // UBSan: GCC >= 4.9 only, and only for native (non-cross) builds
            if host == target
                && !is_windows
                && b.is_flag_supported("-fsanitize=undefined").unwrap_or(false)
            {
                b.flag("-fsanitize=undefined");
                println!("cargo:rustc-link-arg=-fsanitize=undefined");
            }
        } else if !is_msvc {
            // Mirrors CMAKE_C_FLAGS_RELEASE = "-O3 -ffast-math -DNDEBUG -fmerge-all-constants"
            // All flags guarded with is_flag_supported so ICC/unusual compilers
            // degrade gracefully rather than failing the build.
            b.opt_level(3);
            b.define("NDEBUG", None);
            if b.is_flag_supported("-ffast-math").unwrap_or(false) {
                b.flag("-ffast-math");
            }
            if b.is_flag_supported("-fmerge-all-constants")
                .unwrap_or(false)
            {
                b.flag("-fmerge-all-constants");
            }
            // LTO (GCC >= 4.6)
            if b.is_flag_supported("-flto").unwrap_or(false) {
                b.flag("-flto");
                b.flag("-fdata-sections");
                b.flag("-ffunction-sections");
                println!("cargo:rustc-link-arg=-Wl,--gc-sections");
            }
            // Fat LTO objects (GCC >= 4.9)
            if b.is_flag_supported("-ffat-lto-objects").unwrap_or(false) {
                b.flag("-ffat-lto-objects");
            }
            // Unsafe loop optimisations (GCC >= 4.8)
            if b.is_flag_supported("-funsafe-loop-optimizations")
                .unwrap_or(false)
            {
                b.flag("-funsafe-loop-optimizations");
            }
        } else {
            // MSVC release: /O2 is the closest to -O3.
            // /GL enables whole-program optimisation (MSVC's LTO equivalent).
            b.opt_level(2);
            b.define("NDEBUG", None);
            b.flag("/GL");
        }
    };

    // -----------------------------------------------------------------------
    // Compile rdp/n64video.c separately with -fno-strict-aliasing
    //
    // CMakeLists: set_source_files_properties(rdp/n64video.c
    //               PROPERTIES COMPILE_FLAGS -fno-strict-aliasing)
    //
    // cc::Build has no per-file flag API. The idiomatic solution is a second
    // Build instance for that one file, producing a separate static archive
    // that the linker sees alongside the main one.
    // -----------------------------------------------------------------------
    let mut n64video = cc::Build::new();
    configure_build(&mut n64video);
    n64video.file(src.join("rdp/n64video.c"));
    if !is_msvc {
        n64video.flag("-fno-strict-aliasing");
    }
    n64video.compile("cen64_n64video"); // -> libcen64_n64video.a in OUT_DIR

    // -----------------------------------------------------------------------
    // Compile all remaining sources
    // -----------------------------------------------------------------------
    let mut build = cc::Build::new();

    configure_build(&mut build);

    for f in core_sources {
        build.file(src.join(f));
    }
    if is_x86_64 {
        for f in arch_x86_64_sources {
            build.file(src.join(f));
        }
    }

    // OS platform sources — mirrors the if/elseif(DEFINED WIN32/APPLE) block
    if is_windows {
        for f in os_winapi_sources {
            build.file(src.join(f));
        }
    } else if is_macos {
        for f in os_posix_sources {
            build.file(src.join(f));
        }
        for f in os_sdl_sources {
            build.file(src.join(f));
        }
    } else {
        for f in os_posix_sources {
            build.file(src.join(f));
        }
        for f in os_x11_sources {
            build.file(src.join(f));
        }
    }

    build.compile("cen64_core"); // -> libcen64_core.a in OUT_DIR

    // -----------------------------------------------------------------------
    // MSVC MASM assembly (os/windows/x86_64/fpu/*.asm)
    //
    // CMakeLists: enable_language(ASM_MASM)
    //             file(GLOB ASM_SOURCES os/windows/x86_64/fpu/*.asm)
    //
    // The cc crate has zero MASM support. We invoke ml64.exe + lib.exe
    // directly. This is the only mandatory shell-out in the whole script.
    // -----------------------------------------------------------------------
    if is_windows && is_msvc && is_x86_64 {
        assemble_masm(&src, &out_dir);
    }

    // -----------------------------------------------------------------------
    // System library linking
    //
    // Mirrors: target_link_libraries(cen64 ${OPENAL_LIBRARY} ${OPENGL_LIBRARY}
    //            ${ICONV_LIBRARIES} ${VIDEO_LIB} ${CMAKE_THREAD_LIBS_INIT}
    //            ${EXTRA_OS_LIBS})
    //
    // We emit cargo:rustc-link-lib directives, which Cargo passes to the
    // linker when building the final binary.  This is the exact equivalent
    // of what CMake's Find*.cmake modules resolve to on each platform.
    // -----------------------------------------------------------------------

    // Threads (CMAKE_THREAD_LIBS_INIT)
    if !is_windows {
        println!("cargo:rustc-link-lib=pthread");
    }

    // OpenAL
    if is_macos {
        println!("cargo:rustc-link-lib=framework=OpenAL");
    } else if is_windows {
        println!("cargo:rustc-link-lib=OpenAL32");
    } else {
        println!("cargo:rustc-link-lib=openal");
    }

    // OpenGL
    if is_macos {
        println!("cargo:rustc-link-lib=framework=OpenGL");
    } else if is_windows {
        println!("cargo:rustc-link-lib=opengl32");
    } else {
        // Linux: libGL (provided by Mesa or proprietary driver)
        println!("cargo:rustc-link-lib=GL");
    }

    // Iconv — the most platform-variable dependency.
    // On glibc Linux it lives inside libc itself (no explicit link needed).
    // On musl, macOS with Homebrew, and Windows it's a separate library.
    if is_linux && !target.contains("musl") {
        // glibc: already in libc, nothing to emit
    } else if is_macos {
        // Built-in on macOS system toolchain; Homebrew adds a separate one.
        // Try pkg-config first; fall back to -liconv.
        if pkg_config_link("libiconv").is_err() {
            println!("cargo:rustc-link-lib=iconv");
        }
    } else {
        println!("cargo:rustc-link-lib=iconv");
    }

    // Video backend (platform-specific windowing layer)
    if is_macos {
        // CMakeLists: VIDEO_LIB = SDL2_LIBRARIES on macOS
        if pkg_config_link("sdl2").is_err() {
            println!("cargo:rustc-link-lib=SDL2");
        }
    } else if is_linux {
        // CMakeLists: VIDEO_LIB = X11_X11_LIB; also needs Xxf86vm
        println!("cargo:rustc-link-lib=X11");
        println!("cargo:rustc-link-lib=Xxf86vm");
    } else if is_windows {
        // CMakeLists: EXTRA_OS_LIBS = [mingw32] opengl32 winmm ws2_32
        if !is_msvc {
            println!("cargo:rustc-link-lib=mingw32");
        }
        println!("cargo:rustc-link-lib=winmm");
        println!("cargo:rustc-link-lib=ws2_32");
    }
}

// ---------------------------------------------------------------------------
// MASM helper — Windows/MSVC x86_64 only
// ---------------------------------------------------------------------------
fn assemble_masm(src: &std::path::Path, out_dir: &std::path::Path) {
    let fpu_dir = src.join("os/windows/x86_64/fpu");
    if !fpu_dir.exists() {
        return;
    }

    let asm_files: Vec<_> = fs::read_dir(&fpu_dir)
        .expect("could not read fpu dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "asm").unwrap_or(false))
        .map(|e| e.path())
        .collect();

    let mut obj_files = Vec::new();
    for asm in &asm_files {
        let obj = out_dir.join(asm.file_stem().unwrap()).with_extension("obj");
        let status = Command::new("ml64.exe")
            .args(["/c", "/Fo"])
            .arg(&obj)
            .arg(asm)
            .status()
            .expect("ml64.exe not found — ensure MSVC build tools are installed");
        assert!(status.success(), "ml64.exe failed on {:?}", asm);
        obj_files.push(obj);
    }

    let lib_out = out_dir.join("cen64_asm.lib");
    let status = Command::new("lib.exe")
        .arg(format!("/OUT:{}", lib_out.display()))
        .args(&obj_files)
        .status()
        .expect("lib.exe not found");
    assert!(status.success(), "lib.exe archiver failed");

    println!("cargo:rustc-link-lib=static=cen64_asm");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
}

// ---------------------------------------------------------------------------
// Minimal pkg-config helpers (no extra crate required)
// ---------------------------------------------------------------------------

fn pkg_config_cflags(lib: &str) -> Result<Vec<String>, ()> {
    let out = Command::new("pkg-config")
        .args(["--cflags", lib])
        .output()
        .map_err(|_| ())?;
    if !out.status.success() {
        return Err(());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .map(str::to_owned)
        .collect())
}

fn pkg_config_link(lib: &str) -> Result<(), ()> {
    let out = Command::new("pkg-config")
        .args(["--libs", lib])
        .output()
        .map_err(|_| ())?;
    if !out.status.success() {
        return Err(());
    }
    for flag in String::from_utf8_lossy(&out.stdout).split_whitespace() {
        if let Some(p) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={}", p);
        } else if let Some(n) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={}", n);
        }
    }
    Ok(())
}

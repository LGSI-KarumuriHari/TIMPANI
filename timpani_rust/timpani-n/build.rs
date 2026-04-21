/*
 * SPDX-FileCopyrightText: Copyright 2026 LG Electronics Inc.
 * SPDX-License-Identifier: MIT
 */

//! Build script for timpani-n.
//!
//! Responsibilities:
//!   1. Compile `proto/node_service.proto` into tonic client stubs (always).
//!   2. Compile `src/bpf/sigwait.bpf.c` into a Rust skeleton (feature "bpf").
//!
//! Prerequisites
//! -------------
//! - `protoc` on PATH (or set the `PROTOC` env var).
//!   Ubuntu/Debian: sudo apt install -y protobuf-compiler
//!
//! - clang on PATH — required by libbpf-cargo to compile BPF programs.
//!   Ubuntu/Debian: sudo apt install -y clang
//!   (Only needed when building with the default "bpf" feature.)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_proto()?;

    #[cfg(feature = "bpf")]
    compile_bpf()?;

    Ok(())
}

/// Compile `proto/node_service.proto` → tonic gRPC client stubs.
///
/// timpani-n is a pure gRPC client — it calls Timpani-O's NodeService.
/// However, we enable server generation for testing purposes (mock servers).
fn compile_proto() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "./proto";
    let proto_file = format!("{proto_root}/node_service.proto");

    // Re-run this build script if the proto changes.
    println!("cargo:rerun-if-changed={proto_file}");

    tonic_build::configure()
        .build_server(true) // Enable for mock servers in tests
        .build_client(true)
        .compile_protos(&[proto_file.as_str()], &[proto_root])?;

    Ok(())
}

/// Compile `src/bpf/sigwait.bpf.c` → `sigwait.skel.rs` in OUT_DIR.
///
/// libbpf-cargo invokes clang to produce BPF bytecode, then generates a
/// Rust skeleton struct (`SigwaitSkel`) that embeds the bytecode and
/// exposes type-safe map and program accessors.
///
/// The generated file is pulled into the crate via:
///   `include!(concat!(env!("OUT_DIR"), "/sigwait.skel.rs"))`
/// in `src/bpf/mod.rs`.
#[cfg(feature = "bpf")]
fn compile_bpf() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;

    let bpf_src = "./src/bpf/sigwait.bpf.c";
    let bpf_hdr = "./src/bpf/trace_bpf.h";

    println!("cargo:rerun-if-changed={bpf_src}");
    println!("cargo:rerun-if-changed={bpf_hdr}");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    // On Debian/Ubuntu the arch-qualified include dir holds asm/types.h.
    // `/usr/include/asm` is not a symlink on these systems, so clang targeting
    // BPF cannot find it without an explicit -I flag.  This mirrors the
    // `-I/usr/include/${BPF_ARCH}-linux-gnu` line in timpani-n/bpf.cmake.
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let arch_include = format!("/usr/include/{arch}-linux-gnu");

    let mut binding = libbpf_cargo::SkeletonBuilder::new();
    let mut builder = binding.source(bpf_src);
    if std::path::Path::new(&arch_include).exists() {
        builder = builder.clang_args([format!("-I{arch_include}")]);
    }
    builder.build_and_generate(out_dir.join("sigwait.skel.rs"))?;

    Ok(())
}

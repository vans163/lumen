use crate::spec::{EncodingType, Endianness, LinkerFlavor, Target, TargetOptions, TargetResult};

pub fn target() -> TargetResult {
    let mut base = super::linux_musl_base::opts();
    base.cpu = "x86-64".to_string();
    base.max_atomic_width = Some(64);
    base.pre_link_args
        .get_mut(&LinkerFlavor::Gcc)
        .unwrap()
        .push("-m64".to_string());
    base.stack_probes = true;
    base.static_position_independent_executables = true;

    Ok(Target {
        llvm_target: "x86_64-unknown-linux-musl".to_string(),
        target_endian: Endianness::Little,
        target_pointer_width: 64,
        target_c_int_width: "32".to_string(),
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            .to_string(),
        arch: "x86_64".to_string(),
        target_os: "linux".to_string(),
        target_env: "musl".to_string(),
        target_vendor: "unknown".to_string(),
        linker_flavor: LinkerFlavor::Gcc,
        options: TargetOptions {
            encoding: EncodingType::Encoding64Nanboxed,
            ..base
        },
    })
}

use std::{path::Path, process::Command};

use yir_core::YirModule;

use crate::aot::CpuBuildTarget;

pub(crate) fn requires_window_bundle(yir: &YirModule) -> bool {
    yir.nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "window")
}

pub(crate) fn build_window_bundle(
    yir_path: &Path,
    output_dir: &Path,
    exe_path: &Path,
    cpu_target: &CpuBuildTarget,
) -> Result<(String, String), String> {
    if cpu_target.cross_compile {
        return Err(format!(
            "window AOT bundle packaging does not support cross-compiling yet; requested `{}` -> {}",
            cpu_target.abi, cpu_target.clang_target
        ));
    }
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("yir-pack-aot")
        .arg("--")
        .arg(yir_path)
        .arg(output_dir)
        .arg("4")
        .output()
        .map_err(|error| format!("failed to invoke cargo for yir-pack-aot: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "yir-pack-aot failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok((
        exe_path.display().to_string(),
        "window-aot-bundle".to_owned(),
    ))
}

pub(crate) fn compile_native_binary(
    ll_path: &Path,
    shim_path: &Path,
    llvm_object_path: &Path,
    runtime_object_path: &Path,
    exe_path: &Path,
    cpu_target: &CpuBuildTarget,
) -> Result<(), String> {
    run_clang(
        Command::new("clang")
            .arg("-target")
            .arg(&cpu_target.clang_target)
            .arg("-c")
            .arg(ll_path)
            .arg("-O2")
            .arg("-o")
            .arg(llvm_object_path),
        "LLVM program object",
    )?;
    run_clang(
        Command::new("clang")
            .arg("-target")
            .arg(&cpu_target.clang_target)
            .arg("-c")
            .arg(shim_path)
            .arg("-O2")
            .arg("-o")
            .arg(runtime_object_path),
        "runtime shim object",
    )?;
    run_clang(
        Command::new("clang")
            .arg("-target")
            .arg(&cpu_target.clang_target)
            .arg(llvm_object_path)
            .arg(runtime_object_path)
            .arg("-O2")
            .arg("-o")
            .arg(exe_path),
        "compatibility executable",
    )
}

fn run_clang(command: &mut Command, artifact_kind: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|error| format!("failed to invoke clang for {artifact_kind}: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed while producing {artifact_kind}:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

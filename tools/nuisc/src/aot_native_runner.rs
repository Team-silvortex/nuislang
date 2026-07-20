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
    exe_path: &Path,
    cpu_target: &CpuBuildTarget,
) -> Result<(), String> {
    let output = Command::new("clang")
        .arg("-target")
        .arg(&cpu_target.clang_target)
        .arg(ll_path)
        .arg(shim_path)
        .arg("-O2")
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|error| format!("failed to invoke clang: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

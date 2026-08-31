use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use nuis_artifact::{
    build_compiler_component_compile_dispatch_receipt,
    parse_compiler_component_compile_dispatch_receipt, read_compiler_component_build,
    render_compiler_component_compile_dispatch_receipt, resolve_compiler_component_dispatch,
    CompilerComponentCompileDispatchReceiptInput, CompilerComponentDispatchCandidate,
    COMPILER_COMPONENT_BUILD_FILE, COMPILER_COMPONENT_COMPILE_COMMAND,
};

use crate::bootstrap_component_image::{read_image, stage_verified_image, write_new};
use crate::bootstrap_component_replacement::{
    load_verified_component_transition, BootstrapComponentTransitionVerificationInput,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentCompileDispatchInput {
    pub(crate) transition_verification: BootstrapComponentTransitionVerificationInput,
    pub(crate) current_component: PathBuf,
    pub(crate) current_image: PathBuf,
    pub(crate) forward_component: PathBuf,
    pub(crate) forward_image: PathBuf,
    pub(crate) project_input: PathBuf,
    pub(crate) build_output: PathBuf,
    pub(crate) output: PathBuf,
}

pub(crate) fn handle_bootstrap_dispatch_compile(
    input: BootstrapComponentCompileDispatchInput,
) -> Result<(), String> {
    if input.output.exists() {
        return Err(format!(
            "compiler component compile dispatch receipt `{}` already exists",
            input.output.display()
        ));
    }
    if input.build_output.exists() {
        return Err(format!(
            "compiler component compile dispatch requires an absent build output `{}`",
            input.build_output.display()
        ));
    }
    if input.build_output == input.output {
        return Err(
            "compiler component compile dispatch build output and receipt must be distinct"
                .to_owned(),
        );
    }

    let verified = load_verified_component_transition(&input.transition_verification)?;
    let current = read_compiler_component_build(&input.current_component)
        .map_err(|error| format!("failed to verify current compiler component: {error}"))?;
    let forward = read_compiler_component_build(&input.forward_component)
        .map_err(|error| format!("failed to verify forward compiler component: {error}"))?;
    let current_image = read_image(&input.current_image, "current compiler image")?;
    let forward_image = read_image(&input.forward_image, "forward compiler image")?;
    let candidates = [
        CompilerComponentDispatchCandidate {
            component: &forward,
            compiler_image: &forward_image,
        },
        CompilerComponentDispatchCandidate {
            component: &current,
            compiler_image: &current_image,
        },
    ];
    let resolution = resolve_compiler_component_dispatch(
        verified.transition(),
        verified.verification_input(&input.transition_verification),
        &candidates,
    )
    .map_err(|error| format!("failed to resolve compiler component compile dispatch: {error}"))?;
    let request_component_path =
        if resolution.current().component.record_sha256 == current.record_sha256 {
            &input.current_component
        } else {
            &input.forward_component
        };
    let request_compiled_artifact =
        read_compiled_artifact(request_component_path, resolution.current().component)?;

    let staged = stage_verified_image(resolution.current().compiler_image, &input.output)?;
    let process = Command::new(staged.path())
        .arg(COMPILER_COMPONENT_COMPILE_COMMAND)
        .arg(&input.project_input)
        .arg(&input.build_output)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to execute verified current compiler image: {error}"))?;
    let exit_code = process.status.code().ok_or_else(|| {
        "verified current compiler image terminated without a process exit code".to_owned()
    })?;
    let exit_code = usize::try_from(exit_code).map_err(|_| {
        format!("verified current compiler image returned negative exit code {exit_code}")
    })?;
    if exit_code != 0 {
        return Err(format!(
            "verified current compiler image compile request failed with exit code {exit_code}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&process.stdout),
            String::from_utf8_lossy(&process.stderr),
        ));
    }

    let result_path = input.build_output.join(COMPILER_COMPONENT_BUILD_FILE);
    let result = read_compiler_component_build(&result_path)
        .map_err(|error| format!("failed to verify dispatched compiler result: {error}"))?;
    let result_compiled_artifact = read_compiled_artifact(&result_path, &result)?;
    let receipt = build_compiler_component_compile_dispatch_receipt(
        CompilerComponentCompileDispatchReceiptInput {
            transition: verified.transition(),
            resolution: &resolution,
            request: resolution.current().component,
            result: &result,
            request_compiled_artifact: &request_compiled_artifact,
            result_compiled_artifact: &result_compiled_artifact,
            exit_code,
            stdout: &process.stdout,
            stderr: &process.stderr,
        },
    )
    .map_err(|error| {
        format!(
            "verified current compiler image compile dispatch failed: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&process.stdout),
            String::from_utf8_lossy(&process.stderr),
        )
    })?;
    write_new(
        &input.output,
        render_compiler_component_compile_dispatch_receipt(&receipt).as_bytes(),
        "compiler component compile dispatch receipt",
    )?;
    let parsed =
        parse_compiler_component_compile_dispatch_receipt(&input.output).map_err(|error| {
            format!("failed to verify compiler component compile dispatch receipt: {error}")
        })?;
    if parsed != receipt {
        return Err(
            "compiler component compile dispatch receipt changed after persistence".to_owned(),
        );
    }

    println!("bootstrap compiler component: compiled through selected current image");
    println!("  component_id: {}", receipt.component_id);
    println!(
        "  request_reproducible_build_sha256: {}",
        receipt.request_reproducible_build_sha256
    );
    println!("  result_record_sha256: {}", receipt.result_record_sha256);
    println!(
        "  forward_reproducible_build_sha256: {}",
        receipt.forward_reproducible_build_sha256
    );
    println!("  dispatch_sha256: {}", receipt.dispatch_sha256);
    println!("  receipt: {}", input.output.display());
    Ok(())
}

fn read_compiled_artifact(
    component_path: &Path,
    component: &nuis_artifact::CompilerComponentBuild,
) -> Result<Vec<u8>, String> {
    let root = component_path.parent().unwrap_or_else(|| Path::new("."));
    fs::read(root.join(&component.compiled_artifact_file)).map_err(|error| {
        format!(
            "failed to read compiler component compiled artifact `{}`: {error}",
            component.compiled_artifact_file
        )
    })
}

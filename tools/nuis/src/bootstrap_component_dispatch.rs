use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use nuis_artifact::{
    build_compiler_component_dispatch_receipt, parse_compiler_component_build,
    parse_compiler_component_dispatch_receipt, render_compiler_component_dispatch_receipt,
    resolve_compiler_component_dispatch, CompilerComponentDispatchCandidate,
    CompilerComponentDispatchReceiptInput, COMPILER_COMPONENT_DISPATCH_REQUEST_ARGUMENT,
};

use crate::bootstrap_component_image::{read_image, stage_verified_image, write_new};
use crate::bootstrap_component_replacement::{
    load_verified_component_transition, BootstrapComponentTransitionVerificationInput,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentDispatchInput {
    pub(crate) transition_verification: BootstrapComponentTransitionVerificationInput,
    pub(crate) current_component: PathBuf,
    pub(crate) current_image: PathBuf,
    pub(crate) forward_component: PathBuf,
    pub(crate) forward_image: PathBuf,
    pub(crate) output: PathBuf,
}

pub(crate) fn handle_bootstrap_dispatch_component(
    input: BootstrapComponentDispatchInput,
) -> Result<(), String> {
    if input.output.exists() {
        return Err(format!(
            "compiler component dispatch receipt `{}` already exists",
            input.output.display()
        ));
    }
    let verified = load_verified_component_transition(&input.transition_verification)?;
    let current = parse_compiler_component_build(&input.current_component)
        .map_err(|error| format!("failed to parse current compiler component: {error}"))?;
    let forward = parse_compiler_component_build(&input.forward_component)
        .map_err(|error| format!("failed to parse forward compiler component: {error}"))?;
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
    .map_err(|error| format!("failed to resolve compiler component dispatch: {error}"))?;

    let staged = stage_verified_image(resolution.current().compiler_image, &input.output)?;
    let output = Command::new(staged.path())
        .arg(COMPILER_COMPONENT_DISPATCH_REQUEST_ARGUMENT)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to execute verified current compiler image: {error}"))?;
    let exit_code = output.status.code().ok_or_else(|| {
        "verified current compiler image terminated without a process exit code".to_owned()
    })?;
    let exit_code = usize::try_from(exit_code).map_err(|_| {
        format!("verified current compiler image returned negative exit code {exit_code}")
    })?;
    let receipt =
        build_compiler_component_dispatch_receipt(CompilerComponentDispatchReceiptInput {
            transition: verified.transition(),
            resolution: &resolution,
            exit_code,
            stdout: &output.stdout,
            stderr: &output.stderr,
        })
        .map_err(|error| {
            format!(
            "verified current compiler image dispatch failed: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
        })?;
    write_new(
        &input.output,
        render_compiler_component_dispatch_receipt(&receipt).as_bytes(),
        "compiler component dispatch receipt",
    )?;
    let parsed = parse_compiler_component_dispatch_receipt(&input.output).map_err(|error| {
        format!("failed to verify compiler component dispatch receipt: {error}")
    })?;
    if parsed != receipt {
        return Err("compiler component dispatch receipt changed after persistence".to_owned());
    }

    println!("bootstrap compiler component: dispatched");
    println!("  component_id: {}", receipt.component_id);
    println!("  selected_stage_role: {}", receipt.selected_stage_role);
    println!(
        "  selected_reproducible_build_sha256: {}",
        receipt.selected_reproducible_build_sha256
    );
    println!("  forward_stage_role: {}", receipt.forward_stage_role);
    println!(
        "  forward_reproducible_build_sha256: {}",
        receipt.forward_reproducible_build_sha256
    );
    println!("  dispatch_sha256: {}", receipt.dispatch_sha256);
    println!("  receipt: {}", input.output.display());
    Ok(())
}

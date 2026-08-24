use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use nuis_artifact::{
    build_compiler_candidate_execution, read_compiler_candidate_execution,
    read_compiler_component_build, render_compiler_candidate_execution,
    CompilerCandidateExecutionInput, COMPILER_CANDIDATE_EXECUTION_FILE,
    COMPILER_COMPONENT_BUILD_FILE,
};

pub(crate) fn handle_bootstrap_candidate_probe(
    input: PathBuf,
    output_dir: PathBuf,
) -> Result<(), String> {
    let execution_path = output_dir.join(COMPILER_CANDIDATE_EXECUTION_FILE);
    if execution_path.exists() {
        fs::remove_file(&execution_path).map_err(|error| {
            format!(
                "failed to remove stale candidate execution `{}`: {error}",
                execution_path.display()
            )
        })?;
    }
    nuisc::run(nuisc::CommandKind::BootstrapBuild {
        input,
        output_dir: output_dir.clone(),
    })?;

    let component_path = output_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let component = read_compiler_component_build(&component_path)
        .map_err(|error| format!("failed to verify candidate source component: {error}"))?;

    let candidate_path = output_dir.join(&component.native_binary_file);
    let output = Command::new(&candidate_path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!(
                "failed to execute compiler candidate `{}`: {error}",
                candidate_path.display()
            )
        })?;
    let exit_code = output.status.code().ok_or_else(|| {
        format!(
            "compiler candidate `{}` terminated without an exit code",
            candidate_path.display()
        )
    })?;
    let exit_code = usize::try_from(exit_code).map_err(|_| {
        format!(
            "compiler candidate `{}` returned negative exit code {exit_code}",
            candidate_path.display()
        )
    })?;
    let execution = build_compiler_candidate_execution(&CompilerCandidateExecutionInput {
        component: &component,
        exit_code,
        stdout: &output.stdout,
        stderr: &output.stderr,
    })
    .map_err(|error| {
        format!(
            "compiler candidate probe failed: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    })?;
    fs::write(
        &execution_path,
        render_compiler_candidate_execution(&execution),
    )
    .map_err(|error| {
        format!(
            "failed to write compiler candidate execution `{}`: {error}",
            execution_path.display()
        )
    })?;
    let verified = read_compiler_candidate_execution(&execution_path)
        .map_err(|error| format!("failed to verify compiler candidate execution: {error}"))?;

    println!("bootstrap candidate probe: verified");
    println!("  protocol: {}", verified.protocol);
    println!("  role: {}", verified.probe_role);
    println!("  authority: {}", verified.authority);
    println!("  component: {}", verified.component_id);
    println!(
        "  candidate_binary_sha256: {}",
        verified.candidate_binary_sha256
    );
    println!("  execution_sha256: {}", verified.execution_sha256);
    println!("  record: {}", execution_path.display());
    Ok(())
}

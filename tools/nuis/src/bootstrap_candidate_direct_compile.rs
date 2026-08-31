use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use nuis_artifact::{
    build_compiler_candidate_direct_compile_capability,
    parse_compiler_candidate_direct_compile_capability, parse_compiler_candidate_frontend_result,
    read_compiler_stage_handoff, read_compiler_stage_transformations,
    render_compiler_candidate_direct_compile_capability,
    verify_compiler_candidate_direct_compile_capability,
    CompilerCandidateDirectCompileCapabilityInput, COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL,
    COMPILER_STAGE_TRANSFORMATION_FILE,
};

use crate::{
    bootstrap_candidate_compile_capability::load_verified_candidate_compile_lineage,
    bootstrap_component_image::{stage_verified_image, write_new},
};

const CANDIDATE_DIR: &str = "stage1-candidate";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapCandidateDirectCompileInput {
    pub(crate) candidate_root: PathBuf,
    pub(crate) result_output: PathBuf,
    pub(crate) capability_output: PathBuf,
}

pub(crate) fn handle_bootstrap_candidate_direct_compile(
    input: BootstrapCandidateDirectCompileInput,
) -> Result<(), String> {
    validate_outputs(&input)?;
    let lineage = load_verified_candidate_compile_lineage(&input.candidate_root)?;
    let candidate_dir = input.candidate_root.join(CANDIDATE_DIR);
    let (handoff, payloads) =
        read_compiler_stage_handoff(&candidate_dir.join(&lineage.candidate.stage_handoff_file))
            .map_err(|error| format!("failed to verify direct compile handoff: {error}"))?;
    let transformations = read_compiler_stage_transformations(
        &candidate_dir.join(COMPILER_STAGE_TRANSFORMATION_FILE),
        &handoff,
        &payloads,
    )
    .map_err(|error| format!("failed to verify direct compile transformations: {error}"))?;
    let payload_paths = handoff
        .records
        .iter()
        .map(|record| candidate_dir.join(&record.payload_file))
        .collect::<Vec<_>>();
    let staged_adapter = stage_verified_image(&lineage.adapter, &input.capability_output)?;
    let process = Command::new(staged_adapter.path())
        .args(&payload_paths)
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to execute direct stage1 front-end compile: {error}"))?;
    let exit_code = process.status.code().ok_or_else(|| {
        "direct stage1 front-end compile terminated without a process exit code".to_owned()
    })?;
    let exit_code = usize::try_from(exit_code).map_err(|_| {
        format!("direct stage1 front-end compile returned negative exit code {exit_code}")
    })?;
    if exit_code != 0 {
        return Err(format!(
            "direct stage1 front-end compile failed with exit code {exit_code}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&process.stdout),
            String::from_utf8_lossy(&process.stderr),
        ));
    }
    let evidence = CompilerCandidateDirectCompileCapabilityInput {
        candidate: &lineage.candidate,
        production: &lineage.production,
        adapter: &lineage.adapter,
        handoff: &handoff,
        payloads: &payloads,
        transformations: &transformations,
        result: &process.stdout,
        exit_code,
        stderr: &process.stderr,
    };
    let capability = build_compiler_candidate_direct_compile_capability(&evidence)
        .map_err(|error| format!("direct stage1 compile verification failed: {error}"))?;
    write_new(
        &input.result_output,
        &process.stdout,
        "compiler candidate front-end result",
    )?;
    if let Err(error) = write_new(
        &input.capability_output,
        render_compiler_candidate_direct_compile_capability(&capability).as_bytes(),
        "compiler candidate direct compile capability",
    ) {
        let _ = fs::remove_file(&input.result_output);
        return Err(error);
    }
    let persisted_result = parse_compiler_candidate_frontend_result(&input.result_output)
        .map_err(|error| format!("failed to reread candidate front-end result: {error}"))?;
    let persisted = parse_compiler_candidate_direct_compile_capability(&input.capability_output)
        .map_err(|error| format!("failed to reread direct compile capability: {error}"))?;
    verify_compiler_candidate_direct_compile_capability(&persisted, &evidence)
        .map_err(|error| format!("failed to reverify direct compile capability: {error}"))?;
    if persisted != capability
        || persisted_result.protocol != COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL
    {
        return Err("direct stage1 compile evidence changed after persistence".to_owned());
    }

    println!("bootstrap candidate direct front-end compile: verified");
    println!("  component_id: {}", capability.component_id);
    println!(
        "  production_proof_sha256: {}",
        capability.production_proof_sha256
    );
    println!("  result_sha256: {}", capability.result_sha256);
    println!("  provider_dependency_required: false");
    println!("  direct_stage1_compile: true");
    println!("  native_materialization: false");
    println!("  replacement_authorized: false");
    println!("  selection_authorized: false");
    println!("  result: {}", input.result_output.display());
    println!("  capability: {}", input.capability_output.display());
    Ok(())
}

fn validate_outputs(input: &BootstrapCandidateDirectCompileInput) -> Result<(), String> {
    if input.result_output == input.capability_output {
        return Err("direct compile result and capability outputs must be distinct".to_owned());
    }
    for (label, path) in [
        ("direct compile result", &input.result_output),
        ("direct compile capability", &input.capability_output),
    ] {
        if path.exists() {
            return Err(format!("{label} `{}` already exists", path.display()));
        }
    }
    Ok(())
}

use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use nuis_artifact::{
    build_compiler_candidate_fresh_source_capability, build_compiler_candidate_fresh_source_result,
    build_compiler_candidate_nsld_input, build_compiler_candidate_nsld_materialization_capability,
    parse_compiler_candidate_fresh_source_capability, parse_compiler_candidate_fresh_source_result,
    parse_compiler_candidate_fresh_source_result_bytes, parse_compiler_candidate_nsld_input,
    parse_compiler_candidate_nsld_input_bytes,
    parse_compiler_candidate_nsld_materialization_capability, parse_compiler_candidate_successor,
    render_compiler_candidate_fresh_source_capability,
    render_compiler_candidate_fresh_source_result, render_compiler_candidate_nsld_input,
    render_compiler_candidate_nsld_materialization_capability,
    verify_compiler_candidate_fresh_source_capability,
    verify_compiler_candidate_nsld_materialization_capability,
    CompilerCandidateFreshSourceCapabilityInput,
    CompilerCandidateNsldMaterializationCapabilityInput,
};

use crate::{
    bootstrap_candidate_compile_capability::load_verified_candidate_compile_lineage,
    bootstrap_component_image::{stage_verified_image, write_new},
};

const FRESH_SOURCE_COMMAND: &str = "fresh-source-v1";
const NSLD_INPUT_COMMAND: &str = "nsld-input-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapCandidateFreshSourceInput {
    pub(crate) candidate_root: PathBuf,
    pub(crate) successor: PathBuf,
    pub(crate) source: PathBuf,
    pub(crate) result_output: PathBuf,
    pub(crate) capability_output: PathBuf,
    pub(crate) nsld_input_output: PathBuf,
    pub(crate) materialization_capability_output: PathBuf,
}

pub(crate) fn handle_bootstrap_candidate_fresh_source(
    input: BootstrapCandidateFreshSourceInput,
) -> Result<(), String> {
    validate_paths(&input)?;
    let lineage = load_verified_candidate_compile_lineage(&input.candidate_root)?;
    let successor_source = fs::read_to_string(&input.successor).map_err(|error| {
        format!(
            "failed to read candidate successor `{}`: {error}",
            input.successor.display()
        )
    })?;
    let successor = parse_compiler_candidate_successor(&input.successor)
        .map_err(|error| format!("failed to verify candidate successor: {error}"))?;
    let source = fs::read(&input.source).map_err(|error| {
        format!(
            "failed to read candidate fresh source `{}`: {error}",
            input.source.display()
        )
    })?;
    validate_source_encoding(&source)?;

    let staged_adapter = stage_verified_image(&lineage.adapter, &input.capability_output)?;
    let process = Command::new(staged_adapter.path())
        .arg(FRESH_SOURCE_COMMAND)
        .arg(&input.source)
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to execute candidate fresh-source adapter: {error}"))?;
    let exit_code = process.status.code().ok_or_else(|| {
        "candidate fresh-source adapter terminated without a process exit code".to_owned()
    })?;
    let exit_code = usize::try_from(exit_code).map_err(|_| {
        format!("candidate fresh-source adapter returned negative exit code {exit_code}")
    })?;
    if exit_code != 0 {
        return Err(format!(
            "candidate fresh-source request failed with exit code {exit_code}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&process.stdout),
            String::from_utf8_lossy(&process.stderr),
        ));
    }
    let actual_result =
        parse_compiler_candidate_fresh_source_result_bytes(&process.stdout, &input.result_output)
            .map_err(|error| format!("failed to verify candidate fresh-source result: {error}"))?;
    let expected_result = build_compiler_candidate_fresh_source_result(&source)
        .map_err(|error| format!("fresh-source reference verification failed: {error}"))?;
    if actual_result != expected_result
        || render_compiler_candidate_fresh_source_result(&expected_result).as_bytes()
            != process.stdout
    {
        return Err(
            "candidate fresh-source result disagrees with the independent reference compiler"
                .to_owned(),
        );
    }

    let verification = CompilerCandidateFreshSourceCapabilityInput {
        candidate: &lineage.candidate,
        production: &lineage.production,
        successor: &successor,
        successor_source: &successor_source,
        adapter: &lineage.adapter,
        source: &source,
        result: &process.stdout,
        exit_code,
        stderr: &process.stderr,
    };
    let capability = build_compiler_candidate_fresh_source_capability(&verification)
        .map_err(|error| format!("candidate fresh-source capability failed: {error}"))?;
    let fresh_capability_source = render_compiler_candidate_fresh_source_capability(&capability);

    let nsld_process = Command::new(staged_adapter.path())
        .arg(NSLD_INPUT_COMMAND)
        .arg(&input.source)
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to execute candidate Nsld-input adapter: {error}"))?;
    let nsld_exit_code = nsld_process.status.code().ok_or_else(|| {
        "candidate Nsld-input adapter terminated without a process exit code".to_owned()
    })?;
    let nsld_exit_code = usize::try_from(nsld_exit_code).map_err(|_| {
        format!("candidate Nsld-input adapter returned negative exit code {nsld_exit_code}")
    })?;
    if nsld_exit_code != 0 {
        return Err(format!(
            "candidate Nsld-input request failed with exit code {nsld_exit_code}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&nsld_process.stdout),
            String::from_utf8_lossy(&nsld_process.stderr),
        ));
    }
    let actual_nsld_input =
        parse_compiler_candidate_nsld_input_bytes(&nsld_process.stdout, &input.nsld_input_output)
            .map_err(|error| format!("failed to verify candidate Nsld input: {error}"))?;
    let expected_nsld_input = build_compiler_candidate_nsld_input(&source)
        .map_err(|error| format!("candidate Nsld-input reference verification failed: {error}"))?;
    if actual_nsld_input != expected_nsld_input
        || render_compiler_candidate_nsld_input(&expected_nsld_input).as_bytes()
            != nsld_process.stdout
    {
        return Err(
            "candidate Nsld input disagrees with the independent materialization model".to_owned(),
        );
    }
    let nsld_input_source = render_compiler_candidate_nsld_input(&actual_nsld_input);
    let materialization_verification = CompilerCandidateNsldMaterializationCapabilityInput {
        candidate: &lineage.candidate,
        production: &lineage.production,
        successor: &successor,
        fresh_capability: &capability,
        fresh_capability_source: &fresh_capability_source,
        adapter: &lineage.adapter,
        nsld_input: &actual_nsld_input,
        nsld_input_source: &nsld_input_source,
        exit_code: nsld_exit_code,
        stderr: &nsld_process.stderr,
    };
    let materialization_capability =
        build_compiler_candidate_nsld_materialization_capability(&materialization_verification)
            .map_err(|error| {
                format!("candidate Nsld materialization capability failed: {error}")
            })?;
    write_new(
        &input.result_output,
        &process.stdout,
        "compiler candidate fresh-source result",
    )?;
    write_new(
        &input.capability_output,
        fresh_capability_source.as_bytes(),
        "compiler candidate fresh-source capability",
    )?;
    write_new(
        &input.nsld_input_output,
        nsld_input_source.as_bytes(),
        "compiler candidate Nsld input",
    )?;
    write_new(
        &input.materialization_capability_output,
        render_compiler_candidate_nsld_materialization_capability(&materialization_capability)
            .as_bytes(),
        "compiler candidate Nsld materialization capability",
    )?;
    let persisted_result = parse_compiler_candidate_fresh_source_result(&input.result_output)
        .map_err(|error| format!("failed to reread candidate fresh-source result: {error}"))?;
    let persisted_capability = parse_compiler_candidate_fresh_source_capability(
        &input.capability_output,
    )
    .map_err(|error| format!("failed to reread candidate fresh-source capability: {error}"))?;
    verify_compiler_candidate_fresh_source_capability(&persisted_capability, &verification)
        .map_err(|error| format!("failed to replay candidate fresh-source capability: {error}"))?;
    let persisted_nsld_input = parse_compiler_candidate_nsld_input(&input.nsld_input_output)
        .map_err(|error| format!("failed to reread candidate Nsld input: {error}"))?;
    let persisted_materialization = parse_compiler_candidate_nsld_materialization_capability(
        &input.materialization_capability_output,
    )
    .map_err(|error| {
        format!("failed to reread candidate Nsld materialization capability: {error}")
    })?;
    verify_compiler_candidate_nsld_materialization_capability(
        &persisted_materialization,
        &materialization_verification,
    )
    .map_err(|error| format!("failed to replay candidate Nsld materialization: {error}"))?;
    if persisted_result != actual_result
        || persisted_capability != capability
        || persisted_nsld_input != actual_nsld_input
        || persisted_materialization != materialization_capability
    {
        return Err("candidate fresh-source evidence changed after persistence".to_owned());
    }

    println!("bootstrap candidate fresh-source capability: verified");
    println!("  component_id: {}", capability.component_id);
    println!("  source_sha256: {}", capability.source_sha256);
    println!("  token_identity: {}", capability.token_identity);
    println!("  ast_identity: {}", capability.ast_identity);
    println!("  nir_identity: {}", capability.nir_identity);
    println!("  yir_identity: {}", capability.yir_identity);
    println!("  stage0_handoff_required: false");
    println!("  equivalent_nsld_input: true");
    println!("  native_object: false");
    println!("  capability: {}", input.capability_output.display());
    println!("  nsld_input: {}", input.nsld_input_output.display());
    println!(
        "  materialization_capability: {}",
        input.materialization_capability_output.display()
    );
    Ok(())
}

fn validate_paths(input: &BootstrapCandidateFreshSourceInput) -> Result<(), String> {
    for (label, path) in [
        ("fresh-source result", &input.result_output),
        ("fresh-source capability", &input.capability_output),
        ("Nsld input", &input.nsld_input_output),
        (
            "Nsld materialization capability",
            &input.materialization_capability_output,
        ),
    ] {
        if path.exists() {
            return Err(format!(
                "candidate {label} `{}` already exists",
                path.display()
            ));
        }
    }
    let outputs = [
        &input.result_output,
        &input.capability_output,
        &input.nsld_input_output,
        &input.materialization_capability_output,
    ];
    if outputs.iter().any(|path| **path == input.source)
        || outputs
            .iter()
            .enumerate()
            .any(|(index, path)| outputs[index + 1..].contains(path))
    {
        return Err("candidate fresh-source input and outputs must be distinct".to_owned());
    }
    Ok(())
}

fn validate_source_encoding(source: &[u8]) -> Result<(), String> {
    let text = std::str::from_utf8(source)
        .map_err(|error| format!("candidate fresh source is not UTF-8: {error}"))?;
    if source.is_empty()
        || source.len() > 128
        || source.contains(&0)
        || text.contains('\r')
        || !text.ends_with('\n')
    {
        return Err(
            "candidate fresh source must be 1..=128 bytes of canonical UTF-8/LF text".to_owned(),
        );
    }
    Ok(())
}

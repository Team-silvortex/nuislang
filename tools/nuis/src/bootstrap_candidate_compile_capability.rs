use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use nuis_artifact::{
    build_compiler_candidate_compile_capability, parse_compiler_candidate_compile_capability,
    read_compiler_candidate_execution, read_compiler_candidate_production,
    read_compiler_component_build, read_compiler_stage_handoff,
    render_compiler_candidate_compile_capability, verify_compiler_component_build_image,
    CompilerCandidateCompileCapabilityInput, CompilerCandidateProduction, CompilerComponentBuild,
    COMPILER_CANDIDATE_COMPILE_COMMAND, COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT,
    COMPILER_CANDIDATE_EXECUTION_FILE, COMPILER_CANDIDATE_PRODUCTION_FILE,
    COMPILER_COMPONENT_BUILD_FILE,
};

use crate::bootstrap_component_image::{read_image, stage_verified_image, write_new};
use crate::digest_sha256::sha256_hex;

const STAGE0_DIR: &str = "stage0";
const CANDIDATE_DIR: &str = "stage1-candidate";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapCandidateCompileCapabilityInput {
    pub(crate) candidate_root: PathBuf,
    pub(crate) provider_image: PathBuf,
    pub(crate) project_input: PathBuf,
    pub(crate) build_output: PathBuf,
    pub(crate) output: PathBuf,
}

pub(crate) struct VerifiedCandidateCompileLineage {
    pub(crate) stage0_dir: PathBuf,
    pub(crate) stage0: CompilerComponentBuild,
    pub(crate) candidate: CompilerComponentBuild,
    pub(crate) production: CompilerCandidateProduction,
    pub(crate) production_source: String,
    pub(crate) adapter: Vec<u8>,
}

pub(crate) fn handle_bootstrap_candidate_compile_capability(
    input: BootstrapCandidateCompileCapabilityInput,
) -> Result<(), String> {
    validate_output_paths(&input)?;
    let lineage = load_verified_candidate_compile_lineage(&input.candidate_root)?;
    let stage0 = &lineage.stage0;
    let candidate = &lineage.candidate;
    let production = &lineage.production;
    let adapter = &lineage.adapter;
    let provider_image = read_image(&input.provider_image, "stage0 compiler provider image")?;
    verify_compiler_component_build_image(stage0, &provider_image)
        .map_err(|error| format!("failed to verify stage0 compiler provider image: {error}"))?;
    let request_compiled_artifact = read_component_artifact(
        &lineage.stage0_dir.join(COMPILER_COMPONENT_BUILD_FILE),
        &stage0.compiled_artifact_file,
    )?;

    let staged_provider = stage_verified_image(&provider_image, &input.output)?;
    let staged_adapter = stage_verified_image(adapter, &input.output)?;
    let provider_path = fs::canonicalize(staged_provider.path()).map_err(|error| {
        format!("failed to resolve private stage0 compiler provider path: {error}")
    })?;
    let process = Command::new(staged_adapter.path())
        .arg(COMPILER_CANDIDATE_COMPILE_COMMAND)
        .arg(&input.project_input)
        .arg(&input.build_output)
        .env(
            COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT,
            &provider_path,
        )
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!("failed to execute production-bound candidate adapter: {error}")
        })?;
    let exit_code = process.status.code().ok_or_else(|| {
        "production-bound candidate adapter terminated without a process exit code".to_owned()
    })?;
    let exit_code = usize::try_from(exit_code).map_err(|_| {
        format!("production-bound candidate adapter returned negative exit code {exit_code}")
    })?;
    if exit_code != 0 {
        return Err(format!(
            "candidate compile capability request failed with exit code {exit_code}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&process.stdout),
            String::from_utf8_lossy(&process.stderr),
        ));
    }

    let result_path = input.build_output.join(COMPILER_COMPONENT_BUILD_FILE);
    let result = read_compiler_component_build(&result_path)
        .map_err(|error| format!("failed to verify candidate-driven compile result: {error}"))?;
    let result_compiled_artifact =
        read_component_artifact(&result_path, &result.compiled_artifact_file)?;
    let capability =
        build_compiler_candidate_compile_capability(&CompilerCandidateCompileCapabilityInput {
            stage0,
            candidate,
            production,
            adapter,
            provider_image: &provider_image,
            request_compiled_artifact: &request_compiled_artifact,
            result: &result,
            result_compiled_artifact: &result_compiled_artifact,
            exit_code,
            stdout: &process.stdout,
            stderr: &process.stderr,
        })
        .map_err(|error| format!("candidate compile capability verification failed: {error}"))?;
    write_new(
        &input.output,
        render_compiler_candidate_compile_capability(&capability).as_bytes(),
        "compiler candidate compile capability",
    )?;
    let parsed = parse_compiler_candidate_compile_capability(&input.output)
        .map_err(|error| format!("failed to reread candidate compile capability: {error}"))?;
    if parsed != capability {
        return Err("candidate compile capability changed after persistence".to_owned());
    }

    println!("bootstrap candidate compile capability: verified");
    println!("  component_id: {}", capability.component_id);
    println!("  candidate_producer: {}", capability.candidate_producer_id);
    println!(
        "  production_proof_sha256: {}",
        capability.production_proof_sha256
    );
    println!(
        "  result_record_sha256: {}",
        capability.result_record_sha256
    );
    println!("  replacement_authorized: false");
    println!("  selection_authorized: false");
    println!("  capability: {}", input.output.display());
    Ok(())
}

pub(crate) fn load_verified_candidate_compile_lineage(
    candidate_root: &Path,
) -> Result<VerifiedCandidateCompileLineage, String> {
    let stage0_dir = candidate_root.join(STAGE0_DIR);
    let candidate_dir = candidate_root.join(CANDIDATE_DIR);
    let stage0 = read_compiler_component_build(&stage0_dir.join(COMPILER_COMPONENT_BUILD_FILE))
        .map_err(|error| format!("failed to verify stage0 capability provider: {error}"))?;
    let execution =
        read_compiler_candidate_execution(&stage0_dir.join(COMPILER_CANDIDATE_EXECUTION_FILE))
            .map_err(|error| format!("failed to verify candidate execution lineage: {error}"))?;
    let candidate =
        read_compiler_component_build(&candidate_dir.join(COMPILER_COMPONENT_BUILD_FILE))
            .map_err(|error| format!("failed to verify compile candidate component: {error}"))?;
    let (handoff, payloads) =
        read_compiler_stage_handoff(&candidate_dir.join(&candidate.stage_handoff_file))
            .map_err(|error| format!("failed to verify compile candidate handoff: {error}"))?;
    let production = read_compiler_candidate_production(
        &candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE),
        &stage0,
        &execution,
        &candidate,
        &handoff,
        &payloads,
    )
    .map_err(|error| format!("failed to verify compile candidate production: {error}"))?;
    let production_source =
        fs::read_to_string(candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE))
            .map_err(|error| format!("failed to read verified candidate production: {error}"))?;
    let adapter = fs::read(candidate_dir.join(&production.adapter_file)).map_err(|error| {
        format!(
            "failed to read production-bound candidate adapter `{}`: {error}",
            production.adapter_file
        )
    })?;
    if adapter.len() != production.adapter_bytes
        || sha256_hex(&adapter) != production.adapter_sha256
    {
        return Err("production-bound candidate adapter changed before private staging".to_owned());
    }
    Ok(VerifiedCandidateCompileLineage {
        stage0_dir,
        stage0,
        candidate,
        production,
        production_source,
        adapter,
    })
}

fn validate_output_paths(input: &BootstrapCandidateCompileCapabilityInput) -> Result<(), String> {
    if input.output.exists() {
        return Err(format!(
            "compiler candidate compile capability `{}` already exists",
            input.output.display()
        ));
    }
    if input.build_output.exists() {
        return Err(format!(
            "compiler candidate compile capability requires an absent build output `{}`",
            input.build_output.display()
        ));
    }
    if input.build_output == input.output {
        return Err(
            "compiler candidate compile capability build output and receipt must be distinct"
                .to_owned(),
        );
    }
    Ok(())
}

fn read_component_artifact(record_path: &Path, file: &str) -> Result<Vec<u8>, String> {
    let root = record_path.parent().unwrap_or_else(|| Path::new("."));
    fs::read(root.join(file))
        .map_err(|error| format!("failed to read compiler component artifact `{file}`: {error}"))
}

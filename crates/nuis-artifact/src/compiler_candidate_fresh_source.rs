use std::path::Path;

use sha2::{Digest, Sha256};

use crate::{
    build_compiler_candidate_fresh_source_result,
    parse_compiler_candidate_fresh_source_result_bytes,
    parse_compiler_candidate_successor_from_source, render_compiler_candidate_fresh_source_result,
    render_compiler_candidate_successor, toml::escape_toml_string, verify_compiler_component_build,
    ArtifactError, CompilerCandidateFreshSourceResult, CompilerCandidateProduction,
    CompilerCandidateSuccessor, CompilerComponentBuild, COMPILER_CANDIDATE_ADAPTER_FILE,
    COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE, COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL,
    COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT, COMPILER_CANDIDATE_PRODUCTION_PROTOCOL,
    COMPILER_CANDIDATE_SUCCESSOR_FILE, COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};

#[path = "compiler_candidate_fresh_source_parse.rs"]
mod parse;

pub use parse::{
    parse_compiler_candidate_fresh_source_capability,
    parse_compiler_candidate_fresh_source_capability_from_source,
};

pub const COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL: &str =
    "nuis-compiler-candidate-fresh-source-capability-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE: &str =
    "nuis.compiler-candidate-fresh-source-capability.toml";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_DRIVER_CONTRACT: &str =
    "nuis-stage1-candidate-canonical-fresh-source-driver-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_AUTHORITY: &str =
    "single-canonical-fresh-source-front-end-capability-only-no-replacement-or-selection";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_ABI_CONTRACT: &str =
    "v8-exact-exports-reserved-fresh-source-ordinals-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_INPUT_CONTRACT: &str =
    "single-canonical-utf8-lf-source-no-stage0-handoff-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_ARGUMENT_CONTRACT: &str =
    "fresh-source-v1-command-plus-one-source-path-no-shell-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_ENVIRONMENT_CONTRACT: &str =
    "cleared-process-environment-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_NATIVE_CONTRACT: &str =
    "front-end-stage-identities-only-no-native-materialization-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT: &str =
    "candidate-owned-canonical-fresh-source-front-end-verified-no-native-or-selection-authority";

const CLOSED_STDIN_CONTRACT: &str = "closed-stdin-v1";

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateFreshSourceCapabilityInput<'a> {
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub successor: &'a CompilerCandidateSuccessor,
    pub successor_source: &'a str,
    pub adapter: &'a [u8],
    pub source: &'a [u8],
    pub result: &'a [u8],
    pub exit_code: usize,
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateFreshSourceCapability {
    pub protocol: String,
    pub driver_contract: String,
    pub authority: String,
    pub snapshot_contract: String,
    pub abi_contract: String,
    pub input_contract: String,
    pub argument_contract: String,
    pub environment_contract: String,
    pub stdin_contract: String,
    pub native_contract: String,
    pub bootstrap_subset_protocol: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub candidate_record_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub production_protocol: String,
    pub production_proof_sha256: String,
    pub adapter_file: String,
    pub adapter_bytes: usize,
    pub adapter_sha256: String,
    pub predecessor_successor_protocol: String,
    pub predecessor_successor_file: String,
    pub predecessor_successor_file_bytes: usize,
    pub predecessor_successor_file_sha256: String,
    pub predecessor_successor_proof_sha256: String,
    pub source_bytes: usize,
    pub source_lines: usize,
    pub source_sha256: String,
    pub stage_count: usize,
    pub token_record_count: usize,
    pub ast_record_count: usize,
    pub nir_record_count: usize,
    pub yir_record_count: usize,
    pub token_identity: usize,
    pub ast_identity: usize,
    pub nir_identity: usize,
    pub yir_identity: usize,
    pub result_protocol: String,
    pub result_file: String,
    pub result_bytes: usize,
    pub result_sha256: String,
    pub result_bundle_fold: usize,
    pub exit_code: usize,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub stage0_handoff_required: bool,
    pub provider_dependency_required: bool,
    pub candidate_owned_source_processing: bool,
    pub direct_stage1_compile: bool,
    pub fresh_source_compile: bool,
    pub native_materialization: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
}

pub fn build_compiler_candidate_fresh_source_capability(
    input: &CompilerCandidateFreshSourceCapabilityInput<'_>,
) -> Result<CompilerCandidateFreshSourceCapability, ArtifactError> {
    let result = verify_inputs(input)?;
    let mut capability = CompilerCandidateFreshSourceCapability {
        protocol: COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_FRESH_SOURCE_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_FRESH_SOURCE_AUTHORITY.to_owned(),
        snapshot_contract: COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT.to_owned(),
        abi_contract: COMPILER_CANDIDATE_FRESH_SOURCE_ABI_CONTRACT.to_owned(),
        input_contract: COMPILER_CANDIDATE_FRESH_SOURCE_INPUT_CONTRACT.to_owned(),
        argument_contract: COMPILER_CANDIDATE_FRESH_SOURCE_ARGUMENT_CONTRACT.to_owned(),
        environment_contract: COMPILER_CANDIDATE_FRESH_SOURCE_ENVIRONMENT_CONTRACT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        native_contract: COMPILER_CANDIDATE_FRESH_SOURCE_NATIVE_CONTRACT.to_owned(),
        bootstrap_subset_protocol: input.candidate.bootstrap_subset_protocol.clone(),
        component_id: input.candidate.component_id.clone(),
        component_domain: input.candidate.component_domain.clone(),
        component_unit: input.candidate.component_unit.clone(),
        candidate_record_sha256: input.candidate.record_sha256.clone(),
        candidate_reproducible_build_sha256: input.candidate.reproducible_build_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        production_protocol: input.production.protocol.clone(),
        production_proof_sha256: input.production.proof_sha256.clone(),
        adapter_file: input.production.adapter_file.clone(),
        adapter_bytes: input.adapter.len(),
        adapter_sha256: sha256_hex(input.adapter),
        predecessor_successor_protocol: input.successor.protocol.clone(),
        predecessor_successor_file: COMPILER_CANDIDATE_SUCCESSOR_FILE.to_owned(),
        predecessor_successor_file_bytes: input.successor_source.len(),
        predecessor_successor_file_sha256: sha256_hex(input.successor_source.as_bytes()),
        predecessor_successor_proof_sha256: input.successor.proof_sha256.clone(),
        source_bytes: input.source.len(),
        source_lines: result.source_lines,
        source_sha256: sha256_hex(input.source),
        stage_count: result.stage_count,
        token_record_count: result.stages[1].record_count,
        ast_record_count: result.stages[2].record_count,
        nir_record_count: result.stages[3].record_count,
        yir_record_count: result.stages[4].record_count,
        token_identity: result.stages[1].identity,
        ast_identity: result.stages[2].identity,
        nir_identity: result.stages[3].identity,
        yir_identity: result.stages[4].identity,
        result_protocol: result.protocol,
        result_file: COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE.to_owned(),
        result_bytes: input.result.len(),
        result_sha256: sha256_hex(input.result),
        result_bundle_fold: result.bundle_fold,
        exit_code: input.exit_code,
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        stage0_handoff_required: false,
        provider_dependency_required: false,
        candidate_owned_source_processing: true,
        direct_stage1_compile: true,
        fresh_source_compile: true,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    validate_compiler_candidate_fresh_source_capability(&capability)?;
    Ok(capability)
}

pub fn verify_compiler_candidate_fresh_source_capability(
    capability: &CompilerCandidateFreshSourceCapability,
    input: &CompilerCandidateFreshSourceCapabilityInput<'_>,
) -> Result<(), ArtifactError> {
    let expected = build_compiler_candidate_fresh_source_capability(input)?;
    if *capability != expected {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source capability changed its bound evidence",
        ));
    }
    Ok(())
}

fn verify_inputs(
    input: &CompilerCandidateFreshSourceCapabilityInput<'_>,
) -> Result<CompilerCandidateFreshSourceResult, ArtifactError> {
    verify_compiler_component_build(input.candidate)?;
    let production = input.production;
    let successor = input.successor;
    let parsed_successor = parse_compiler_candidate_successor_from_source(
        input.successor_source,
        Path::new(COMPILER_CANDIDATE_SUCCESSOR_FILE),
    )?;
    if &parsed_successor != successor
        || render_compiler_candidate_successor(successor) != input.successor_source
        || input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || production.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || production.candidate_component_sha256 != input.candidate.record_sha256
        || production.candidate_producer_id != input.candidate.producer_id
        || production.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || production.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || production.adapter_bytes != input.adapter.len()
        || production.adapter_sha256 != sha256_hex(input.adapter)
        || production.replacement_authorized
        || successor.protocol != COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL
        || successor.component_id != input.candidate.component_id
        || successor.component_domain != input.candidate.component_domain
        || successor.component_unit != input.candidate.component_unit
        || successor.candidate_record_sha256 != input.candidate.record_sha256
        || successor.candidate_reproducible_build_sha256
            != input.candidate.reproducible_build_sha256
        || successor.candidate_producer_id != input.candidate.producer_id
        || successor.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || successor.production_proof_sha256 != production.proof_sha256
        || successor.provider_dependency_required
        || !successor.direct_stage1_compile
        || successor.fresh_source_compile
        || successor.native_materialization
        || successor.replacement_authorized
        || successor.selection_authorized
        || !successor.successor_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source lineage is inconsistent",
        ));
    }
    let result = parse_compiler_candidate_fresh_source_result_bytes(
        input.result,
        Path::new(COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE),
    )?;
    let expected = build_compiler_candidate_fresh_source_result(input.source)?;
    if result != expected
        || render_compiler_candidate_fresh_source_result(&expected).as_bytes() != input.result
        || input.exit_code != 0
        || !input.stderr.is_empty()
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source execution result is inconsistent",
        ));
    }
    Ok(result)
}

pub fn render_compiler_candidate_fresh_source_capability(
    capability: &CompilerCandidateFreshSourceCapability,
) -> String {
    format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nauthority = \"{}\"\nsnapshot_contract = \"{}\"\nabi_contract = \"{}\"\ninput_contract = \"{}\"\nargument_contract = \"{}\"\nenvironment_contract = \"{}\"\nstdin_contract = \"{}\"\nnative_contract = \"{}\"\nbootstrap_subset_protocol = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nproduction_protocol = \"{}\"\nproduction_proof_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\npredecessor_successor_protocol = \"{}\"\npredecessor_successor_file = \"{}\"\npredecessor_successor_file_bytes = {}\npredecessor_successor_file_sha256 = \"{}\"\npredecessor_successor_proof_sha256 = \"{}\"\nsource_bytes = {}\nsource_lines = {}\nsource_sha256 = \"{}\"\nstage_count = {}\ntoken_record_count = {}\nast_record_count = {}\nnir_record_count = {}\nyir_record_count = {}\ntoken_identity = {}\nast_identity = {}\nnir_identity = {}\nyir_identity = {}\nresult_protocol = \"{}\"\nresult_file = \"{}\"\nresult_bytes = {}\nresult_sha256 = \"{}\"\nresult_bundle_fold = {}\nexit_code = {}\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\nstage0_handoff_required = {}\nprovider_dependency_required = {}\ncandidate_owned_source_processing = {}\ndirect_stage1_compile = {}\nfresh_source_compile = {}\nnative_materialization = {}\nreplacement_authorized = {}\nselection_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        capability.protocol,
        capability.driver_contract,
        capability.authority,
        capability.snapshot_contract,
        capability.abi_contract,
        capability.input_contract,
        capability.argument_contract,
        capability.environment_contract,
        capability.stdin_contract,
        capability.native_contract,
        escape_toml_string(&capability.bootstrap_subset_protocol),
        escape_toml_string(&capability.component_id),
        escape_toml_string(&capability.component_domain),
        escape_toml_string(&capability.component_unit),
        capability.candidate_record_sha256,
        capability.candidate_reproducible_build_sha256,
        escape_toml_string(&capability.candidate_producer_id),
        capability.candidate_compiler_image_sha256,
        capability.production_protocol,
        capability.production_proof_sha256,
        escape_toml_string(&capability.adapter_file),
        capability.adapter_bytes,
        capability.adapter_sha256,
        capability.predecessor_successor_protocol,
        capability.predecessor_successor_file,
        capability.predecessor_successor_file_bytes,
        capability.predecessor_successor_file_sha256,
        capability.predecessor_successor_proof_sha256,
        capability.source_bytes,
        capability.source_lines,
        capability.source_sha256,
        capability.stage_count,
        capability.token_record_count,
        capability.ast_record_count,
        capability.nir_record_count,
        capability.yir_record_count,
        capability.token_identity,
        capability.ast_identity,
        capability.nir_identity,
        capability.yir_identity,
        capability.result_protocol,
        escape_toml_string(&capability.result_file),
        capability.result_bytes,
        capability.result_sha256,
        capability.result_bundle_fold,
        capability.exit_code,
        capability.stderr_bytes,
        capability.stderr_sha256,
        capability.stage0_handoff_required,
        capability.provider_dependency_required,
        capability.candidate_owned_source_processing,
        capability.direct_stage1_compile,
        capability.fresh_source_compile,
        capability.native_materialization,
        capability.replacement_authorized,
        capability.selection_authorized,
        capability.verdict,
        capability.proof_sha256,
    )
}

pub(super) fn validate_compiler_candidate_fresh_source_capability(
    capability: &CompilerCandidateFreshSourceCapability,
) -> Result<(), ArtifactError> {
    if capability.protocol != COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL
        || capability.driver_contract != COMPILER_CANDIDATE_FRESH_SOURCE_DRIVER_CONTRACT
        || capability.authority != COMPILER_CANDIDATE_FRESH_SOURCE_AUTHORITY
        || capability.snapshot_contract != COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT
        || capability.abi_contract != COMPILER_CANDIDATE_FRESH_SOURCE_ABI_CONTRACT
        || capability.input_contract != COMPILER_CANDIDATE_FRESH_SOURCE_INPUT_CONTRACT
        || capability.argument_contract != COMPILER_CANDIDATE_FRESH_SOURCE_ARGUMENT_CONTRACT
        || capability.environment_contract != COMPILER_CANDIDATE_FRESH_SOURCE_ENVIRONMENT_CONTRACT
        || capability.stdin_contract != CLOSED_STDIN_CONTRACT
        || capability.native_contract != COMPILER_CANDIDATE_FRESH_SOURCE_NATIVE_CONTRACT
        || capability.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || capability.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || capability.predecessor_successor_protocol != COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL
        || capability.predecessor_successor_file != COMPILER_CANDIDATE_SUCCESSOR_FILE
        || capability.result_protocol != COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL
        || capability.result_file != COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE
        || capability.stage0_handoff_required
        || capability.provider_dependency_required
        || !capability.candidate_owned_source_processing
        || !capability.direct_stage1_compile
        || !capability.fresh_source_compile
        || capability.native_materialization
        || capability.replacement_authorized
        || capability.selection_authorized
        || capability.verdict != COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source capability declares an unsupported contract",
        ));
    }
    for value in [
        &capability.bootstrap_subset_protocol,
        &capability.component_id,
        &capability.component_domain,
        &capability.component_unit,
        &capability.candidate_producer_id,
    ] {
        if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
            return Err(ArtifactError::new(
                "compiler candidate fresh-source text identity is invalid",
            ));
        }
    }
    if capability.adapter_bytes == 0
        || capability.predecessor_successor_file_bytes == 0
        || capability.source_bytes == 0
        || capability.source_lines != 5
        || capability.stage_count != 5
        || [
            capability.token_record_count,
            capability.ast_record_count,
            capability.nir_record_count,
            capability.yir_record_count,
            capability.token_identity,
            capability.ast_identity,
            capability.nir_identity,
            capability.yir_identity,
            capability.result_bytes,
            capability.result_bundle_fold,
        ]
        .contains(&0)
        || capability.exit_code != 0
        || capability.stderr_bytes != 0
        || capability.stderr_sha256 != sha256_hex(&[])
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source execution identity is invalid",
        ));
    }
    for value in capability_hashes(capability) {
        validate_sha256(value)?;
    }
    if capability.proof_sha256 != capability_identity(capability) {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source capability proof identity mismatch",
        ));
    }
    Ok(())
}

fn capability_hashes(capability: &CompilerCandidateFreshSourceCapability) -> [&str; 11] {
    [
        &capability.candidate_record_sha256,
        &capability.candidate_reproducible_build_sha256,
        &capability.candidate_compiler_image_sha256,
        &capability.production_proof_sha256,
        &capability.adapter_sha256,
        &capability.predecessor_successor_file_sha256,
        &capability.predecessor_successor_proof_sha256,
        &capability.source_sha256,
        &capability.result_sha256,
        &capability.stderr_sha256,
        &capability.proof_sha256,
    ]
}

fn capability_identity(capability: &CompilerCandidateFreshSourceCapability) -> String {
    let mut hash = Sha256::new();
    for value in [
        capability.protocol.as_bytes(),
        capability.driver_contract.as_bytes(),
        capability.authority.as_bytes(),
        capability.snapshot_contract.as_bytes(),
        capability.abi_contract.as_bytes(),
        capability.input_contract.as_bytes(),
        capability.argument_contract.as_bytes(),
        capability.environment_contract.as_bytes(),
        capability.stdin_contract.as_bytes(),
        capability.native_contract.as_bytes(),
        capability.bootstrap_subset_protocol.as_bytes(),
        capability.component_id.as_bytes(),
        capability.component_domain.as_bytes(),
        capability.component_unit.as_bytes(),
        capability.candidate_record_sha256.as_bytes(),
        capability.candidate_reproducible_build_sha256.as_bytes(),
        capability.candidate_producer_id.as_bytes(),
        capability.candidate_compiler_image_sha256.as_bytes(),
        capability.production_protocol.as_bytes(),
        capability.production_proof_sha256.as_bytes(),
        capability.adapter_file.as_bytes(),
        capability.adapter_sha256.as_bytes(),
        capability.predecessor_successor_protocol.as_bytes(),
        capability.predecessor_successor_file.as_bytes(),
        capability.predecessor_successor_file_sha256.as_bytes(),
        capability.predecessor_successor_proof_sha256.as_bytes(),
        capability.source_sha256.as_bytes(),
        capability.result_protocol.as_bytes(),
        capability.result_file.as_bytes(),
        capability.result_sha256.as_bytes(),
        capability.stderr_sha256.as_bytes(),
        capability.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        capability.adapter_bytes,
        capability.predecessor_successor_file_bytes,
        capability.source_bytes,
        capability.source_lines,
        capability.stage_count,
        capability.token_record_count,
        capability.ast_record_count,
        capability.nir_record_count,
        capability.yir_record_count,
        capability.token_identity,
        capability.ast_identity,
        capability.nir_identity,
        capability.yir_identity,
        capability.result_bytes,
        capability.result_bundle_fold,
        capability.exit_code,
        capability.stderr_bytes,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for value in [
        capability.stage0_handoff_required,
        capability.provider_dependency_required,
        capability.candidate_owned_source_processing,
        capability.direct_stage1_compile,
        capability.fresh_source_compile,
        capability.native_materialization,
        capability.replacement_authorized,
        capability.selection_authorized,
    ] {
        hash_field(&mut hash, &[u8::from(value)]);
    }
    encode_hex(&hash.finalize())
}

fn validate_sha256(value: &str) -> Result<(), ArtifactError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source SHA-256 identity is invalid",
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
#[path = "compiler_candidate_fresh_source_tests.rs"]
mod tests;

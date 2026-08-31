use std::path::Path;

use sha2::{Digest, Sha256};

use crate::{
    build_compiler_candidate_frontend_result, parse_compiler_candidate_frontend_result_bytes,
    render_compiler_candidate_frontend_result, toml::escape_toml_string,
    verify_compiler_component_build, verify_compiler_stage_transformations, ArtifactError,
    CompilerCandidateProduction, CompilerComponentBuild, CompilerStageHandoff,
    CompilerStageTransformations, VerifiedCompilerStagePayload, COMPILER_CANDIDATE_ADAPTER_FILE,
    COMPILER_CANDIDATE_FRONTEND_RESULT_FILE, COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL,
    COMPILER_CANDIDATE_PRODUCTION_PROTOCOL, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
    COMPILER_STAGE_HANDOFF_PROTOCOL,
};

#[path = "compiler_candidate_direct_compile_parse.rs"]
mod parse;

pub use parse::{
    parse_compiler_candidate_direct_compile_capability,
    parse_compiler_candidate_direct_compile_capability_from_source,
};

pub const COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL: &str =
    "nuis-compiler-candidate-compile-capability-v2";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE: &str =
    "nuis.compiler-candidate-direct-compile-capability.toml";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT: &str =
    "nuis-stage1-candidate-direct-front-end-driver-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_AUTHORITY: &str =
    "front-end-compile-capability-only-no-replacement-or-selection";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_REQUEST_CONTRACT: &str =
    "canonical-five-stage-handoff-through-production-bound-candidate-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT: &str =
    "no-runtime-compiler-provider-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_ENVIRONMENT_CONTRACT: &str =
    "cleared-process-environment-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_INPUT_CONTRACT: &str =
    "verified-candidate-handoff-five-stage-payloads-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_ARGUMENT_CONTRACT: &str =
    "exact-five-stage-payloads-no-shell-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_NATIVE_CONTRACT: &str =
    "front-end-result-only-no-native-materialization-v1";
pub const COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT: &str =
    "candidate-direct-front-end-compile-verified-no-selection-authority";
const CLOSED_STDIN_CONTRACT: &str = "closed-stdin-v1";
const EXPECTED_STAGE_COUNT: usize = 5;

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateDirectCompileCapabilityInput<'a> {
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub adapter: &'a [u8],
    pub handoff: &'a CompilerStageHandoff,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub transformations: &'a CompilerStageTransformations,
    pub result: &'a [u8],
    pub exit_code: usize,
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateDirectCompileCapability {
    pub protocol: String,
    pub driver_contract: String,
    pub authority: String,
    pub request_contract: String,
    pub provider_contract: String,
    pub environment_contract: String,
    pub input_contract: String,
    pub argument_contract: String,
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
    pub handoff_protocol: String,
    pub handoff_bundle_sha256: String,
    pub input_record_count: usize,
    pub input_identity_sha256: String,
    pub result_protocol: String,
    pub result_file: String,
    pub result_bytes: usize,
    pub result_sha256: String,
    pub result_bundle_fold: usize,
    pub exit_code: usize,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub provider_dependency_required: bool,
    pub direct_stage1_compile: bool,
    pub native_materialization: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
}

pub fn build_compiler_candidate_direct_compile_capability(
    input: &CompilerCandidateDirectCompileCapabilityInput<'_>,
) -> Result<CompilerCandidateDirectCompileCapability, ArtifactError> {
    verify_inputs(input)?;
    let result = parse_compiler_candidate_frontend_result_bytes(
        input.result,
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )?;
    let mut capability = CompilerCandidateDirectCompileCapability {
        protocol: COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_DIRECT_COMPILE_AUTHORITY.to_owned(),
        request_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_REQUEST_CONTRACT.to_owned(),
        provider_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT.to_owned(),
        environment_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_ENVIRONMENT_CONTRACT.to_owned(),
        input_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_INPUT_CONTRACT.to_owned(),
        argument_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_ARGUMENT_CONTRACT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        native_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_NATIVE_CONTRACT.to_owned(),
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
        handoff_protocol: input.handoff.protocol.clone(),
        handoff_bundle_sha256: input.handoff.bundle_sha256.clone(),
        input_record_count: input.handoff.records.len(),
        input_identity_sha256: input_identity(input),
        result_protocol: result.protocol,
        result_file: COMPILER_CANDIDATE_FRONTEND_RESULT_FILE.to_owned(),
        result_bytes: input.result.len(),
        result_sha256: sha256_hex(input.result),
        result_bundle_fold: result.bundle_fold,
        exit_code: input.exit_code,
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        provider_dependency_required: false,
        direct_stage1_compile: true,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    validate_compiler_candidate_direct_compile_capability(&capability)?;
    Ok(capability)
}

pub fn verify_compiler_candidate_direct_compile_capability(
    capability: &CompilerCandidateDirectCompileCapability,
    input: &CompilerCandidateDirectCompileCapabilityInput<'_>,
) -> Result<(), ArtifactError> {
    let expected = build_compiler_candidate_direct_compile_capability(input)?;
    if *capability != expected {
        return Err(ArtifactError::new(
            "compiler candidate direct compile capability changed its bound evidence",
        ));
    }
    Ok(())
}

fn verify_inputs(
    input: &CompilerCandidateDirectCompileCapabilityInput<'_>,
) -> Result<(), ArtifactError> {
    verify_compiler_component_build(input.candidate)?;
    if input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || input.production.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || input.production.candidate_component_sha256 != input.candidate.record_sha256
        || input.production.candidate_producer_id != input.candidate.producer_id
        || input.production.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || input.production.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || input.production.adapter_bytes != input.adapter.len()
        || input.production.adapter_sha256 != sha256_hex(input.adapter)
        || input.production.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate direct compile lineage is inconsistent",
        ));
    }
    if input.handoff.protocol != COMPILER_STAGE_HANDOFF_PROTOCOL
        || input.handoff.producer_id != input.candidate.producer_id
        || input.handoff.module_domain != input.candidate.component_domain
        || input.handoff.module_unit != input.candidate.component_unit
        || input.handoff.bundle_sha256 != input.candidate.stage_handoff_bundle_sha256
        || input.handoff.bundle_sha256 != input.production.stage_handoff_bundle_sha256
        || input.handoff.records.len() != EXPECTED_STAGE_COUNT
        || input.payloads.len() != EXPECTED_STAGE_COUNT
        || input.production.records.len() != EXPECTED_STAGE_COUNT
    {
        return Err(ArtifactError::new(
            "compiler candidate direct compile handoff is inconsistent",
        ));
    }
    for (ordinal, ((record, payload), production)) in input
        .handoff
        .records
        .iter()
        .zip(input.payloads)
        .zip(&input.production.records)
        .enumerate()
    {
        if record.ordinal != ordinal
            || production.ordinal != ordinal
            || record.stage != payload.stage
            || production.stage != record.stage.as_str()
            || record.payload_bytes != payload.bytes.len()
            || production.payload_bytes != payload.bytes.len()
            || record.payload_sha256 != sha256_hex(&payload.bytes)
            || production.payload_sha256 != record.payload_sha256
            || production.fold != crate::compiler_candidate_stage_fold(ordinal, &payload.bytes)
        {
            return Err(ArtifactError::new(format!(
                "compiler candidate direct compile stage {ordinal} identity mismatch"
            )));
        }
    }
    verify_compiler_stage_transformations(input.transformations, input.handoff, input.payloads)?;
    let expected_result =
        build_compiler_candidate_frontend_result(input.production, input.transformations)?;
    if render_compiler_candidate_frontend_result(&expected_result).as_bytes() != input.result
        || input.exit_code != 0
        || !input.stderr.is_empty()
    {
        return Err(ArtifactError::new(
            "compiler candidate direct compile execution result is inconsistent",
        ));
    }
    Ok(())
}

pub fn render_compiler_candidate_direct_compile_capability(
    capability: &CompilerCandidateDirectCompileCapability,
) -> String {
    format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nauthority = \"{}\"\nrequest_contract = \"{}\"\nprovider_contract = \"{}\"\nenvironment_contract = \"{}\"\ninput_contract = \"{}\"\nargument_contract = \"{}\"\nstdin_contract = \"{}\"\nnative_contract = \"{}\"\nbootstrap_subset_protocol = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nproduction_protocol = \"{}\"\nproduction_proof_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nhandoff_protocol = \"{}\"\nhandoff_bundle_sha256 = \"{}\"\ninput_record_count = {}\ninput_identity_sha256 = \"{}\"\nresult_protocol = \"{}\"\nresult_file = \"{}\"\nresult_bytes = {}\nresult_sha256 = \"{}\"\nresult_bundle_fold = {}\nexit_code = {}\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\nprovider_dependency_required = {}\ndirect_stage1_compile = {}\nnative_materialization = {}\nreplacement_authorized = {}\nselection_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        capability.protocol,
        capability.driver_contract,
        capability.authority,
        capability.request_contract,
        capability.provider_contract,
        capability.environment_contract,
        capability.input_contract,
        capability.argument_contract,
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
        capability.handoff_protocol,
        capability.handoff_bundle_sha256,
        capability.input_record_count,
        capability.input_identity_sha256,
        capability.result_protocol,
        escape_toml_string(&capability.result_file),
        capability.result_bytes,
        capability.result_sha256,
        capability.result_bundle_fold,
        capability.exit_code,
        capability.stderr_bytes,
        capability.stderr_sha256,
        capability.provider_dependency_required,
        capability.direct_stage1_compile,
        capability.native_materialization,
        capability.replacement_authorized,
        capability.selection_authorized,
        capability.verdict,
        capability.proof_sha256,
    )
}

pub(super) fn validate_compiler_candidate_direct_compile_capability(
    capability: &CompilerCandidateDirectCompileCapability,
) -> Result<(), ArtifactError> {
    if capability.protocol != COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL
        || capability.driver_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT
        || capability.authority != COMPILER_CANDIDATE_DIRECT_COMPILE_AUTHORITY
        || capability.request_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_REQUEST_CONTRACT
        || capability.provider_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT
        || capability.environment_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_ENVIRONMENT_CONTRACT
        || capability.input_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_INPUT_CONTRACT
        || capability.argument_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_ARGUMENT_CONTRACT
        || capability.stdin_contract != CLOSED_STDIN_CONTRACT
        || capability.native_contract != COMPILER_CANDIDATE_DIRECT_COMPILE_NATIVE_CONTRACT
        || capability.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || capability.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || capability.handoff_protocol != COMPILER_STAGE_HANDOFF_PROTOCOL
        || capability.result_protocol != COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL
        || capability.result_file != COMPILER_CANDIDATE_FRONTEND_RESULT_FILE
        || capability.provider_dependency_required
        || !capability.direct_stage1_compile
        || capability.native_materialization
        || capability.replacement_authorized
        || capability.selection_authorized
        || capability.verdict != COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler candidate direct compile capability declares an unsupported contract",
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
                "compiler candidate direct compile text identity is invalid",
            ));
        }
    }
    if capability.adapter_bytes == 0
        || capability.input_record_count != EXPECTED_STAGE_COUNT
        || capability.result_bytes == 0
        || capability.result_bundle_fold == 0
        || capability.exit_code != 0
        || capability.stderr_bytes != 0
        || capability.stderr_sha256 != sha256_hex(&[])
    {
        return Err(ArtifactError::new(
            "compiler candidate direct compile execution identity is invalid",
        ));
    }
    for value in capability_hashes(capability) {
        validate_sha256(value)?;
    }
    if capability_identity(capability) != capability.proof_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate direct compile capability proof identity mismatch",
        ));
    }
    Ok(())
}

fn capability_hashes(capability: &CompilerCandidateDirectCompileCapability) -> [&str; 10] {
    [
        &capability.candidate_record_sha256,
        &capability.candidate_reproducible_build_sha256,
        &capability.candidate_compiler_image_sha256,
        &capability.production_proof_sha256,
        &capability.adapter_sha256,
        &capability.handoff_bundle_sha256,
        &capability.input_identity_sha256,
        &capability.result_sha256,
        &capability.stderr_sha256,
        &capability.proof_sha256,
    ]
}

fn input_identity(input: &CompilerCandidateDirectCompileCapabilityInput<'_>) -> String {
    let mut hash = Sha256::new();
    for ((record, payload), production) in input
        .handoff
        .records
        .iter()
        .zip(input.payloads)
        .zip(&input.production.records)
    {
        for value in [
            record.stage.as_str().as_bytes(),
            record.encoding.as_bytes(),
            record.payload_sha256.as_bytes(),
            record.record_sha256.as_bytes(),
        ] {
            hash_field(&mut hash, value);
        }
        for value in [record.ordinal, payload.bytes.len(), production.fold] {
            hash_field(&mut hash, &(value as u64).to_le_bytes());
        }
    }
    encode_hex(&hash.finalize())
}

fn capability_identity(capability: &CompilerCandidateDirectCompileCapability) -> String {
    let mut hash = Sha256::new();
    for value in [
        capability.protocol.as_bytes(),
        capability.driver_contract.as_bytes(),
        capability.authority.as_bytes(),
        capability.request_contract.as_bytes(),
        capability.provider_contract.as_bytes(),
        capability.environment_contract.as_bytes(),
        capability.input_contract.as_bytes(),
        capability.argument_contract.as_bytes(),
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
        capability.handoff_protocol.as_bytes(),
        capability.handoff_bundle_sha256.as_bytes(),
        capability.input_identity_sha256.as_bytes(),
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
        capability.input_record_count,
        capability.result_bytes,
        capability.result_bundle_fold,
        capability.exit_code,
        capability.stderr_bytes,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for value in [
        capability.provider_dependency_required,
        capability.direct_stage1_compile,
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
            "compiler candidate direct compile SHA-256 identity is invalid",
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
#[path = "compiler_candidate_direct_compile_tests.rs"]
mod tests;

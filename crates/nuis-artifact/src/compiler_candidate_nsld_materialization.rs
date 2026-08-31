use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    parse_compiler_candidate_fresh_source_capability_from_source,
    parse_compiler_candidate_nsld_input_from_source,
    render_compiler_candidate_fresh_source_capability, render_compiler_candidate_nsld_input,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError, CompilerCandidateFreshSourceCapability, CompilerCandidateNsldInput,
    CompilerCandidateProduction, CompilerCandidateSuccessor, CompilerComponentBuild,
    COMPILER_CANDIDATE_ADAPTER_FILE, COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE,
    COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL, COMPILER_CANDIDATE_NSLD_INPUT_FILE,
    COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL, COMPILER_CANDIDATE_PRODUCTION_PROTOCOL,
    COMPILER_CANDIDATE_SUCCESSOR_FILE, COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL,
};

pub const COMPILER_CANDIDATE_NSLD_MATERIALIZATION_PROTOCOL: &str =
    "nuis-compiler-candidate-nsld-materialization-capability-v1";
pub const COMPILER_CANDIDATE_NSLD_MATERIALIZATION_FILE: &str =
    "nuis.compiler-candidate-nsld-materialization-capability.toml";
pub const COMPILER_CANDIDATE_NSLD_MATERIALIZATION_DRIVER: &str =
    "nuis-stage1-candidate-nsld-input-driver-v1";
pub const COMPILER_CANDIDATE_NSLD_MATERIALIZATION_AUTHORITY: &str =
    "candidate-owned-equivalent-nsld-input-only-no-replacement-or-selection";
pub const COMPILER_CANDIDATE_NSLD_MATERIALIZATION_ARGUMENT_CONTRACT: &str =
    "nsld-input-v1-command-plus-one-source-path-no-shell-v1";
pub const COMPILER_CANDIDATE_NSLD_MATERIALIZATION_VERDICT: &str =
    "candidate-owned-yir-to-nsld-input-verified-no-native-object-or-selection-authority";

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateNsldMaterializationCapabilityInput<'a> {
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub successor: &'a CompilerCandidateSuccessor,
    pub fresh_capability: &'a CompilerCandidateFreshSourceCapability,
    pub fresh_capability_source: &'a str,
    pub adapter: &'a [u8],
    pub nsld_input: &'a CompilerCandidateNsldInput,
    pub nsld_input_source: &'a str,
    pub exit_code: usize,
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateNsldMaterializationCapability {
    pub protocol: String,
    pub driver_contract: String,
    pub authority: String,
    pub argument_contract: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub candidate_record_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub production_protocol: String,
    pub production_proof_sha256: String,
    pub predecessor_successor_protocol: String,
    pub predecessor_successor_file: String,
    pub predecessor_successor_proof_sha256: String,
    pub fresh_source_capability_protocol: String,
    pub fresh_source_capability_file: String,
    pub fresh_source_capability_bytes: usize,
    pub fresh_source_capability_sha256: String,
    pub fresh_source_capability_proof_sha256: String,
    pub adapter_file: String,
    pub adapter_bytes: usize,
    pub adapter_sha256: String,
    pub nsld_input_protocol: String,
    pub nsld_input_file: String,
    pub nsld_input_bytes: usize,
    pub nsld_input_sha256: String,
    pub source_sha256: String,
    pub source_identity: usize,
    pub yir_identity: usize,
    pub materialization_fold: usize,
    pub exit_code: usize,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub candidate_owned_yir_materialization: bool,
    pub equivalent_nsld_input: bool,
    pub native_object: bool,
    pub stage0_handoff_required: bool,
    pub provider_dependency_required: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
}

pub fn build_compiler_candidate_nsld_materialization_capability(
    input: &CompilerCandidateNsldMaterializationCapabilityInput<'_>,
) -> Result<CompilerCandidateNsldMaterializationCapability, ArtifactError> {
    validate_evidence(input)?;
    let mut capability = CompilerCandidateNsldMaterializationCapability {
        protocol: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_DRIVER.to_owned(),
        authority: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_AUTHORITY.to_owned(),
        argument_contract: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_ARGUMENT_CONTRACT.to_owned(),
        component_id: input.candidate.component_id.clone(),
        component_domain: input.candidate.component_domain.clone(),
        component_unit: input.candidate.component_unit.clone(),
        candidate_record_sha256: input.candidate.record_sha256.clone(),
        candidate_reproducible_build_sha256: input.candidate.reproducible_build_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        production_protocol: input.production.protocol.clone(),
        production_proof_sha256: input.production.proof_sha256.clone(),
        predecessor_successor_protocol: input.successor.protocol.clone(),
        predecessor_successor_file: COMPILER_CANDIDATE_SUCCESSOR_FILE.to_owned(),
        predecessor_successor_proof_sha256: input.successor.proof_sha256.clone(),
        fresh_source_capability_protocol: input.fresh_capability.protocol.clone(),
        fresh_source_capability_file: COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE.to_owned(),
        fresh_source_capability_bytes: input.fresh_capability_source.len(),
        fresh_source_capability_sha256: sha256_hex(input.fresh_capability_source.as_bytes()),
        fresh_source_capability_proof_sha256: input.fresh_capability.proof_sha256.clone(),
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE.to_owned(),
        adapter_bytes: input.adapter.len(),
        adapter_sha256: sha256_hex(input.adapter),
        nsld_input_protocol: input.nsld_input.protocol.clone(),
        nsld_input_file: COMPILER_CANDIDATE_NSLD_INPUT_FILE.to_owned(),
        nsld_input_bytes: input.nsld_input_source.len(),
        nsld_input_sha256: sha256_hex(input.nsld_input_source.as_bytes()),
        source_sha256: input.fresh_capability.source_sha256.clone(),
        source_identity: input.nsld_input.source_identity,
        yir_identity: input.nsld_input.yir_identity,
        materialization_fold: input.nsld_input.materialization_fold,
        exit_code: input.exit_code,
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        candidate_owned_yir_materialization: true,
        equivalent_nsld_input: true,
        native_object: false,
        stage0_handoff_required: false,
        provider_dependency_required: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    validate_capability(&capability)?;
    Ok(capability)
}

pub fn verify_compiler_candidate_nsld_materialization_capability(
    capability: &CompilerCandidateNsldMaterializationCapability,
    input: &CompilerCandidateNsldMaterializationCapabilityInput<'_>,
) -> Result<(), ArtifactError> {
    let expected = build_compiler_candidate_nsld_materialization_capability(input)?;
    if capability != &expected {
        return Err(ArtifactError::new(
            "compiler candidate Nsld materialization capability changed its bound evidence",
        ));
    }
    Ok(())
}

fn validate_evidence(
    input: &CompilerCandidateNsldMaterializationCapabilityInput<'_>,
) -> Result<(), ArtifactError> {
    let parsed_fresh = parse_compiler_candidate_fresh_source_capability_from_source(
        input.fresh_capability_source,
        Path::new(COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE),
    )?;
    let parsed_nsld = parse_compiler_candidate_nsld_input_from_source(
        input.nsld_input_source,
        Path::new(COMPILER_CANDIDATE_NSLD_INPUT_FILE),
    )?;
    if &parsed_fresh != input.fresh_capability
        || &parsed_nsld != input.nsld_input
        || render_compiler_candidate_fresh_source_capability(input.fresh_capability)
            != input.fresh_capability_source
        || render_compiler_candidate_nsld_input(input.nsld_input) != input.nsld_input_source
    {
        return Err(ArtifactError::new(
            "compiler candidate Nsld materialization evidence is not canonical",
        ));
    }
    if input.production.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || input.successor.protocol != COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL
        || input.fresh_capability.protocol != COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL
        || input.nsld_input.protocol != COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL
        || input.candidate.record_sha256 != input.fresh_capability.candidate_record_sha256
        || input.candidate.compiler_image_sha256
            != input.fresh_capability.candidate_compiler_image_sha256
        || input.production.proof_sha256 != input.fresh_capability.production_proof_sha256
        || input.successor.proof_sha256 != input.fresh_capability.predecessor_successor_proof_sha256
        || sha256_hex(input.adapter) != input.fresh_capability.adapter_sha256
        || input.nsld_input.yir_identity != input.fresh_capability.yir_identity
        || input.exit_code != 0
        || !input.stderr.is_empty()
    {
        return Err(ArtifactError::new(
            "compiler candidate Nsld materialization lineage is inconsistent",
        ));
    }
    Ok(())
}

pub fn render_compiler_candidate_nsld_materialization_capability(
    capability: &CompilerCandidateNsldMaterializationCapability,
) -> String {
    format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nauthority = \"{}\"\nargument_contract = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nproduction_protocol = \"{}\"\nproduction_proof_sha256 = \"{}\"\npredecessor_successor_protocol = \"{}\"\npredecessor_successor_file = \"{}\"\npredecessor_successor_proof_sha256 = \"{}\"\nfresh_source_capability_protocol = \"{}\"\nfresh_source_capability_file = \"{}\"\nfresh_source_capability_bytes = {}\nfresh_source_capability_sha256 = \"{}\"\nfresh_source_capability_proof_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nnsld_input_protocol = \"{}\"\nnsld_input_file = \"{}\"\nnsld_input_bytes = {}\nnsld_input_sha256 = \"{}\"\nsource_sha256 = \"{}\"\nsource_identity = {}\nyir_identity = {}\nmaterialization_fold = {}\nexit_code = {}\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\ncandidate_owned_yir_materialization = {}\nequivalent_nsld_input = {}\nnative_object = {}\nstage0_handoff_required = {}\nprovider_dependency_required = {}\nreplacement_authorized = {}\nselection_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        escape_toml_string(&capability.protocol),
        escape_toml_string(&capability.driver_contract),
        escape_toml_string(&capability.authority),
        escape_toml_string(&capability.argument_contract),
        escape_toml_string(&capability.component_id),
        escape_toml_string(&capability.component_domain),
        escape_toml_string(&capability.component_unit),
        capability.candidate_record_sha256,
        capability.candidate_reproducible_build_sha256,
        escape_toml_string(&capability.candidate_producer_id),
        capability.candidate_compiler_image_sha256,
        escape_toml_string(&capability.production_protocol),
        capability.production_proof_sha256,
        escape_toml_string(&capability.predecessor_successor_protocol),
        escape_toml_string(&capability.predecessor_successor_file),
        capability.predecessor_successor_proof_sha256,
        escape_toml_string(&capability.fresh_source_capability_protocol),
        escape_toml_string(&capability.fresh_source_capability_file),
        capability.fresh_source_capability_bytes,
        capability.fresh_source_capability_sha256,
        capability.fresh_source_capability_proof_sha256,
        escape_toml_string(&capability.adapter_file),
        capability.adapter_bytes,
        capability.adapter_sha256,
        escape_toml_string(&capability.nsld_input_protocol),
        escape_toml_string(&capability.nsld_input_file),
        capability.nsld_input_bytes,
        capability.nsld_input_sha256,
        capability.source_sha256,
        capability.source_identity,
        capability.yir_identity,
        capability.materialization_fold,
        capability.exit_code,
        capability.stderr_bytes,
        capability.stderr_sha256,
        capability.candidate_owned_yir_materialization,
        capability.equivalent_nsld_input,
        capability.native_object,
        capability.stage0_handoff_required,
        capability.provider_dependency_required,
        capability.replacement_authorized,
        capability.selection_authorized,
        escape_toml_string(&capability.verdict),
        capability.proof_sha256,
    )
}

pub fn parse_compiler_candidate_nsld_materialization_capability(
    path: &Path,
) -> Result<CompilerCandidateNsldMaterializationCapability, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate Nsld materialization capability `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_nsld_materialization_capability_from_source(&source, path)
}

pub fn parse_compiler_candidate_nsld_materialization_capability_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateNsldMaterializationCapability, ArtifactError> {
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let boolean = |key| parse_required_toml_bool(source, key, path);
    let capability = CompilerCandidateNsldMaterializationCapability {
        protocol: string("protocol")?,
        driver_contract: string("driver_contract")?,
        authority: string("authority")?,
        argument_contract: string("argument_contract")?,
        component_id: string("component_id")?,
        component_domain: string("component_domain")?,
        component_unit: string("component_unit")?,
        candidate_record_sha256: string("candidate_record_sha256")?,
        candidate_reproducible_build_sha256: string("candidate_reproducible_build_sha256")?,
        candidate_producer_id: string("candidate_producer_id")?,
        candidate_compiler_image_sha256: string("candidate_compiler_image_sha256")?,
        production_protocol: string("production_protocol")?,
        production_proof_sha256: string("production_proof_sha256")?,
        predecessor_successor_protocol: string("predecessor_successor_protocol")?,
        predecessor_successor_file: string("predecessor_successor_file")?,
        predecessor_successor_proof_sha256: string("predecessor_successor_proof_sha256")?,
        fresh_source_capability_protocol: string("fresh_source_capability_protocol")?,
        fresh_source_capability_file: string("fresh_source_capability_file")?,
        fresh_source_capability_bytes: number("fresh_source_capability_bytes")?,
        fresh_source_capability_sha256: string("fresh_source_capability_sha256")?,
        fresh_source_capability_proof_sha256: string("fresh_source_capability_proof_sha256")?,
        adapter_file: string("adapter_file")?,
        adapter_bytes: number("adapter_bytes")?,
        adapter_sha256: string("adapter_sha256")?,
        nsld_input_protocol: string("nsld_input_protocol")?,
        nsld_input_file: string("nsld_input_file")?,
        nsld_input_bytes: number("nsld_input_bytes")?,
        nsld_input_sha256: string("nsld_input_sha256")?,
        source_sha256: string("source_sha256")?,
        source_identity: number("source_identity")?,
        yir_identity: number("yir_identity")?,
        materialization_fold: number("materialization_fold")?,
        exit_code: number("exit_code")?,
        stderr_bytes: number("stderr_bytes")?,
        stderr_sha256: string("stderr_sha256")?,
        candidate_owned_yir_materialization: boolean("candidate_owned_yir_materialization")?,
        equivalent_nsld_input: boolean("equivalent_nsld_input")?,
        native_object: boolean("native_object")?,
        stage0_handoff_required: boolean("stage0_handoff_required")?,
        provider_dependency_required: boolean("provider_dependency_required")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
        verdict: string("verdict")?,
        proof_sha256: string("proof_sha256")?,
    };
    validate_capability(&capability)?;
    if render_compiler_candidate_nsld_materialization_capability(&capability) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate Nsld materialization capability `{}` is not canonical",
            path.display()
        )));
    }
    Ok(capability)
}

fn validate_capability(
    capability: &CompilerCandidateNsldMaterializationCapability,
) -> Result<(), ArtifactError> {
    if capability.component_id.is_empty()
        || capability.component_domain.is_empty()
        || capability.component_unit.is_empty()
        || capability.candidate_producer_id.is_empty()
        || capability.fresh_source_capability_bytes == 0
        || capability.adapter_bytes == 0
        || capability.nsld_input_bytes == 0
        || capability.source_identity == 0
        || capability.yir_identity == 0
        || capability.materialization_fold == 0
        || capability.protocol != COMPILER_CANDIDATE_NSLD_MATERIALIZATION_PROTOCOL
        || capability.driver_contract != COMPILER_CANDIDATE_NSLD_MATERIALIZATION_DRIVER
        || capability.authority != COMPILER_CANDIDATE_NSLD_MATERIALIZATION_AUTHORITY
        || capability.argument_contract != COMPILER_CANDIDATE_NSLD_MATERIALIZATION_ARGUMENT_CONTRACT
        || capability.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || capability.predecessor_successor_protocol != COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL
        || capability.predecessor_successor_file != COMPILER_CANDIDATE_SUCCESSOR_FILE
        || capability.fresh_source_capability_protocol
            != COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL
        || capability.fresh_source_capability_file
            != COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE
        || capability.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || capability.nsld_input_protocol != COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL
        || capability.nsld_input_file != COMPILER_CANDIDATE_NSLD_INPUT_FILE
        || capability.exit_code != 0
        || capability.stderr_bytes != 0
        || capability.stderr_sha256 != sha256_hex(&[])
        || !capability.candidate_owned_yir_materialization
        || !capability.equivalent_nsld_input
        || capability.native_object
        || capability.stage0_handoff_required
        || capability.provider_dependency_required
        || capability.replacement_authorized
        || capability.selection_authorized
        || capability.verdict != COMPILER_CANDIDATE_NSLD_MATERIALIZATION_VERDICT
        || capability.proof_sha256 != capability_identity(capability)
    {
        return Err(ArtifactError::new(
            "compiler candidate Nsld materialization capability is invalid",
        ));
    }
    for hash in capability_hashes(capability) {
        validate_sha256(hash)?;
    }
    Ok(())
}

fn capability_hashes(capability: &CompilerCandidateNsldMaterializationCapability) -> [&str; 12] {
    [
        &capability.candidate_record_sha256,
        &capability.candidate_reproducible_build_sha256,
        &capability.candidate_compiler_image_sha256,
        &capability.production_proof_sha256,
        &capability.predecessor_successor_proof_sha256,
        &capability.fresh_source_capability_sha256,
        &capability.fresh_source_capability_proof_sha256,
        &capability.adapter_sha256,
        &capability.nsld_input_sha256,
        &capability.source_sha256,
        &capability.stderr_sha256,
        &capability.proof_sha256,
    ]
}

fn capability_identity(capability: &CompilerCandidateNsldMaterializationCapability) -> String {
    let mut identity_record = capability.clone();
    identity_record.proof_sha256.clear();
    sha256_hex(
        render_compiler_candidate_nsld_materialization_capability(&identity_record).as_bytes(),
    )
}

fn validate_sha256(value: &str) -> Result<(), ArtifactError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ArtifactError::new(
            "compiler candidate Nsld materialization SHA-256 identity is invalid",
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
#[path = "compiler_candidate_nsld_materialization_tests.rs"]
mod tests;

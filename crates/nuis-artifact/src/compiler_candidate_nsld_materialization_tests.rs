use std::path::Path;

use super::{
    capability_identity, parse_compiler_candidate_nsld_materialization_capability_from_source,
    render_compiler_candidate_nsld_materialization_capability,
    CompilerCandidateNsldMaterializationCapability,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_ARGUMENT_CONTRACT,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_AUTHORITY,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_DRIVER,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_PROTOCOL,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_VERDICT,
};
use crate::{
    COMPILER_CANDIDATE_ADAPTER_FILE, COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE,
    COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL, COMPILER_CANDIDATE_NSLD_INPUT_FILE,
    COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL, COMPILER_CANDIDATE_PRODUCTION_PROTOCOL,
    COMPILER_CANDIDATE_SUCCESSOR_FILE, COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL,
};

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn capability() -> CompilerCandidateNsldMaterializationCapability {
    let mut capability = CompilerCandidateNsldMaterializationCapability {
        protocol: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_DRIVER.to_owned(),
        authority: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_AUTHORITY.to_owned(),
        argument_contract: COMPILER_CANDIDATE_NSLD_MATERIALIZATION_ARGUMENT_CONTRACT.to_owned(),
        component_id: "nuisc".to_owned(),
        component_domain: "compiler-toolchain".to_owned(),
        component_unit: "bootstrap".to_owned(),
        candidate_record_sha256: hash('a'),
        candidate_reproducible_build_sha256: hash('b'),
        candidate_producer_id: "candidate".to_owned(),
        candidate_compiler_image_sha256: hash('c'),
        production_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        production_proof_sha256: hash('d'),
        predecessor_successor_protocol: COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL.to_owned(),
        predecessor_successor_file: COMPILER_CANDIDATE_SUCCESSOR_FILE.to_owned(),
        predecessor_successor_proof_sha256: hash('e'),
        fresh_source_capability_protocol: COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL
            .to_owned(),
        fresh_source_capability_file: COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE.to_owned(),
        fresh_source_capability_bytes: 100,
        fresh_source_capability_sha256: hash('f'),
        fresh_source_capability_proof_sha256: hash('1'),
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE.to_owned(),
        adapter_bytes: 200,
        adapter_sha256: hash('2'),
        nsld_input_protocol: COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL.to_owned(),
        nsld_input_file: COMPILER_CANDIDATE_NSLD_INPUT_FILE.to_owned(),
        nsld_input_bytes: 300,
        nsld_input_sha256: hash('3'),
        source_sha256: hash('4'),
        source_identity: 12_832_741_133,
        yir_identity: 9_279_238_763,
        materialization_fold: 1_403_051_547,
        exit_code: 0,
        stderr_bytes: 0,
        stderr_sha256: super::sha256_hex(&[]),
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
    capability
}

#[test]
fn materialization_capability_roundtrips_canonically() {
    let capability = capability();
    let source = render_compiler_candidate_nsld_materialization_capability(&capability);
    let parsed = parse_compiler_candidate_nsld_materialization_capability_from_source(
        &source,
        Path::new("candidate-nsld-materialization.toml"),
    )
    .expect("parse canonical materialization capability");

    assert_eq!(parsed, capability);
    assert!(parsed.candidate_owned_yir_materialization);
    assert!(parsed.equivalent_nsld_input);
    assert!(!parsed.native_object);
}

#[test]
fn materialization_capability_rejects_authority_drift() {
    let source = render_compiler_candidate_nsld_materialization_capability(&capability());
    let damaged = source.replacen(
        "replacement_authorized = false",
        "replacement_authorized = true",
        1,
    );
    assert!(
        parse_compiler_candidate_nsld_materialization_capability_from_source(
            &damaged,
            Path::new("damaged-materialization-capability.toml"),
        )
        .is_err()
    );
}

#[test]
fn materialization_capability_rejects_incomplete_bound_evidence() {
    let mut capability = capability();
    capability.adapter_bytes = 0;
    capability.proof_sha256 = capability_identity(&capability);
    let source = render_compiler_candidate_nsld_materialization_capability(&capability);

    assert!(
        parse_compiler_candidate_nsld_materialization_capability_from_source(
            &source,
            Path::new("incomplete-materialization-capability.toml"),
        )
        .is_err()
    );
}

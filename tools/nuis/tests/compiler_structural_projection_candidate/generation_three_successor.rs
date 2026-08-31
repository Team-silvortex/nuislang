use std::{fs, path::Path, process::Command};

use nuis_artifact::{
    compiler_component_attester_trust_registry_sha256,
    compiler_component_replacement_authorizer_registry_sha256,
    parse_compiler_candidate_direct_compile_capability,
    parse_compiler_candidate_fresh_source_capability, parse_compiler_candidate_fresh_source_result,
    parse_compiler_candidate_successor, COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT,
    COMPILER_CANDIDATE_SUCCESSOR_FILE, COMPILER_CANDIDATE_SUCCESSOR_VERDICT,
};

use super::assert_success;

pub(super) fn assert_signed_direct_successor(
    output_dir: &Path,
    selected_root: &Path,
    output_dir_text: &str,
) {
    let direct_result = output_dir.join("candidate-direct-front-end-result.txt");
    let direct_capability = output_dir.join("candidate-direct-capability.toml");
    let direct = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-candidate-direct-compile")
        .arg(selected_root)
        .arg(&direct_result)
        .arg(&direct_capability)
        .env(
            "NUIS_BOOTSTRAP_STAGE0_PROVIDER_V1",
            "/provider-must-not-be-observed",
        )
        .output()
        .expect("execute direct stage1 front-end compile");
    assert_success(&direct, "direct stage1 front-end compile");
    let capability = parse_compiler_candidate_direct_compile_capability(&direct_capability)
        .expect("verify direct compile capability");
    assert!(!capability.provider_dependency_required);
    assert!(capability.direct_stage1_compile);
    assert!(!capability.native_materialization);

    let aggregate = output_dir.join("nuis.compiler-component-reproducibility.toml");
    let attestation = output_dir.join("nuis.compiler-component-attestation.toml");
    let attester_registry = output_dir.join("attester-registry.toml");
    let attester_registry_source =
        fs::read_to_string(&attester_registry).expect("read attester registry source");
    let attester_registry_sha256 =
        compiler_component_attester_trust_registry_sha256(&attester_registry_source);
    let authorization = output_dir.join("component-authorization.toml");
    let owner_registry = output_dir.join("component-owner-registry.toml");
    let owner_registry_source =
        fs::read_to_string(&owner_registry).expect("read component-owner registry source");
    let owner_registry_sha256 =
        compiler_component_replacement_authorizer_registry_sha256(&owner_registry_source);
    let active_state = output_dir.join("component-active-state.toml");
    let transition = output_dir.join("component-transition.toml");
    let delegated_capability = output_dir.join("preselection-capability.toml");
    let preselection = output_dir.join("candidate-preselection.toml");
    let successor_path = output_dir.join("candidate-successor.toml");
    let transition_before = fs::read(&transition).expect("snapshot successor transition");
    let preselection_before = fs::read(&preselection).expect("snapshot predecessor preselection");
    let delegated_before =
        fs::read(&delegated_capability).expect("snapshot delegated capability v1");

    let signed = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-sign-candidate-successor")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(&attester_registry_sha256)
        .arg("f".repeat(64))
        .arg(&authorization)
        .arg(&owner_registry)
        .arg(&owner_registry_sha256)
        .arg("d".repeat(64))
        .arg(&active_state)
        .arg(&transition)
        .arg("e".repeat(64))
        .arg(selected_root)
        .arg(&delegated_capability)
        .arg(&preselection)
        .arg("f".repeat(64))
        .arg(&direct_capability)
        .arg(&direct_result)
        .arg("b".repeat(64))
        .arg("compiler-owner-1")
        .arg("release-control")
        .arg("clean-build-direct-successor-3")
        .arg(&successor_path)
        .env("NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX", "09".repeat(32))
        .output()
        .expect("sign generation-three direct successor");
    assert_success(&signed, "signed generation-three direct successor");

    let successor = parse_compiler_candidate_successor(&successor_path)
        .expect("verify generation-three candidate successor");
    assert_eq!(successor.target_generation, 3);
    assert_eq!(successor.verdict, COMPILER_CANDIDATE_SUCCESSOR_VERDICT);
    assert!(successor.successor_authorized);
    assert!(successor.direct_stage1_compile);
    assert!(!successor.provider_dependency_required);
    assert!(!successor.fresh_source_compile);
    assert!(!successor.native_materialization);
    assert!(!successor.replacement_authorized);
    assert!(!successor.selection_authorized);
    let successor_source =
        fs::read_to_string(&successor_path).expect("read candidate successor source");
    assert!(!successor_source.contains(output_dir_text));
    assert_eq!(
        fs::read(&transition).expect("reread successor transition"),
        transition_before
    );
    assert_eq!(
        fs::read(&preselection).expect("reread predecessor preselection"),
        preselection_before
    );
    assert_eq!(
        fs::read(&delegated_capability).expect("reread delegated capability v1"),
        delegated_before
    );
    assert_fresh_source_capability(output_dir, selected_root, &successor_path, output_dir_text);
}

fn assert_fresh_source_capability(
    output_dir: &Path,
    selected_root: &Path,
    successor: &Path,
    output_dir_text: &str,
) {
    let source =
        Path::new("../../tests/fixtures/bootstrap/accepted/compiler_candidate_fresh_snapshot.ns");
    let result_path = output_dir.join("candidate-fresh-source-result.txt");
    let capability_path = output_dir.join("candidate-fresh-source-capability.toml");
    let successor_before = fs::read(successor).expect("snapshot signed candidate successor");
    let output = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-candidate-fresh-source")
        .arg(selected_root)
        .arg(successor)
        .arg(source)
        .arg(&result_path)
        .arg(&capability_path)
        .env(
            "NUIS_BOOTSTRAP_STAGE0_PROVIDER_V1",
            "/provider-must-not-be-observed",
        )
        .output()
        .expect("execute candidate-owned fresh-source compiler");
    assert_success(&output, "candidate-owned fresh-source compiler");

    let result = parse_compiler_candidate_fresh_source_result(&result_path)
        .expect("verify candidate fresh-source result");
    assert_eq!(result.stages[1].record_count, 16);
    assert_eq!(result.stages[2].record_count, 5);
    assert_eq!(result.stages[3].record_count, 6);
    assert_eq!(result.stages[4].record_count, 6);
    assert_eq!(result.stages[1].identity, 8_634_151_688);
    assert_eq!(result.stages[2].identity, 16_043_672_006);
    assert_eq!(result.stages[3].identity, 12_661_455_449);
    assert_eq!(result.stages[4].identity, 9_279_238_763);
    assert_eq!(result.bundle_fold, 357_450_558);
    assert!(!result.stage0_handoff_required);
    assert!(result.candidate_owned_source_processing);
    assert!(result.fresh_source_compile);
    assert!(!result.native_materialization);

    let capability = parse_compiler_candidate_fresh_source_capability(&capability_path)
        .expect("verify candidate fresh-source capability");
    assert_eq!(capability.verdict, COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT);
    assert_eq!(
        capability.predecessor_successor_file,
        COMPILER_CANDIDATE_SUCCESSOR_FILE
    );
    assert!(!capability.stage0_handoff_required);
    assert!(!capability.provider_dependency_required);
    assert!(capability.candidate_owned_source_processing);
    assert!(capability.direct_stage1_compile);
    assert!(capability.fresh_source_compile);
    assert!(!capability.native_materialization);
    assert!(!capability.replacement_authorized);
    assert!(!capability.selection_authorized);
    let capability_source =
        fs::read_to_string(&capability_path).expect("read fresh-source capability source");
    assert!(!capability_source.contains(output_dir_text));
    assert_eq!(
        fs::read(successor).expect("reread signed candidate successor"),
        successor_before
    );

    let tampered_source = output_dir.join("candidate-fresh-source-tampered.ns");
    let mut tampered = fs::read(source).expect("read canonical fresh-source snapshot");
    let literal = tampered
        .iter()
        .position(|byte| *byte == b'7')
        .expect("find canonical source literal");
    tampered[literal] = b'8';
    fs::write(&tampered_source, tampered).expect("write tampered fresh-source snapshot");
    let rejected_result = output_dir.join("candidate-fresh-source-rejected.txt");
    let rejected_capability = output_dir.join("candidate-fresh-source-rejected.toml");
    let rejected = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-candidate-fresh-source")
        .arg(selected_root)
        .arg(successor)
        .arg(&tampered_source)
        .arg(&rejected_result)
        .arg(&rejected_capability)
        .output()
        .expect("reject drifted fresh-source snapshot");
    assert!(!rejected.status.success());
    assert!(!rejected_result.exists());
    assert!(!rejected_capability.exists());
}

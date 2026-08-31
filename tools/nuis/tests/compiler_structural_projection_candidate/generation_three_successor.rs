use std::{fs, path::Path, process::Command};

use nuis_artifact::{
    compiler_component_attester_trust_registry_sha256,
    compiler_component_replacement_authorizer_registry_sha256,
    parse_compiler_candidate_direct_compile_capability, parse_compiler_candidate_successor,
    COMPILER_CANDIDATE_SUCCESSOR_VERDICT,
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
}

use std::{path::PathBuf, process::Command};

use ed25519_dalek::SigningKey;
use nuis_artifact::{
    build_compiler_component_replacement_authorizer_registry,
    compiler_component_attester_trust_registry_sha256,
    compiler_component_replacement_authorizer_registry_sha256,
    parse_compiler_component_active_state, parse_compiler_component_attester_trust_registry,
    parse_compiler_component_replacement_authorization, parse_compiler_component_transition,
    read_compiler_component_attestation, render_compiler_component_replacement_authorizer_registry,
    select_compiler_component_active_target, select_compiler_component_transition_target,
    CompilerComponentActiveSelection, CompilerComponentReplacementAuthorizerEntryInput,
    CompilerComponentTransitionSelection, CompilerComponentTransitionVerificationInput,
};

const CHALLENGE_SHA256: &str = "d5aeef8c1d33a5b473f11142197fd361df26dc0f2ec1c0188362f9ece139338c";
const REGISTRY_SHA256: &str = "90b8f7f4c9d336c72caa7dc4dc9a91c41ec263a7bfffa282ee8211088b164f01";

fn evidence_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/evidence/compiler-attestation/linux-amd64-cleanroom/generation-1")
}

#[test]
fn checked_in_remote_attestation_verifies_and_fails_closed() {
    let evidence = evidence_dir();
    let aggregate = evidence.join("nuis.compiler-component-reproducibility.toml");
    let attestation = evidence.join("nuis.compiler-component-attestation.toml");
    let registry = evidence.join("nuis.compiler-component-attester-trust-registry.toml");

    let registry_source = std::fs::read_to_string(&registry).expect("read trust registry");
    assert_eq!(
        compiler_component_attester_trust_registry_sha256(&registry_source),
        REGISTRY_SHA256
    );
    let parsed_registry = parse_compiler_component_attester_trust_registry(&registry)
        .expect("parse canonical trust registry");
    assert_eq!(parsed_registry.generation, 1);
    assert_eq!(parsed_registry.entries.len(), 1);

    let claim = read_compiler_component_attestation(
        &attestation,
        &aggregate,
        &registry,
        REGISTRY_SHA256,
        CHALLENGE_SHA256,
    )
    .expect("verify checked-in remote attestation");
    assert_eq!(claim.attester_id, "kyuubiki-lab-1");
    assert_eq!(claim.environment_id, "linux-amd64-cleanroom");
    assert_eq!(claim.run_count, 2);
    assert!(!claim.replacement_authorized);

    let wrong_challenge = "0".repeat(64);
    let error = read_compiler_component_attestation(
        &attestation,
        &aggregate,
        &registry,
        REGISTRY_SHA256,
        &wrong_challenge,
    )
    .expect_err("challenge replay must fail closed");
    assert!(error.to_string().contains("verifier request"));

    let wrong_registry_pin = "0".repeat(64);
    let error = read_compiler_component_attestation(
        &attestation,
        &aggregate,
        &registry,
        &wrong_registry_pin,
        CHALLENGE_SHA256,
    )
    .expect_err("unpinned registry bytes must fail closed");
    assert!(error.to_string().contains("pinned SHA-256"));
}

#[test]
fn remote_attestation_requires_a_distinct_pinned_replacement_authorizer() {
    let evidence = evidence_dir();
    let aggregate = evidence.join("nuis.compiler-component-reproducibility.toml");
    let attestation = evidence.join("nuis.compiler-component-attestation.toml");
    let attester_registry = evidence.join("nuis.compiler-component-attester-trust-registry.toml");
    let scratch =
        std::env::temp_dir().join(format!("nuis_component_replacement_{}", std::process::id()));
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).expect("clear stale replacement scratch");
    }
    std::fs::create_dir_all(&scratch).expect("create replacement scratch");
    let authorizer_registry_path = scratch.join("authorizers.toml");
    let authorization_path = scratch.join("authorization.toml");
    let active_state_path = scratch.join("active-state.toml");
    let transition_path = scratch.join("transition.toml");

    let signing_key = SigningKey::from_bytes(&[9; 32]);
    let public_key_hex: String = signing_key
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let authorizer_registry = build_compiler_component_replacement_authorizer_registry(
        1,
        &[CompilerComponentReplacementAuthorizerEntryInput {
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
            component_id: "bootstrap_structural_projection_candidate",
            public_key_hex: &public_key_hex,
            status: "active",
        }],
    )
    .expect("build authorizer registry");
    let authorizer_registry_source =
        render_compiler_component_replacement_authorizer_registry(&authorizer_registry);
    let authorizer_registry_sha256 =
        compiler_component_replacement_authorizer_registry_sha256(&authorizer_registry_source);
    std::fs::write(&authorizer_registry_path, &authorizer_registry_source)
        .expect("write authorizer registry");
    let authorization_challenge = "e".repeat(64);

    let authorize = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-authorize-component-replacement")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .arg("compiler-owner-1")
        .arg("release-control")
        .arg("projection-relay-genesis")
        .arg(&authorization_path)
        .env("NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX", "09".repeat(32))
        .output()
        .expect("run replacement authorization frontdoor");
    assert!(
        authorize.status.success(),
        "authorization failed: {}",
        String::from_utf8_lossy(&authorize.stderr)
    );
    assert!(String::from_utf8_lossy(&authorize.stdout)
        .contains("bootstrap component replacement: authorized"));
    let authorization = parse_compiler_component_replacement_authorization(&authorization_path)
        .expect("parse emitted replacement authorization");
    assert!(authorization.replacement_authorized);
    assert!(authorization.reversible);
    assert!(!authorization.attestation_replacement_authorized);
    assert_ne!(
        authorization.authorizer_public_key_id,
        authorization.attester_public_key_id
    );

    let verify = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-verify-component-replacement")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .output()
        .expect("run replacement verification frontdoor");
    assert!(
        verify.status.success(),
        "verification failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout)
        .contains("bootstrap component replacement: verified"));

    let attestation_source_before =
        std::fs::read_to_string(&attestation).expect("read immutable attestation");
    let authorization_source_before =
        std::fs::read_to_string(&authorization_path).expect("read immutable authorization");
    let activate = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-activate-component")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .output()
        .expect("run active-component consumer");
    assert!(
        activate.status.success(),
        "activation failed: {}",
        String::from_utf8_lossy(&activate.stderr)
    );
    let activation_stdout = String::from_utf8_lossy(&activate.stdout);
    assert!(activation_stdout.contains("bootstrap compiler component: activated"));
    assert!(activation_stdout.contains("active_stage_role: stage1-candidate"));
    assert!(activation_stdout.contains("rollback_stage_role: stage0"));

    let active_state = parse_compiler_component_active_state(&active_state_path)
        .expect("parse emitted active-component state");
    let active = select_compiler_component_active_target(
        &active_state,
        &authorization,
        &authorization_source_before,
        CompilerComponentActiveSelection::Active,
    )
    .expect("resolve active candidate");
    let rollback = select_compiler_component_active_target(
        &active_state,
        &authorization,
        &authorization_source_before,
        CompilerComponentActiveSelection::Rollback,
    )
    .expect("resolve stage0 rollback");
    assert_eq!(active.stage_role, "stage1-candidate");
    assert_eq!(
        active.reproducible_build_sha256,
        authorization.to_reproducible_build_sha256
    );
    assert_eq!(rollback.stage_role, "stage0");
    assert_eq!(
        rollback.reproducible_build_sha256,
        authorization.from_reproducible_build_sha256
    );
    assert_eq!(
        std::fs::read_to_string(&attestation).expect("reread immutable attestation"),
        attestation_source_before
    );
    assert_eq!(
        std::fs::read_to_string(&authorization_path).expect("reread immutable authorization"),
        authorization_source_before
    );

    let replay_state = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-activate-component")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .output()
        .expect("run active-state replacement rejection");
    assert!(!replay_state.status.success());
    assert!(String::from_utf8_lossy(&replay_state.stderr).contains("without replacement"));

    let active_state_source_before =
        std::fs::read_to_string(&active_state_path).expect("read immutable active state");
    let transition_challenge = "f".repeat(64);
    let rollback_component = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-rollback-component")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_challenge)
        .arg("compiler-owner-1")
        .arg("release-control")
        .arg("projection-relay-rollback-2")
        .arg(&transition_path)
        .env("NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX", "09".repeat(32))
        .output()
        .expect("run generation-two rollback frontdoor");
    assert!(
        rollback_component.status.success(),
        "rollback failed: {}",
        String::from_utf8_lossy(&rollback_component.stderr)
    );
    let rollback_stdout = String::from_utf8_lossy(&rollback_component.stdout);
    assert!(rollback_stdout.contains("bootstrap compiler component: rolled back"));
    assert!(rollback_stdout.contains("generation: 2"));
    assert!(rollback_stdout.contains("current_stage_role: stage0"));
    assert!(rollback_stdout.contains("forward_stage_role: stage1-candidate"));

    let transition = parse_compiler_component_transition(&transition_path)
        .expect("parse emitted generation-two transition");
    let transition_verification = CompilerComponentTransitionVerificationInput {
        authorization: &authorization,
        authorization_source: &authorization_source_before,
        active_state: &active_state,
        active_state_source: &active_state_source_before,
        authorizer_registry: &authorizer_registry,
        authorizer_registry_source: &authorizer_registry_source,
        expected_authorizer_registry_sha256: &authorizer_registry_sha256,
        expected_transition_challenge_sha256: &transition_challenge,
    };
    let current = select_compiler_component_transition_target(
        &transition,
        transition_verification,
        CompilerComponentTransitionSelection::Current,
    )
    .expect("resolve restored stage0 transition target");
    let forward = select_compiler_component_transition_target(
        &transition,
        transition_verification,
        CompilerComponentTransitionSelection::Forward,
    )
    .expect("resolve retained candidate transition target");
    assert_eq!(transition.generation, 2);
    assert_eq!(
        transition.predecessor_authorization_proof_sha256,
        authorization.proof_sha256
    );
    assert_eq!(
        transition.predecessor_state_sha256,
        active_state.state_sha256
    );
    assert_eq!(current.stage_role, "stage0");
    assert_eq!(
        current.reproducible_build_sha256,
        authorization.from_reproducible_build_sha256
    );
    assert_eq!(forward.stage_role, "stage1-candidate");
    assert_eq!(
        forward.reproducible_build_sha256,
        authorization.to_reproducible_build_sha256
    );

    let verify_transition = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-verify-component-transition")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_path)
        .arg(&transition_challenge)
        .output()
        .expect("run generation-two transition verification");
    assert!(
        verify_transition.status.success(),
        "transition verification failed: {}",
        String::from_utf8_lossy(&verify_transition.stderr)
    );
    assert!(String::from_utf8_lossy(&verify_transition.stdout)
        .contains("bootstrap compiler component transition: verified"));
    assert_eq!(
        std::fs::read_to_string(&attestation).expect("reread transition attestation"),
        attestation_source_before
    );
    assert_eq!(
        std::fs::read_to_string(&authorization_path).expect("reread transition authorization"),
        authorization_source_before
    );
    assert_eq!(
        std::fs::read_to_string(&active_state_path).expect("reread transition active state"),
        active_state_source_before
    );

    let transition_replay = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-verify-component-transition")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_path)
        .arg("0".repeat(64))
        .output()
        .expect("run transition challenge replay rejection");
    assert!(!transition_replay.status.success());
    assert!(String::from_utf8_lossy(&transition_replay.stderr).contains("verifier request"));

    let replay = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-verify-component-replacement")
        .arg(&aggregate)
        .arg(&attestation)
        .arg(&attester_registry)
        .arg(REGISTRY_SHA256)
        .arg(CHALLENGE_SHA256)
        .arg(&authorization_path)
        .arg(&authorizer_registry_path)
        .arg(&authorizer_registry_sha256)
        .arg("0".repeat(64))
        .output()
        .expect("run replacement replay rejection");
    assert!(!replay.status.success());
    assert!(String::from_utf8_lossy(&replay.stderr).contains("verifier request"));

    std::fs::remove_dir_all(&scratch).expect("remove replacement scratch");
}

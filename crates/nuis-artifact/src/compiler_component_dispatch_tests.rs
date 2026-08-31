use std::path::Path;

use ed25519_dalek::SigningKey;

use super::*;
use crate::{
    build_compiler_component_active_state, build_compiler_component_attestation,
    build_compiler_component_build, build_compiler_component_compile_dispatch_receipt,
    build_compiler_component_replacement_authorization,
    build_compiler_component_replacement_authorizer_registry, build_compiler_component_transition,
    compiler_component_replacement_authorizer_registry_sha256,
    compiler_component_reproducibility::build_from_runs, encode_nuis_compiled_artifact_binary,
    promote_compiler_component_candidate, render_compiler_component_active_state,
    render_compiler_component_attestation, render_compiler_component_compile_dispatch_receipt,
    render_compiler_component_replacement_authorization,
    render_compiler_component_replacement_authorizer_registry,
    render_compiler_component_reproducibility, CompilerComponentAttestationInput,
    CompilerComponentBuildInput, CompilerComponentCandidatePromotionInput,
    CompilerComponentCompileDispatchReceiptInput, CompilerComponentDependencyInput,
    CompilerComponentReplacementAuthorizationInput,
    CompilerComponentReplacementAuthorizerEntryInput, CompilerComponentReproducibilityRun,
    CompilerComponentTransitionInput, NuisCompiledArtifact, NuisExecutableEnvelope,
    NuisLifecycleContract, COMPILER_COMPONENT_COMPILE_DISPATCH_FILE,
    COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT, COMPILER_COMPONENT_STAGE0_ROLE,
};

const TRANSITION_CHALLENGE: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const STAGE0_IMAGE: &[u8] = b"stage0 frontdoor compiler image";
const FORWARD_IMAGE: &[u8] = b"stage1 candidate compiler image";

fn hash(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn signing_key_hex(seed: u8) -> String {
    std::iter::repeat_n(format!("{seed:02x}"), 32).collect()
}

fn public_key_hex(seed: u8) -> String {
    SigningKey::from_bytes(&[seed; 32])
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn component_pair() -> (CompilerComponentBuild, CompilerComponentBuild) {
    let compiled_artifact = compiled_artifact("request build manifest", b"native compiler output");
    let dependencies = [CompilerComponentDependencyInput {
        kind: "component-source",
        identity: "main.ns",
        bytes: b"mod cpu Main {}\n",
    }];
    let stage0 = build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role: COMPILER_COMPONENT_STAGE0_ROLE,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v8",
        component_id: "projection_relay",
        component_domain: "cpu",
        component_unit: "Main",
        producer_id: "nuisc-stage0-reference",
        compiler_image: STAGE0_IMAGE,
        stage_handoff_file: "nuis.compiler-stage-handoff.toml",
        stage_handoff_bundle_sha256: &hash('1'),
        build_manifest_file: "nuis.build.manifest.toml",
        build_manifest: b"build manifest",
        compiled_artifact_file: "nuis.compiled.artifact",
        compiled_artifact: &compiled_artifact,
        native_binary_file: "projection_relay",
        native_binary: b"native compiler output",
        dependencies: &dependencies,
    })
    .expect("build stage0 component");
    let candidate =
        promote_compiler_component_candidate(&CompilerComponentCandidatePromotionInput {
            stage0: &stage0,
            producer_id: "nuis-stage1-producer",
            compiler_image: FORWARD_IMAGE,
            stage_handoff_bundle_sha256: &stage0.stage_handoff_bundle_sha256,
        })
        .expect("promote candidate component");
    (stage0, candidate)
}

fn rebuilt_stage0(native_binary: &[u8]) -> CompilerComponentBuild {
    let compiled_artifact = compiled_artifact("result build manifest", native_binary);
    rebuilt_stage0_with_artifact(native_binary, &compiled_artifact)
}

fn rebuilt_stage0_with_artifact(
    native_binary: &[u8],
    compiled_artifact: &[u8],
) -> CompilerComponentBuild {
    let dependencies = [CompilerComponentDependencyInput {
        kind: "component-source",
        identity: "main.ns",
        bytes: b"mod cpu Main {}\n",
    }];
    build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role: COMPILER_COMPONENT_STAGE0_ROLE,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v8",
        component_id: "projection_relay",
        component_domain: "cpu",
        component_unit: "Main",
        producer_id: "nuisc-stage0-reference",
        compiler_image: STAGE0_IMAGE,
        stage_handoff_file: "nuis.compiler-stage-handoff.toml",
        stage_handoff_bundle_sha256: &hash('1'),
        build_manifest_file: "nuis.build.manifest.toml",
        build_manifest: b"cache-hit build manifest",
        compiled_artifact_file: "nuis.compiled.artifact",
        compiled_artifact,
        native_binary_file: "projection_relay",
        native_binary,
        dependencies: &dependencies,
    })
    .expect("build dispatched stage0 result")
}

fn compiled_artifact(build_manifest_source: &str, binary_blob: &[u8]) -> Vec<u8> {
    compiled_artifact_with_mode(build_manifest_source, binary_blob, "native-cpu-llvm")
}

fn compiled_artifact_with_mode(
    build_manifest_source: &str,
    binary_blob: &[u8],
    packaging_mode: &str,
) -> Vec<u8> {
    encode_nuis_compiled_artifact_binary(&NuisCompiledArtifact {
        schema: "nuis-compiled-artifact-v1".to_owned(),
        packaging_mode: packaging_mode.to_owned(),
        cpu_target_abi: "cpu.test".to_owned(),
        cpu_target_machine_arch: "test".to_owned(),
        cpu_target_machine_os: "test".to_owned(),
        cpu_target_object_format: "test".to_owned(),
        cpu_target_calling_abi: "test".to_owned(),
        binary_name: "projection_relay".to_owned(),
        binary_bytes: binary_blob.len(),
        build_manifest_bytes: build_manifest_source.len(),
        envelope: NuisExecutableEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            executable_kind: "native-cpu-llvm".to_owned(),
            package_count: 1,
            domain_families: vec!["cpu".to_owned()],
            contract_families: vec!["cpu".to_owned()],
            function_kind: "function-node".to_owned(),
            graph_kind: "minimal-function-graph".to_owned(),
            default_time_mode: "logical".to_owned(),
        },
        lifecycle: NuisLifecycleContract {
            schema: "nuis-lifecycle-contract-v1".to_owned(),
            bootstrap_entry: "main".to_owned(),
            tick_policy: "owned".to_owned(),
            shutdown_policy: "drain".to_owned(),
            yalivia_rpc: "disabled".to_owned(),
            hook_surface: vec!["on_bootstrap".to_owned()],
            export_surface: vec!["main".to_owned()],
            runtime_capability_flags: vec!["runtime.bootstrap".to_owned()],
        },
        build_manifest_source: build_manifest_source.to_owned(),
        binary_blob: binary_blob.to_vec(),
        host_objects: Vec::new(),
    })
    .expect("encode test compiled artifact")
}

fn run(
    ordinal: usize,
    stage0: &CompilerComponentBuild,
    candidate: &CompilerComponentBuild,
) -> CompilerComponentReproducibilityRun {
    CompilerComponentReproducibilityRun {
        ordinal,
        run_id: format!("clean-build-{ordinal}"),
        component_id: stage0.component_id.clone(),
        clean_root_state: "absent-or-empty-before-build".to_owned(),
        clean_root_witness_sha256: hash(if ordinal == 0 { '2' } else { '3' }),
        stage0_record_sha256: if ordinal == 0 {
            stage0.record_sha256.clone()
        } else {
            hash('4')
        },
        stage0_reproducible_build_sha256: stage0.reproducible_build_sha256.clone(),
        candidate_record_sha256: if ordinal == 0 {
            candidate.record_sha256.clone()
        } else {
            hash('5')
        },
        candidate_reproducible_build_sha256: candidate.reproducible_build_sha256.clone(),
        candidate_compiler_image_sha256: candidate.compiler_image_sha256.clone(),
        native_output_sha256: candidate.native_binary_sha256.clone(),
        production_proof_sha256: hash(if ordinal == 0 { '6' } else { '7' }),
        differential_report_sha256: hash(if ordinal == 0 { '8' } else { '9' }),
        comparison_count: 13,
        equivalent_count: 13,
        deterministic_artifact_equivalent: true,
        differential_verdict: "equivalent-awaiting-authorization".to_owned(),
        replacement_authorized: false,
    }
}

struct Fixture {
    stage0: CompilerComponentBuild,
    candidate: CompilerComponentBuild,
    transition: CompilerComponentTransition,
    authorization: crate::CompilerComponentReplacementAuthorization,
    authorization_source: String,
    active_state: crate::CompilerComponentActiveState,
    active_state_source: String,
    authorizer_registry: crate::CompilerComponentReplacementAuthorizerRegistry,
    authorizer_registry_source: String,
    authorizer_registry_sha256: String,
}

fn fixture() -> Fixture {
    let (stage0, candidate) = component_pair();
    let report = build_from_runs(vec![
        run(0, &stage0, &candidate),
        run(1, &stage0, &candidate),
    ])
    .expect("build reproducibility report");
    let report_source = render_compiler_component_reproducibility(&report);
    let attestation = build_compiler_component_attestation(
        CompilerComponentAttestationInput {
            reproducibility: &report,
            reproducibility_source: &report_source,
            challenge_sha256: &hash('a'),
            attester_id: "linux-builder-1",
            environment_id: "linux-amd64-cleanroom",
        },
        &signing_key_hex(7),
    )
    .expect("build attestation");
    let attestation_source = render_compiler_component_attestation(&attestation);
    let authorization = build_compiler_component_replacement_authorization(
        CompilerComponentReplacementAuthorizationInput {
            reproducibility: &report,
            reproducibility_source: &report_source,
            attestation: &attestation,
            attestation_source: &attestation_source,
            challenge_sha256: &hash('b'),
            authorization_id: "projection-relay-genesis",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect("build authorization");
    let authorization_source = render_compiler_component_replacement_authorization(&authorization);
    let active_state = build_compiler_component_active_state(&authorization, &authorization_source)
        .expect("build active state");
    let active_state_source = render_compiler_component_active_state(&active_state);
    let authorizer_registry = build_compiler_component_replacement_authorizer_registry(
        1,
        &[CompilerComponentReplacementAuthorizerEntryInput {
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
            component_id: "projection_relay",
            public_key_hex: &public_key_hex(9),
            status: "active",
        }],
    )
    .expect("build authorizer registry");
    let authorizer_registry_source =
        render_compiler_component_replacement_authorizer_registry(&authorizer_registry);
    let authorizer_registry_sha256 =
        compiler_component_replacement_authorizer_registry_sha256(&authorizer_registry_source);
    let transition = build_compiler_component_transition(
        CompilerComponentTransitionInput {
            authorization: &authorization,
            authorization_source: &authorization_source,
            active_state: &active_state,
            active_state_source: &active_state_source,
            challenge_sha256: TRANSITION_CHALLENGE,
            transition_id: "projection-relay-rollback-2",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect("build transition");
    Fixture {
        stage0,
        candidate,
        transition,
        authorization,
        authorization_source,
        active_state,
        active_state_source,
        authorizer_registry,
        authorizer_registry_source,
        authorizer_registry_sha256,
    }
}

fn verification_input(fixture: &Fixture) -> CompilerComponentTransitionVerificationInput<'_> {
    CompilerComponentTransitionVerificationInput {
        authorization: &fixture.authorization,
        authorization_source: &fixture.authorization_source,
        active_state: &fixture.active_state,
        active_state_source: &fixture.active_state_source,
        authorizer_registry: &fixture.authorizer_registry,
        authorizer_registry_source: &fixture.authorizer_registry_source,
        expected_authorizer_registry_sha256: &fixture.authorizer_registry_sha256,
        expected_transition_challenge_sha256: TRANSITION_CHALLENGE,
    }
}

#[test]
fn signed_targets_resolve_without_inventory_order_or_paths() {
    let fixture = fixture();
    let candidates = [
        CompilerComponentDispatchCandidate {
            component: &fixture.candidate,
            compiler_image: FORWARD_IMAGE,
        },
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: STAGE0_IMAGE,
        },
    ];
    let resolution = resolve_compiler_component_dispatch(
        &fixture.transition,
        verification_input(&fixture),
        &candidates,
    )
    .expect("resolve path-free dispatch");
    assert_eq!(
        resolution.current().component.reproducible_build_sha256,
        fixture.transition.current_reproducible_build_sha256
    );
    assert_eq!(resolution.current().compiler_image, STAGE0_IMAGE);
    assert_eq!(resolution.forward().compiler_image, FORWARD_IMAGE);

    let receipt =
        build_compiler_component_dispatch_receipt(CompilerComponentDispatchReceiptInput {
            transition: &fixture.transition,
            resolution: &resolution,
            exit_code: 0,
            stdout: b"nuis toolchain frontdoor\n",
            stderr: b"",
        })
        .expect("build dispatch receipt");
    let source = render_compiler_component_dispatch_receipt(&receipt);
    assert!(!source.contains("/"));
    assert!(!source.contains("timestamp"));
    let parsed = parse_compiler_component_dispatch_receipt_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_DISPATCH_FILE),
    )
    .expect("parse canonical dispatch receipt");
    assert_eq!(parsed, receipt);
    assert_eq!(parsed.verdict, COMPILER_COMPONENT_DISPATCH_VERDICT);
    assert_eq!(
        parsed.forward_reproducible_build_sha256,
        fixture.transition.forward_reproducible_build_sha256
    );
}

#[test]
fn image_tampering_duplicate_registration_and_receipt_drift_fail_closed() {
    let fixture = fixture();
    let tampered = [
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: b"tampered stage0 image",
        },
        CompilerComponentDispatchCandidate {
            component: &fixture.candidate,
            compiler_image: FORWARD_IMAGE,
        },
    ];
    let error = resolve_compiler_component_dispatch(
        &fixture.transition,
        verification_input(&fixture),
        &tampered,
    )
    .expect_err("tampered current image must fail");
    assert!(error.to_string().contains("image identity mismatch"));

    let duplicate = [
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: STAGE0_IMAGE,
        },
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: STAGE0_IMAGE,
        },
    ];
    let error = resolve_compiler_component_dispatch(
        &fixture.transition,
        verification_input(&fixture),
        &duplicate,
    )
    .expect_err("duplicate current registration must fail");
    assert!(error.to_string().contains("repeats the signed current"));

    let valid = [
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: STAGE0_IMAGE,
        },
        CompilerComponentDispatchCandidate {
            component: &fixture.candidate,
            compiler_image: FORWARD_IMAGE,
        },
    ];
    let resolution = resolve_compiler_component_dispatch(
        &fixture.transition,
        verification_input(&fixture),
        &valid,
    )
    .expect("resolve valid inventory");
    let receipt =
        build_compiler_component_dispatch_receipt(CompilerComponentDispatchReceiptInput {
            transition: &fixture.transition,
            resolution: &resolution,
            exit_code: 0,
            stdout: b"nuis help\n",
            stderr: b"",
        })
        .expect("build receipt");
    let source = render_compiler_component_dispatch_receipt(&receipt);
    let tampered_source = source.replacen(
        "selected_stage_role = \"stage0\"",
        "selected_stage_role = \"stage1-candidate\"",
        1,
    );
    let error = parse_compiler_component_dispatch_receipt_from_source(
        &tampered_source,
        Path::new(COMPILER_COMPONENT_DISPATCH_FILE),
    )
    .expect_err("receipt target drift must fail");
    assert!(error.to_string().contains("unsupported contract"));
}

#[test]
fn selected_current_rebuilds_one_canonical_request_without_persisting_paths() {
    let fixture = fixture();
    let candidates = [
        CompilerComponentDispatchCandidate {
            component: &fixture.candidate,
            compiler_image: FORWARD_IMAGE,
        },
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: STAGE0_IMAGE,
        },
    ];
    let resolution = resolve_compiler_component_dispatch(
        &fixture.transition,
        verification_input(&fixture),
        &candidates,
    )
    .expect("resolve compile dispatch");
    let result = rebuilt_stage0(b"native compiler output");
    let request_artifact = compiled_artifact("request build manifest", b"native compiler output");
    let result_artifact = compiled_artifact("result build manifest", b"native compiler output");
    assert_ne!(result.record_sha256, fixture.stage0.record_sha256);
    assert_eq!(
        result.reproducible_build_sha256,
        fixture.stage0.reproducible_build_sha256
    );

    let receipt = build_compiler_component_compile_dispatch_receipt(
        CompilerComponentCompileDispatchReceiptInput {
            transition: &fixture.transition,
            resolution: &resolution,
            request: &fixture.stage0,
            result: &result,
            request_compiled_artifact: &request_artifact,
            result_compiled_artifact: &result_artifact,
            exit_code: 0,
            stdout: b"bootstrap component build: recorded\n",
            stderr: b"",
        },
    )
    .expect("build compile dispatch receipt");
    let source = render_compiler_component_compile_dispatch_receipt(&receipt);
    assert!(!source.contains('/'));
    assert!(!source.contains("timestamp"));
    let parsed = crate::parse_compiler_component_compile_dispatch_receipt_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_COMPILE_DISPATCH_FILE),
    )
    .expect("parse canonical compile dispatch receipt");
    assert_eq!(parsed, receipt);
    assert_eq!(parsed.verdict, COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT);
    assert_eq!(
        parsed.request_reproducible_build_sha256,
        parsed.result_reproducible_build_sha256
    );
    assert_eq!(
        parsed.forward_reproducible_build_sha256,
        fixture.transition.forward_reproducible_build_sha256
    );
}

#[test]
fn compile_result_and_receipt_semantic_drift_fail_closed() {
    let fixture = fixture();
    let candidates = [
        CompilerComponentDispatchCandidate {
            component: &fixture.stage0,
            compiler_image: STAGE0_IMAGE,
        },
        CompilerComponentDispatchCandidate {
            component: &fixture.candidate,
            compiler_image: FORWARD_IMAGE,
        },
    ];
    let resolution = resolve_compiler_component_dispatch(
        &fixture.transition,
        verification_input(&fixture),
        &candidates,
    )
    .expect("resolve compile dispatch");
    let drifted = rebuilt_stage0(b"different native compiler output");
    let request_artifact = compiled_artifact("request build manifest", b"native compiler output");
    let drifted_artifact =
        compiled_artifact("result build manifest", b"different native compiler output");
    let error = build_compiler_component_compile_dispatch_receipt(
        CompilerComponentCompileDispatchReceiptInput {
            transition: &fixture.transition,
            resolution: &resolution,
            request: &fixture.stage0,
            result: &drifted,
            request_compiled_artifact: &request_artifact,
            result_compiled_artifact: &drifted_artifact,
            exit_code: 0,
            stdout: b"bootstrap component build: recorded\n",
            stderr: b"",
        },
    )
    .expect_err("native result drift must fail closed");
    assert!(error
        .to_string()
        .contains("does not satisfy the canonical rebuild request"));

    let semantic_drift_artifact = compiled_artifact_with_mode(
        "result build manifest",
        b"native compiler output",
        "native-cpu-drift",
    );
    let semantic_drift =
        rebuilt_stage0_with_artifact(b"native compiler output", &semantic_drift_artifact);
    let error = build_compiler_component_compile_dispatch_receipt(
        CompilerComponentCompileDispatchReceiptInput {
            transition: &fixture.transition,
            resolution: &resolution,
            request: &fixture.stage0,
            result: &semantic_drift,
            request_compiled_artifact: &request_artifact,
            result_compiled_artifact: &semantic_drift_artifact,
            exit_code: 0,
            stdout: b"bootstrap component build: recorded\n",
            stderr: b"",
        },
    )
    .expect_err("compiled artifact semantic drift must fail closed");
    assert!(error
        .to_string()
        .contains("changed the path-neutral compiled artifact semantics"));

    let result = rebuilt_stage0(b"native compiler output");
    let result_artifact = compiled_artifact("result build manifest", b"native compiler output");
    let receipt = build_compiler_component_compile_dispatch_receipt(
        CompilerComponentCompileDispatchReceiptInput {
            transition: &fixture.transition,
            resolution: &resolution,
            request: &fixture.stage0,
            result: &result,
            request_compiled_artifact: &request_artifact,
            result_compiled_artifact: &result_artifact,
            exit_code: 0,
            stdout: b"bootstrap component build: recorded\n",
            stderr: b"",
        },
    )
    .expect("build valid compile dispatch receipt");
    let source = render_compiler_component_compile_dispatch_receipt(&receipt);
    let tampered = source.replacen(
        &format!(
            "result_native_binary_sha256 = \"{}\"",
            receipt.result_native_binary_sha256
        ),
        &format!("result_native_binary_sha256 = \"{}\"", hash('f')),
        1,
    );
    let error = crate::parse_compiler_component_compile_dispatch_receipt_from_source(
        &tampered,
        Path::new(COMPILER_COMPONENT_COMPILE_DISPATCH_FILE),
    )
    .expect_err("receipt request/result drift must fail closed");
    assert!(error.to_string().contains("inconsistent request"));
}

use super::{
    parse_args, BootstrapComponentReplacementInput, BootstrapComponentReplacementVerificationInput,
    CommandKind, GalaxyCommand,
};
use std::path::PathBuf;

#[test]
fn parses_build_with_self_contained_packaging_mode() {
    let command = parse_args(
        [
            "build".to_owned(),
            "--packaging-mode".to_owned(),
            "nuis-self-contained-image".to_owned(),
            "examples/demo.ns".to_owned(),
            "target/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build packaging mode parses");
    assert_eq!(
        command,
        CommandKind::Build {
            input: PathBuf::from("examples/demo.ns"),
            output_dir: PathBuf::from("target/demo"),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
            packaging_mode: Some("nuis-self-contained-image".to_owned()),
        }
    );
}

#[test]
fn parses_workflow_with_default_input() {
    let command = parse_args(["workflow".to_owned()].into_iter()).expect("workflow parses");
    assert_eq!(
        command,
        CommandKind::Workflow {
            input: PathBuf::from("."),
            json: false,
        }
    );
}

#[test]
fn parses_dev_tensor_json() {
    let command = parse_args(["dev-tensor".to_owned(), "--json".to_owned()].into_iter())
        .expect("dev-tensor parses");
    assert_eq!(command, CommandKind::DevTensor { json: true });
}

#[test]
fn parses_bootstrap_status_default_manifest() {
    let command =
        parse_args(["bootstrap-status".to_owned()].into_iter()).expect("bootstrap-status parses");
    assert_eq!(
        command,
        CommandKind::BootstrapStatus {
            input: PathBuf::from("docs/reference/nuis-self-hosting-readiness.toml"),
            json: false,
        }
    );
}

#[test]
fn parses_bootstrap_status_json_with_explicit_manifest() {
    let command = parse_args(
        [
            "bootstrap-status".to_owned(),
            "--json".to_owned(),
            "target/readiness.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap-status json parses");
    assert_eq!(
        command,
        CommandKind::BootstrapStatus {
            input: PathBuf::from("target/readiness.toml"),
            json: true,
        }
    );
}

#[test]
fn rejects_unknown_bootstrap_status_option() {
    let error = parse_args(["bootstrap-status".to_owned(), "--ready".to_owned()].into_iter())
        .expect_err("unknown option must fail");
    assert_eq!(error, "usage: nuis bootstrap-status [--json] [manifest]");
}

#[test]
fn parses_bootstrap_build_command() {
    let command = parse_args(
        [
            "bootstrap-build".to_owned(),
            "compiler-project".to_owned(),
            "build/compiler".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap-build parses");
    assert_eq!(
        command,
        CommandKind::BootstrapBuild {
            input: PathBuf::from("compiler-project"),
            output_dir: PathBuf::from("build/compiler"),
        }
    );
}

#[test]
fn parses_bootstrap_candidate_probe_command() {
    let command = parse_args(
        [
            "bootstrap-candidate-probe".to_owned(),
            "compiler-project".to_owned(),
            "build/candidate-probe".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap-candidate-probe parses");
    assert_eq!(
        command,
        CommandKind::BootstrapCandidateProbe {
            input: PathBuf::from("compiler-project"),
            output_dir: PathBuf::from("build/candidate-probe"),
        }
    );
}

#[test]
fn parses_bootstrap_candidate_build_command() {
    let command = parse_args(
        [
            "bootstrap-candidate-build".to_owned(),
            "compiler-project".to_owned(),
            "build/candidate-component".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap-candidate-build parses");
    assert_eq!(
        command,
        CommandKind::BootstrapCandidateBuild {
            input: PathBuf::from("compiler-project"),
            output_dir: PathBuf::from("build/candidate-component"),
        }
    );
}

#[test]
fn parses_bootstrap_reproducibility_command() {
    let command = parse_args(
        [
            "bootstrap-reproducibility".to_owned(),
            "compiler-project".to_owned(),
            "build/reproducibility".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap-reproducibility parses");
    assert_eq!(
        command,
        CommandKind::BootstrapReproducibility {
            input: PathBuf::from("compiler-project"),
            output_dir: PathBuf::from("build/reproducibility"),
        }
    );
}

#[test]
fn parses_bootstrap_attestation_commands() {
    let command = parse_args(
        [
            "bootstrap-attest-reproducibility".to_owned(),
            "build/repro/nuis.compiler-component-reproducibility.toml".to_owned(),
            "build/repro/clean-build-0".to_owned(),
            "build/repro/clean-build-1".to_owned(),
            "f".repeat(64),
            "linux-builder-1".to_owned(),
            "linux-amd64-cleanroom".to_owned(),
            "build/repro/nuis.compiler-component-attestation.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap attestation parses");
    assert_eq!(
        command,
        CommandKind::BootstrapAttestReproducibility {
            aggregate: PathBuf::from("build/repro/nuis.compiler-component-reproducibility.toml"),
            first_root: PathBuf::from("build/repro/clean-build-0"),
            second_root: PathBuf::from("build/repro/clean-build-1"),
            challenge_sha256: "f".repeat(64),
            attester_id: "linux-builder-1".to_owned(),
            environment_id: "linux-amd64-cleanroom".to_owned(),
            output: PathBuf::from("build/repro/nuis.compiler-component-attestation.toml"),
        }
    );

    let command = parse_args(
        [
            "bootstrap-verify-reproducibility-attestation".to_owned(),
            "aggregate.toml".to_owned(),
            "attestation.toml".to_owned(),
            "registry.toml".to_owned(),
            "a".repeat(64),
            "f".repeat(64),
        ]
        .into_iter(),
    )
    .expect("bootstrap attestation verification parses");
    assert_eq!(
        command,
        CommandKind::BootstrapVerifyReproducibilityAttestation {
            aggregate: PathBuf::from("aggregate.toml"),
            attestation: PathBuf::from("attestation.toml"),
            trust_registry: PathBuf::from("registry.toml"),
            registry_sha256: "a".repeat(64),
            challenge_sha256: "f".repeat(64),
        }
    );
}

#[test]
fn parses_bootstrap_component_replacement_commands() {
    let command = parse_args(
        [
            "bootstrap-authorize-component-replacement".to_owned(),
            "aggregate.toml".to_owned(),
            "attestation.toml".to_owned(),
            "attesters.toml".to_owned(),
            "a".repeat(64),
            "b".repeat(64),
            "authorizers.toml".to_owned(),
            "c".repeat(64),
            "d".repeat(64),
            "compiler-owner-1".to_owned(),
            "release-control".to_owned(),
            "projection-relay-genesis".to_owned(),
            "authorization.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap replacement authorization parses");
    assert_eq!(
        command,
        CommandKind::BootstrapAuthorizeComponentReplacement(BootstrapComponentReplacementInput {
            aggregate: PathBuf::from("aggregate.toml"),
            attestation: PathBuf::from("attestation.toml"),
            attester_registry: PathBuf::from("attesters.toml"),
            attester_registry_sha256: "a".repeat(64),
            attestation_challenge_sha256: "b".repeat(64),
            authorizer_registry: PathBuf::from("authorizers.toml"),
            authorizer_registry_sha256: "c".repeat(64),
            authorization_challenge_sha256: "d".repeat(64),
            authorizer_id: "compiler-owner-1".to_owned(),
            environment_id: "release-control".to_owned(),
            authorization_id: "projection-relay-genesis".to_owned(),
            output: PathBuf::from("authorization.toml"),
        },)
    );

    let command = parse_args(
        [
            "bootstrap-verify-component-replacement".to_owned(),
            "aggregate.toml".to_owned(),
            "attestation.toml".to_owned(),
            "attesters.toml".to_owned(),
            "a".repeat(64),
            "b".repeat(64),
            "authorization.toml".to_owned(),
            "authorizers.toml".to_owned(),
            "c".repeat(64),
            "d".repeat(64),
        ]
        .into_iter(),
    )
    .expect("bootstrap replacement verification parses");
    assert_eq!(
        command,
        CommandKind::BootstrapVerifyComponentReplacement(
            BootstrapComponentReplacementVerificationInput {
                aggregate: PathBuf::from("aggregate.toml"),
                attestation: PathBuf::from("attestation.toml"),
                attester_registry: PathBuf::from("attesters.toml"),
                attester_registry_sha256: "a".repeat(64),
                attestation_challenge_sha256: "b".repeat(64),
                authorization: PathBuf::from("authorization.toml"),
                authorizer_registry: PathBuf::from("authorizers.toml"),
                authorizer_registry_sha256: "c".repeat(64),
                authorization_challenge_sha256: "d".repeat(64),
            },
        )
    );
}

#[test]
fn parses_bootstrap_diff_command() {
    let command = parse_args(
        [
            "bootstrap-diff".to_owned(),
            "stage0/nuis.compiler-component-build.toml".to_owned(),
            "stage1/nuis.compiler-component-build.toml".to_owned(),
            "audit/nuis.compiler-component-diff.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bootstrap-diff parses");
    assert_eq!(
        command,
        CommandKind::BootstrapDiff {
            stage0_record: PathBuf::from("stage0/nuis.compiler-component-build.toml"),
            candidate_record: PathBuf::from("stage1/nuis.compiler-component-build.toml"),
            report: PathBuf::from("audit/nuis.compiler-component-diff.toml"),
        }
    );
}

#[test]
fn parses_galaxy_provider_resolution() {
    let command = parse_args(
        [
            "galaxy".to_owned(),
            "resolve-deps".to_owned(),
            "examples/project".to_owned(),
            "--provider-root".to_owned(),
            "mirror".to_owned(),
            "--provider-id".to_owned(),
            "fixture.offline".to_owned(),
        ]
        .into_iter(),
    )
    .expect("Galaxy provider resolution parses");
    assert_eq!(
        command,
        CommandKind::Galaxy(GalaxyCommand::ResolveDeps {
            input: PathBuf::from("examples/project"),
            provider_root: PathBuf::from("mirror"),
            provider_id: "fixture.offline".to_owned(),
            provider_kind: "offline-layout".to_owned(),
        })
    );
}

#[test]
fn parses_workflow_json_with_explicit_input() {
    let command = parse_args(
        [
            "workflow".to_owned(),
            "--json".to_owned(),
            "examples/demo.ns".to_owned(),
        ]
        .into_iter(),
    )
    .expect("workflow json parses");
    assert_eq!(
        command,
        CommandKind::Workflow {
            input: PathBuf::from("examples/demo.ns"),
            json: true,
        }
    );
}

#[test]
fn parses_project_imports_json_with_explicit_input() {
    let command = parse_args(
        [
            "project-imports".to_owned(),
            "--json".to_owned(),
            "examples/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("project-imports parses");
    assert_eq!(
        command,
        CommandKind::ProjectImports {
            input: PathBuf::from("examples/demo"),
            json: true,
            apply_suggested: false,
        }
    );
}

#[test]
fn parses_project_imports_apply_suggested_with_default_input() {
    let command =
        parse_args(["project-imports".to_owned(), "--apply-suggested".to_owned()].into_iter())
            .expect("project-imports apply parses");
    assert_eq!(
        command,
        CommandKind::ProjectImports {
            input: PathBuf::from("."),
            json: false,
            apply_suggested: true,
        }
    );
}

#[test]
fn parses_inspect_artifact_json_with_manifest_input() {
    let command = parse_args(
        [
            "inspect-artifact".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("inspect-artifact parses");
    assert_eq!(
        command,
        CommandKind::InspectArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: true,
        }
    );
}

#[test]
fn parses_verify_artifact_with_compiled_artifact_input() {
    let command = parse_args(
        [
            "verify-artifact".to_owned(),
            "target/demo/nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .expect("verify-artifact parses");
    assert_eq!(
        command,
        CommandKind::VerifyArtifact {
            input: PathBuf::from("target/demo/nuis.compiled.artifact"),
            json: false,
        }
    );
}

#[test]
fn parses_run_artifact_with_manifest_input() {
    let command = parse_args(
        [
            "run-artifact".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("run-artifact parses");
    assert_eq!(
        command,
        CommandKind::RunArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: false,
        }
    );
}

#[test]
fn parses_run_artifact_json_with_manifest_input() {
    let command = parse_args(
        [
            "run-artifact".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("run-artifact json parses");
    assert_eq!(
        command,
        CommandKind::RunArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: true,
        }
    );
}

#[test]
fn parses_debug_resume_json_with_manifest_input() {
    let command = parse_args(
        [
            "debug-resume".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("debug-resume json parses");
    assert_eq!(
        command,
        CommandKind::DebugResume {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: true,
            breakpoint: None,
            breakpoint_phase: None,
            breakpoint_entry: None,
            cursor_output: None,
        }
    );
}

#[test]
fn parses_debug_request_selector() {
    let command = parse_args(
        [
            "debug-request".to_owned(),
            "target/demo".to_owned(),
            "--request-id".to_owned(),
            "kernel.cuda.copy".to_owned(),
            "--save-cursor".to_owned(),
            "target/demo/request.cursor.toml".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    )
    .expect("debug-request parses");
    assert_eq!(
        command,
        CommandKind::DebugRequest {
            input: PathBuf::from("target/demo"),
            request_id: "kernel.cuda.copy".to_owned(),
            json: true,
            cursor_output: Some(PathBuf::from("target/demo/request.cursor.toml")),
        }
    );
}

#[test]
fn parses_debug_lineage_repair_json_with_manifest_input() {
    let command = parse_args(
        [
            "debug-lineage-repair".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("debug-lineage-repair json parses");
    assert_eq!(
        command,
        CommandKind::DebugLineageRepair {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: true,
        }
    );
}

#[test]
fn parses_debug_resume_typed_stop_and_cursor_output() {
    let command = parse_args(
        [
            "debug-resume".to_owned(),
            "target/demo".to_owned(),
            "--break-phase".to_owned(),
            "device-dispatch".to_owned(),
            "--break-entry".to_owned(),
            "pixelmagic.blur".to_owned(),
            "--save-cursor".to_owned(),
            "next.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("debug-resume typed stop parses");
    assert_eq!(
        command,
        CommandKind::DebugResume {
            input: PathBuf::from("target/demo"),
            json: false,
            breakpoint: None,
            breakpoint_phase: Some("device-dispatch".to_owned()),
            breakpoint_entry: Some("pixelmagic.blur".to_owned()),
            cursor_output: Some(PathBuf::from("next.toml")),
        }
    );

    let error = parse_args(
        [
            "debug-resume".to_owned(),
            "target/demo".to_owned(),
            "--break-at".to_owned(),
            "1".to_owned(),
            "--break-phase".to_owned(),
            "device-dispatch".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap_err();
    assert!(error.contains("mutually exclusive"));
}

#[test]
fn parses_artifact_doctor_json_with_output_dir() {
    let command = parse_args(
        [
            "artifact-doctor".to_owned(),
            "--json".to_owned(),
            "target/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("artifact-doctor parses");
    assert_eq!(
        command,
        CommandKind::ArtifactDoctor {
            input: PathBuf::from("target/demo"),
            json: true,
        }
    );
}

#[test]
fn parses_build_report_json_with_output_dir() {
    let command = parse_args(
        [
            "build-report".to_owned(),
            "--json".to_owned(),
            "target/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build-report parses");
    assert_eq!(
        command,
        CommandKind::BuildReport {
            input: PathBuf::from("target/demo"),
            json: true,
        }
    );
}

#[test]
fn parses_release_check_json_with_target_options() {
    let command = parse_args(
        [
            "release-check".to_owned(),
            "--json".to_owned(),
            "--cpu-abi".to_owned(),
            "cpu.arm64.apple_aapcs64".to_owned(),
            "--target".to_owned(),
            "aarch64-apple-darwin".to_owned(),
            "examples/demo".to_owned(),
            "target/release-check-demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("release-check json parses");
    assert_eq!(
        command,
        CommandKind::ReleaseCheck {
            input: PathBuf::from("examples/demo"),
            output_dir: PathBuf::from("target/release-check-demo"),
            cpu_abi: Some("cpu.arm64.apple_aapcs64".to_owned()),
            target: Some("aarch64-apple-darwin".to_owned()),
            json: true,
        }
    );
}

#[test]
fn parses_unpack_artifact_support_json_with_output_dir() {
    let command = parse_args(
        [
            "unpack-artifact-support".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.compiled.artifact".to_owned(),
            "target/unpacked".to_owned(),
        ]
        .into_iter(),
    )
    .expect("unpack-artifact-support parses");
    assert_eq!(
        command,
        CommandKind::UnpackArtifactSupport {
            input: PathBuf::from("target/demo/nuis.compiled.artifact"),
            output_dir: PathBuf::from("target/unpacked"),
            json: true,
        }
    );
}
#[test]
fn parses_materialize_artifact_with_output_dir() {
    let command = parse_args(
        [
            "materialize-artifact".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
            "target/materialized".to_owned(),
        ]
        .into_iter(),
    )
    .expect("materialize-artifact parses");
    assert_eq!(
        command,
        CommandKind::MaterializeArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            output_dir: PathBuf::from("target/materialized"),
            json: false,
        }
    );
}
#[test]
fn parses_bench_with_default_input() {
    let command = parse_args(["bench".to_owned()].into_iter()).expect("bench parses");
    assert_eq!(
        command,
        CommandKind::Bench {
            input: PathBuf::from("."),
            list: false,
            json: false,
            exact: false,
            filter: None,
        }
    );
}

#[test]
fn parses_bench_with_list_exact_and_filter() {
    let command = parse_args(
        [
            "bench".to_owned(),
            "--list".to_owned(),
            "--json".to_owned(),
            "--exact".to_owned(),
            "examples/demo.ns".to_owned(),
            "sum_loop".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bench with filter parses");
    assert_eq!(
        command,
        CommandKind::Bench {
            input: PathBuf::from("examples/demo.ns"),
            list: true,
            json: true,
            exact: true,
            filter: Some("sum_loop".to_owned()),
        }
    );
}

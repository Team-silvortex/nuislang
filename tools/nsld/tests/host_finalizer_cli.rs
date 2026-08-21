#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuisc::aot::{
    host_cpu_build_target, write_build_manifest, BuildManifestContext, CompileArtifacts,
    CompileHostObject,
};

#[test]
fn cli_materializes_and_runs_registered_internal_macho_artifact_image() {
    let dir = unique_temp_dir("nsld-cli-internal-macho-finalizer");
    fs::create_dir_all(&dir).unwrap();
    let source_executable = find_path_executable("true");
    let expected_binary = fs::read(&source_executable).unwrap();
    let manifest = write_native_cpu_fixture(&dir, &source_executable);
    let final_binary = dir.join("demo.bin");

    let mut non_executable = fs::metadata(&final_binary).unwrap().permissions();
    non_executable.set_mode(0o644);
    fs::set_permissions(&final_binary, non_executable).unwrap();

    let drive = run_nsld_args(&[
        "drive",
        manifest.to_str().unwrap(),
        "--apply",
        "--until-clean",
        "--json",
    ]);
    let output = run_nsld("final-executable-output", &manifest);
    let invoke_plan = run_nsld("final-executable-host-invoke-plan", &manifest);
    let invoke_plan_text = run_nsld_args(&[
        "final-executable-host-invoke-plan",
        manifest.to_str().unwrap(),
    ]);
    let private_image_probe = run_nsld_args(&[
        "final-executable-private-image-loader-probe",
        manifest.to_str().unwrap(),
        "--json",
    ]);
    let private_image_probe_text = run_nsld_args(&[
        "final-executable-private-image-loader-probe",
        manifest.to_str().unwrap(),
    ]);
    let check = run_nsld("check", &manifest);
    let artifact_chain = run_nsld("artifact-chain", &manifest);
    let launcher = run_nsld("final-executable-launcher-manifest", &manifest);
    let launcher_dry_run = run_nsld("final-executable-launcher-dry-run", &manifest);

    let actual_binary = fs::read(&final_binary).unwrap();
    let persisted_invoke_plan =
        fs::read_to_string(dir.join("nuis.nsld.final-executable-host-invoke-plan.toml")).unwrap();
    let executable_mode = fs::metadata(&final_binary).unwrap().permissions().mode();
    let launched = Command::new(&final_binary).output().unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(actual_binary, expected_binary);
    assert_ne!(executable_mode & 0o111, 0);
    assert!(launched.status.success());
    assert!(invoke_plan
        .contains("\"finalizer_contract\":\"nuis-nsld-executable-finalizer-registry-v1\""));
    assert!(invoke_plan
        .contains("\"finalizer_provider_id\":\"nsld.finalizer.mach-o.arm64.artifact-image-v1\""));
    assert!(invoke_plan
        .contains("\"finalizer_execution_kind\":\"registered-nsld-artifact-image-writer\""));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-host-object-linkage-v1\""));
    assert!(invoke_plan.contains("\"relocation_count\":2"));
    assert!(invoke_plan.contains("\"internally_resolved_symbol_count\":1"));
    assert!(invoke_plan.contains("\"unresolved_external_symbols\":[\"_puts\"]"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-placement-binding-v2\""));
    assert!(
        invoke_plan.contains("\"status\":\"placement-ready-with-external-compatibility-boundary\"")
    );
    assert!(invoke_plan.contains("\"merged_section_count\":1"));
    assert!(invoke_plan.contains("\"section_placement_count\":2"));
    assert!(invoke_plan.contains("\"symbol_binding_count\":2"));
    assert!(invoke_plan.contains("\"internally_bound_symbol_count\":1"));
    assert!(invoke_plan.contains("\"external_compatibility_symbol_count\":1"));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-relocation-application-v1\"")
    );
    assert!(invoke_plan.contains("\"status\":\"planned-with-platform-structure-boundary\""));
    assert!(invoke_plan.contains("\"relocation_count\":2"));
    assert!(invoke_plan.contains("\"registered_kind_count\":1"));
    assert!(invoke_plan.contains("\"ready_application_count\":1"));
    assert!(invoke_plan.contains("\"platform_structure_count\":1"));
    assert!(invoke_plan.contains("\"relocation_kind\":\"arm64-branch26\""));
    assert!(invoke_plan.contains("\"action_kind\":\"rewrite-branch26\""));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-materialization-preview-v1\"")
    );
    assert!(invoke_plan.contains("\"status\":\"preview-ready-with-platform-structure-boundary\""));
    assert!(invoke_plan.contains("\"image_span_bytes\":16"));
    assert!(invoke_plan.contains("\"previewed_patch_count\":1"));
    assert!(invoke_plan.contains("\"deferred_patch_count\":1"));
    assert!(invoke_plan.contains("\"source_bytes_hex\":\"00000094\""));
    assert!(invoke_plan.contains("\"encoded_bytes_hex\":\"02000094\""));
    assert!(invoke_plan.contains("\"source_output_offset\":0"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-patch-application-v1\""));
    assert!(invoke_plan
        .contains("\"status\":\"direct-patches-applied-with-platform-structure-boundary\""));
    assert!(invoke_plan.contains("\"expected_patch_count\":1"));
    assert!(invoke_plan.contains("\"applied_patch_count\":1"));
    assert!(invoke_plan.contains("\"write_once_span_count\":1"));
    assert!(invoke_plan.contains("\"post_write_bytes_hash\":"));
    assert!(invoke_plan.contains("\"application_ledger_hash\":"));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-platform-structure-plan-v1\"")
    );
    assert!(invoke_plan.contains("\"status\":\"allocated-ready-for-platform-patching\""));
    assert!(invoke_plan.contains("\"deferred_relocation_count\":1"));
    assert!(invoke_plan.contains("\"target_count\":1"));
    assert!(invoke_plan.contains("\"base_image_span_bytes\":16"));
    assert!(invoke_plan.contains("\"planned_image_span_bytes\":40"));
    assert!(invoke_plan.contains("\"stub_region_offset\":16"));
    assert!(invoke_plan.contains("\"stub_entry_count\":1"));
    assert!(invoke_plan.contains("\"got_region_offset\":32"));
    assert!(invoke_plan.contains("\"got_entry_count\":1"));
    assert!(invoke_plan.contains("\"target_symbol\":\"_puts\""));
    assert!(invoke_plan.contains("\"got_output_offset\":32"));
    assert!(invoke_plan.contains("\"stub_output_offset\":16"));
    assert!(invoke_plan.contains("\"patch_target_kind\":\"branch-stub\""));
    assert!(invoke_plan.contains("\"patch_target_output_offset\":16"));
    assert!(invoke_plan
        .contains("\"contract\":\"nuis-nsld-macho-arm64-platform-patch-application-v1\""));
    assert!(invoke_plan.contains("\"status\":\"platform-patches-applied-with-unresolved-binds\""));
    assert!(invoke_plan.contains("\"platform_image_span_bytes\":40"));
    assert!(invoke_plan.contains("\"expected_deferred_patch_count\":1"));
    assert!(invoke_plan.contains("\"applied_deferred_patch_count\":1"));
    assert!(invoke_plan.contains("\"stub_write_count\":1"));
    assert!(invoke_plan.contains("\"got_write_count\":1"));
    assert!(invoke_plan.contains("\"unresolved_bind_count\":1"));
    assert!(invoke_plan.contains("\"write_once_span_count\":3"));
    assert!(invoke_plan.contains("\"write_kind\":\"arm64-branch-stub\""));
    assert!(invoke_plan.contains("\"encoded_bytes_hex\":\"10000090101240f900021fd6\""));
    assert!(invoke_plan.contains("\"source_output_offset\":8"));
    assert!(invoke_plan.contains("\"patch_target_output_offset\":16"));
    assert!(invoke_plan.contains("\"status\":\"unresolved-external\""));
    assert!(invoke_plan.contains("\"got_output_offset\":32"));
    assert!(invoke_plan.contains("\"shell_layout_plan\":{"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-shell-layout-plan-v1\""));
    assert!(invoke_plan.contains("\"status\":\"layout-planned-with-code-signature-boundary\""));
    assert!(invoke_plan.contains("\"entry_symbol\":\"_nuis_entry\""));
    assert!(invoke_plan.contains("\"segment_name\":\"__LINKEDIT\""));
    assert!(invoke_plan.contains("\"section_name\":\"__stubs\""));
    assert!(invoke_plan.contains("\"section_name\":\"__got\""));
    assert!(invoke_plan.contains("\"record_kind\":\"external-undefined\""));
    assert!(invoke_plan.contains("\"target_symbol\":\"_puts\",\"dylib_ordinal\":1"));
    assert!(invoke_plan.contains("\"command_kind\":\"code-signature\""));
    assert!(invoke_plan.contains("\"code_signature_status\":\"required-payload-pending\""));
    assert!(invoke_plan.contains("\"shell_image_serialization\":{"));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-shell-image-serialization-v2\"")
    );
    assert!(invoke_plan.contains("\"status\":\"signed-private-image-validated\""));
    assert!(invoke_plan.contains("\"publication_status\":\"private-not-published\""));
    assert!(invoke_plan.contains("\"code_signature_status\":\"ad-hoc-payload-validated\""));
    assert!(invoke_plan.contains("\"code_signature\":{"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-ad-hoc-signature-v1\""));
    assert!(
        invoke_plan.contains("\"validation_status\":\"signed-private-image-structurally-valid\"")
    );
    assert!(invoke_plan.contains("\"publication_eligible\":false"));
    assert!(invoke_plan
        .contains("\"publication_blockers\":[\"independent-os-load-validation-pending\"]"));
    assert!(invoke_plan.contains("\"slots\":[{"));
    assert!(invoke_plan.contains("\"relocation_rewrite_count\":2"));
    assert!(invoke_plan.contains("\"stub_rewrite_count\":1"));
    assert!(invoke_plan.contains("\"got_rewrite_count\":0"));
    assert!(invoke_plan.contains("\"rewrite_kind\":\"relocation-final-address\""));
    assert!(invoke_plan.contains("\"rewrite_kind\":\"stub-final-address\""));
    assert!(invoke_plan.contains("\"target_symbol\":\"_nuis_runtime\""));
    assert!(invoke_plan
        .contains("\"symbol\":\"_nuis_runtime\",\"reference_object_id\":\"host.program-llvm\""));
    assert!(invoke_plan.contains("\"target_object_id\":\"host.runtime-shim\""));
    assert!(
        invoke_plan.contains("\"symbol\":\"_puts\",\"reference_object_id\":\"host.runtime-shim\"")
    );
    assert!(invoke_plan.contains("\"status\":\"external-compatibility\""));
    assert!(invoke_plan_text
        .contains("finalizer_input_placement_contract: nuis-nsld-macho-placement-binding-v2"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_relocation_application_contract: nuis-nsld-macho-arm64-relocation-application-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_relocation_application_count: 2"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_materialization: contract=nuis-nsld-macho-arm64-materialization-preview-v1"
    ));
    assert!(invoke_plan_text.contains(
        "finalizer_input_patch_application: contract=nuis-nsld-macho-arm64-patch-application-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_applied_patch: id="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_platform_structure: contract=nuis-nsld-macho-arm64-platform-structure-plan-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_platform_target: id="));
    assert!(invoke_plan_text.contains("symbol=_puts"));
    assert!(invoke_plan_text.contains("finalizer_input_platform_binding: relocation="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_platform_patch_application: contract=nuis-nsld-macho-arm64-platform-patch-application-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_platform_write: id="));
    assert!(invoke_plan_text.contains("kind=arm64-branch-stub"));
    assert!(invoke_plan_text.contains("finalizer_input_platform_patch: relocation="));
    assert!(invoke_plan_text.contains("finalizer_input_platform_bind: id="));
    assert!(invoke_plan_text.contains("status=unresolved-external"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_layout: contract=nuis-nsld-macho-arm64-shell-layout-plan-v1"
    ));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_entry: rule=arm64.macho.program-entry.v1 symbol=_nuis_entry"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_shell_segment: id="));
    assert!(invoke_plan_text.contains("name=__LINKEDIT"));
    assert!(invoke_plan_text.contains("finalizer_input_shell_bind: id="));
    assert!(invoke_plan_text.contains("finalizer_input_shell_command: id="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_image: contract=nuis-nsld-macho-arm64-shell-image-serialization-v2"
    ));
    assert!(invoke_plan_text.contains("publication=private-not-published"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_image_code_signature: contract=nuis-nsld-macho-arm64-ad-hoc-signature-v1 status=ad-hoc-payload-validated"
    ));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_image_publication_eligibility: contract=nuis-nsld-macho-arm64-publication-eligibility-v1"
    ));
    assert!(
        invoke_plan_text.contains("eligible=false blockers=independent-os-load-validation-pending")
    );
    assert!(invoke_plan_text.contains("finalizer_input_shell_image_signature_slot: index=0"));
    assert!(invoke_plan_text.contains("finalizer_input_shell_image_rewrite: id="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_symbol_binding: symbol=_nuis_runtime reference=host.program-llvm:1 status=internal"
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_placement_contract = \"nuis-nsld-macho-placement-binding-v2\""));
    assert!(persisted_invoke_plan.contains("finalizer_input_merged_section_count = 1"));
    assert!(persisted_invoke_plan.contains("finalizer_input_section_placement_count = 2"));
    assert!(persisted_invoke_plan.contains("finalizer_input_symbol_binding_count = 2"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_relocation_application_contract = \"nuis-nsld-macho-arm64-relocation-application-v1\""
    ));
    assert!(persisted_invoke_plan.contains("finalizer_input_relocation_application_count = 2"));
    assert!(persisted_invoke_plan.contains("arm64-branch26|rewrite-branch26"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_materialization_contract = \"nuis-nsld-macho-arm64-materialization-preview-v1\""
    ));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_materialization_previewed_patch_count = 1")
    );
    assert!(persisted_invoke_plan.contains("00000094|02000094"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_patch_application_contract = \"nuis-nsld-macho-arm64-patch-application-v1\""
    ));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_patch_application_applied_patch_count = 1")
    );
    assert!(persisted_invoke_plan.contains("finalizer_input_patch_application_patches = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_platform_structure_contract = \"nuis-nsld-macho-arm64-platform-structure-plan-v1\""
    ));
    assert!(persisted_invoke_plan.contains("finalizer_input_platform_structure_target_count = 1"));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_platform_structure_stub_entry_count = 1")
    );
    assert!(
        persisted_invoke_plan.contains("finalizer_input_platform_structure_got_entry_count = 1")
    );
    assert!(persisted_invoke_plan.contains("finalizer_input_platform_structure_targets = ["));
    assert!(persisted_invoke_plan.contains("finalizer_input_platform_structure_bindings = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_platform_patch_application_contract = \"nuis-nsld-macho-arm64-platform-patch-application-v1\""
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_platform_image_span_bytes = 40"));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_applied_deferred_patch_count = 1"));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_unresolved_bind_count = 1"));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_structure_writes = ["));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_platform_patch_application_patches = [")
    );
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_bind_records = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_layout_contract = \"nuis-nsld-macho-arm64-shell-layout-plan-v1\""
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_shell_layout_entry_symbol = \"_nuis_entry\""));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_segments = ["));
    assert!(persisted_invoke_plan.contains("|__LINKEDIT|"));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_sections = ["));
    assert!(persisted_invoke_plan.contains("|__stubs|"));
    assert!(persisted_invoke_plan.contains("|__got|"));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_binds = ["));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_load_commands = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_image_contract = \"nuis-nsld-macho-arm64-shell-image-serialization-v2\""
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_shell_image_publication_status = \"private-not-published\""));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_image_code_signature_status = \"ad-hoc-payload-validated\""
    ));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_image_signature_contract = \"nuis-nsld-macho-arm64-ad-hoc-signature-v1\""
    ));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_image_signature_validation_status = \"signed-private-image-structurally-valid\""
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_shell_image_signature_publication_eligible = false"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_image_signature_publication_blockers = [\"independent-os-load-validation-pending\"]"
    ));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_image_signature_slots = ["));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_image_rewrite_count = 3"));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_image_rewrites = ["));
    assert!(
        private_image_probe.contains("\"contract\":\"nuis-nsld-macho-arm64-os-loader-probe-v1\"")
    );
    assert!(private_image_probe.contains("\"status\":\"blocked-external-compatibility-input\""));
    assert!(private_image_probe.contains("\"probe_mode\":\"plan-only\""));
    assert!(private_image_probe.contains("\"input_eligible\":false"));
    assert!(private_image_probe.contains("\"attempted\":false"));
    assert!(private_image_probe.contains("\"publication_eligible\":false"));
    assert!(private_image_probe.contains(
        "\"publication_blockers\":[\"private-image-has-external-compatibility-bindings\"]"
    ));
    assert!(private_image_probe_text.contains("Nsld Mach-O arm64 private-image loader probe"));
    assert!(private_image_probe_text.contains("status=blocked-external-compatibility-input"));
    assert!(private_image_probe_text.contains("attempted=false"));
    assert!(invoke_plan.contains("\"invocation_kind\":\"registered-internal-finalizer\""));
    assert!(invoke_plan.contains("\"invocation_policy\":\"registered-internal\""));
    assert!(invoke_plan.contains("\"requires_explicit_allow\":false"));
    assert!(invoke_plan.contains("\"explicit_allow_present\":false"));
    assert!(invoke_plan.contains("\"would_invoke\":true"));
    assert!(drive.contains("\"kind\":\"nsld_drive_until_clean\""));
    assert!(drive.contains("\"completed\":true"), "{drive}");
    assert!(drive.contains("\"stop_reason\":\"clean\""), "{drive}");
    assert!(output.contains("\"output_kind\":\"host-native-executable\""));
    assert!(output.contains("\"present\":true"), "{output}");
    assert!(output.contains("\"nsld_owned_output\":true"), "{output}");
    assert!(output.contains("\"runnable_candidate\":true"), "{output}");
    assert!(check.contains("\"final_executable_output_present\":true"));
    assert!(check.contains("\"final_executable_output_runnable_candidate\":true"));
    assert!(artifact_chain.contains("\"stage_id\":\"final-executable-output\""));
    assert!(artifact_chain.contains("\"present\":true"));
    assert!(launcher.contains("\"final_output_present\":true"));
    assert!(launcher.contains("\"ready\":true"));
    assert!(launcher_dry_run.contains("\"final_output_readable\":true"));
    assert!(launcher_dry_run.contains("\"dry_run_ready\":true"));
}

#[test]
fn cli_persists_and_replays_internal_private_image_admission_receipt() {
    let dir = unique_temp_dir("nsld-cli-private-image-admission");
    fs::create_dir_all(&dir).unwrap();
    let source_executable = find_path_executable("true");
    let manifest = write_internal_native_cpu_fixture(&dir, &source_executable);
    let compatibility_before = fs::read(dir.join("demo.bin")).unwrap();
    let receipt_path = dir.join("nuis.nsld.macho-arm64-publication-admission.toml");
    let invoke_plan = run_nsld("final-executable-host-invoke-plan", &manifest);

    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-placement-binding-v2\""));
    assert!(invoke_plan.contains("\"common_allocation_count\":1"));
    assert!(invoke_plan.contains("\"section_name\":\"__nuis_common\""));
    assert!(invoke_plan.contains("\"symbol\":\"_nuis_state\""));
    assert!(invoke_plan.contains("\"declaration_count\":1"));
    assert!(invoke_plan.contains("\"size_bytes\":8,\"alignment\":8"));
    assert!(invoke_plan.contains("\"relocation_count\":3"));

    let planned = run_nsld("final-executable-private-image-loader-probe", &manifest);
    assert!(planned.contains("\"probe_mode\":\"plan-only\""));
    assert!(planned.contains("\"admission_receipt_persisted\":false"));
    assert!(planned.contains("\"admission_receipt_validation_status\":\"not-requested\""));
    assert!(!receipt_path.exists());

    let probe = run_nsld_args(&[
        "final-executable-private-image-loader-probe",
        manifest.to_str().unwrap(),
        "--apply",
        "--json",
    ]);
    let receipt = fs::read_to_string(&receipt_path).unwrap();
    let receipt_mode = fs::metadata(&receipt_path).unwrap().permissions().mode() & 0o777;
    let verification = run_nsld("verify-final-executable-private-image-admission", &manifest);
    let verification_text = run_nsld_args(&[
        "verify-final-executable-private-image-admission",
        manifest.to_str().unwrap(),
    ]);
    let compatibility_after = fs::read(dir.join("demo.bin")).unwrap();

    assert!(probe.contains("\"publication_eligible\":true"), "{probe}");
    assert!(
        probe.contains("\"admission_receipt_persisted\":true"),
        "{probe}"
    );
    assert!(probe.contains(
        "\"admission_receipt_file\":\"nuis.nsld.macho-arm64-publication-admission.toml\""
    ));
    assert!(probe.contains(
        "\"admission_receipt_validation_status\":\"publication-admission-replay-verified\""
    ));
    assert!(receipt.contains("contract = \"nuis-nsld-macho-arm64-publication-admission-v1\""));
    assert!(receipt.contains("probe_kernel_accepted = true"));
    assert!(receipt.contains("unresolved_external_symbol_count = 0"));
    assert!(receipt.contains("bind_count = 0"));
    assert!(receipt.contains("shell_image_sha256 = \""));
    assert!(receipt.contains("receipt_hash_sha256 = \""));
    assert!(!receipt.contains(dir.to_str().unwrap()));
    assert_eq!(receipt_mode, 0o600);
    assert!(verification.contains("\"valid\":true"), "{verification}");
    assert!(verification.contains("\"receipt_hash_matches\":true"));
    assert!(verification.contains("\"private_image_matches\":true"));
    assert!(verification.contains("\"probe_evidence_valid\":true"));
    assert!(verification_text.contains("probe=true valid=true"));
    assert_eq!(compatibility_after, compatibility_before);

    let publication_plan = run_nsld("final-executable-private-image-publication", &manifest);
    let publication_plan_text = run_nsld_args(&[
        "final-executable-private-image-publication",
        manifest.to_str().unwrap(),
    ]);
    assert!(publication_plan
        .contains("\"contract\":\"nuis-nsld-registered-private-image-publication-v1\""));
    assert!(publication_plan.contains("\"status\":\"ready-private-image-publication-plan\""));
    assert!(publication_plan.contains("\"apply_requested\":false"));
    assert!(publication_plan.contains("\"publication_ready\":true"));
    assert!(publication_plan.contains("\"installation_attempted\":false"));
    assert!(publication_plan.contains("\"installed\":false"));
    assert!(publication_plan.contains("\"output_changed\":false"));
    assert!(publication_plan_text.contains("Nsld registered private-image publication"));
    assert!(publication_plan_text.contains("attempted=false installed=false"));
    assert_eq!(
        fs::read(dir.join("demo.bin")).unwrap(),
        compatibility_before
    );

    let damaged = receipt.replace(
        "probe_kernel_accepted = true",
        "probe_kernel_accepted = false",
    );
    fs::write(&receipt_path, damaged).unwrap();
    let rejected = run_nsld_failure(&[
        "verify-final-executable-private-image-admission",
        manifest.to_str().unwrap(),
        "--json",
    ]);
    let rejected_stdout = String::from_utf8(rejected.stdout).unwrap();
    assert!(rejected_stdout.contains("\"valid\":false"));
    assert!(rejected_stdout.contains("receipt-hash-mismatch"));
    assert!(rejected_stdout.contains("loader-probe-evidence-invalid"));
    let rejected_publication = run_nsld_failure(&[
        "final-executable-private-image-publication",
        manifest.to_str().unwrap(),
        "--apply",
        "--json",
    ]);
    let rejected_publication_stdout = String::from_utf8(rejected_publication.stdout).unwrap();
    assert!(rejected_publication_stdout
        .contains("\"status\":\"blocked-publication-admission-invalid\""));
    assert!(rejected_publication_stdout.contains("\"installation_attempted\":false"));
    assert!(rejected_publication_stdout.contains("\"installed\":false"));
    assert!(rejected_publication_stdout.contains("\"output_changed\":false"));
    assert!(rejected_publication_stdout.contains("publication-admission:receipt-hash-mismatch"));
    assert_eq!(
        fs::read(dir.join("demo.bin")).unwrap(),
        compatibility_before
    );

    fs::write(&receipt_path, &receipt).unwrap();
    let publication = run_nsld_args(&[
        "final-executable-private-image-publication",
        manifest.to_str().unwrap(),
        "--apply",
        "--json",
    ]);
    let private_image = fs::read(dir.join("demo.bin")).unwrap();
    let private_image_mode = fs::metadata(dir.join("demo.bin"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    let private_execution = Command::new(dir.join("demo.bin")).output().unwrap();
    assert!(publication.contains("\"status\":\"private-image-published\""));
    assert!(publication.contains(
        "\"capability_id\":\"nsld.finalizer.mach-o.arm64.private-image-publication-v1\""
    ));
    assert!(publication.contains("\"apply_requested\":true"));
    assert!(publication.contains("\"installation_attempted\":true"));
    assert!(publication.contains("\"installed\":true"));
    assert!(publication.contains("\"output_matches_private_image\":true"));
    assert!(publication.contains("\"output_executable\":true"));
    assert!(publication.contains("\"output_changed\":true"));
    assert_ne!(private_image, compatibility_before);
    assert_eq!(private_image_mode, 0o700);
    assert!(private_execution.status.success());
    assert!(private_execution.stdout.is_empty());
    assert!(private_execution.stderr.is_empty());

    let rebuilt_manifest = write_internal_native_cpu_fixture_returning(&dir, &source_executable, 1);
    assert_eq!(rebuilt_manifest, manifest);
    let drifted = run_nsld_failure(&[
        "verify-final-executable-private-image-admission",
        manifest.to_str().unwrap(),
        "--json",
    ]);
    let drifted_stdout = String::from_utf8(drifted.stdout).unwrap();
    assert!(drifted_stdout.contains("\"receipt_hash_matches\":true"));
    assert!(drifted_stdout.contains("\"private_image_matches\":false"));
    assert!(drifted_stdout.contains("\"signature_identity_matches\":false"));
    assert!(drifted_stdout.contains("private-image-identity-mismatch"));

    fs::remove_dir_all(dir).unwrap();
}

fn run_nsld(command: &str, manifest: &Path) -> String {
    run_nsld_args(&[command, manifest.to_str().unwrap(), "--json"])
}

fn run_nsld_args(args: &[&str]) -> String {
    let command_label = args.join(" ");
    let output = Command::new(env!("CARGO_BIN_EXE_nsld"))
        .args(args)
        .env_remove("NUIS_NSLD_HOST_FINALIZER_POLICY")
        .env_remove("NUIS_NSLD_ALLOW_HOST_FINALIZER")
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsld {command_label}: {error}"));
    if !output.status.success() {
        panic!(
            "nsld {command_label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).unwrap()
}

fn run_nsld_failure(args: &[&str]) -> std::process::Output {
    let command_label = args.join(" ");
    let output = Command::new(env!("CARGO_BIN_EXE_nsld"))
        .args(args)
        .env_remove("NUIS_NSLD_HOST_FINALIZER_POLICY")
        .env_remove("NUIS_NSLD_ALLOW_HOST_FINALIZER")
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsld {command_label}: {error}"));
    assert!(
        !output.status.success(),
        "nsld {command_label} unexpectedly succeeded"
    );
    output
}

fn write_native_cpu_fixture(dir: &Path, source_executable: &Path) -> PathBuf {
    write_native_cpu_fixture_with_objects(
        dir,
        source_executable,
        arm64_object("_nuis_entry", "_nuis_runtime"),
        arm64_object("_nuis_runtime", "_puts"),
    )
}

fn write_internal_native_cpu_fixture(dir: &Path, source_executable: &Path) -> PathBuf {
    write_internal_native_cpu_fixture_returning(dir, source_executable, 0)
}

fn write_internal_native_cpu_fixture_returning(
    dir: &Path,
    source_executable: &Path,
    return_value: u16,
) -> PathBuf {
    write_native_cpu_fixture_with_objects(
        dir,
        source_executable,
        arm64_tail_branch_object("_nuis_entry", "_nuis_runtime"),
        arm64_common_leaf_object_returning("_nuis_runtime", "_nuis_state", return_value),
    )
}

fn write_native_cpu_fixture_with_objects(
    dir: &Path,
    source_executable: &Path,
    program_bytes: Vec<u8>,
    runtime_bytes: Vec<u8>,
) -> PathBuf {
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    let source = dir.join("demo.ns");
    let program_object = dir.join("demo.host-program.o");
    let runtime_object = dir.join("demo.host-runtime.o");
    fs::write(&source, "fn main() -> i64 { 0 }\n").unwrap();
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&program_object, program_bytes).unwrap();
    fs::write(&runtime_object, runtime_bytes).unwrap();
    fs::copy(source_executable, &bin).unwrap();

    let manifest = write_build_manifest(
        dir,
        &CompileArtifacts {
            ast_path: ast.display().to_string(),
            nir_path: nir.display().to_string(),
            yir_path: yir.display().to_string(),
            llvm_ir_path: ll.display().to_string(),
            binary_path: bin.display().to_string(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            host_objects: vec![
                CompileHostObject {
                    object_id: "host.program-llvm".to_owned(),
                    role: "program-llvm".to_owned(),
                    path: program_object.display().to_string(),
                },
                CompileHostObject {
                    object_id: "host.runtime-shim".to_owned(),
                    role: "runtime-shim".to_owned(),
                    path: runtime_object.display().to_string(),
                },
            ],
        },
        &BuildManifestContext {
            input_path: source.display().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target: host_cpu_build_target(),
        },
    )
    .unwrap();
    PathBuf::from(manifest)
}

fn arm64_object(defined: &str, undefined: &str) -> Vec<u8> {
    arm64_branch_object(defined, undefined, 0x9400_0000, 0)
}

fn arm64_tail_branch_object(defined: &str, undefined: &str) -> Vec<u8> {
    arm64_branch_object(defined, undefined, 0x1400_0000, 0xd503_201f)
}

fn arm64_branch_object(
    defined: &str,
    undefined: &str,
    branch_instruction: u32,
    trailing_instruction: u32,
) -> Vec<u8> {
    const SEGMENT_OFFSET: usize = 32;
    const SECTION_OFFSET: usize = 104;
    const SYMTAB_OFFSET: usize = 184;
    const PAYLOAD_OFFSET: usize = 208;
    const RELOCATION_OFFSET: usize = 216;
    const SYMBOL_OFFSET: usize = 224;
    const STRING_OFFSET: usize = 256;
    let mut strings = vec![0];
    let defined_index = strings.len() as u32;
    strings.extend_from_slice(defined.as_bytes());
    strings.push(0);
    let undefined_index = strings.len() as u32;
    strings.extend_from_slice(undefined.as_bytes());
    strings.push(0);
    let mut bytes = vec![0u8; STRING_OFFSET + strings.len()];
    bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
    bytes[4..8].copy_from_slice(&0x0100_000cu32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    write_u32(&mut bytes, 16, 2);
    write_u32(&mut bytes, 20, 176);
    write_u32(&mut bytes, SEGMENT_OFFSET, 0x19);
    write_u32(&mut bytes, SEGMENT_OFFSET + 4, 152);
    write_u32(&mut bytes, SEGMENT_OFFSET + 64, 1);
    bytes[SECTION_OFFSET..SECTION_OFFSET + 6].copy_from_slice(b"__text");
    bytes[SECTION_OFFSET + 16..SECTION_OFFSET + 22].copy_from_slice(b"__TEXT");
    write_u64(&mut bytes, SECTION_OFFSET + 40, 8);
    write_u32(&mut bytes, SECTION_OFFSET + 48, PAYLOAD_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 56, RELOCATION_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 60, 1);
    write_u32(&mut bytes, PAYLOAD_OFFSET, branch_instruction);
    write_u32(&mut bytes, PAYLOAD_OFFSET + 4, trailing_instruction);
    write_u32(&mut bytes, SYMTAB_OFFSET, 0x2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 4, 24);
    write_u32(&mut bytes, SYMTAB_OFFSET + 8, SYMBOL_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 12, 2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 16, STRING_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 20, strings.len() as u32);
    write_u32(&mut bytes, RELOCATION_OFFSET, 0);
    write_u32(
        &mut bytes,
        RELOCATION_OFFSET + 4,
        1 | (1 << 24) | (2 << 25) | (1 << 27) | (2 << 28),
    );
    write_u32(&mut bytes, SYMBOL_OFFSET, defined_index);
    bytes[SYMBOL_OFFSET + 4] = 0x0f;
    bytes[SYMBOL_OFFSET + 5] = 1;
    write_u32(&mut bytes, SYMBOL_OFFSET + 16, undefined_index);
    bytes[SYMBOL_OFFSET + 20] = 0x01;
    bytes[STRING_OFFSET..].copy_from_slice(&strings);
    bytes
}

fn arm64_common_leaf_object_returning(defined: &str, common: &str, return_value: u16) -> Vec<u8> {
    const SEGMENT_OFFSET: usize = 32;
    const SECTION_OFFSET: usize = 104;
    const SYMTAB_OFFSET: usize = 184;
    const PAYLOAD_OFFSET: usize = 208;
    const RELOCATION_OFFSET: usize = 228;
    const SYMBOL_OFFSET: usize = 244;
    const STRING_OFFSET: usize = 276;
    let mut strings = vec![0];
    let defined_index = strings.len() as u32;
    strings.extend_from_slice(defined.as_bytes());
    strings.push(0);
    let common_index = strings.len() as u32;
    strings.extend_from_slice(common.as_bytes());
    strings.push(0);
    let mut bytes = vec![0u8; STRING_OFFSET + strings.len()];
    bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
    bytes[4..8].copy_from_slice(&0x0100_000cu32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    write_u32(&mut bytes, 16, 2);
    write_u32(&mut bytes, 20, 176);
    write_u32(&mut bytes, SEGMENT_OFFSET, 0x19);
    write_u32(&mut bytes, SEGMENT_OFFSET + 4, 152);
    write_u32(&mut bytes, SEGMENT_OFFSET + 64, 1);
    bytes[SECTION_OFFSET..SECTION_OFFSET + 6].copy_from_slice(b"__text");
    bytes[SECTION_OFFSET + 16..SECTION_OFFSET + 22].copy_from_slice(b"__TEXT");
    write_u64(&mut bytes, SECTION_OFFSET + 40, 20);
    write_u32(&mut bytes, SECTION_OFFSET + 48, PAYLOAD_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 52, 2);
    write_u32(&mut bytes, SECTION_OFFSET + 56, RELOCATION_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 60, 2);
    write_u32(&mut bytes, PAYLOAD_OFFSET, 0x9000_0008);
    write_u32(&mut bytes, PAYLOAD_OFFSET + 4, 0x9100_0108);
    write_u32(&mut bytes, PAYLOAD_OFFSET + 8, 0xf900_011f);
    write_u32(
        &mut bytes,
        PAYLOAD_OFFSET + 12,
        0x5280_0000 | (u32::from(return_value) << 5),
    );
    write_u32(&mut bytes, PAYLOAD_OFFSET + 16, 0xd65f_03c0);
    write_u32(&mut bytes, SYMTAB_OFFSET, 0x2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 4, 24);
    write_u32(&mut bytes, SYMTAB_OFFSET + 8, SYMBOL_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 12, 2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 16, STRING_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 20, strings.len() as u32);
    write_u32(&mut bytes, SYMBOL_OFFSET, defined_index);
    bytes[SYMBOL_OFFSET + 4] = 0x0f;
    bytes[SYMBOL_OFFSET + 5] = 1;
    write_u32(&mut bytes, RELOCATION_OFFSET, 0);
    write_u32(
        &mut bytes,
        RELOCATION_OFFSET + 4,
        1 | (1 << 24) | (2 << 25) | (1 << 27) | (3 << 28),
    );
    write_u32(&mut bytes, RELOCATION_OFFSET + 8, 4);
    write_u32(
        &mut bytes,
        RELOCATION_OFFSET + 12,
        1 | (2 << 25) | (1 << 27) | (4 << 28),
    );
    write_u32(&mut bytes, SYMBOL_OFFSET + 16, common_index);
    bytes[SYMBOL_OFFSET + 20] = 0x01;
    bytes[SYMBOL_OFFSET + 22..SYMBOL_OFFSET + 24].copy_from_slice(&(3u16 << 8).to_le_bytes());
    write_u64(&mut bytes, SYMBOL_OFFSET + 24, 8);
    bytes[STRING_OFFSET..].copy_from_slice(&strings);
    bytes
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}

fn find_path_executable(name: &str) -> PathBuf {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("required test executable `{name}` was not found on PATH"))
}

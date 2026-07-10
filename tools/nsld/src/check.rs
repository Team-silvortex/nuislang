use super::{
    artifact_chain::{
        nsld_artifact_chain_issues, nsld_artifact_stage_kind_path,
        nsld_artifact_stage_kind_path_for_plan, nsld_artifact_stages_for_plan,
        NsldArtifactStageKind,
    },
    assembly::{
        nsld_verify_assemble_plan_report, nsld_verify_link_bundle_report,
        nsld_verify_section_manifest_report,
    },
    check_container::{nsld_check_container_snapshot, push_container_snapshot_issues},
    check_core::{nsld_check_core_snapshot, push_optional_check_failure},
    check_final::{nsld_check_final_snapshot, push_final_snapshot_issues},
    link_units::{nsld_verify_link_inputs_report, nsld_verify_link_units_report},
    object_byte_layout::nsld_verify_object_byte_layout_report,
    object_emit::nsld_verify_object_emit_report,
    object_file_layout::nsld_verify_object_file_layout_report,
    object_image_dry_run::nsld_verify_object_image_dry_run_report,
    object_output::nsld_verify_object_output_report,
    object_plan::nsld_verify_object_plan_report,
    object_writer_input::{
        nsld_verify_object_writer_dry_run_report, nsld_verify_object_writer_input_report,
    },
    reports::NsldCheckReport,
};
use std::path::Path;
pub(crate) fn nsld_check_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldCheckReport {
    let core_snapshot = nsld_check_core_snapshot(plan);
    let link_input_table_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::LinkInputs);
    let link_input_table_present = link_input_table_path.exists();
    let link_input_verify_report =
        link_input_table_present.then(|| nsld_verify_link_inputs_report(manifest, plan));
    let link_input_table_valid = link_input_verify_report.as_ref().map(|report| report.valid);
    let link_input_table_issues = link_input_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let link_unit_table_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::LinkUnits);
    let link_unit_table_present = link_unit_table_path.exists();
    let link_unit_verify_report =
        link_unit_table_present.then(|| nsld_verify_link_units_report(manifest, plan));
    let link_unit_table_valid = link_unit_verify_report.as_ref().map(|report| report.valid);
    let link_unit_table_issues = link_unit_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let link_bundle_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::LinkBundle);
    let link_bundle_present = link_bundle_path.exists();
    let link_bundle_verify_report =
        link_bundle_present.then(|| nsld_verify_link_bundle_report(manifest, plan));
    let link_bundle_valid = link_bundle_verify_report
        .as_ref()
        .map(|report| report.valid);
    let link_bundle_issues = link_bundle_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let assemble_plan_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::AssemblePlan);
    let assemble_plan_present = assemble_plan_path.exists();
    let assemble_plan_verify_report =
        assemble_plan_present.then(|| nsld_verify_assemble_plan_report(manifest, plan));
    let assemble_plan_valid = assemble_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let assemble_plan_issues = assemble_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let section_manifest_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::SectionManifest);
    let section_manifest_present = section_manifest_path.exists();
    let section_manifest_verify_report =
        section_manifest_present.then(|| nsld_verify_section_manifest_report(manifest, plan));
    let section_manifest_valid = section_manifest_verify_report
        .as_ref()
        .map(|report| report.valid);
    let section_manifest_issues = section_manifest_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_plan_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectPlan);
    let object_plan_present = object_plan_path.exists();
    let object_plan_verify_report =
        object_plan_present.then(|| nsld_verify_object_plan_report(manifest, plan));
    let object_plan_valid = object_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_plan_issues = object_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_writer_input_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectWriterInput);
    let object_writer_input_present = object_writer_input_path.exists();
    let object_writer_input_verify_report =
        object_writer_input_present.then(|| nsld_verify_object_writer_input_report(manifest, plan));
    let object_writer_input_valid = object_writer_input_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_writer_input_issues = object_writer_input_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_byte_layout_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectByteLayout);
    let object_byte_layout_present = object_byte_layout_path.exists();
    let object_byte_layout_verify_report =
        object_byte_layout_present.then(|| nsld_verify_object_byte_layout_report(manifest, plan));
    let object_byte_layout_valid = object_byte_layout_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_byte_layout_issues = object_byte_layout_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_file_layout_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectFileLayout);
    let object_file_layout_present = object_file_layout_path.exists();
    let object_file_layout_verify_report =
        object_file_layout_present.then(|| nsld_verify_object_file_layout_report(manifest, plan));
    let object_file_layout_valid = object_file_layout_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_file_layout_issues = object_file_layout_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_image_dry_run_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectImageDryRun);
    let object_image_dry_run_present = object_image_dry_run_path.exists();
    let object_image_dry_run_verify_report = object_image_dry_run_present
        .then(|| nsld_verify_object_image_dry_run_report(manifest, plan));
    let object_image_dry_run_valid = object_image_dry_run_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_image_dry_run_issues = object_image_dry_run_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_image_relocation_lowering_valid = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_valid);
    let object_image_relocation_lowering_rule_count = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_rule_count);
    let object_image_relocation_lowering_rules = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_rules.clone())
        .unwrap_or_default();
    let object_image_relocation_lowering_issues = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_issues.clone())
        .unwrap_or_default();
    let object_image_relocation_record_count = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_record_count);
    let object_image_relocation_record_table_hash = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_record_table_hash.clone());
    let object_image_relocation_records = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_records.clone())
        .unwrap_or_default();
    let object_image_dry_run_bytes_present = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::ObjectImageDryRunBytes,
    )
    .exists();
    let object_emit_blocked_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectEmitBlocked);
    let object_emit_blocked_present = object_emit_blocked_path.exists();
    let object_emit_blocked_verify_report =
        object_emit_blocked_present.then(|| nsld_verify_object_emit_report(manifest, plan));
    let object_emit_blocked_valid = object_emit_blocked_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_emit_blocked_issues = object_emit_blocked_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_output_path =
        nsld_artifact_stage_kind_path_for_plan(plan, NsldArtifactStageKind::ObjectOutput);
    let object_output_present = object_output_path.exists();
    let object_output_verify_report =
        object_output_present.then(|| nsld_verify_object_output_report(manifest, plan));
    let object_output_valid = object_output_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_output_expected_size_bytes = object_output_verify_report
        .as_ref()
        .and_then(|report| report.expected_size_bytes);
    let object_output_actual_size_bytes = object_output_verify_report
        .as_ref()
        .and_then(|report| report.actual_size_bytes);
    let object_output_expected_hash = object_output_verify_report
        .as_ref()
        .and_then(|report| report.expected_hash.clone());
    let object_output_actual_hash = object_output_verify_report
        .as_ref()
        .and_then(|report| report.actual_hash.clone());
    let object_output_issues = if let Some(report) = object_output_verify_report.as_ref() {
        report.issues.clone()
    } else {
        Vec::new()
    };
    let object_writer_dry_run_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectWriterDryRun);
    let object_writer_dry_run_present = object_writer_dry_run_path.exists();
    let object_writer_dry_run_verify_report = object_writer_dry_run_present
        .then(|| nsld_verify_object_writer_dry_run_report(manifest, plan));
    let object_writer_dry_run_valid = object_writer_dry_run_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_writer_dry_run_issues = object_writer_dry_run_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let container_snapshot = nsld_check_container_snapshot(manifest, plan);
    let final_snapshot = nsld_check_final_snapshot(manifest, plan);
    let artifact_chain_issues = nsld_artifact_chain_issues(&nsld_artifact_stages_for_plan(plan));
    let artifact_chain_valid = artifact_chain_issues.is_empty();
    let mut issues = Vec::new();

    if !core_snapshot.artifact_lowering_alignment_consistent {
        issues.push(format!(
            "artifact lowering alignment has {} mismatch(es)",
            core_snapshot.artifact_lowering_alignment_mismatches
        ));
        for check in &plan.artifact_lowering_alignment.checks {
            for issue in &check.issues {
                issues.push(format!(
                    "{}:{}: {}",
                    check.package_id, check.domain_family, issue
                ));
            }
        }
    }
    if !core_snapshot.clock_protocol_valid {
        issues.push("clock protocol validation failed".to_owned());
        issues.extend(core_snapshot.clock_protocol_issues.iter().cloned());
    }
    if !core_snapshot.hetero_calculate_valid {
        issues.push("hetero calculate validation failed".to_owned());
        issues.extend(core_snapshot.hetero_calculate_issues.iter().cloned());
    }
    if !core_snapshot.static_link {
        issues.push("hetero calculate plan is not static-link".to_owned());
    }
    if !core_snapshot.lifecycle_driven {
        issues.push("hetero calculate plan is not lifecycle-driven".to_owned());
    }
    if !core_snapshot.sidecar_capability_valid {
        issues.push("sidecar capability validation failed".to_owned());
        issues.extend(core_snapshot.sidecar_capability_issues.iter().cloned());
    }
    macro_rules! push_failure {
        ($valid:expr, $headline:literal, $details:expr) => {
            push_optional_check_failure(&mut issues, $valid, $headline, $details);
        };
    }
    push_failure!(
        link_input_table_valid,
        "link input table verification failed",
        &link_input_table_issues
    );
    push_failure!(
        link_unit_table_valid,
        "link unit table verification failed",
        &link_unit_table_issues
    );
    push_failure!(
        link_bundle_valid,
        "link bundle verification failed",
        &link_bundle_issues
    );
    push_failure!(
        assemble_plan_valid,
        "assemble plan verification failed",
        &assemble_plan_issues
    );
    push_failure!(
        section_manifest_valid,
        "section manifest verification failed",
        &section_manifest_issues
    );
    push_failure!(
        object_plan_valid,
        "object plan verification failed",
        &object_plan_issues
    );
    push_failure!(
        object_writer_input_valid,
        "object writer input verification failed",
        &object_writer_input_issues
    );
    push_failure!(
        object_byte_layout_valid,
        "object byte layout verification failed",
        &object_byte_layout_issues
    );
    push_failure!(
        object_file_layout_valid,
        "object file layout verification failed",
        &object_file_layout_issues
    );
    push_failure!(
        object_image_dry_run_valid,
        "object image dry-run verification failed",
        &object_image_dry_run_issues
    );
    push_failure!(
        object_emit_blocked_valid,
        "object emit blocked report verification failed",
        &object_emit_blocked_issues
    );
    push_failure!(
        object_output_valid,
        "object output verification failed",
        &object_output_issues
    );
    push_failure!(
        object_writer_dry_run_valid,
        "object writer dry-run verification failed",
        &object_writer_dry_run_issues
    );
    push_container_snapshot_issues(&mut issues, &container_snapshot);
    push_final_snapshot_issues(&mut issues, &final_snapshot);
    if !artifact_chain_valid {
        issues.push("nsld artifact chain is incomplete".to_owned());
        issues.extend(artifact_chain_issues.iter().cloned());
    }

    let checks = 6 + usize::from(link_input_table_present) + usize::from(link_unit_table_present);
    let checks = checks + usize::from(link_bundle_present);
    let checks = checks + usize::from(assemble_plan_present);
    let checks = checks + usize::from(section_manifest_present);
    let checks = checks + usize::from(object_plan_present);
    let checks = checks + usize::from(object_writer_input_present);
    let checks = checks + usize::from(object_byte_layout_present);
    let checks = checks + usize::from(object_file_layout_present);
    let checks = checks + usize::from(object_image_dry_run_present);
    let checks = checks + usize::from(object_image_dry_run_bytes_present);
    let checks = checks + usize::from(object_emit_blocked_present);
    let checks = checks + usize::from(object_output_present);
    let checks = checks + usize::from(object_writer_dry_run_present);
    let checks = checks + usize::from(container_snapshot.container_plan_present);
    let checks = checks + usize::from(container_snapshot.container_present);
    let checks = checks
        + usize::from(
            container_snapshot.container_present || container_snapshot.container_payload_present,
        );
    let checks = checks + usize::from(container_snapshot.closure_snapshot_present);
    let checks = checks + usize::from(final_snapshot.final_stage_plan_present);
    let checks = checks + usize::from(final_snapshot.final_executable_writer_input_present);
    let checks = checks + usize::from(final_snapshot.final_executable_host_invoke_plan_present);
    let checks = checks + usize::from(final_snapshot.final_executable_layout_plan_present);
    let checks = checks + usize::from(final_snapshot.final_executable_image_dry_run_present);
    let checks = checks + usize::from(final_snapshot.final_executable_blocked_present);
    let checks = checks + usize::from(final_snapshot.final_executable_output_present);
    let failures = issues.len();
    NsldCheckReport {
        manifest: manifest.display().to_string(),
        valid: failures == 0,
        checks,
        failures,
        artifact_lowering_alignment_consistent: core_snapshot
            .artifact_lowering_alignment_consistent,
        artifact_lowering_alignment_mismatches: core_snapshot
            .artifact_lowering_alignment_mismatches,
        clock_protocol_valid: core_snapshot.clock_protocol_valid,
        clock_protocol_issues: core_snapshot.clock_protocol_issues.clone(),
        hetero_calculate_valid: core_snapshot.hetero_calculate_valid,
        hetero_calculate_issues: core_snapshot.hetero_calculate_issues.clone(),
        static_link: core_snapshot.static_link,
        lifecycle_driven: core_snapshot.lifecycle_driven,
        sidecar_capability_valid: core_snapshot.sidecar_capability_valid,
        sidecar_capability_issues: core_snapshot.sidecar_capability_issues.clone(),
        link_input_table_present,
        link_input_table_valid,
        link_input_table_issues,
        link_unit_table_present,
        link_unit_table_valid,
        link_unit_table_issues,
        link_bundle_present,
        link_bundle_valid,
        link_bundle_issues,
        assemble_plan_present,
        assemble_plan_valid,
        assemble_plan_issues,
        section_manifest_present,
        section_manifest_valid,
        section_manifest_issues,
        object_plan_present,
        object_plan_valid,
        object_plan_issues,
        object_writer_input_present,
        object_writer_input_valid,
        object_writer_input_issues,
        object_byte_layout_present,
        object_byte_layout_valid,
        object_byte_layout_issues,
        object_file_layout_present,
        object_file_layout_valid,
        object_file_layout_issues,
        object_image_dry_run_present,
        object_image_dry_run_valid,
        object_image_dry_run_issues,
        object_image_relocation_lowering_valid,
        object_image_relocation_lowering_rule_count,
        object_image_relocation_lowering_rules,
        object_image_relocation_lowering_issues,
        object_image_relocation_record_count,
        object_image_relocation_record_table_hash,
        object_image_relocation_records,
        object_image_dry_run_bytes_present,
        object_emit_blocked_present,
        object_emit_blocked_valid,
        object_emit_blocked_issues,
        object_output_present,
        object_output_valid,
        object_output_expected_size_bytes,
        object_output_actual_size_bytes,
        object_output_expected_hash,
        object_output_actual_hash,
        object_output_issues,
        object_writer_dry_run_present,
        object_writer_dry_run_valid,
        object_writer_dry_run_issues,
        container_plan_present: container_snapshot.container_plan_present,
        container_plan_valid: container_snapshot.container_plan_valid,
        container_plan_issues: container_snapshot.container_plan_issues,
        container_present: container_snapshot.container_present,
        container_valid: container_snapshot.container_valid,
        container_issues: container_snapshot.container_issues,
        container_section_issues: container_snapshot.container_section_issues,
        container_loader_symbol_issues: container_snapshot.container_loader_symbol_issues,
        container_relocation_issues: container_snapshot.container_relocation_issues,
        container_compatibility_domain_issues: container_snapshot
            .container_compatibility_domain_issues,
        container_external_import_issues: container_snapshot.container_external_import_issues,
        container_payload_present: container_snapshot.container_payload_present,
        container_payload_issues: container_snapshot.container_payload_issues,
        closure_snapshot_present: container_snapshot.closure_snapshot_present,
        closure_snapshot_valid: container_snapshot.closure_snapshot_valid,
        closure_snapshot_issues: container_snapshot.closure_snapshot_issues,
        closure_snapshot_linker_contract_hash: container_snapshot
            .closure_snapshot_linker_contract_hash,
        closure_snapshot_container_hash: container_snapshot.closure_snapshot_container_hash,
        closure_snapshot_payload_size_bytes: container_snapshot.closure_snapshot_payload_size_bytes,
        closure_snapshot_payload_hash: container_snapshot.closure_snapshot_payload_hash,
        final_stage_plan_present: final_snapshot.final_stage_plan_present,
        final_stage_plan_valid: final_snapshot.final_stage_plan_valid,
        final_stage_plan_ready: final_snapshot.final_stage_plan_ready,
        final_stage_plan_hash: final_snapshot.final_stage_plan_hash,
        final_stage_plan_blocker_count: final_snapshot.final_stage_plan_blocker_count,
        final_stage_plan_issues: final_snapshot.final_stage_plan_issues,
        final_executable_writer_input_present: final_snapshot.final_executable_writer_input_present,
        final_executable_writer_input_valid: final_snapshot.final_executable_writer_input_valid,
        final_executable_writer_input_hash: final_snapshot.final_executable_writer_input_hash,
        final_executable_writer_input_command_arg_count: final_snapshot
            .final_executable_writer_input_command_arg_count,
        final_executable_writer_input_issues: final_snapshot.final_executable_writer_input_issues,
        final_executable_host_invoke_plan_present: final_snapshot
            .final_executable_host_invoke_plan_present,
        final_executable_host_invoke_plan_valid: final_snapshot
            .final_executable_host_invoke_plan_valid,
        final_executable_host_invoke_plan_hash: final_snapshot
            .final_executable_host_invoke_plan_hash,
        final_executable_host_invoke_plan_invocation_policy: final_snapshot
            .final_executable_host_invoke_plan_invocation_policy,
        final_executable_host_invoke_plan_requires_explicit_allow: final_snapshot
            .final_executable_host_invoke_plan_requires_explicit_allow,
        final_executable_host_invoke_plan_explicit_allow_present: final_snapshot
            .final_executable_host_invoke_plan_explicit_allow_present,
        final_executable_host_invoke_plan_would_invoke: final_snapshot
            .final_executable_host_invoke_plan_would_invoke,
        final_executable_host_invoke_plan_blocker_count: final_snapshot
            .final_executable_host_invoke_plan_blocker_count,
        final_executable_host_invoke_plan_issues: final_snapshot
            .final_executable_host_invoke_plan_issues,
        final_executable_layout_plan_present: final_snapshot.final_executable_layout_plan_present,
        final_executable_layout_plan_valid: final_snapshot.final_executable_layout_plan_valid,
        final_executable_layout_plan_hash: final_snapshot.final_executable_layout_plan_hash,
        final_executable_layout_plan_payload_count: final_snapshot
            .final_executable_layout_plan_payload_count,
        final_executable_layout_plan_issues: final_snapshot.final_executable_layout_plan_issues,
        final_executable_image_dry_run_present: final_snapshot
            .final_executable_image_dry_run_present,
        final_executable_image_dry_run_valid: final_snapshot.final_executable_image_dry_run_valid,
        final_executable_image_dry_run_hash: final_snapshot.final_executable_image_dry_run_hash,
        final_executable_image_dry_run_size_bytes: final_snapshot
            .final_executable_image_dry_run_size_bytes,
        final_executable_image_dry_run_issues: final_snapshot.final_executable_image_dry_run_issues,
        final_executable_blocked_present: final_snapshot.final_executable_blocked_present,
        final_executable_blocked_valid: final_snapshot.final_executable_blocked_valid,
        final_executable_blocked_emitted: final_snapshot.final_executable_blocked_emitted,
        final_executable_blocked_plan_hash: final_snapshot.final_executable_blocked_plan_hash,
        final_executable_blocked_blocker_count: final_snapshot
            .final_executable_blocked_blocker_count,
        final_executable_blocked_issues: final_snapshot.final_executable_blocked_issues,
        final_executable_output_present: final_snapshot.final_executable_output_present,
        final_executable_output_size_bytes: final_snapshot.final_executable_output_size_bytes,
        final_executable_output_hash: final_snapshot.final_executable_output_hash,
        final_executable_output_runnable_candidate: final_snapshot
            .final_executable_output_runnable_candidate,
        final_executable_output_blocker_count: final_snapshot.final_executable_output_blocker_count,
        final_executable_output_issues: final_snapshot.final_executable_output_issues,
        container_loader_readiness: container_snapshot.container_loader_readiness,
        container_loader_blockers: container_snapshot.container_loader_blockers,
        container_metadata_table_hash: container_snapshot.container_metadata_table_hash,
        container_compatibility_domain_count: container_snapshot
            .container_compatibility_domain_count,
        container_compatibility_domain_table_hash: container_snapshot
            .container_compatibility_domain_table_hash,
        container_compatibility_domain_id: container_snapshot.container_compatibility_domain_id,
        container_compatibility_domain_kind: container_snapshot.container_compatibility_domain_kind,
        container_compatibility_domain_paradigm: container_snapshot
            .container_compatibility_domain_paradigm,
        container_compatibility_domain_lifecycle_hook: container_snapshot
            .container_compatibility_domain_lifecycle_hook,
        container_compatibility_domain_abi_family: container_snapshot
            .container_compatibility_domain_abi_family,
        container_compatibility_domain_wrapper_policy: container_snapshot
            .container_compatibility_domain_wrapper_policy,
        container_compatibility_domain_required: container_snapshot
            .container_compatibility_domain_required,
        container_external_import_count: container_snapshot.container_external_import_count,
        container_native_object_section_present: container_snapshot
            .container_native_object_section_present,
        container_native_object_section_id: container_snapshot.container_native_object_section_id,
        container_native_object_loader_symbol_present: container_snapshot
            .container_native_object_loader_symbol_present,
        container_native_object_loader_symbol_id: container_snapshot
            .container_native_object_loader_symbol_id,
        container_native_object_relocation_present: container_snapshot
            .container_native_object_relocation_present,
        container_native_object_relocation_id: container_snapshot
            .container_native_object_relocation_id,
        artifact_chain_valid,
        artifact_chain_issues,
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        domains: core_snapshot.domains,
        sidecar_capabilities: core_snapshot.sidecar_capabilities,
        clock_edges: core_snapshot.clock_edges,
        data_segments: core_snapshot.data_segments,
        issues,
    }
}

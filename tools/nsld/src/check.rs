use super::{
    artifact_chain::{
        nsld_artifact_chain_issues, nsld_artifact_stage_kind_path, nsld_artifact_stages,
        NsldArtifactStageKind,
    },
    assembly::{
        nsld_verify_assemble_plan_report, nsld_verify_link_bundle_report,
        nsld_verify_section_manifest_report,
    },
    container_pipeline::{
        nsld_container_report, nsld_verify_container_plan_report, nsld_verify_container_report,
    },
    link_units::{
        nsld_domain_diagnostics, nsld_sidecar_capability_diagnostics,
        nsld_verify_link_inputs_report, nsld_verify_link_units_report,
    },
    object_byte_layout::nsld_verify_object_byte_layout_report,
    object_emit::nsld_verify_object_emit_report,
    object_file_layout::nsld_verify_object_file_layout_report,
    object_image_dry_run::nsld_verify_object_image_dry_run_report,
    object_output::nsld_verify_object_output_report,
    object_plan::nsld_verify_object_plan_report,
    object_writer_input::{
        nsld_verify_object_writer_dry_run_report, nsld_verify_object_writer_input_report,
    },
    reports::{NsldCheckReport, NsldClockEdgeDiagnostic, NsldDataSegmentDiagnostic},
};
use std::path::Path;

pub(crate) fn nsld_check_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldCheckReport {
    let artifact_lowering_alignment_consistent = plan.artifact_lowering_alignment.consistent;
    let artifact_lowering_alignment_mismatches = plan.artifact_lowering_alignment.mismatches;
    let clock_protocol_valid = plan.clock_protocol.validation.valid;
    let clock_protocol_issues = plan.clock_protocol.validation.issues.clone();
    let hetero_calculate_valid = plan.hetero_calculate.validation.valid;
    let hetero_calculate_issues = plan.hetero_calculate.validation.issues.clone();
    let static_link = plan.hetero_calculate.static_link;
    let lifecycle_driven = plan.hetero_calculate.lifecycle_driven;
    let domains = nsld_domain_diagnostics(plan);
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let sidecar_capability_issues = sidecar_capabilities
        .iter()
        .flat_map(|capability| {
            capability
                .issues
                .iter()
                .map(|issue| {
                    format!(
                        "{}:{}: {}",
                        capability.package_id, capability.domain_family, issue
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let sidecar_capability_valid = sidecar_capability_issues.is_empty();
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
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectOutput);
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
    let container_plan_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ContainerPlan);
    let container_plan_present = container_plan_path.exists();
    let container_plan_verify_report =
        container_plan_present.then(|| nsld_verify_container_plan_report(manifest, plan));
    let container_plan_valid = container_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let container_plan_issues = container_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let container_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::Container);
    let container_present = container_path.exists();
    let container_verify_report =
        container_present.then(|| nsld_verify_container_report(manifest, plan));
    let container_valid = container_verify_report.as_ref().map(|report| report.valid);
    let container_issues = container_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let container_section_issues = container_verify_report
        .as_ref()
        .map(|report| report.container_section_issues.clone())
        .unwrap_or_default();
    let container_loader_symbol_issues = container_verify_report
        .as_ref()
        .map(|report| report.loader_symbol_issues.clone())
        .unwrap_or_default();
    let container_relocation_issues = container_verify_report
        .as_ref()
        .map(|report| report.relocation_issues.clone())
        .unwrap_or_default();
    let container_compatibility_domain_issues = container_verify_report
        .as_ref()
        .map(|report| report.compatibility_domain_issues.clone())
        .unwrap_or_default();
    let container_external_import_issues = container_verify_report
        .as_ref()
        .map(|report| report.external_import_issues.clone())
        .unwrap_or_default();
    let container_compatibility_domain_count = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_count);
    let container_compatibility_domain_table_hash = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_table_hash.clone());
    let container_compatibility_domain_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_id.clone());
    let container_compatibility_domain_kind = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_kind.clone());
    let container_compatibility_domain_paradigm = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_paradigm.clone());
    let container_compatibility_domain_lifecycle_hook = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_lifecycle_hook.clone());
    let container_compatibility_domain_abi_family = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_abi_family.clone());
    let container_compatibility_domain_wrapper_policy = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_wrapper_policy.clone());
    let container_compatibility_domain_required = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_compatibility_domain_required);
    let container_native_object_section_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_native_object_section_present);
    let container_native_object_section_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_native_object_section_id.clone());
    let container_native_object_loader_symbol_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_native_object_loader_symbol_present);
    let container_native_object_loader_symbol_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_native_object_loader_symbol_id.clone());
    let container_native_object_relocation_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_native_object_relocation_present);
    let container_native_object_relocation_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_native_object_relocation_id.clone());
    let expected_container_report =
        container_present.then(|| nsld_container_report(manifest, plan));
    let container_loader_readiness = expected_container_report
        .as_ref()
        .map(|report| report.loader_readiness.clone());
    let container_loader_blockers = expected_container_report
        .as_ref()
        .map(|report| report.loader_blockers.clone())
        .unwrap_or_default();
    let container_metadata_table_hash = expected_container_report
        .as_ref()
        .map(|report| report.metadata_table_hash.clone());
    let container_external_import_count = expected_container_report
        .as_ref()
        .map(|report| report.external_imports.len());
    let container_payload_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ContainerPayload);
    let container_payload_present = container_payload_path.exists();
    let mut container_payload_issues = Vec::new();
    if container_payload_present && !container_present {
        container_payload_issues.push("container payload is present without container".to_owned());
    }
    if container_present && !container_payload_present {
        container_payload_issues
            .push("container payload is missing for present container".to_owned());
    }
    let artifact_chain_issues = nsld_artifact_chain_issues(&nsld_artifact_stages(&plan.output_dir));
    let artifact_chain_valid = artifact_chain_issues.is_empty();
    let clock_edges = plan
        .clock_protocol
        .edges
        .iter()
        .map(|edge| NsldClockEdgeDiagnostic {
            index: edge.index,
            from: edge.from.clone(),
            to: edge.to.clone(),
            relation: edge.relation.clone(),
            source: edge.source.clone(),
        })
        .collect::<Vec<_>>();
    let data_segments = plan
        .hetero_calculate
        .data_segments
        .iter()
        .map(|segment| NsldDataSegmentDiagnostic {
            index: segment.index,
            segment_id: segment.segment_id.clone(),
            domain_family: segment.domain_family.clone(),
            owner_package: segment.owner_package.clone(),
            order_key: segment.order_key.clone(),
            access_phase: segment.access_phase.clone(),
            source_path: segment
                .source_path
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect::<Vec<_>>();
    let mut issues = Vec::new();

    if !artifact_lowering_alignment_consistent {
        issues.push(format!(
            "artifact lowering alignment has {} mismatch(es)",
            artifact_lowering_alignment_mismatches
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
    if !clock_protocol_valid {
        issues.push("clock protocol validation failed".to_owned());
        issues.extend(clock_protocol_issues.iter().cloned());
    }
    if !hetero_calculate_valid {
        issues.push("hetero calculate validation failed".to_owned());
        issues.extend(hetero_calculate_issues.iter().cloned());
    }
    if !static_link {
        issues.push("hetero calculate plan is not static-link".to_owned());
    }
    if !lifecycle_driven {
        issues.push("hetero calculate plan is not lifecycle-driven".to_owned());
    }
    if !sidecar_capability_valid {
        issues.push("sidecar capability validation failed".to_owned());
        issues.extend(sidecar_capability_issues.iter().cloned());
    }
    if link_input_table_valid == Some(false) {
        issues.push("link input table verification failed".to_owned());
        issues.extend(link_input_table_issues.iter().cloned());
    }
    if link_unit_table_valid == Some(false) {
        issues.push("link unit table verification failed".to_owned());
        issues.extend(link_unit_table_issues.iter().cloned());
    }
    if link_bundle_valid == Some(false) {
        issues.push("link bundle verification failed".to_owned());
        issues.extend(link_bundle_issues.iter().cloned());
    }
    if assemble_plan_valid == Some(false) {
        issues.push("assemble plan verification failed".to_owned());
        issues.extend(assemble_plan_issues.iter().cloned());
    }
    if section_manifest_valid == Some(false) {
        issues.push("section manifest verification failed".to_owned());
        issues.extend(section_manifest_issues.iter().cloned());
    }
    if object_plan_valid == Some(false) {
        issues.push("object plan verification failed".to_owned());
        issues.extend(object_plan_issues.iter().cloned());
    }
    if object_writer_input_valid == Some(false) {
        issues.push("object writer input verification failed".to_owned());
        issues.extend(object_writer_input_issues.iter().cloned());
    }
    if object_byte_layout_valid == Some(false) {
        issues.push("object byte layout verification failed".to_owned());
        issues.extend(object_byte_layout_issues.iter().cloned());
    }
    if object_file_layout_valid == Some(false) {
        issues.push("object file layout verification failed".to_owned());
        issues.extend(object_file_layout_issues.iter().cloned());
    }
    if object_image_dry_run_valid == Some(false) {
        issues.push("object image dry-run verification failed".to_owned());
        issues.extend(object_image_dry_run_issues.iter().cloned());
    }
    if object_emit_blocked_valid == Some(false) {
        issues.push("object emit blocked report verification failed".to_owned());
        issues.extend(object_emit_blocked_issues.iter().cloned());
    }
    if object_output_valid == Some(false) {
        issues.push("object output verification failed".to_owned());
        issues.extend(object_output_issues.iter().cloned());
    }
    if object_writer_dry_run_valid == Some(false) {
        issues.push("object writer dry-run verification failed".to_owned());
        issues.extend(object_writer_dry_run_issues.iter().cloned());
    }
    if container_plan_valid == Some(false) {
        issues.push("container plan verification failed".to_owned());
        issues.extend(container_plan_issues.iter().cloned());
    }
    if container_valid == Some(false) {
        issues.push("container verification failed".to_owned());
        issues.extend(container_issues.iter().cloned());
    }
    if container_loader_readiness.as_deref() == Some("blocked") {
        issues.push("container loader readiness is blocked".to_owned());
        issues.extend(container_loader_blockers.iter().cloned());
    }
    if !container_payload_issues.is_empty() {
        issues.push("container payload state is inconsistent".to_owned());
        issues.extend(container_payload_issues.iter().cloned());
    }
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
    let checks = checks + usize::from(container_plan_present);
    let checks = checks + usize::from(container_present);
    let checks = checks + usize::from(container_present || container_payload_present);
    let failures = issues.len();
    NsldCheckReport {
        manifest: manifest.display().to_string(),
        valid: failures == 0,
        checks,
        failures,
        artifact_lowering_alignment_consistent,
        artifact_lowering_alignment_mismatches,
        clock_protocol_valid,
        clock_protocol_issues,
        hetero_calculate_valid,
        hetero_calculate_issues,
        static_link,
        lifecycle_driven,
        sidecar_capability_valid,
        sidecar_capability_issues,
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
        container_plan_present,
        container_plan_valid,
        container_plan_issues,
        container_present,
        container_valid,
        container_issues,
        container_section_issues,
        container_loader_symbol_issues,
        container_relocation_issues,
        container_compatibility_domain_issues,
        container_external_import_issues,
        container_payload_present,
        container_payload_issues,
        container_loader_readiness,
        container_loader_blockers,
        container_metadata_table_hash,
        container_compatibility_domain_count,
        container_compatibility_domain_table_hash,
        container_compatibility_domain_id,
        container_compatibility_domain_kind,
        container_compatibility_domain_paradigm,
        container_compatibility_domain_lifecycle_hook,
        container_compatibility_domain_abi_family,
        container_compatibility_domain_wrapper_policy,
        container_compatibility_domain_required,
        container_external_import_count,
        container_native_object_section_present,
        container_native_object_section_id,
        container_native_object_loader_symbol_present,
        container_native_object_loader_symbol_id,
        container_native_object_relocation_present,
        container_native_object_relocation_id,
        artifact_chain_valid,
        artifact_chain_issues,
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        domains,
        sidecar_capabilities,
        clock_edges,
        data_segments,
        issues,
    }
}

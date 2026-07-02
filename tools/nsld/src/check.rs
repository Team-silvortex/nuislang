use super::{
    assembly::{
        nsld_verify_assemble_plan_report, nsld_verify_link_bundle_report,
        nsld_verify_section_manifest_report,
    },
    closure::nsld_artifact_chain_issues,
    container_pipeline::{
        nsld_container_report, nsld_verify_container_plan_report, nsld_verify_container_report,
    },
    link_units::{
        nsld_domain_diagnostics, nsld_sidecar_capability_diagnostics,
        nsld_verify_link_inputs_report, nsld_verify_link_units_report,
    },
    reports::{NsldCheckReport, NsldClockEdgeDiagnostic, NsldDataSegmentDiagnostic},
};
use std::path::{Path, PathBuf};

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
    let link_input_table_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let link_input_table_present = link_input_table_path.exists();
    let link_input_verify_report =
        link_input_table_present.then(|| nsld_verify_link_inputs_report(manifest, plan));
    let link_input_table_valid = link_input_verify_report.as_ref().map(|report| report.valid);
    let link_input_table_issues = link_input_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let link_unit_table_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-units.toml");
    let link_unit_table_present = link_unit_table_path.exists();
    let link_unit_verify_report =
        link_unit_table_present.then(|| nsld_verify_link_units_report(manifest, plan));
    let link_unit_table_valid = link_unit_verify_report.as_ref().map(|report| report.valid);
    let link_unit_table_issues = link_unit_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let link_bundle_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-bundle.toml");
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
    let assemble_plan_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.assemble-plan.toml");
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
        PathBuf::from(&plan.output_dir).join("nuis.nsld.section-manifest.toml");
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
    let container_plan_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container-plan.toml");
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
    let container_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container");
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
    let container_external_import_issues = container_verify_report
        .as_ref()
        .map(|report| report.external_import_issues.clone())
        .unwrap_or_default();
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
        PathBuf::from(&plan.output_dir).join("nuis.nsld.container.payload");
    let container_payload_present = container_payload_path.exists();
    let mut container_payload_issues = Vec::new();
    if container_payload_present && !container_present {
        container_payload_issues.push("container payload is present without container".to_owned());
    }
    if container_present && !container_payload_present {
        container_payload_issues
            .push("container payload is missing for present container".to_owned());
    }
    let artifact_chain_issues = nsld_artifact_chain_issues(&[
        ("nuis.nsld.link-inputs.toml", link_input_table_present),
        ("nuis.nsld.link-units.toml", link_unit_table_present),
        ("nuis.nsld.link-bundle.toml", link_bundle_present),
        ("nuis.nsld.assemble-plan.toml", assemble_plan_present),
        ("nuis.nsld.section-manifest.toml", section_manifest_present),
        ("nuis.nsld.container-plan.toml", container_plan_present),
        ("nuis.nsld.container", container_present),
        ("nuis.nsld.container.payload", container_payload_present),
    ]);
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
        container_plan_present,
        container_plan_valid,
        container_plan_issues,
        container_present,
        container_valid,
        container_issues,
        container_section_issues,
        container_loader_symbol_issues,
        container_relocation_issues,
        container_external_import_issues,
        container_payload_present,
        container_payload_issues,
        container_loader_readiness,
        container_loader_blockers,
        container_metadata_table_hash,
        container_external_import_count,
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

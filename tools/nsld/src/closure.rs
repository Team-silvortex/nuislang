use super::{
    container_pipeline::nsld_container_report,
    link_units::{
        nsld_link_input_summary, nsld_sidecar_capability_diagnostics,
        nsld_verify_link_inputs_report,
    },
    reports::NsldClosureReport,
};
use std::path::{Path, PathBuf};

pub(crate) fn nsld_artifact_chain_issues(stages: &[(&str, bool)]) -> Vec<String> {
    let mut first_missing_before_present = None;
    let mut issues = Vec::new();

    for (name, present) in stages {
        if *present {
            if let Some(missing) = first_missing_before_present {
                issues.push(format!(
                    "artifact `{name}` is present but prerequisite `{missing}` is missing"
                ));
            }
        } else if first_missing_before_present.is_none() {
            first_missing_before_present = Some(*name);
        }
    }

    issues
}

pub(crate) fn nsld_closure_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldClosureReport {
    let mut internal_contracts = vec![
        "build-manifest".to_owned(),
        "compiled-artifact-envelope".to_owned(),
        "artifact-lowering-alignment".to_owned(),
        "clock-protocol".to_owned(),
        "hetero-calculate-plan".to_owned(),
        "deterministic-data-segment-order".to_owned(),
    ];
    if plan.bridge_registry_path.is_some() {
        internal_contracts.push("bridge-registry".to_owned());
    }
    if plan.host_bridge_plan_index_path.is_some() {
        internal_contracts.push("host-bridge-plan-index".to_owned());
    }
    if plan.lowering_plan_index_path.is_some() {
        internal_contracts.push("lowering-plan-index".to_owned());
    }
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    if !sidecar_capabilities.is_empty()
        && sidecar_capabilities
            .iter()
            .all(|capability| capability.valid)
    {
        internal_contracts.push("lowering-sidecar-capabilities".to_owned());
        internal_contracts.push("link-input-sidecar-table".to_owned());
    }
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let link_input_table_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let link_input_verify_report = link_input_table_path
        .exists()
        .then(|| nsld_verify_link_inputs_report(manifest, plan));
    let link_input_table_present = link_input_verify_report.is_some();
    let link_input_table_valid = link_input_verify_report.as_ref().map(|report| report.valid);
    if link_input_table_valid == Some(true) {
        internal_contracts.push("verified-link-input-table".to_owned());
    }
    let container_report = nsld_container_report(manifest, plan);

    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut external_dependencies = Vec::new();
    if host_wrapper_required {
        external_dependencies.push(format!("final-stage:{}", plan.final_stage.driver));
    }
    if !plan.cpu_target.clang_target.is_empty() {
        external_dependencies.push(format!("clang-target:{}", plan.cpu_target.clang_target));
    }
    if plan.final_stage.link_mode == "bundle-packaging" {
        external_dependencies.push("host-launcher-wrapper".to_owned());
    }

    let mut unresolved = Vec::new();
    if host_wrapper_required {
        unresolved.push("self-owned-final-native-linker".to_owned());
    }
    if plan.compiled_artifact.container_kind.is_none() {
        unresolved.push("nuis-owned-container-kind".to_owned());
    }
    if !plan.artifact_lowering_alignment.consistent {
        unresolved.push("artifact-lowering-alignment-mismatch".to_owned());
    }
    if !plan.clock_protocol.validation.valid {
        unresolved.push("clock-protocol-validation".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        unresolved.push("hetero-calculate-validation".to_owned());
    }
    for capability in &sidecar_capabilities {
        for issue in &capability.issues {
            unresolved.push(format!(
                "sidecar-capability:{}:{}:{}",
                capability.package_id, capability.domain_family, issue
            ));
        }
    }
    if let Some(report) = &link_input_verify_report {
        for issue in &report.issues {
            unresolved.push(format!("link-input-table:{issue}"));
        }
    }

    NsldClosureReport {
        manifest: manifest.display().to_string(),
        closed: unresolved.is_empty(),
        internal_contracts,
        link_inputs: link_input_summary.inputs,
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
        link_input_table_present,
        link_input_table_valid,
        container_metadata_table_hash: container_report.metadata_table_hash,
        container_loader_readiness: container_report.loader_readiness,
        external_dependencies,
        unresolved,
        host_wrapper_required,
        domain_count: plan.domain_units.len(),
        hetero_domain_count: plan
            .domain_units
            .iter()
            .filter(|unit| unit.kind == "heterogeneous")
            .count(),
        sidecar_capability_count: sidecar_capabilities.len(),
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
    }
}

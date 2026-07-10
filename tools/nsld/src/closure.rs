use super::{
    artifact_chain::{
        nsld_artifact_chain_issues, nsld_artifact_stage_kind_path, nsld_artifact_stage_present,
        nsld_artifact_stages_for_plan, NsldArtifactStageKind,
    },
    closure_snapshot_helpers::{
        closure_snapshot_path, nsld_linker_contract_hash, push_bool_mismatch, push_string_mismatch,
        push_usize_mismatch, render_closure_snapshot,
    },
    container_pipeline::nsld_container_report,
    link_units::{
        nsld_link_input_summary, nsld_sidecar_capability_diagnostics,
        nsld_verify_link_inputs_report,
    },
    object_byte_layout::nsld_verify_object_byte_layout_report,
    object_emit::nsld_verify_object_emit_report,
    object_file_layout::nsld_verify_object_file_layout_report,
    object_image_dry_run::nsld_verify_object_image_dry_run_report,
    object_output::nsld_object_output_issues,
    object_plan::nsld_verify_object_plan_report,
    object_writer_input::{
        nsld_verify_object_writer_dry_run_report, nsld_verify_object_writer_input_report,
    },
    reports::{NsldClosureEmitReport, NsldClosureReport, NsldClosureVerifyReport},
    toml,
};
use std::{fs, path::Path};
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
    let link_input_table_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::LinkInputs);
    let link_input_verify_report = link_input_table_path
        .exists()
        .then(|| nsld_verify_link_inputs_report(manifest, plan));
    let link_input_table_present = link_input_verify_report.is_some();
    let link_input_table_valid = link_input_verify_report.as_ref().map(|report| report.valid);
    if link_input_table_valid == Some(true) {
        internal_contracts.push("verified-link-input-table".to_owned());
    }
    let container_report = nsld_container_report(manifest, plan);
    let compatibility_domain = container_report.compatibility_domains.first();

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
    let prepared_artifact_stages = nsld_artifact_stages_for_plan(plan)
        .into_iter()
        .filter(|stage| {
            stage.kind != NsldArtifactStageKind::ClosureSnapshot
                && stage.kind != NsldArtifactStageKind::FinalStagePlan
                && stage.kind != NsldArtifactStageKind::FinalExecutableBlocked
        })
        .collect::<Vec<_>>();
    let prepared_artifact_chain_issues = nsld_artifact_chain_issues(&prepared_artifact_stages);
    let prepared_artifact_chain_valid = prepared_artifact_chain_issues.is_empty();
    if prepared_artifact_chain_valid
        && prepared_artifact_stages
            .iter()
            .all(|stage| stage.present || !stage.required)
    {
        internal_contracts.push("verified-prepared-artifact-chain".to_owned());
    }
    for issue in &prepared_artifact_chain_issues {
        unresolved.push(format!("prepared-artifact-chain:{issue}"));
    }
    let object_image_dry_run_present = nsld_artifact_stage_present(
        &prepared_artifact_stages,
        NsldArtifactStageKind::ObjectImageDryRun,
    );
    let object_image_dry_run_verify_report = object_image_dry_run_present
        .then(|| nsld_verify_object_image_dry_run_report(manifest, plan));
    if object_image_dry_run_verify_report
        .as_ref()
        .is_some_and(|report| {
            report.valid
                && report
                    .actual_relocation_record_table_hash
                    .as_deref()
                    .is_some_and(|hash| hash.starts_with("0x"))
        })
    {
        internal_contracts.push("verified-object-image-relocation-record-table".to_owned());
    }
    verify_prepared_artifact(
        "object-plan",
        nsld_artifact_stage_present(&prepared_artifact_stages, NsldArtifactStageKind::ObjectPlan),
        || nsld_verify_object_plan_report(manifest, plan).valid,
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-writer-input",
        nsld_artifact_stage_present(
            &prepared_artifact_stages,
            NsldArtifactStageKind::ObjectWriterInput,
        ),
        || nsld_verify_object_writer_input_report(manifest, plan).valid,
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-byte-layout",
        nsld_artifact_stage_present(
            &prepared_artifact_stages,
            NsldArtifactStageKind::ObjectByteLayout,
        ),
        || nsld_verify_object_byte_layout_report(manifest, plan).valid,
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-file-layout",
        nsld_artifact_stage_present(
            &prepared_artifact_stages,
            NsldArtifactStageKind::ObjectFileLayout,
        ),
        || nsld_verify_object_file_layout_report(manifest, plan).valid,
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-image-dry-run",
        object_image_dry_run_present,
        || {
            object_image_dry_run_verify_report
                .as_ref()
                .is_some_and(|report| report.valid)
        },
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-emit-blocked",
        nsld_artifact_stage_present(
            &prepared_artifact_stages,
            NsldArtifactStageKind::ObjectEmitBlocked,
        ),
        || nsld_verify_object_emit_report(manifest, plan).valid,
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-output",
        nsld_artifact_stage_present(
            &prepared_artifact_stages,
            NsldArtifactStageKind::ObjectOutput,
        ),
        || nsld_object_output_issues(plan).is_empty(),
        &mut internal_contracts,
        &mut unresolved,
    );
    verify_prepared_artifact(
        "object-writer-dry-run",
        nsld_artifact_stage_present(
            &prepared_artifact_stages,
            NsldArtifactStageKind::ObjectWriterDryRun,
        ),
        || nsld_verify_object_writer_dry_run_report(manifest, plan).valid,
        &mut internal_contracts,
        &mut unresolved,
    );
    let object_image_relocation_record_table_hash = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_record_table_hash.clone());
    let linker_contract_hash = nsld_linker_contract_hash(
        &internal_contracts,
        &link_input_summary.table_hash,
        &container_report.metadata_table_hash,
        &container_report.container_hash,
        &container_report.payload_hash,
        &container_report.loader_readiness,
        object_image_relocation_record_table_hash.as_deref(),
        &external_dependencies,
        &unresolved,
        &plan.final_stage.link_mode,
    );

    NsldClosureReport {
        manifest: manifest.display().to_string(),
        closed: unresolved.is_empty(),
        internal_contracts,
        linker_contract_hash,
        link_inputs: link_input_summary.inputs,
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
        link_input_table_present,
        link_input_table_valid,
        prepared_artifact_chain_valid,
        prepared_artifact_chain_issues,
        container_metadata_table_hash: container_report.metadata_table_hash,
        container_layout_hash: container_report.container_layout_hash,
        container_hash: container_report.container_hash,
        payload_size_bytes: container_report.payload_size_bytes,
        payload_hash: container_report.payload_hash,
        container_loader_readiness: container_report.loader_readiness,
        compatibility_domain_count: container_report.compatibility_domains.len(),
        compatibility_domain_table_hash: container_report.compatibility_domain_table_hash,
        compatibility_domain_id: compatibility_domain.map(|domain| domain.domain_id.clone()),
        compatibility_domain_kind: compatibility_domain.map(|domain| domain.domain_kind.clone()),
        compatibility_domain_paradigm: compatibility_domain.map(|domain| domain.paradigm.clone()),
        compatibility_domain_lifecycle_hook: compatibility_domain
            .map(|domain| domain.lifecycle_hook.clone()),
        compatibility_domain_abi_family: compatibility_domain
            .map(|domain| domain.abi_family.clone()),
        compatibility_domain_wrapper_policy: compatibility_domain
            .map(|domain| domain.wrapper_policy.clone()),
        compatibility_domain_required: compatibility_domain.map(|domain| domain.required),
        object_image_relocation_lowering_valid: object_image_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_relocation_lowering_valid),
        object_image_relocation_lowering_rule_count: object_image_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_relocation_lowering_rule_count),
        object_image_relocation_lowering_rules: object_image_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_relocation_lowering_rules.clone())
            .unwrap_or_default(),
        object_image_relocation_lowering_issues: object_image_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_relocation_lowering_issues.clone())
            .unwrap_or_default(),
        object_image_relocation_record_count: object_image_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_relocation_record_count),
        object_image_relocation_record_table_hash,
        object_image_relocation_records: object_image_dry_run_verify_report
            .as_ref()
            .and_then(|report| report.actual_relocation_records.clone())
            .unwrap_or_default(),
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

pub(crate) fn nsld_emit_closure_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldClosureEmitReport, String> {
    let report = nsld_closure_report(manifest, plan);
    let output_path = closure_snapshot_path(plan);
    fs::write(&output_path, render_closure_snapshot(&report)).map_err(|error| {
        format!(
            "failed to write nsld closure snapshot `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldClosureEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        linker_contract_hash: report.linker_contract_hash,
        closed: report.closed,
        internal_contract_count: report.internal_contracts.len(),
        unresolved_count: report.unresolved.len(),
    })
}

pub(crate) fn nsld_verify_closure_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldClosureVerifyReport {
    let expected_report = nsld_closure_report(manifest, plan);
    let input_path = closure_snapshot_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_closure_snapshot `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_linker_contract_hash,
        actual_container_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_closed,
        actual_internal_contract_count,
        actual_unresolved_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "linker_contract_hash"),
            toml::string_value(source, "container_hash"),
            toml::usize_value(source, "payload_size_bytes"),
            toml::string_value(source, "payload_hash"),
            toml::bool_value(source, "closed"),
            toml::usize_value(source, "internal_contract_count"),
            toml::usize_value(source, "unresolved_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        let expected = render_closure_snapshot(&expected_report);
        if actual != expected {
            issues.push("closure-snapshot-content-mismatch".to_owned());
        }
        push_string_mismatch(
            &mut issues,
            "linker_contract_hash",
            &expected_report.linker_contract_hash,
            actual_linker_contract_hash.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "container_hash",
            &expected_report.container_hash,
            actual_container_hash.as_deref(),
        );
        push_usize_mismatch(
            &mut issues,
            "payload_size_bytes",
            expected_report.payload_size_bytes,
            actual_payload_size_bytes,
        );
        push_string_mismatch(
            &mut issues,
            "payload_hash",
            &expected_report.payload_hash,
            actual_payload_hash.as_deref(),
        );
        push_bool_mismatch(&mut issues, "closed", expected_report.closed, actual_closed);
        push_usize_mismatch(
            &mut issues,
            "internal_contract_count",
            expected_report.internal_contracts.len(),
            actual_internal_contract_count,
        );
        push_usize_mismatch(
            &mut issues,
            "unresolved_count",
            expected_report.unresolved.len(),
            actual_unresolved_count,
        );
    }

    NsldClosureVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_linker_contract_hash: expected_report.linker_contract_hash,
        expected_container_hash: expected_report.container_hash,
        expected_payload_size_bytes: expected_report.payload_size_bytes,
        expected_payload_hash: expected_report.payload_hash,
        expected_closed: expected_report.closed,
        expected_internal_contract_count: expected_report.internal_contracts.len(),
        expected_unresolved_count: expected_report.unresolved.len(),
        actual_linker_contract_hash,
        actual_container_hash,
        actual_payload_size_bytes,
        actual_payload_hash,
        actual_closed,
        actual_internal_contract_count,
        actual_unresolved_count,
        issues,
    }
}

fn verify_prepared_artifact(
    contract: &str,
    present: bool,
    verify: impl FnOnce() -> bool,
    internal_contracts: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) {
    if !present {
        return;
    }
    if verify() {
        internal_contracts.push(format!("verified-{contract}"));
    } else {
        unresolved.push(format!("{contract}:verification"));
    }
}

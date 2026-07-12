use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    closure::nsld_verify_closure_report,
    container_pipeline::{
        nsld_container_report, nsld_verify_container_plan_report, nsld_verify_container_report,
    },
};
use std::path::Path;

pub(crate) struct NsldCheckContainerSnapshot {
    pub(crate) container_plan_present: bool,
    pub(crate) container_plan_valid: Option<bool>,
    pub(crate) container_plan_issues: Vec<String>,
    pub(crate) container_present: bool,
    pub(crate) container_valid: Option<bool>,
    pub(crate) container_issues: Vec<String>,
    pub(crate) container_section_issues: Vec<String>,
    pub(crate) container_loader_symbol_issues: Vec<String>,
    pub(crate) container_relocation_issues: Vec<String>,
    pub(crate) container_compatibility_domain_issues: Vec<String>,
    pub(crate) container_external_import_issues: Vec<String>,
    pub(crate) container_payload_present: bool,
    pub(crate) container_payload_issues: Vec<String>,
    pub(crate) closure_snapshot_present: bool,
    pub(crate) closure_snapshot_valid: Option<bool>,
    pub(crate) closure_snapshot_issues: Vec<String>,
    pub(crate) closure_snapshot_linker_contract_hash: Option<String>,
    pub(crate) closure_snapshot_container_hash: Option<String>,
    pub(crate) closure_snapshot_payload_size_bytes: Option<usize>,
    pub(crate) closure_snapshot_payload_hash: Option<String>,
    pub(crate) container_loader_readiness: Option<String>,
    pub(crate) container_loader_blockers: Vec<String>,
    pub(crate) container_metadata_table_hash: Option<String>,
    pub(crate) container_compatibility_domain_count: Option<usize>,
    pub(crate) container_compatibility_domain_table_hash: Option<String>,
    pub(crate) container_compatibility_domain_id: Option<String>,
    pub(crate) container_compatibility_domain_kind: Option<String>,
    pub(crate) container_compatibility_domain_paradigm: Option<String>,
    pub(crate) container_compatibility_domain_lifecycle_hook: Option<String>,
    pub(crate) container_compatibility_domain_abi_family: Option<String>,
    pub(crate) container_compatibility_domain_wrapper_policy: Option<String>,
    pub(crate) container_compatibility_domain_required: Option<bool>,
    pub(crate) container_external_import_count: Option<usize>,
    pub(crate) container_native_object_section_present: bool,
    pub(crate) container_native_object_section_id: Option<String>,
    pub(crate) container_native_object_loader_symbol_present: bool,
    pub(crate) container_native_object_loader_symbol_id: Option<String>,
    pub(crate) container_native_object_relocation_present: bool,
    pub(crate) container_native_object_relocation_id: Option<String>,
    pub(crate) container_shader_section_present: bool,
    pub(crate) container_shader_section_id: Option<String>,
    pub(crate) container_shader_loader_symbol_present: bool,
    pub(crate) container_shader_loader_symbol_id: Option<String>,
    pub(crate) container_shader_relocation_present: bool,
    pub(crate) container_shader_relocation_id: Option<String>,
    pub(crate) container_kernel_section_present: bool,
    pub(crate) container_kernel_section_id: Option<String>,
    pub(crate) container_kernel_loader_symbol_present: bool,
    pub(crate) container_kernel_loader_symbol_id: Option<String>,
    pub(crate) container_kernel_relocation_present: bool,
    pub(crate) container_kernel_relocation_id: Option<String>,
}

pub(crate) fn nsld_check_container_snapshot(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldCheckContainerSnapshot {
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
    let container_shader_section_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_shader_section_present);
    let container_shader_section_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_shader_section_id.clone());
    let container_shader_loader_symbol_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_shader_loader_symbol_present);
    let container_shader_loader_symbol_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_shader_loader_symbol_id.clone());
    let container_shader_relocation_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_shader_relocation_present);
    let container_shader_relocation_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_shader_relocation_id.clone());
    let container_kernel_section_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_kernel_section_present);
    let container_kernel_section_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_kernel_section_id.clone());
    let container_kernel_loader_symbol_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_kernel_loader_symbol_present);
    let container_kernel_loader_symbol_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_kernel_loader_symbol_id.clone());
    let container_kernel_relocation_present = container_verify_report
        .as_ref()
        .is_some_and(|report| report.actual_kernel_relocation_present);
    let container_kernel_relocation_id = container_verify_report
        .as_ref()
        .and_then(|report| report.actual_kernel_relocation_id.clone());
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
    let closure_snapshot_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ClosureSnapshot);
    let closure_snapshot_present = closure_snapshot_path.exists();
    let closure_snapshot_verify_report =
        closure_snapshot_present.then(|| nsld_verify_closure_report(manifest, plan));
    let closure_snapshot_valid = closure_snapshot_verify_report
        .as_ref()
        .map(|report| report.valid);
    let closure_snapshot_issues = closure_snapshot_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let closure_snapshot_linker_contract_hash = closure_snapshot_verify_report
        .as_ref()
        .and_then(|report| report.actual_linker_contract_hash.clone());
    let closure_snapshot_container_hash = closure_snapshot_verify_report
        .as_ref()
        .and_then(|report| report.actual_container_hash.clone());
    let closure_snapshot_payload_size_bytes = closure_snapshot_verify_report
        .as_ref()
        .and_then(|report| report.actual_payload_size_bytes);
    let closure_snapshot_payload_hash = closure_snapshot_verify_report
        .as_ref()
        .and_then(|report| report.actual_payload_hash.clone());

    NsldCheckContainerSnapshot {
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
        closure_snapshot_present,
        closure_snapshot_valid,
        closure_snapshot_issues,
        closure_snapshot_linker_contract_hash,
        closure_snapshot_container_hash,
        closure_snapshot_payload_size_bytes,
        closure_snapshot_payload_hash,
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
        container_shader_section_present,
        container_shader_section_id,
        container_shader_loader_symbol_present,
        container_shader_loader_symbol_id,
        container_shader_relocation_present,
        container_shader_relocation_id,
        container_kernel_section_present,
        container_kernel_section_id,
        container_kernel_loader_symbol_present,
        container_kernel_loader_symbol_id,
        container_kernel_relocation_present,
        container_kernel_relocation_id,
    }
}

pub(crate) fn push_container_snapshot_issues(
    issues: &mut Vec<String>,
    snapshot: &NsldCheckContainerSnapshot,
) {
    if snapshot.container_plan_valid == Some(false) {
        issues.push("container plan verification failed".to_owned());
        issues.extend(snapshot.container_plan_issues.iter().cloned());
    }
    if snapshot.container_valid == Some(false) {
        issues.push("container verification failed".to_owned());
        issues.extend(snapshot.container_issues.iter().cloned());
    }
    if snapshot.container_loader_readiness.as_deref() == Some("blocked") {
        issues.push("container loader readiness is blocked".to_owned());
        issues.extend(snapshot.container_loader_blockers.iter().cloned());
    }
    if !snapshot.container_payload_issues.is_empty() {
        issues.push("container payload state is inconsistent".to_owned());
        issues.extend(snapshot.container_payload_issues.iter().cloned());
    }
    if snapshot.closure_snapshot_valid == Some(false) {
        issues.push("closure snapshot verification failed".to_owned());
        issues.extend(snapshot.closure_snapshot_issues.iter().cloned());
    }
}

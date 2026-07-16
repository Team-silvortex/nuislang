use super::{
    final_executable_container_loader::final_executable_container_loader_evidence,
    final_executable_image::{
        parse_final_executable_image_header, FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT, FINAL_EXECUTABLE_IMAGE_VERSION,
    },
    final_executable_image_stage::nsld_verify_final_executable_image_dry_run_report,
    final_executable_layout_stage::nsld_final_executable_layout_plan_report,
    final_executable_output_backend::{
        nsld_backend_artifact_assembly_boundary, nsld_backend_artifact_candidates,
    },
    final_executable_output_handoff::{
        final_executable_first_payload_execution, final_executable_output_boundary_status,
        final_executable_output_entrypoint_materialization_evidence_status,
        final_executable_output_execution_handoff, final_executable_output_materialization_status,
        final_executable_output_recommended_next_action,
    },
    final_executable_paths::{
        nsld_final_executable_launcher_dry_run_path, nsld_final_executable_launcher_manifest_path,
    },
    final_executable_provider_sample::nsld_device_provider_sample_evidence,
    final_stage::{nsld_verify_final_executable_emit_report, nsld_verify_final_stage_plan_report},
    fnv1a64_hex,
    object_output::nsld_verify_object_output_report,
    reports::NsldFinalExecutableOutputReport,
    toml,
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_output_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableOutputReport {
    let final_stage = nsld_verify_final_stage_plan_report(manifest, plan);
    let final_emit = nsld_verify_final_executable_emit_report(manifest, plan);
    let object_output = nsld_verify_object_output_report(manifest, plan);
    let final_layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let image_dry_run = nsld_verify_final_executable_image_dry_run_report(manifest, plan);
    let output_path = plan.final_stage.output_path.clone();
    let host_native_output = plan.final_stage.link_mode == "host-toolchain-finalize";
    let output_kind = if host_native_output {
        "host-native-executable"
    } else {
        "nuis-image"
    }
    .to_owned();
    let output_validation_mode = if host_native_output {
        "host-native-presence-and-invoke-plan"
    } else {
        "nuis-image-header-size-and-hash"
    }
    .to_owned();
    let output_image_header_required = !host_native_output;
    let emitted = final_emit.actual_emitted == Some(true);
    let path_present = Path::new(&output_path).exists();
    let nsld_owned_output = emitted && path_present;
    let output_bytes = if emitted {
        fs::read(&output_path).ok()
    } else {
        None
    };
    let present = nsld_owned_output && output_bytes.is_some();
    let size_bytes = output_bytes.as_ref().map(Vec::len);
    let output_hash = output_bytes.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let output_header = output_bytes
        .as_ref()
        .and_then(|bytes| parse_final_executable_image_header(bytes));
    let output_image_magic = output_header.as_ref().map(|header| header.magic.clone());
    let output_image_version = output_header.as_ref().map(|header| header.version as usize);
    let output_image_header_size = output_header.as_ref().map(|header| header.header_size);
    let output_payload_byte_offset = output_header.as_ref().map(|header| header.payload_offset);
    let output_payload_byte_span = output_header.as_ref().map(|header| header.payload_span);
    let output_layout_hash = output_header
        .as_ref()
        .map(|header| header.layout_hash.clone());
    let output_byte_map_hash = output_header
        .as_ref()
        .map(|header| header.byte_map_hash.clone());
    let scheduler_metadata_payload_id = image_dry_run.actual_scheduler_metadata_payload_id.clone();
    let scheduler_metadata_present = image_dry_run.actual_scheduler_metadata_present;
    let scheduler_metadata_offset = image_dry_run.actual_scheduler_metadata_offset;
    let scheduler_metadata_hash = image_dry_run.actual_scheduler_metadata_hash.clone();
    let output_image_header_valid = output_header.as_ref().is_some_and(|header| {
        let payload_end = header.payload_offset.saturating_add(header.payload_span);
        header.magic == FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT
            && header.version == FINAL_EXECUTABLE_IMAGE_VERSION
            && header.header_size == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
            && header.payload_offset == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
            && size_bytes.is_some_and(|size| payload_end <= size)
    });
    let expected_image_size_bytes = final_emit.actual_image_dry_run_size_bytes;
    let expected_image_hash = final_emit.actual_image_dry_run_hash.clone();
    let expected_image_resolver_status = final_emit.actual_image_dry_run_resolver_status.clone();
    let expected_image_patch_application_status = final_emit
        .actual_image_dry_run_patch_application_status
        .clone();
    let expected_image_patch_byte_audit_status = final_emit
        .actual_image_dry_run_patch_byte_audit_status
        .clone();
    let expected_image_patch_byte_audit_hash = final_emit
        .actual_image_dry_run_patch_byte_audit_hash
        .clone();
    let matches_expected_image = present
        && size_bytes == expected_image_size_bytes
        && output_hash == expected_image_hash
        && expected_image_hash.is_some();
    let matches_verified_patched_image = matches_expected_image
        && expected_image_resolver_status.as_deref() == Some("resolved")
        && expected_image_patch_application_status.as_deref() == Some("applied")
        && expected_image_patch_byte_audit_status.as_deref() == Some("verified")
        && expected_image_patch_byte_audit_hash.is_some();
    let mut blockers = Vec::new();
    let mut issues = Vec::new();

    if !final_stage.valid {
        blockers.push("final-stage-plan:invalid".to_owned());
        issues.extend(
            final_stage
                .issues
                .iter()
                .map(|issue| format!("final-stage-plan:{issue}")),
        );
    }
    if !final_emit.valid {
        blockers.push("final-executable-emit:invalid".to_owned());
        issues.extend(
            final_emit
                .issues
                .iter()
                .map(|issue| format!("final-executable-emit:{issue}")),
        );
    }
    if final_emit.actual_emitted != Some(true) {
        blockers.push("final-executable-emit:not-emitted".to_owned());
    }
    if !present {
        if !path_present {
            blockers.push("final-executable-output:missing".to_owned());
        } else if !nsld_owned_output {
            blockers.push("final-executable-output:not-nsld-owned".to_owned());
            issues.push(format!(
                "final executable output path exists but was not emitted by Nsld `{output_path}`"
            ));
        } else {
            blockers.push("final-executable-output:unreadable".to_owned());
            issues.push(format!(
                "missing_or_unreadable_final_executable_output `{output_path}`"
            ));
        }
    }
    if present && !host_native_output && !output_image_header_valid {
        blockers.push("final-executable-output:image-header-invalid".to_owned());
        issues.push(format!(
            "final executable output image header invalid: magic {} version {} header_size {} payload_offset {} payload_span {}",
            output_image_magic.as_deref().unwrap_or("missing"),
            output_image_version
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            output_image_header_size
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            output_payload_byte_offset
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            output_payload_byte_span
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if !host_native_output && final_emit.valid && expected_image_hash.is_none() {
        blockers.push("final-executable-output:expected-image-hash-missing".to_owned());
        issues.push("final executable output cannot be compared because verified image dry-run hash is missing".to_owned());
    }
    if !host_native_output
        && present
        && expected_image_size_bytes.is_some()
        && size_bytes != expected_image_size_bytes
    {
        blockers.push("final-executable-output:size-mismatch".to_owned());
        issues.push(format!(
            "final executable output size mismatch: expected {}, found {}",
            expected_image_size_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            size_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if !host_native_output
        && present
        && expected_image_hash.is_some()
        && output_hash != expected_image_hash
    {
        blockers.push("final-executable-output:hash-mismatch".to_owned());
        issues.push(format!(
            "final executable output hash mismatch: expected {}, found {}",
            expected_image_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned()),
            output_hash.clone().unwrap_or_else(|| "missing".to_owned())
        ));
    }
    if !host_native_output && present && matches_expected_image && !matches_verified_patched_image {
        blockers.push("final-executable-output:verified-patch-evidence-missing".to_owned());
        issues.push(format!(
            "final executable output patch evidence incomplete: resolver={} application={} byte_audit={} byte_audit_hash={}",
            expected_image_resolver_status.as_deref().unwrap_or("missing"),
            expected_image_patch_application_status
                .as_deref()
                .unwrap_or("missing"),
            expected_image_patch_byte_audit_status
                .as_deref()
                .unwrap_or("missing"),
            expected_image_patch_byte_audit_hash
                .as_deref()
                .unwrap_or("missing")
        ));
    }
    let backend_artifacts = nsld_backend_artifact_candidates(plan);
    let backend_artifact_assembly =
        nsld_backend_artifact_assembly_boundary(&backend_artifacts, &final_layout);
    if !backend_artifacts.blockers.is_empty() {
        let mut ordered_blockers = backend_artifacts.blockers.clone();
        ordered_blockers.append(&mut blockers);
        blockers = ordered_blockers;
        issues.extend(backend_artifacts.issues.clone());
    }
    if !backend_artifact_assembly.blockers.is_empty() {
        let mut ordered_blockers = backend_artifact_assembly.blockers.clone();
        ordered_blockers.append(&mut blockers);
        blockers = ordered_blockers;
        issues.extend(backend_artifact_assembly.issues.clone());
    }
    let device_provider_sample = nsld_device_provider_sample_evidence(&plan.output_dir);
    if let Some(blocker) = device_provider_sample.first_blocker.clone() {
        blockers.insert(0, blocker.clone());
        issues.push(format!(
            "device provider sample manifest {} is {}: {}",
            device_provider_sample.path, device_provider_sample.status, blocker
        ));
    }
    let dispatch_blockers = nsld_nustar_dispatch_blockers(plan);
    if !dispatch_blockers.blockers.is_empty() {
        let mut ordered_blockers = dispatch_blockers.blockers;
        ordered_blockers.append(&mut blockers);
        blockers = ordered_blockers;
        issues.extend(dispatch_blockers.issues);
    }

    let runnable_candidate = present
        && final_stage.valid
        && final_emit.valid
        && final_emit.actual_emitted == Some(true)
        && if host_native_output {
            final_emit.actual_host_invoke_plan_would_invoke == Some(true)
        } else {
            matches_verified_patched_image && output_image_header_valid
        };
    let boundary_status = final_executable_output_boundary_status(
        runnable_candidate,
        path_present,
        nsld_owned_output,
        present,
        &blockers,
    )
    .to_owned();
    let materialization_status = final_executable_output_materialization_status(
        boundary_status.as_str(),
        host_native_output,
        output_image_header_valid,
        matches_verified_patched_image,
    )
    .to_owned();
    let launcher_manifest_path = nsld_final_executable_launcher_manifest_path(plan);
    let launcher_manifest_source = fs::read_to_string(&launcher_manifest_path).ok();
    let launcher_manifest_present = launcher_manifest_source.is_some();
    let launcher_manifest_ready = launcher_manifest_source
        .as_deref()
        .and_then(|source| toml::bool_value(source, "ready"));
    let launcher_manifest_blocker_count = launcher_manifest_source
        .as_deref()
        .and_then(|source| toml::usize_value(source, "blocker_count"));
    let launcher_dry_run_path = nsld_final_executable_launcher_dry_run_path(plan);
    let launcher_dry_run_source = fs::read_to_string(&launcher_dry_run_path).ok();
    let launcher_dry_run_present = launcher_dry_run_source.is_some();
    let launcher_dry_run_ready = launcher_dry_run_source
        .as_deref()
        .and_then(|source| toml::bool_value(source, "dry_run_ready"));
    let launcher_dry_run_would_enter_lifecycle_hook = launcher_dry_run_source
        .as_deref()
        .and_then(|source| toml::bool_value(source, "would_enter_lifecycle_hook"));
    let launcher_dry_run_blocker_count = launcher_dry_run_source
        .as_deref()
        .and_then(|source| toml::usize_value(source, "blocker_count"));
    let entrypoint_materialization_evidence_status =
        final_executable_output_entrypoint_materialization_evidence_status(
            boundary_status.as_str(),
            host_native_output,
            launcher_manifest_ready,
            launcher_dry_run_ready,
            launcher_dry_run_would_enter_lifecycle_hook,
        )
        .to_owned();
    let container_loader_evidence =
        final_executable_container_loader_evidence(output_bytes.as_deref(), host_native_output);
    let first_payload_execution = final_executable_first_payload_execution(
        boundary_status.as_str(),
        host_native_output,
        &container_loader_evidence,
    );
    let execution_handoff = final_executable_output_execution_handoff(
        boundary_status.as_str(),
        host_native_output,
        &blockers,
        &first_payload_execution,
    );
    let recommended_next_action = final_executable_output_recommended_next_action(
        boundary_status.as_str(),
        host_native_output,
        entrypoint_materialization_evidence_status.as_str(),
        &first_payload_execution,
    )
    .to_owned();

    NsldFinalExecutableOutputReport {
        manifest: manifest.display().to_string(),
        output_path,
        output_kind,
        output_validation_mode,
        boundary_status,
        materialization_status,
        execution_handoff_contract: execution_handoff.contract,
        execution_handoff_ready: execution_handoff.ready,
        execution_handoff_status: execution_handoff.status,
        execution_handoff_target: execution_handoff.target,
        execution_handoff_evidence_status: execution_handoff.evidence_status,
        execution_handoff_first_blocker: execution_handoff.first_blocker,
        execution_handoff_decision_code: execution_handoff.decision_code,
        entrypoint_materialization_evidence_status,
        launcher_manifest_path: launcher_manifest_path.display().to_string(),
        launcher_manifest_present,
        launcher_manifest_ready,
        launcher_manifest_blocker_count,
        launcher_dry_run_path: launcher_dry_run_path.display().to_string(),
        launcher_dry_run_present,
        launcher_dry_run_ready,
        launcher_dry_run_would_enter_lifecycle_hook,
        launcher_dry_run_blocker_count,
        container_loader_status: container_loader_evidence.status,
        container_loader_payload_scan_kind: container_loader_evidence.payload_scan_kind,
        container_loader_parsed: container_loader_evidence.parsed,
        container_loader_readiness: container_loader_evidence.readiness,
        container_loader_ready: container_loader_evidence.ready,
        container_loader_handoff_status: container_loader_evidence.handoff_status,
        container_loader_handoff_ready: container_loader_evidence.handoff_ready,
        container_loader_handoff_first_blocker: container_loader_evidence.handoff_first_blocker,
        container_loader_entry_symbol: container_loader_evidence.entry_symbol,
        container_loader_entry_kind: container_loader_evidence.entry_kind,
        container_loader_entry_section_id: container_loader_evidence.entry_section_id,
        container_loader_symbol_count: container_loader_evidence.symbol_count,
        first_payload_execution_status: first_payload_execution.status,
        first_payload_execution_ready: first_payload_execution.ready,
        first_payload_execution_target: first_payload_execution.target,
        first_payload_execution_entry_symbol: first_payload_execution.entry_symbol,
        first_payload_execution_entry_kind: first_payload_execution.entry_kind,
        first_payload_execution_entry_section_id: first_payload_execution.entry_section_id,
        first_payload_execution_first_blocker: first_payload_execution.first_blocker,
        device_provider_sample_manifest_available: device_provider_sample.available,
        device_provider_sample_manifest_path: device_provider_sample.path,
        device_provider_sample_manifest_status: device_provider_sample.status,
        device_provider_sample_manifest_record_count: device_provider_sample.record_count,
        device_provider_sample_manifest_ready_record_count: device_provider_sample
            .ready_record_count,
        device_provider_sample_manifest_pending_record_count: device_provider_sample
            .pending_record_count,
        device_provider_sample_manifest_first_provider_family: device_provider_sample
            .first_provider_family,
        device_provider_sample_manifest_first_materialization_status: device_provider_sample
            .first_materialization_status,
        device_provider_sample_manifest_first_blocker: device_provider_sample.first_blocker,
        recommended_next_action,
        path_present,
        nsld_owned_output,
        present,
        size_bytes,
        output_hash,
        output_image_header_required,
        output_image_header_valid,
        output_image_magic,
        output_image_version,
        output_image_header_size,
        output_payload_byte_offset,
        output_payload_byte_span,
        output_layout_hash,
        output_byte_map_hash,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_offset,
        scheduler_metadata_hash,
        expected_image_size_bytes,
        expected_image_hash,
        matches_expected_image,
        expected_image_resolver_status,
        expected_image_patch_application_status,
        expected_image_patch_byte_audit_status,
        expected_image_patch_byte_audit_hash,
        matches_verified_patched_image,
        final_stage_plan_valid: final_stage.valid,
        final_stage_plan_hash: final_stage.actual_plan_hash,
        final_executable_emit_valid: final_emit.valid,
        final_executable_emitted: final_emit.actual_emitted,
        final_executable_blocker_count: final_emit.actual_blocker_count,
        object_output_valid: object_output.valid,
        object_output_path: object_output.object_output_path,
        object_output_expected_size_bytes: object_output.expected_size_bytes,
        object_output_actual_size_bytes: object_output.actual_size_bytes,
        object_output_expected_hash: object_output.expected_hash,
        object_output_actual_hash: object_output.actual_hash,
        object_output_issues: object_output.issues,
        runnable_candidate,
        backend_artifact_candidate_count: backend_artifacts.candidate_count,
        backend_artifact_ready_count: backend_artifacts.ready_count,
        backend_artifact_selection_status: backend_artifacts.selection_status,
        backend_artifact_ordered_candidates: backend_artifacts.ordered_candidates,
        backend_artifact_selected_candidate: backend_artifacts.selected_candidate,
        backend_artifact_selection_reason: backend_artifacts.selection_reason,
        backend_artifact_first_unready: backend_artifacts.first_unready,
        backend_artifact_assembly_status: backend_artifact_assembly.status,
        backend_artifact_selected_payload_path: backend_artifact_assembly.selected_payload_path,
        backend_artifact_selected_payload_consumed: backend_artifact_assembly.consumed,
        backend_artifact_assembly_first_blocker: backend_artifact_assembly.first_blocker,
        blockers,
        issues,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldNustarDispatchBlockers {
    blockers: Vec<String>,
    issues: Vec<String>,
}

fn nsld_nustar_dispatch_blockers(plan: &nuisc::linker::LinkPlan) -> NsldNustarDispatchBlockers {
    let mut blockers = Vec::new();
    let mut issues = Vec::new();
    for unit in plan
        .domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
    {
        match nuisc::registry::load_manifest_for_domain(
            Path::new("nustar-packages"),
            &unit.domain_family,
        ) {
            Ok(manifest) => {
                let dispatch = nuisc::registry::dispatch_readiness_summary(&manifest);
                if dispatch.status != "ready" {
                    blockers.push(format!(
                        "nustar-dispatch:{}:{}",
                        nsld_nustar_dispatch_unit_id(unit),
                        dispatch.status
                    ));
                    issues.push(format!(
                        "Nustar dispatch readiness blocked for {} domain {} with status {}",
                        unit.package_id, unit.domain_family, dispatch.status
                    ));
                }
                for signal in dispatch.missing_signals {
                    blockers.push(format!(
                        "nustar-dispatch:{}:missing:{}",
                        nsld_nustar_dispatch_unit_id(unit),
                        signal
                    ));
                }
            }
            Err(error) => {
                blockers.push(format!(
                    "nustar-dispatch:{}:registry-unavailable",
                    nsld_nustar_dispatch_unit_id(unit)
                ));
                issues.push(format!(
                    "Nustar registry manifest unavailable for {} domain {}: {error}",
                    unit.package_id, unit.domain_family
                ));
            }
        }
    }
    NsldNustarDispatchBlockers { blockers, issues }
}

fn nsld_nustar_dispatch_unit_id(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    if unit.package_id.is_empty() {
        unit.domain_family.clone()
    } else {
        unit.package_id.clone()
    }
}

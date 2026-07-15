use super::{
    artifact_chain::{nsld_artifact_stage_kind_path_for_plan, NsldArtifactStageKind},
    container_verify,
    final_executable_layout::{
        final_executable_backend_artifact_payloads, final_executable_byte_map_entries,
        final_executable_payloads, nsld_final_executable_byte_map_hash,
        nsld_final_executable_layout_hash, nsld_final_executable_relocation_application_table_hash,
    },
    final_executable_paths::nsld_final_executable_layout_plan_path,
    final_executable_render::render_final_executable_layout_plan,
    final_stage::nsld_final_stage_plan_report,
    reports::{
        NsldFinalExecutableLayoutPlanEmitReport, NsldFinalExecutableLayoutPlanReport,
        NsldFinalExecutableLayoutPlanVerifyReport, NsldFinalExecutableRelocationApplicationRecord,
        NsldFinalStagePlanReport,
    },
    toml,
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_layout_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLayoutPlanReport {
    let final_stage = nsld_final_stage_plan_report(manifest, plan);
    let native_object = final_stage
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0003.native-object");
    let native_object_path = native_object
        .map(|input| input.path.clone())
        .unwrap_or_else(|| {
            nsld_artifact_stage_kind_path_for_plan(plan, NsldArtifactStageKind::ObjectOutput)
                .display()
                .to_string()
        });
    let mut payloads = final_executable_payloads(&final_stage);
    payloads.extend(final_executable_backend_artifact_payloads(
        plan,
        payloads.len(),
    ));
    let payload_names = payloads
        .iter()
        .map(|payload| payload.payload_kind.clone())
        .collect::<Vec<_>>();
    let byte_alignment = 16;
    let byte_map_entries = final_executable_byte_map_entries(&payloads, byte_alignment);
    let byte_span = byte_map_entries
        .last()
        .map(|entry| entry.offset + entry.size_bytes)
        .unwrap_or(0);
    let byte_map_hash = nsld_final_executable_byte_map_hash(&byte_map_entries);
    let relocation_applications =
        final_executable_relocation_application_records(&final_stage, &byte_map_entries);
    let platform_envelope_family = if plan.cpu_target.object_format.is_empty() {
        "host-native".to_owned()
    } else {
        plan.cpu_target.object_format.clone()
    };
    let platform_envelope_policy = if final_stage.host_wrapper_required {
        "compatibility-envelope".to_owned()
    } else {
        "self-contained-envelope".to_owned()
    };
    let internal_binary_format = "nuis-hetero-unified-binary".to_owned();
    let lifecycle_entry_hook = "on_process_start".to_owned();
    let scheduler_contract = "deterministic-lifecycle-hook-order".to_owned();
    let scheduler_metadata_payload = "payload0004.scheduler-metadata".to_owned();
    let scheduler_metadata_lifecycle_hook = "on_scheduler_metadata_load".to_owned();
    let scheduler_hetero_node_count = plan.hetero_calculate.nodes.len();
    let scheduler_wait_event_count = plan
        .hetero_calculate
        .nodes
        .iter()
        .map(|node| node.wait_on.len())
        .sum::<usize>();
    let scheduler_emit_event_count = plan
        .hetero_calculate
        .nodes
        .iter()
        .map(|node| node.emits.len())
        .sum::<usize>();
    let data_segment_ordering = "deterministic-data-segment-order".to_owned();
    let relocation_application_count = relocation_applications.len();
    let relocation_application_table_hash =
        nsld_final_executable_relocation_application_table_hash(&relocation_applications);
    let relocation_application_table_source = if relocation_applications.is_empty() {
        "none".to_owned()
    } else {
        "payload0000.container".to_owned()
    };
    let relocation_application_strategy = "nsb-loader-relocation-table".to_owned();
    let compatibility_domain = if final_stage.native_object_required {
        "cffi-native-object".to_owned()
    } else {
        "none".to_owned()
    };
    let compatibility_lifecycle_hook = if final_stage.native_object_required {
        "on_cffi_native_object".to_owned()
    } else {
        "none".to_owned()
    };
    let mut notes = final_stage.notes.clone();
    notes.push("final-executable-layout-is-nsld-owned-protocol".to_owned());
    notes.push("platform-envelope-is-compatibility-shell".to_owned());

    let layout_hash = nsld_final_executable_layout_hash(
        &final_stage.plan_hash,
        &final_stage.final_output_path,
        &final_stage.final_stage_link_mode,
        &platform_envelope_family,
        &platform_envelope_policy,
        &internal_binary_format,
        &lifecycle_entry_hook,
        &scheduler_contract,
        &scheduler_metadata_payload,
        &scheduler_metadata_lifecycle_hook,
        scheduler_hetero_node_count,
        scheduler_wait_event_count,
        scheduler_emit_event_count,
        &data_segment_ordering,
        &relocation_application_strategy,
        &relocation_application_table_source,
        relocation_application_count,
        &relocation_application_table_hash,
        &relocation_applications,
        &native_object_path,
        final_stage.native_object_required,
        final_stage.native_object_present,
        &compatibility_domain,
        &compatibility_lifecycle_hook,
        &payloads,
        byte_alignment,
        byte_span,
        &byte_map_hash,
        &byte_map_entries,
        &notes,
    );

    NsldFinalExecutableLayoutPlanReport {
        manifest: final_stage.manifest,
        output_path: final_stage.final_output_path,
        layout_hash,
        final_stage_plan_hash: final_stage.plan_hash,
        final_stage_link_mode: final_stage.final_stage_link_mode,
        platform_envelope_family,
        platform_envelope_policy,
        internal_binary_format,
        lifecycle_entry_hook,
        scheduler_contract,
        scheduler_metadata_payload,
        scheduler_metadata_lifecycle_hook,
        scheduler_hetero_node_count,
        scheduler_wait_event_count,
        scheduler_emit_event_count,
        data_segment_ordering,
        relocation_application_strategy,
        relocation_application_table_source,
        relocation_application_count,
        relocation_application_table_hash,
        relocation_applications,
        native_object_path,
        native_object_required: final_stage.native_object_required,
        native_object_present: final_stage.native_object_present,
        compatibility_domain,
        compatibility_lifecycle_hook,
        payload_count: payloads.len(),
        payloads,
        payload_names,
        byte_alignment,
        byte_span,
        byte_map_hash,
        byte_map_entries,
        notes,
    }
}

pub(crate) fn nsld_emit_final_executable_layout_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableLayoutPlanEmitReport, String> {
    let report = nsld_final_executable_layout_plan_report(manifest, plan);
    let output_path = nsld_final_executable_layout_plan_path(plan);
    fs::write(&output_path, render_final_executable_layout_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld final executable layout plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldFinalExecutableLayoutPlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        layout_hash: report.layout_hash,
        final_stage_plan_hash: report.final_stage_plan_hash,
        payload_count: report.payload_count,
        native_object_present: report.native_object_present,
    })
}

pub(crate) fn nsld_verify_final_executable_layout_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLayoutPlanVerifyReport {
    let expected = nsld_final_executable_layout_plan_report(manifest, plan);
    let expected_source = render_final_executable_layout_plan(&expected);
    let input_path = nsld_final_executable_layout_plan_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_layout_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_layout_hash,
        actual_payload_count,
        actual_payloads,
        actual_payload_entry_count,
        actual_byte_map_entry_count,
        actual_byte_span,
        actual_byte_map_hash,
        actual_lifecycle_entry_hook,
        actual_scheduler_hetero_node_count,
        actual_relocation_application_strategy,
        actual_relocation_application_count,
        actual_relocation_application_table_hash,
        actual_platform_envelope_family,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "layout_hash"),
            toml::usize_value(source, "payload_count"),
            toml::string_array_value(source, "payloads"),
            table_entry_count(source, "payload"),
            table_entry_count(source, "byte_map_entry"),
            toml::usize_value(source, "byte_span"),
            toml::string_value(source, "byte_map_hash"),
            toml::string_value(source, "lifecycle_entry_hook"),
            toml::usize_value(source, "scheduler_hetero_node_count"),
            toml::string_value(source, "relocation_application_strategy"),
            toml::usize_value(source, "relocation_application_count"),
            toml::string_value(source, "relocation_application_table_hash"),
            toml::string_value(source, "platform_envelope_family"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None,
                None,
                Vec::new(),
                0,
                0,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-layout-plan-content-mismatch".to_owned());
        }
        if actual_layout_hash.as_deref() != Some(expected.layout_hash.as_str()) {
            issues.push(format!(
                "layout_hash mismatch: expected {}, found {}",
                expected.layout_hash,
                actual_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payload_count != Some(expected.payload_count) {
            issues.push(format!(
                "payload_count mismatch: expected {}, found {}",
                expected.payload_count,
                actual_payload_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payloads != expected.payload_names {
            issues.push(format!(
                "payloads mismatch: expected [{}], found [{}]",
                expected.payload_names.join(", "),
                actual_payloads.join(", ")
            ));
        }
        if actual_payload_entry_count != expected.payloads.len() {
            issues.push(format!(
                "payload_entry_count mismatch: expected {}, found {}",
                expected.payloads.len(),
                actual_payload_entry_count
            ));
        }
        if actual_byte_map_entry_count != expected.byte_map_entries.len() {
            issues.push(format!(
                "byte_map_entry_count mismatch: expected {}, found {}",
                expected.byte_map_entries.len(),
                actual_byte_map_entry_count
            ));
        }
        if actual_byte_span != Some(expected.byte_span) {
            issues.push(format!(
                "byte_span mismatch: expected {}, found {}",
                expected.byte_span,
                actual_byte_span
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_byte_map_hash.as_deref() != Some(expected.byte_map_hash.as_str()) {
            issues.push(format!(
                "byte_map_hash mismatch: expected {}, found {}",
                expected.byte_map_hash,
                actual_byte_map_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_lifecycle_entry_hook.as_deref() != Some(expected.lifecycle_entry_hook.as_str()) {
            issues.push(format!(
                "lifecycle_entry_hook mismatch: expected {}, found {}",
                expected.lifecycle_entry_hook,
                actual_lifecycle_entry_hook
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_scheduler_hetero_node_count != Some(expected.scheduler_hetero_node_count) {
            issues.push(format!(
                "scheduler_hetero_node_count mismatch: expected {}, found {}",
                expected.scheduler_hetero_node_count,
                actual_scheduler_hetero_node_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_relocation_application_strategy.as_deref()
            != Some(expected.relocation_application_strategy.as_str())
        {
            issues.push(format!(
                "relocation_application_strategy mismatch: expected {}, found {}",
                expected.relocation_application_strategy,
                actual_relocation_application_strategy
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_relocation_application_count != Some(expected.relocation_application_count) {
            issues.push(format!(
                "relocation_application_count mismatch: expected {}, found {}",
                expected.relocation_application_count,
                actual_relocation_application_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_relocation_application_table_hash.as_deref()
            != Some(expected.relocation_application_table_hash.as_str())
        {
            issues.push(format!(
                "relocation_application_table_hash mismatch: expected {}, found {}",
                expected.relocation_application_table_hash,
                actual_relocation_application_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_platform_envelope_family.as_deref()
            != Some(expected.platform_envelope_family.as_str())
        {
            issues.push(format!(
                "platform_envelope_family mismatch: expected {}, found {}",
                expected.platform_envelope_family,
                actual_platform_envelope_family
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldFinalExecutableLayoutPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_layout_hash: expected.layout_hash,
        actual_layout_hash,
        expected_payload_count: expected.payload_count,
        actual_payload_count,
        expected_payloads: expected.payload_names,
        actual_payloads,
        expected_payload_entry_count: expected.payloads.len(),
        actual_payload_entry_count,
        expected_byte_map_entry_count: expected.byte_map_entries.len(),
        actual_byte_map_entry_count,
        expected_byte_span: expected.byte_span,
        actual_byte_span,
        expected_byte_map_hash: expected.byte_map_hash,
        actual_byte_map_hash,
        expected_lifecycle_entry_hook: expected.lifecycle_entry_hook,
        actual_lifecycle_entry_hook,
        expected_scheduler_hetero_node_count: expected.scheduler_hetero_node_count,
        actual_scheduler_hetero_node_count,
        expected_relocation_application_strategy: expected.relocation_application_strategy,
        actual_relocation_application_strategy,
        expected_relocation_application_count: expected.relocation_application_count,
        actual_relocation_application_count,
        expected_relocation_application_table_hash: expected.relocation_application_table_hash,
        actual_relocation_application_table_hash,
        expected_platform_envelope_family: expected.platform_envelope_family,
        actual_platform_envelope_family,
        issues,
    }
}

fn final_executable_relocation_application_records(
    final_stage: &NsldFinalStagePlanReport,
    byte_map_entries: &[super::reports::NsldFinalExecutableByteMapEntry],
) -> Vec<NsldFinalExecutableRelocationApplicationRecord> {
    let Some(container_input) = final_stage
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0000.container")
    else {
        return Vec::new();
    };
    let source = fs::read_to_string(&container_input.path).unwrap_or_default();
    let container_payload_offset = byte_map_entries
        .iter()
        .find(|entry| entry.payload_id == "payload0001.container-payload")
        .map(|entry| entry.offset)
        .unwrap_or(0);

    container_verify::relocation_entries(&source)
        .into_iter()
        .enumerate()
        .map(
            |(index, relocation)| NsldFinalExecutableRelocationApplicationRecord {
                order_index: index,
                relocation_id: relocation.relocation_id,
                relocation_kind: relocation.relocation_kind,
                source_payload_id: "payload0001.container-payload".to_owned(),
                source_section_id: relocation.source_section_id,
                source_offset: relocation.source_offset,
                image_offset: container_payload_offset.saturating_add(relocation.source_offset),
                target_symbol_id: relocation.target_symbol_id,
                addend: relocation.addend,
                application_status: "planned".to_owned(),
            },
        )
        .collect()
}

fn table_entry_count(source: &str, table: &str) -> usize {
    let header = format!("[[{table}]]");
    source.lines().filter(|line| line.trim() == header).count()
}

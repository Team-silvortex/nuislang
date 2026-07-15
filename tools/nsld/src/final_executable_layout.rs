use super::{
    final_executable_image::final_executable_payload_size,
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableByteMapEntry, NsldFinalExecutablePayloadDiagnostic,
        NsldFinalExecutableRelocationApplicationRecord, NsldFinalStagePlanReport,
    },
};

pub(crate) fn final_executable_payloads(
    final_stage: &NsldFinalStagePlanReport,
) -> Vec<NsldFinalExecutablePayloadDiagnostic> {
    let mut payloads = Vec::with_capacity(final_stage.inputs.len());
    for input in &final_stage.inputs {
        let (payload_id, lifecycle_hook) = match input.input_id.as_str() {
            "fsi0000.container" => ("payload0000.container", "on_process_start"),
            "fsi0001.container-payload" => ("payload0001.container-payload", "on_process_start"),
            "fsi0002.closure-snapshot" => ("payload0002.closure-snapshot", "on_debug_metadata"),
            "fsi0003.native-object" => ("payload0003.native-object", "on_cffi_native_object"),
            "fsi0004.scheduler-metadata" => (
                "payload0004.scheduler-metadata",
                "on_scheduler_metadata_load",
            ),
            _ => continue,
        };
        if input.input_id == "fsi0003.native-object" && !final_stage.native_object_required {
            continue;
        }
        payloads.push(NsldFinalExecutablePayloadDiagnostic {
            order_index: input.order_index,
            payload_id: payload_id.to_owned(),
            payload_kind: input.input_kind.clone(),
            lifecycle_hook: lifecycle_hook.to_owned(),
            path: input.path.clone(),
            content_hash: input.content_hash.clone(),
            required: input.required,
            present: input.present,
        });
    }
    payloads
}

pub(crate) fn final_executable_backend_artifact_payloads(
    plan: &nuisc::linker::LinkPlan,
    start_order_index: usize,
) -> Vec<NsldFinalExecutablePayloadDiagnostic> {
    let mut units = plan
        .domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
        .filter(|unit| {
            unit.backend_family.is_some()
                && unit.target_device.is_some()
                && unit.artifact_payload_blob_path.is_some()
                && unit.artifact_payload_format.is_some()
                && unit.artifact_bridge_stub_path.is_some()
        })
        .collect::<Vec<_>>();
    units.sort_by(|left, right| {
        left.backend_priority
            .unwrap_or(usize::MAX)
            .cmp(&right.backend_priority.unwrap_or(usize::MAX))
            .then_with(|| {
                backend_artifact_payload_key(left).cmp(&backend_artifact_payload_key(right))
            })
    });

    units
        .into_iter()
        .enumerate()
        .map(|(index, unit)| {
            let path = unit.artifact_payload_blob_path.clone().unwrap_or_default();
            let present = !path.is_empty() && std::path::Path::new(&path).exists();
            let content_hash = if present {
                std::fs::read(&path)
                    .map(|bytes| fnv1a64_hex(&bytes))
                    .unwrap_or_else(|_| "missing".to_owned())
            } else {
                "missing".to_owned()
            };
            NsldFinalExecutablePayloadDiagnostic {
                order_index: start_order_index + index,
                payload_id: format!("payload{:04}.backend-artifact", start_order_index + index),
                payload_kind: format!(
                    "nustar-backend-artifact:{}",
                    backend_artifact_payload_key(unit)
                ),
                lifecycle_hook: "on_backend_artifact_load".to_owned(),
                path,
                content_hash,
                required: true,
                present,
            }
        })
        .collect()
}

fn backend_artifact_payload_key(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    format!(
        "{}:{}:{}",
        unit.domain_family,
        unit.backend_family.as_deref().unwrap_or("none"),
        unit.target_device.as_deref().unwrap_or("none")
    )
}

pub(crate) fn final_executable_byte_map_entries(
    payloads: &[NsldFinalExecutablePayloadDiagnostic],
    alignment: usize,
) -> Vec<NsldFinalExecutableByteMapEntry> {
    let mut offset = 0usize;
    let mut entries = Vec::with_capacity(payloads.len());
    for payload in payloads {
        offset = align_to(offset, alignment);
        let size_bytes = final_executable_payload_size(payload);
        entries.push(NsldFinalExecutableByteMapEntry {
            order_index: payload.order_index,
            payload_id: payload.payload_id.clone(),
            payload_kind: payload.payload_kind.clone(),
            offset,
            size_bytes,
            alignment,
            content_hash: payload.content_hash.clone(),
        });
        offset += size_bytes;
    }
    entries
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn nsld_final_executable_layout_hash(
    final_stage_plan_hash: &str,
    output_path: &str,
    final_stage_link_mode: &str,
    platform_envelope_family: &str,
    platform_envelope_policy: &str,
    internal_binary_format: &str,
    lifecycle_entry_hook: &str,
    scheduler_contract: &str,
    scheduler_metadata_payload: &str,
    scheduler_metadata_lifecycle_hook: &str,
    scheduler_hetero_node_count: usize,
    scheduler_wait_event_count: usize,
    scheduler_emit_event_count: usize,
    data_segment_ordering: &str,
    relocation_application_strategy: &str,
    relocation_application_table_source: &str,
    relocation_application_count: usize,
    relocation_application_table_hash: &str,
    relocation_applications: &[NsldFinalExecutableRelocationApplicationRecord],
    native_object_path: &str,
    native_object_required: bool,
    native_object_present: bool,
    compatibility_domain: &str,
    compatibility_lifecycle_hook: &str,
    payloads: &[NsldFinalExecutablePayloadDiagnostic],
    byte_alignment: usize,
    byte_span: usize,
    byte_map_hash: &str,
    byte_map_entries: &[NsldFinalExecutableByteMapEntry],
    notes: &[String],
) -> String {
    let mut material = String::new();
    material.push_str(final_stage_plan_hash);
    material.push('\t');
    material.push_str(output_path);
    material.push('\t');
    material.push_str(final_stage_link_mode);
    material.push('\n');
    material.push_str(platform_envelope_family);
    material.push('\t');
    material.push_str(platform_envelope_policy);
    material.push('\t');
    material.push_str(internal_binary_format);
    material.push('\n');
    material.push_str(lifecycle_entry_hook);
    material.push('\t');
    material.push_str(scheduler_contract);
    material.push('\t');
    material.push_str(scheduler_metadata_payload);
    material.push('\t');
    material.push_str(scheduler_metadata_lifecycle_hook);
    material.push('\t');
    material.push_str(&scheduler_hetero_node_count.to_string());
    material.push('\t');
    material.push_str(&scheduler_wait_event_count.to_string());
    material.push('\t');
    material.push_str(&scheduler_emit_event_count.to_string());
    material.push('\t');
    material.push_str(data_segment_ordering);
    material.push('\n');
    material.push_str(relocation_application_strategy);
    material.push('\t');
    material.push_str(relocation_application_table_source);
    material.push('\t');
    material.push_str(&relocation_application_count.to_string());
    material.push('\t');
    material.push_str(relocation_application_table_hash);
    material.push('\n');
    for record in relocation_applications {
        material.push_str("relocation-application\t");
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.relocation_kind);
        material.push('\t');
        material.push_str(&record.source_payload_id);
        material.push('\t');
        material.push_str(&record.source_section_id);
        material.push('\t');
        material.push_str(&record.source_offset.to_string());
        material.push('\t');
        material.push_str(&record.image_offset.to_string());
        material.push('\t');
        material.push_str(&record.target_symbol_id);
        material.push('\t');
        material.push_str(&record.addend.to_string());
        material.push('\t');
        material.push_str(&record.application_status);
        material.push('\n');
    }
    material.push_str(native_object_path);
    material.push('\t');
    material.push_str(if native_object_required {
        "native-object-required"
    } else {
        "native-object-optional"
    });
    material.push('\t');
    material.push_str(if native_object_present {
        "native-object-present"
    } else {
        "native-object-missing"
    });
    material.push('\n');
    material.push_str(compatibility_domain);
    material.push('\t');
    material.push_str(compatibility_lifecycle_hook);
    material.push('\n');
    material.push_str(&byte_alignment.to_string());
    material.push('\t');
    material.push_str(&byte_span.to_string());
    material.push('\t');
    material.push_str(byte_map_hash);
    material.push('\n');
    for payload in payloads {
        material.push_str("payload\t");
        material.push_str(&payload.payload_id);
        material.push('\t');
        material.push_str(&payload.payload_kind);
        material.push('\t');
        material.push_str(&payload.lifecycle_hook);
        material.push('\t');
        material.push_str(&payload.path);
        material.push('\t');
        material.push_str(&payload.content_hash);
        material.push('\t');
        material.push_str(if payload.required {
            "required"
        } else {
            "optional"
        });
        material.push('\t');
        material.push_str(if payload.present {
            "present"
        } else {
            "missing"
        });
        material.push('\n');
    }
    for entry in byte_map_entries {
        material.push_str("byte-map\t");
        material.push_str(&entry.order_index.to_string());
        material.push('\t');
        material.push_str(&entry.payload_id);
        material.push('\t');
        material.push_str(&entry.payload_kind);
        material.push('\t');
        material.push_str(&entry.offset.to_string());
        material.push('\t');
        material.push_str(&entry.size_bytes.to_string());
        material.push('\t');
        material.push_str(&entry.alignment.to_string());
        material.push('\t');
        material.push_str(&entry.content_hash);
        material.push('\n');
    }
    for note in notes {
        material.push_str("note\t");
        material.push_str(note);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

pub(crate) fn nsld_final_executable_byte_map_hash(
    entries: &[NsldFinalExecutableByteMapEntry],
) -> String {
    let mut material = String::new();
    for entry in entries {
        material.push_str(&entry.order_index.to_string());
        material.push('\t');
        material.push_str(&entry.payload_id);
        material.push('\t');
        material.push_str(&entry.payload_kind);
        material.push('\t');
        material.push_str(&entry.offset.to_string());
        material.push('\t');
        material.push_str(&entry.size_bytes.to_string());
        material.push('\t');
        material.push_str(&entry.alignment.to_string());
        material.push('\t');
        material.push_str(&entry.content_hash);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

pub(crate) fn nsld_final_executable_relocation_application_table_hash(
    records: &[NsldFinalExecutableRelocationApplicationRecord],
) -> String {
    let mut material = String::new();
    for record in records {
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.relocation_kind);
        material.push('\t');
        material.push_str(&record.source_payload_id);
        material.push('\t');
        material.push_str(&record.source_section_id);
        material.push('\t');
        material.push_str(&record.source_offset.to_string());
        material.push('\t');
        material.push_str(&record.image_offset.to_string());
        material.push('\t');
        material.push_str(&record.target_symbol_id);
        material.push('\t');
        material.push_str(&record.addend.to_string());
        material.push('\t');
        material.push_str(&record.application_status);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn align_to(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    value.div_ceil(alignment) * alignment
}

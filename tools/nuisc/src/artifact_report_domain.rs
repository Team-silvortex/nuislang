use super::*;

pub(crate) fn domain_build_contract_summary_json(
    summary: &registry::NustarDomainBuildContractSummary,
) -> String {
    let lowering_fields = vec![
        json_string_field("lane_policy", &summary.lowering.lane_policy),
        json_string_field("bridge_surface", &summary.lowering.bridge_surface),
        json_string_field("emission_kind", &summary.lowering.emission_kind),
    ];
    let backend_fields = vec![
        json_string_field("stub_kind", &summary.backend.stub_kind),
        json_string_field("bridge_entry", &summary.backend.bridge_entry),
        json_string_field("submission_mode", &summary.backend.submission_mode),
        json_string_field("wake_policy", &summary.backend.wake_policy),
        json_string_field("scheduler_binding", &summary.backend.scheduler_binding),
        json_optional_string_field("phase_bind", summary.backend.phase_bind.as_deref()),
        json_optional_string_field("phase_submit", summary.backend.phase_submit.as_deref()),
        json_optional_string_field("phase_wait", summary.backend.phase_wait.as_deref()),
        json_optional_string_field("phase_finalize", summary.backend.phase_finalize.as_deref()),
        json_optional_string_field(
            "transport_model",
            summary.backend.transport_model.as_deref(),
        ),
        json_optional_string_field("request_shape", summary.backend.request_shape.as_deref()),
        json_optional_string_field("response_shape", summary.backend.response_shape.as_deref()),
        json_optional_string_field("dispatch_shape", summary.backend.dispatch_shape.as_deref()),
        json_optional_string_field("memory_binding", summary.backend.memory_binding.as_deref()),
        json_optional_string_field(
            "resource_binding",
            summary.backend.resource_binding.as_deref(),
        ),
        json_optional_string_field(
            "completion_model",
            summary.backend.completion_model.as_deref(),
        ),
    ];
    let bridge_fields = vec![
        json_string_field("bridge_surface", &summary.bridge.bridge_surface),
        json_string_field("bridge_entry", &summary.bridge.bridge_entry),
        json_string_field("scheduler_binding", &summary.bridge.scheduler_binding),
        json_string_field("phase_bind", &summary.bridge.phase_bind),
        json_string_field("phase_submit", &summary.bridge.phase_submit),
        json_string_field("phase_wait", &summary.bridge.phase_wait),
        json_string_field("phase_finalize", &summary.bridge.phase_finalize),
        json_string_field("bridge_kind", &summary.bridge.bridge_kind),
    ];
    let host_bridge_fields = vec![
        json_string_field("host_ffi_surface", &summary.host_bridge.host_ffi_surface),
        json_string_field("handle_family", &summary.host_bridge.handle_family),
        json_string_array_field("phase_order", &summary.host_bridge.phase_order),
        json_string_array_field("phase_bind_inputs", &summary.host_bridge.phase_bind_inputs),
        json_string_array_field(
            "phase_bind_outputs",
            &summary.host_bridge.phase_bind_outputs,
        ),
        json_string_array_field(
            "phase_submit_inputs",
            &summary.host_bridge.phase_submit_inputs,
        ),
        json_string_array_field(
            "phase_submit_outputs",
            &summary.host_bridge.phase_submit_outputs,
        ),
        json_string_array_field("phase_wait_inputs", &summary.host_bridge.phase_wait_inputs),
        json_string_array_field(
            "phase_wait_outputs",
            &summary.host_bridge.phase_wait_outputs,
        ),
        json_string_array_field(
            "phase_finalize_inputs",
            &summary.host_bridge.phase_finalize_inputs,
        ),
        json_string_array_field(
            "phase_finalize_outputs",
            &summary.host_bridge.phase_finalize_outputs,
        ),
        json_string_field("phase_bind_wake", &summary.host_bridge.phase_bind_wake),
        json_string_field("phase_submit_wake", &summary.host_bridge.phase_submit_wake),
        json_string_field("phase_wait_wake", &summary.host_bridge.phase_wait_wake),
        json_string_field(
            "phase_finalize_wake",
            &summary.host_bridge.phase_finalize_wake,
        ),
        json_bool_field("bridge_plan_begin", summary.host_bridge.bridge_plan_begin),
        json_bool_field("bridge_plan_end", summary.host_bridge.bridge_plan_end),
    ];
    format!(
        "{{\"lowering\":{{{}}},\"backend\":{{{}}},\"bridge\":{{{}}},\"host_bridge\":{{{}}}}}",
        lowering_fields.join(","),
        backend_fields.join(","),
        bridge_fields.join(","),
        host_bridge_fields.join(","),
    )
}

pub(crate) fn domain_registry_json(
    registration: &registry::NustarDomainRegistration,
    manifest: &registry::NustarPackageManifest,
) -> String {
    let mut fields = registry::domain_registration_json(registration);
    fields.pop();
    fields.push_str(&format!(
        ",\"build_contract\":{}",
        domain_build_contract_summary_json(&registry::domain_build_contract_summary(manifest))
    ));
    fields.push('}');
    fields
}

pub(crate) fn domain_build_unit_json(unit: &aot::BuildManifestDomainBuildUnit) -> String {
    let fields = vec![
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_optional_string_field("abi", unit.abi.as_deref()),
        json_optional_string_field("machine_arch", unit.machine_arch.as_deref()),
        json_optional_string_field("machine_os", unit.machine_os.as_deref()),
        json_optional_string_field("backend_family", unit.backend_family.as_deref()),
        json_optional_string_field(
            "selected_lowering_target",
            unit.selected_lowering_target.as_deref(),
        ),
        json_optional_string_field("artifact_stub_path", unit.artifact_stub_path.as_deref()),
        json_optional_string_field(
            "artifact_payload_path",
            unit.artifact_payload_path.as_deref(),
        ),
        json_optional_string_field(
            "artifact_bridge_stub_path",
            unit.artifact_bridge_stub_path.as_deref(),
        ),
        json_optional_string_field(
            "artifact_payload_blob_path",
            unit.artifact_payload_blob_path.as_deref(),
        ),
        match unit.artifact_payload_blob_bytes {
            Some(value) => json_usize_field("artifact_payload_blob_bytes", value),
            None => "\"artifact_payload_blob_bytes\":null".to_owned(),
        },
        json_optional_string_field(
            "artifact_payload_format",
            unit.artifact_payload_format.as_deref(),
        ),
        json_string_field("contract_family", &unit.contract_family),
        json_string_field("packaging_role", &unit.packaging_role),
    ];
    format!("{{{}}}", fields.join(","))
}

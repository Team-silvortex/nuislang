use crate::{
    append_json_field_strings,
    artifact_doctor::{collect_artifact_output_diagnostics, probe_artifact_doctor},
    artifact_doctor_render::append_artifact_output_diagnostic_json_fields,
    build_report_runtime, json_bool_field, json_field, json_object_array_field,
    json_optional_string_field, json_string_array_field, json_usize_field, runtime_host_yir,
    workflow::append_workflow_link_plan_json_fields,
};
use std::path::Path;

fn build_report_domain_unit_record(unit: &nuisc::aot::BuildManifestDomainBuildUnit) -> String {
    let mut fields = vec![
        json_field("package_id", &unit.package_id),
        json_field("domain_family", &unit.domain_family),
        json_field("contract_family", &unit.contract_family),
        json_field("packaging_role", &unit.packaging_role),
        json_bool_field("heterogeneous", unit.is_heterogeneous()),
    ];
    if let Some(value) = unit.abi.as_deref() {
        fields.push(json_field("abi", value));
    }
    if let Some(value) = unit.machine_arch.as_deref() {
        fields.push(json_field("machine_arch", value));
    }
    if let Some(value) = unit.machine_os.as_deref() {
        fields.push(json_field("machine_os", value));
    }
    if let Some(value) = unit.backend_family.as_deref() {
        fields.push(json_field("backend_family", value));
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        fields.push(json_field("selected_lowering_target", value));
    }
    if let Some(value) = unit.artifact_payload_format.as_deref() {
        fields.push(json_field("artifact_payload_format", value));
    }
    if let Some(value) = unit.artifact_payload_blob_bytes {
        fields.push(json_usize_field("artifact_payload_blob_bytes", value));
    }
    format!("{{{}}}", fields.join(","))
}

fn runtime_session_json_fields(
    manifest_verify: Option<&nuisc::aot::BuildManifestVerifyReport>,
) -> Vec<String> {
    let Some(report) = manifest_verify else {
        return vec![
            json_usize_field("heterogeneous_domain_count", 0),
            json_optional_string_field("bridge_registry_path", None),
            json_usize_field("bridge_registry_units", 0),
            json_usize_field("bridge_registry_checked", 0),
            json_usize_field("bridge_registry_entries_checked", 0),
            json_optional_string_field("host_bridge_plan_index_path", None),
            json_usize_field("host_bridge_plan_units", 0),
            json_usize_field("host_bridge_plan_checked", 0),
            json_usize_field("host_bridge_plan_entries_checked", 0),
            json_optional_string_field("lowering_plan_index_path", None),
            json_usize_field("lowering_plan_units", 0),
            json_usize_field("lowering_plan_index_checked", 0),
            json_usize_field("lowering_plan_entries_checked", 0),
            json_usize_field("domain_payload_blobs_checked", 0),
            json_usize_field("domain_payload_blob_sections_checked", 0),
            json_usize_field("domain_payload_contract_sections_checked", 0),
            json_usize_field("domain_payload_lowering_plans_checked", 0),
            json_usize_field("domain_payload_backend_stubs_checked", 0),
            json_usize_field("domain_payload_bridge_plans_checked", 0),
            json_usize_field("domain_bridge_stubs_checked", 0),
        ];
    };
    vec![
        json_usize_field(
            "heterogeneous_domain_count",
            report.heterogeneous_domain_count,
        ),
        json_optional_string_field(
            "bridge_registry_path",
            report.bridge_registry_path.as_deref(),
        ),
        json_usize_field("bridge_registry_units", report.bridge_registry_units),
        json_usize_field("bridge_registry_checked", report.bridge_registry_checked),
        json_usize_field(
            "bridge_registry_entries_checked",
            report.bridge_registry_entries_checked,
        ),
        json_optional_string_field(
            "host_bridge_plan_index_path",
            report.host_bridge_plan_index_path.as_deref(),
        ),
        json_usize_field("host_bridge_plan_units", report.host_bridge_plan_units),
        json_usize_field("host_bridge_plan_checked", report.host_bridge_plan_checked),
        json_usize_field(
            "host_bridge_plan_entries_checked",
            report.host_bridge_plan_entries_checked,
        ),
        json_optional_string_field(
            "lowering_plan_index_path",
            report.lowering_plan_index_path.as_deref(),
        ),
        json_usize_field("lowering_plan_units", report.lowering_plan_units),
        json_usize_field(
            "lowering_plan_index_checked",
            report.lowering_plan_index_checked,
        ),
        json_usize_field(
            "lowering_plan_entries_checked",
            report.lowering_plan_entries_checked,
        ),
        json_usize_field(
            "domain_payload_blobs_checked",
            report.domain_payload_blobs_checked,
        ),
        json_usize_field(
            "domain_payload_blob_sections_checked",
            report.domain_payload_blob_sections_checked,
        ),
        json_usize_field(
            "domain_payload_contract_sections_checked",
            report.domain_payload_contract_sections_checked,
        ),
        json_usize_field(
            "domain_payload_lowering_plans_checked",
            report.domain_payload_lowering_plans_checked,
        ),
        json_usize_field(
            "domain_payload_backend_stubs_checked",
            report.domain_payload_backend_stubs_checked,
        ),
        json_usize_field(
            "domain_payload_bridge_plans_checked",
            report.domain_payload_bridge_plans_checked,
        ),
        json_usize_field(
            "domain_bridge_stubs_checked",
            report.domain_bridge_stubs_checked,
        ),
    ]
}

pub(crate) fn append_runtime_session_json_fields(
    out: &mut String,
    manifest_verify: Option<&nuisc::aot::BuildManifestVerifyReport>,
) {
    append_json_field_strings(out, runtime_session_json_fields(manifest_verify));
}

fn runtime_load_json_fields(artifact_path: Option<&Path>, artifact_verified: bool) -> Vec<String> {
    if !artifact_verified {
        return runtime_load_unavailable_fields(false, None);
    }
    let Some(path) = artifact_path else {
        return runtime_load_unavailable_fields(false, None);
    };
    match nuis_runtime::RuntimeLoader.load_from_artifact_path(path) {
        Ok(loaded) => {
            let host_consumable = loaded.host_consumable_summary();
            vec![
                json_bool_field("runtime_load_attempted", true),
                json_bool_field("runtime_load_ok", true),
                json_optional_string_field("runtime_load_error", None),
                json_field(
                    "runtime_loaded_lifecycle_entry",
                    &loaded.artifact.lifecycle.bootstrap_entry,
                ),
                json_usize_field("runtime_loaded_domain_units", loaded.domain_units.len()),
                json_usize_field(
                    "runtime_loaded_heterogeneous_units",
                    loaded.heterogeneous_units().count(),
                ),
                json_usize_field(
                    "runtime_loaded_payload_blobs",
                    loaded.domain_payload_blobs.len(),
                ),
                json_usize_field(
                    "runtime_payload_backed_heterogeneous_units",
                    host_consumable.payload_backed_units,
                ),
                json_usize_field(
                    "runtime_cpu_fallback_units",
                    host_consumable.cpu_fallback_units,
                ),
                json_usize_field(
                    "runtime_host_consumable_units",
                    host_consumable.host_consumable_units,
                ),
                json_bool_field(
                    "runtime_loaded_bridge_registry",
                    loaded.bridge_registry.is_some(),
                ),
                json_bool_field(
                    "runtime_loaded_host_bridge_plan_index",
                    loaded.host_bridge_plan_index.is_some(),
                ),
            ]
        }
        Err(error) => runtime_load_unavailable_fields(true, Some(&error.to_string())),
    }
}

fn runtime_load_unavailable_fields(attempted: bool, error: Option<&str>) -> Vec<String> {
    vec![
        json_bool_field("runtime_load_attempted", attempted),
        json_bool_field("runtime_load_ok", false),
        json_optional_string_field("runtime_load_error", error),
        json_usize_field("runtime_loaded_domain_units", 0),
        json_usize_field("runtime_loaded_heterogeneous_units", 0),
        json_usize_field("runtime_loaded_payload_blobs", 0),
        json_usize_field("runtime_payload_backed_heterogeneous_units", 0),
        json_usize_field("runtime_cpu_fallback_units", 0),
        json_usize_field("runtime_host_consumable_units", 0),
        json_bool_field("runtime_loaded_bridge_registry", false),
        json_bool_field("runtime_loaded_host_bridge_plan_index", false),
    ]
}

fn runtime_execution_json_fields(
    artifact_path: Option<&Path>,
    artifact_verified: bool,
) -> Vec<String> {
    if !artifact_verified {
        return runtime_execution_unavailable_fields(false, None);
    }
    let Some(path) = artifact_path else {
        return runtime_execution_unavailable_fields(false, None);
    };
    match runtime_execution_summary(path) {
        Ok((
            domains,
            plan_phases,
            trace_events,
            host_fallback_events,
            kernel_host_reference_events,
        )) => vec![
            json_bool_field("runtime_execution_attempted", true),
            json_bool_field("runtime_execution_ok", true),
            json_optional_string_field("runtime_execution_error", None),
            json_usize_field("runtime_execution_domains", domains),
            json_usize_field("runtime_execution_plan_phases", plan_phases),
            json_usize_field("runtime_execution_trace_events", trace_events),
            json_usize_field(
                "runtime_execution_host_fallback_events",
                host_fallback_events,
            ),
            json_usize_field(
                "runtime_execution_kernel_host_reference_events",
                kernel_host_reference_events,
            ),
        ],
        Err(error) => runtime_execution_unavailable_fields(true, Some(&error)),
    }
}

fn runtime_execution_unavailable_fields(attempted: bool, error: Option<&str>) -> Vec<String> {
    vec![
        json_bool_field("runtime_execution_attempted", attempted),
        json_bool_field("runtime_execution_ok", false),
        json_optional_string_field("runtime_execution_error", error),
        json_usize_field("runtime_execution_domains", 0),
        json_usize_field("runtime_execution_plan_phases", 0),
        json_usize_field("runtime_execution_trace_events", 0),
        json_usize_field("runtime_execution_host_fallback_events", 0),
        json_usize_field("runtime_execution_kernel_host_reference_events", 0),
    ]
}

pub(crate) fn runtime_execution_summary(
    path: &Path,
) -> Result<(usize, usize, usize, usize, usize), String> {
    let loaded = nuis_runtime::RuntimeLoader
        .load_from_artifact_path(path)
        .map_err(|error| error.to_string())?;
    let mut adapters = nuis_runtime::AdapterRegistry::new();
    adapters.register(Box::new(build_report_runtime::BuildReportRuntimeAdapter));
    let bridge = nuis_runtime::BridgeExecutor;
    let executor = nuis_runtime::Executor;
    let mut domains = 0usize;
    let mut plan_phases = 0usize;
    let mut trace_events = 0usize;
    let mut host_fallback_events = 0usize;
    let mut kernel_host_reference_events = 0usize;
    for unit in loaded.heterogeneous_units() {
        let prepared = bridge
            .prepare(&loaded, &adapters, &unit.domain_family)
            .map_err(|error| error.to_string())?;
        let plan = executor
            .plan(&prepared)
            .map_err(|error| error.to_string())?;
        let trace = executor
            .execute_prepared_plan(prepared.adapter, &plan)
            .map_err(|error| error.to_string())?;
        host_fallback_events += trace
            .events
            .iter()
            .filter(|event| event.outcome.status == "host-cpu-fallback-complete")
            .count();
        kernel_host_reference_events += trace
            .events
            .iter()
            .filter(|event| event.outcome.status == "kernel-host-reference-dispatch-complete")
            .count();
        domains += 1;
        plan_phases += plan.phases.len();
        trace_events += trace.events.len();
    }
    Ok((
        domains,
        plan_phases,
        trace_events,
        host_fallback_events,
        kernel_host_reference_events,
    ))
}

pub(crate) fn render_build_report_json(input: &Path) -> String {
    let doctor = probe_artifact_doctor(input);
    let diagnostics = collect_artifact_output_diagnostics(input, &doctor);
    let manifest_verify = doctor
        .manifest_path
        .as_ref()
        .filter(|_| doctor.manifest_verified)
        .and_then(|path| nuisc::aot::verify_build_manifest(path).ok());
    let artifact_verify = doctor
        .artifact_path
        .as_ref()
        .filter(|_| doctor.artifact_verified)
        .and_then(|path| nuisc::aot::verify_nuis_compiled_artifact(path).ok());
    let domain_unit_records = manifest_verify
        .as_ref()
        .map(|report| {
            report
                .domain_build_units
                .iter()
                .map(build_report_domain_unit_record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "build_report"),
            json_field("source_kind", &doctor.source_kind),
            json_field("input", &doctor.input.display().to_string()),
            json_optional_string_field(
                "output_dir",
                doctor
                    .output_dir
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_optional_string_field(
                "manifest_path",
                doctor
                    .manifest_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_optional_string_field(
                "artifact_path",
                doctor
                    .artifact_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_optional_string_field(
                "binary_path",
                doctor
                    .binary_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_bool_field("manifest_verified", doctor.manifest_verified),
            json_bool_field("artifact_verified", doctor.artifact_verified),
            json_bool_field("ready_to_run", doctor.ready_to_run),
            json_field("recommended_next_step", &doctor.recommended_next_step),
            json_field("recommended_command", &doctor.recommended_command),
            json_field("recommended_reason", &doctor.recommended_reason),
            json_usize_field("domain_units_count", domain_unit_records.len()),
            json_object_array_field("domain_units", &domain_unit_records),
        ],
    );
    append_artifact_output_diagnostic_json_fields(
        &mut out,
        &diagnostics,
        "self_check_ready",
        "self_check_code",
        "self_check_error",
        true,
    );
    if let Some(report) = manifest_verify.as_ref() {
        append_json_field_strings(
            &mut out,
            vec![
                json_usize_field(
                    "text_handle_rewrite_helper_hits",
                    report.project_text_handle_rewrite_helper_hits,
                ),
                json_usize_field(
                    "text_handle_rewrite_local_hits",
                    report.project_text_handle_rewrite_local_hits,
                ),
                json_usize_field(
                    "text_handle_rewrite_total_hits",
                    report.project_text_handle_rewrite_helper_hits
                        + report.project_text_handle_rewrite_local_hits,
                ),
                json_field("packaging_mode", &report.packaging_mode),
                json_field("binary_name", &report.artifact_binary_name),
                json_usize_field("binary_bytes", report.artifact_binary_bytes),
                json_field("lifecycle_schema", &report.lifecycle_schema),
                json_field(
                    "lifecycle_bootstrap_entry",
                    &report.lifecycle_bootstrap_entry,
                ),
                json_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
                json_field(
                    "lifecycle_shutdown_policy",
                    &report.lifecycle_shutdown_policy,
                ),
                json_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
                json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
                json_string_array_field(
                    "lifecycle_export_surface",
                    &report.lifecycle_export_surface,
                ),
                json_string_array_field(
                    "lifecycle_runtime_capability_flags",
                    &report.lifecycle_runtime_capability_flags,
                ),
                json_field("cpu_target_abi", &report.cpu_target_abi),
                json_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
                json_field("cpu_target_machine_os", &report.cpu_target_machine_os),
            ],
        );
    }
    if let Some(report) = artifact_verify.as_ref() {
        append_json_field_strings(
            &mut out,
            vec![
                json_bool_field(
                    "artifact_roundtrip_verified",
                    report.artifact_roundtrip_verified,
                ),
                json_bool_field(
                    "lifecycle_contract_consistent",
                    report.lifecycle_contract_consistent,
                ),
                json_bool_field(
                    "lifecycle_runtime_capability_flags_consistent",
                    report.lifecycle_runtime_capability_flags_consistent,
                ),
            ],
        );
    }
    append_runtime_session_json_fields(&mut out, manifest_verify.as_ref());
    append_json_field_strings(
        &mut out,
        runtime_load_json_fields(doctor.artifact_path.as_deref(), doctor.artifact_verified),
    );
    append_json_field_strings(
        &mut out,
        runtime_execution_json_fields(doctor.artifact_path.as_deref(), doctor.artifact_verified),
    );
    append_json_field_strings(
        &mut out,
        runtime_host_yir::runtime_host_yir_json_fields(
            doctor.artifact_path.as_deref(),
            doctor.artifact_verified,
        ),
    );
    append_workflow_link_plan_json_fields(&mut out, diagnostics.link_plan.plan.as_ref());
    out.push('}');
    out
}

use crate::{
    append_json_field_strings,
    artifact_doctor::{ArtifactOutputDiagnostics, ProjectValidationSnapshot},
    json_bool_field, json_field, json_object_array_field, json_optional_bool_field,
    json_optional_string_field, json_string_array_field, json_surface, json_usize_field,
    run_artifact::run_artifact_prelaunch_summary,
    workflow::{
        append_workflow_link_plan_json_fields, artifact_lowering_units_json,
        project_abi_checks_json, project_domain_registry_checks_json, project_lowering_checks_json,
    },
};
use std::path::Path;

pub(crate) fn append_artifact_output_diagnostic_json_fields(
    out: &mut String,
    diagnostics: &ArtifactOutputDiagnostics,
    self_check_ready_key: &str,
    self_check_code_key: &str,
    self_check_error_key: &str,
    include_project_details: bool,
) {
    append_json_field_strings(
        out,
        vec![
            json_field(
                "artifact_diagnostic_code",
                diagnostics.artifact_diagnostic_code,
            ),
            json_bool_field(self_check_ready_key, diagnostics.self_check.ready),
            json_field(self_check_code_key, diagnostics.self_check.code),
            json_optional_string_field(
                self_check_error_key,
                diagnostics.self_check.error.as_deref(),
            ),
        ],
    );
    append_project_validation_summary_json_fields(
        out,
        diagnostics.project_checks.snapshot.as_ref(),
        include_project_details,
    );
    append_json_field_strings(
        out,
        vec![json_field(
            "project_checks_code",
            diagnostics.project_checks.code,
        )],
    );
}

fn append_project_validation_summary_json_fields(
    out: &mut String,
    snapshot: Option<&ProjectValidationSnapshot>,
    include_details: bool,
) {
    append_json_field_strings(
        out,
        vec![json_bool_field(
            "project_checks_available",
            snapshot.is_some(),
        )],
    );
    if let Some(snapshot) = snapshot {
        append_json_field_strings(
            out,
            vec![json_field(
                "project_checks_root",
                &snapshot.project_root.display().to_string(),
            )],
        );
        append_json_field_strings(
            out,
            json_surface::project_check_summary_json_fields(
                &snapshot.abi_checks,
                &snapshot.registry_checks,
                &snapshot.lowering_checks,
            ),
        );
        if include_details {
            append_json_field_strings(
                out,
                vec![
                    json_object_array_field(
                        "abi_checks",
                        &project_abi_checks_json(&snapshot.abi_checks),
                    ),
                    json_object_array_field(
                        "registry_checks",
                        &project_domain_registry_checks_json(&snapshot.registry_checks),
                    ),
                    json_object_array_field(
                        "lowering_checks",
                        &project_lowering_checks_json(&snapshot.lowering_checks),
                    ),
                ],
            );
        }
    }
}

pub(crate) fn render_artifact_doctor_json(input: &Path) -> String {
    let report = crate::artifact_doctor::probe_artifact_doctor(input);
    let diagnostics = crate::artifact_doctor::collect_artifact_output_diagnostics(input, &report);
    let output_dir = report
        .output_dir
        .as_ref()
        .map(|path| path.display().to_string());
    let manifest_path = report
        .manifest_path
        .as_ref()
        .map(|path| path.display().to_string());
    let artifact_path = report
        .artifact_path
        .as_ref()
        .map(|path| path.display().to_string());
    let binary_path = report
        .binary_path
        .as_ref()
        .map(|path| path.display().to_string());
    let resolved_binary = report.binary_path.as_deref().filter(|path| path.exists());
    let artifact_closure =
        run_artifact_prelaunch_summary(report.output_dir.as_deref(), resolved_binary);
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "artifact_doctor"),
            json_field("source_kind", &report.source_kind),
            json_field("input", &report.input.display().to_string()),
            json_optional_string_field("output_dir", output_dir.as_deref()),
            json_optional_string_field("manifest_path", manifest_path.as_deref()),
            json_optional_string_field("artifact_path", artifact_path.as_deref()),
            json_optional_string_field("binary_path", binary_path.as_deref()),
            json_bool_field("manifest_exists", report.manifest_exists),
            json_bool_field("artifact_exists", report.artifact_exists),
            json_bool_field("binary_exists", report.binary_exists),
            json_bool_field("manifest_verified", report.manifest_verified),
            json_bool_field("artifact_verified", report.artifact_verified),
            json_optional_string_field(
                "artifact_container_kind",
                report.artifact_container_kind.as_deref(),
            ),
            match report.artifact_container_version {
                Some(version) => format!("\"artifact_container_version\":{}", version),
                None => "\"artifact_container_version\":null".to_owned(),
            },
            match report.artifact_section_count {
                Some(count) => json_usize_field("artifact_section_count", count),
                None => "\"artifact_section_count\":null".to_owned(),
            },
            json_string_array_field("artifact_section_names", &report.artifact_section_names),
            match report.artifact_section_table_valid {
                Some(valid) => json_bool_field("artifact_section_table_valid", valid),
                None => "\"artifact_section_table_valid\":null".to_owned(),
            },
            match report.lowering_unit_count {
                Some(count) => json_usize_field("lowering_unit_count", count),
                None => "\"lowering_unit_count\":null".to_owned(),
            },
            json_string_array_field("lowering_domain_families", &report.lowering_domain_families),
            json_string_array_field("lowering_targets", &report.lowering_targets),
            artifact_lowering_units_json(&report.lowering_units),
            json_bool_field("ready_to_run", report.ready_to_run),
            json_field("artifact_closure_kind", &artifact_closure.kind),
            json_field("artifact_closure_status", &artifact_closure.status),
            json_field(
                "artifact_closure_evidence_status",
                &artifact_closure.evidence_status,
            ),
            json_optional_string_field(
                "artifact_closure_command",
                artifact_closure.command.as_deref(),
            ),
            json_bool_field(
                "artifact_closure_runner_command_present",
                artifact_closure.runner_command_present,
            ),
            json_optional_string_field(
                "artifact_closure_entrypoint_path",
                artifact_closure.entrypoint_path.as_deref(),
            ),
            json_bool_field(
                "artifact_closure_entrypoint_present",
                artifact_closure.entrypoint_present,
            ),
            json_optional_string_field(
                "artifact_closure_entrypoint_protocol",
                artifact_closure.entrypoint_protocol.as_deref(),
            ),
            json_optional_bool_field(
                "artifact_closure_entrypoint_protocol_valid",
                artifact_closure.entrypoint_protocol_valid,
            ),
            json_field("artifact_closure_reason", &artifact_closure.reason),
            json_field("recommended_next_step", &report.recommended_next_step),
            json_field("recommended_command", &report.recommended_command),
            json_field("recommended_reason", &report.recommended_reason),
            json_optional_string_field(
                "manifest_verify_error",
                report.manifest_verify_error.as_deref(),
            ),
            json_optional_string_field(
                "artifact_verify_error",
                report.artifact_verify_error.as_deref(),
            ),
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
    append_workflow_link_plan_json_fields(&mut out, diagnostics.link_plan.plan.as_ref());
    out.push('}');
    out
}

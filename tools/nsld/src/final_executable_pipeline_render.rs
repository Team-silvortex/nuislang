use super::{reports::NsldFinalExecutablePipelineEmitReport, toml};

pub(crate) fn render_final_executable_pipeline(
    report: &NsldFinalExecutablePipelineEmitReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-pipeline-v1\"\n");
    push_str_field(&mut out, "manifest", &report.manifest);
    out.push_str(&format!("valid = {}\n", report.valid));
    push_str_field(
        &mut out,
        "final_stage_plan_path",
        &report.final_stage_plan_path,
    );
    push_str_field(&mut out, "final_output_path", &report.final_output_path);
    push_str_field(&mut out, "writer_input_path", &report.writer_input_path);
    push_str_field(
        &mut out,
        "host_invoke_plan_path",
        &report.host_invoke_plan_path,
    );
    push_str_field(&mut out, "layout_plan_path", &report.layout_plan_path);
    push_str_field(&mut out, "image_dry_run_path", &report.image_dry_run_path);
    push_str_field(
        &mut out,
        "final_executable_blocked_path",
        &report.final_executable_blocked_path,
    );
    push_str_field(
        &mut out,
        "launcher_manifest_path",
        &report.launcher_manifest_path,
    );
    push_str_field(
        &mut out,
        "launcher_dry_run_path",
        &report.launcher_dry_run_path,
    );
    out.push_str(&format!(
        "final_executable_emitted = {}\n",
        report.final_executable_emitted
    ));
    out.push_str(&format!(
        "launcher_manifest_ready = {}\n",
        report.launcher_manifest_ready
    ));
    out.push_str(&format!(
        "launcher_dry_run_ready = {}\n",
        report.launcher_dry_run_ready
    ));
    out.push_str(&format!(
        "would_enter_lifecycle_hook = {}\n",
        report.would_enter_lifecycle_hook
    ));
    push_str_field(
        &mut out,
        "self_owned_image_status",
        &report.self_owned_image_status,
    );
    push_str_field(
        &mut out,
        "entrypoint_materialization_status",
        &report.entrypoint_materialization_status,
    );
    push_str_field(
        &mut out,
        "entrypoint_materialization_kind",
        &report.entrypoint_materialization_kind,
    );
    push_optional_str_field(
        &mut out,
        "entrypoint_materialization_path",
        report.entrypoint_materialization_path.as_deref(),
    );
    out.push_str(&format!(
        "entrypoint_materialization_ready = {}\n",
        report.entrypoint_materialization_ready
    ));
    push_optional_str_field(
        &mut out,
        "entrypoint_materialization_first_blocker",
        report.entrypoint_materialization_first_blocker.as_deref(),
    );
    out.push_str(&format!(
        "entrypoint_materialization_present = {}\n",
        report.entrypoint_materialization_present.unwrap_or(false)
    ));
    push_optional_str_field(
        &mut out,
        "entrypoint_materialization_hash",
        report.entrypoint_materialization_hash.as_deref(),
    );
    push_optional_str_field(
        &mut out,
        "entrypoint_materialization_runner_command",
        report.entrypoint_materialization_runner_command.as_deref(),
    );
    push_str_field(
        &mut out,
        "execution_handoff_contract",
        &report.execution_handoff_contract,
    );
    out.push_str(&format!(
        "execution_handoff_ready = {}\n",
        report.execution_handoff_ready
    ));
    push_str_field(
        &mut out,
        "execution_handoff_status",
        &report.execution_handoff_status,
    );
    push_str_field(
        &mut out,
        "execution_handoff_target",
        &report.execution_handoff_target,
    );
    push_str_field(
        &mut out,
        "execution_handoff_evidence_status",
        &report.execution_handoff_evidence_status,
    );
    push_optional_str_field(
        &mut out,
        "execution_handoff_first_blocker",
        report.execution_handoff_first_blocker.as_deref(),
    );
    push_str_field(
        &mut out,
        "execution_handoff_decision_code",
        &report.execution_handoff_decision_code,
    );
    push_optional_str_field(
        &mut out,
        "scheduler_metadata_payload_id",
        report.scheduler_metadata_payload_id.as_deref(),
    );
    out.push_str(&format!(
        "scheduler_metadata_present = {}\n",
        report.scheduler_metadata_present.unwrap_or(false)
    ));
    push_optional_str_field(
        &mut out,
        "scheduler_metadata_hash",
        report.scheduler_metadata_hash.as_deref(),
    );
    out.push_str(&format!(
        "required_stage_path_count = {}\n",
        report.required_stage_path_count
    ));
    out.push_str(&format!(
        "required_stage_path_present_count = {}\n",
        report.required_stage_path_present_count
    ));
    out.push_str(&format!(
        "missing_required_stage_paths = [{}]\n",
        quoted_array(&report.missing_required_stage_paths)
    ));
    out.push_str(&format!("blocker_count = {}\n", report.blockers.len()));
    out.push_str(&format!(
        "blockers = [{}]\n",
        quoted_array(&report.blockers)
    ));
    out
}

fn push_str_field(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!(
        "{key} = \"{}\"\n",
        toml::escape_toml_string(value)
    ));
}

fn push_optional_str_field(out: &mut String, key: &str, value: Option<&str>) {
    push_str_field(out, key, value.unwrap_or(""));
}

fn quoted_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", toml::escape_toml_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

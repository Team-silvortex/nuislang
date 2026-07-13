use super::{
    final_executable_paths::{
        nsld_final_executable_blocked_path, nsld_final_executable_host_invoke_plan_path,
        nsld_final_executable_image_dry_run_path, nsld_final_executable_layout_plan_path,
        nsld_final_executable_pipeline_path, nsld_final_executable_writer_input_path,
        nsld_final_stage_plan_path,
    },
    final_stage::{
        nsld_emit_final_executable_host_invoke_plan_report,
        nsld_emit_final_executable_image_dry_run_report,
        nsld_emit_final_executable_launcher_dry_run_report,
        nsld_emit_final_executable_launcher_manifest_report,
        nsld_emit_final_executable_layout_plan_report, nsld_emit_final_executable_report,
        nsld_emit_final_executable_writer_input_report, nsld_emit_final_stage_plan_report,
        nsld_final_executable_launcher_dry_run_report,
        nsld_final_executable_launcher_manifest_report,
    },
    fnv1a64_hex,
    reports::{NsldFinalExecutablePipelineEmitReport, NsldFinalExecutablePipelineVerifyReport},
    toml,
};
use std::{fs, path::Path};

pub(crate) fn nsld_emit_final_executable_pipeline_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutablePipelineEmitReport, String> {
    let final_stage_plan = nsld_emit_final_stage_plan_report(manifest, plan)?;
    let writer_input = nsld_emit_final_executable_writer_input_report(manifest, plan)?;
    let host_invoke_plan = nsld_emit_final_executable_host_invoke_plan_report(manifest, plan)?;
    let layout_plan = nsld_emit_final_executable_layout_plan_report(manifest, plan)?;
    let image_dry_run = nsld_emit_final_executable_image_dry_run_report(manifest, plan)?;
    let final_executable = nsld_emit_final_executable_report(manifest, plan)?;
    let launcher_manifest = nsld_emit_final_executable_launcher_manifest_report(manifest, plan)?;
    let launcher_dry_run = nsld_emit_final_executable_launcher_dry_run_report(manifest, plan)?;
    let launcher_manifest_report = nsld_final_executable_launcher_manifest_report(manifest, plan);
    let launcher_dry_run_report = nsld_final_executable_launcher_dry_run_report(manifest, plan);
    let mut blockers = final_executable.blockers.clone();
    let required_stage_paths = final_executable_pipeline_required_stage_paths(
        final_executable.emitted,
        &final_stage_plan.output_path,
        &final_executable.output_path,
        &writer_input.output_path,
        &host_invoke_plan.output_path,
        &layout_plan.output_path,
        &image_dry_run.output_path,
        &final_executable.blocked_report_path,
        &launcher_manifest.output_path,
        &launcher_dry_run.output_path,
    );
    let missing_required_stage_paths = missing_paths(&required_stage_paths);
    if !launcher_manifest.ready {
        blockers.push("final-executable-launcher-manifest:not-ready".to_owned());
    }
    if !launcher_dry_run.dry_run_ready {
        blockers.push("final-executable-launcher-dry-run:not-ready".to_owned());
    }
    blockers.extend(
        missing_required_stage_paths
            .iter()
            .map(|path| format!("required-stage-path-missing:{path}")),
    );
    let issues = blockers
        .iter()
        .map(|blocker| format!("pipeline:{blocker}"))
        .collect::<Vec<_>>();

    let self_owned_image_status = nsld_pipeline_self_owned_image_status(
        launcher_manifest.ready,
        launcher_manifest_report.nsb_path.as_str(),
        launcher_manifest_report.nsb_present,
        launcher_manifest_report.nsb_hash.as_deref(),
        launcher_manifest_report.image_header_valid,
    )
    .to_owned();
    let entrypoint_materialization_status = nsld_pipeline_entrypoint_materialization_status(
        self_owned_image_status.as_str(),
        launcher_dry_run.dry_run_ready,
        launcher_dry_run_report.would_enter_lifecycle_hook,
    )
    .to_owned();

    let report = NsldFinalExecutablePipelineEmitReport {
        manifest: manifest.display().to_string(),
        valid: blockers.is_empty(),
        final_stage_plan_path: final_stage_plan.output_path,
        final_output_path: final_executable.output_path,
        writer_input_path: writer_input.output_path,
        host_invoke_plan_path: host_invoke_plan.output_path,
        layout_plan_path: layout_plan.output_path,
        image_dry_run_path: image_dry_run.output_path,
        final_executable_blocked_path: final_executable.blocked_report_path,
        launcher_manifest_path: launcher_manifest.output_path,
        launcher_dry_run_path: launcher_dry_run.output_path,
        final_executable_emitted: final_executable.emitted,
        launcher_manifest_ready: launcher_manifest.ready,
        launcher_dry_run_ready: launcher_dry_run.dry_run_ready,
        would_enter_lifecycle_hook: launcher_dry_run.dry_run_ready,
        self_owned_image_status,
        entrypoint_materialization_status,
        scheduler_metadata_payload_id: launcher_manifest_report
            .scheduler_metadata_payload_id
            .clone()
            .or_else(|| {
                launcher_dry_run_report
                    .scheduler_metadata_payload_id
                    .clone()
            }),
        scheduler_metadata_present: launcher_manifest_report
            .scheduler_metadata_present
            .or(launcher_dry_run_report.scheduler_metadata_present),
        scheduler_metadata_hash: launcher_manifest_report
            .scheduler_metadata_hash
            .clone()
            .or_else(|| launcher_dry_run_report.scheduler_metadata_hash.clone()),
        required_stage_path_count: required_stage_paths.len(),
        required_stage_path_present_count: required_stage_paths.len()
            - missing_required_stage_paths.len(),
        missing_required_stage_paths,
        blocker_count: blockers.len(),
        blockers,
        issues,
    };
    let source = render_final_executable_pipeline(&report);
    let output_path = nsld_final_executable_pipeline_path(plan);
    fs::write(&output_path, source).map_err(|error| {
        format!(
            "failed to write nsld final executable pipeline `{}`: {error}",
            output_path.display()
        )
    })?;
    Ok(report)
}

pub(crate) fn nsld_verify_final_executable_pipeline_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutablePipelineVerifyReport {
    let expected = nsld_final_executable_pipeline_snapshot(manifest, plan);
    let expected_source = render_final_executable_pipeline(&expected);
    let expected_hash = fnv1a64_hex(expected_source.as_bytes());
    let input_path = nsld_final_executable_pipeline_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_pipeline `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_hash,
        actual_valid,
        actual_final_executable_emitted,
        actual_launcher_manifest_ready,
        actual_launcher_dry_run_ready,
        actual_would_enter_lifecycle_hook,
        actual_self_owned_image_status,
        actual_entrypoint_materialization_status,
        actual_scheduler_metadata_payload_id,
        actual_scheduler_metadata_present,
        actual_scheduler_metadata_hash,
        actual_required_stage_path_count,
        actual_required_stage_path_present_count,
        actual_missing_required_stage_paths,
        actual_blocker_count,
        actual_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::bool_value(source, "valid"),
            toml::bool_value(source, "final_executable_emitted"),
            toml::bool_value(source, "launcher_manifest_ready"),
            toml::bool_value(source, "launcher_dry_run_ready"),
            toml::bool_value(source, "would_enter_lifecycle_hook"),
            non_empty_toml_string(source, "self_owned_image_status"),
            non_empty_toml_string(source, "entrypoint_materialization_status"),
            non_empty_toml_string(source, "scheduler_metadata_payload_id"),
            toml::bool_value(source, "scheduler_metadata_present"),
            non_empty_toml_string(source, "scheduler_metadata_hash"),
            toml::usize_value(source, "required_stage_path_count"),
            toml::usize_value(source, "required_stage_path_present_count"),
            toml::string_array_value(source, "missing_required_stage_paths"),
            toml::usize_value(source, "blocker_count"),
            toml::string_array_value(source, "blockers"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                None,
                Vec::new(),
            )
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-pipeline-content-mismatch".to_owned());
        }
        push_bool_mismatch(&mut issues, "valid", expected.valid, actual_valid);
        push_bool_mismatch(
            &mut issues,
            "final_executable_emitted",
            expected.final_executable_emitted,
            actual_final_executable_emitted,
        );
        push_bool_mismatch(
            &mut issues,
            "launcher_manifest_ready",
            expected.launcher_manifest_ready,
            actual_launcher_manifest_ready,
        );
        push_bool_mismatch(
            &mut issues,
            "launcher_dry_run_ready",
            expected.launcher_dry_run_ready,
            actual_launcher_dry_run_ready,
        );
        push_bool_mismatch(
            &mut issues,
            "would_enter_lifecycle_hook",
            expected.would_enter_lifecycle_hook,
            actual_would_enter_lifecycle_hook,
        );
        push_optional_string_mismatch(
            &mut issues,
            "self_owned_image_status",
            Some(expected.self_owned_image_status.as_str()),
            actual_self_owned_image_status.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "entrypoint_materialization_status",
            Some(expected.entrypoint_materialization_status.as_str()),
            actual_entrypoint_materialization_status.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "scheduler_metadata_payload_id",
            expected.scheduler_metadata_payload_id.as_deref(),
            actual_scheduler_metadata_payload_id.as_deref(),
        );
        if actual_scheduler_metadata_present != expected.scheduler_metadata_present {
            issues.push(format!(
                "scheduler_metadata_present mismatch: expected {}, found {}",
                optional_bool_text(expected.scheduler_metadata_present),
                optional_bool_text(actual_scheduler_metadata_present)
            ));
        }
        if actual_scheduler_metadata_hash != expected.scheduler_metadata_hash {
            issues.push(format!(
                "scheduler_metadata_hash mismatch: expected {}, found {}",
                expected
                    .scheduler_metadata_hash
                    .as_deref()
                    .unwrap_or("missing"),
                actual_scheduler_metadata_hash
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        push_usize_mismatch(
            &mut issues,
            "required_stage_path_count",
            expected.required_stage_path_count,
            actual_required_stage_path_count,
        );
        push_usize_mismatch(
            &mut issues,
            "required_stage_path_present_count",
            expected.required_stage_path_present_count,
            actual_required_stage_path_present_count,
        );
        if actual_missing_required_stage_paths != expected.missing_required_stage_paths {
            issues.push(format!(
                "missing_required_stage_paths mismatch: expected [{}], found [{}]",
                expected.missing_required_stage_paths.join(", "),
                actual_missing_required_stage_paths.join(", ")
            ));
        }
        if actual_blocker_count != Some(expected.blockers.len()) {
            issues.push(format!(
                "blocker_count mismatch: expected {}, found {}",
                expected.blockers.len(),
                actual_blocker_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_blockers != expected.blockers {
            issues.push(format!(
                "blockers mismatch: expected [{}], found [{}]",
                expected.blockers.join(", "),
                actual_blockers.join(", ")
            ));
        }
    }
    NsldFinalExecutablePipelineVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_pipeline_hash: expected_hash,
        actual_pipeline_hash: actual_hash,
        expected_valid: expected.valid,
        actual_valid,
        expected_final_executable_emitted: expected.final_executable_emitted,
        actual_final_executable_emitted,
        expected_launcher_manifest_ready: expected.launcher_manifest_ready,
        actual_launcher_manifest_ready,
        expected_launcher_dry_run_ready: expected.launcher_dry_run_ready,
        actual_launcher_dry_run_ready,
        expected_would_enter_lifecycle_hook: expected.would_enter_lifecycle_hook,
        actual_would_enter_lifecycle_hook,
        expected_self_owned_image_status: expected.self_owned_image_status,
        actual_self_owned_image_status,
        expected_entrypoint_materialization_status: expected.entrypoint_materialization_status,
        actual_entrypoint_materialization_status,
        expected_scheduler_metadata_payload_id: expected.scheduler_metadata_payload_id,
        actual_scheduler_metadata_payload_id,
        expected_scheduler_metadata_present: expected.scheduler_metadata_present,
        actual_scheduler_metadata_present,
        expected_scheduler_metadata_hash: expected.scheduler_metadata_hash,
        actual_scheduler_metadata_hash,
        expected_required_stage_path_count: expected.required_stage_path_count,
        actual_required_stage_path_count,
        expected_required_stage_path_present_count: expected.required_stage_path_present_count,
        actual_required_stage_path_present_count,
        expected_missing_required_stage_paths: expected.missing_required_stage_paths,
        actual_missing_required_stage_paths,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
    }
}

fn nsld_final_executable_pipeline_snapshot(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutablePipelineEmitReport {
    let final_executable =
        super::final_stage::nsld_verify_final_executable_emit_report(manifest, plan);
    let launcher_manifest =
        super::final_stage::nsld_verify_final_executable_launcher_manifest_report(manifest, plan);
    let launcher_dry_run =
        super::final_stage::nsld_verify_final_executable_launcher_dry_run_report(manifest, plan);
    let mut blockers = final_executable.expected_blockers.clone();
    if launcher_manifest.actual_ready != Some(true) {
        blockers.push("final-executable-launcher-manifest:not-ready".to_owned());
    }
    if launcher_dry_run.actual_dry_run_ready != Some(true) {
        blockers.push("final-executable-launcher-dry-run:not-ready".to_owned());
    }
    let required_stage_paths = final_executable_pipeline_required_stage_paths(
        final_executable.expected_emitted,
        &nsld_final_stage_plan_path(plan).display().to_string(),
        &plan.final_stage.output_path,
        &nsld_final_executable_writer_input_path(plan)
            .display()
            .to_string(),
        &nsld_final_executable_host_invoke_plan_path(plan)
            .display()
            .to_string(),
        &nsld_final_executable_layout_plan_path(plan)
            .display()
            .to_string(),
        &nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        &nsld_final_executable_blocked_path(plan)
            .display()
            .to_string(),
        &launcher_manifest.input_path,
        &launcher_dry_run.input_path,
    );
    let missing_required_stage_paths = missing_paths(&required_stage_paths);
    blockers.extend(
        missing_required_stage_paths
            .iter()
            .map(|path| format!("required-stage-path-missing:{path}")),
    );
    let issues = blockers
        .iter()
        .map(|blocker| format!("pipeline:{blocker}"))
        .collect::<Vec<_>>();
    let self_owned_image_status = nsld_pipeline_self_owned_image_status(
        launcher_manifest.actual_ready == Some(true),
        launcher_manifest.actual_nsb_path.as_deref().unwrap_or(""),
        launcher_manifest.actual_nsb_size_bytes.is_some(),
        launcher_manifest.actual_nsb_hash.as_deref(),
        launcher_manifest.actual_image_header_valid == Some(true),
    )
    .to_owned();
    let entrypoint_materialization_status = nsld_pipeline_entrypoint_materialization_status(
        self_owned_image_status.as_str(),
        launcher_dry_run.actual_dry_run_ready == Some(true),
        launcher_dry_run.actual_would_enter_lifecycle_hook == Some(true),
    )
    .to_owned();

    NsldFinalExecutablePipelineEmitReport {
        manifest: manifest.display().to_string(),
        valid: blockers.is_empty(),
        final_stage_plan_path: nsld_final_stage_plan_path(plan).display().to_string(),
        final_output_path: plan.final_stage.output_path.clone(),
        writer_input_path: nsld_final_executable_writer_input_path(plan)
            .display()
            .to_string(),
        host_invoke_plan_path: nsld_final_executable_host_invoke_plan_path(plan)
            .display()
            .to_string(),
        layout_plan_path: nsld_final_executable_layout_plan_path(plan)
            .display()
            .to_string(),
        image_dry_run_path: nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        final_executable_blocked_path: nsld_final_executable_blocked_path(plan)
            .display()
            .to_string(),
        launcher_manifest_path: launcher_manifest.input_path,
        launcher_dry_run_path: launcher_dry_run.input_path,
        final_executable_emitted: final_executable.expected_emitted,
        launcher_manifest_ready: launcher_manifest.actual_ready == Some(true),
        launcher_dry_run_ready: launcher_dry_run.actual_dry_run_ready == Some(true),
        would_enter_lifecycle_hook: launcher_dry_run.actual_would_enter_lifecycle_hook
            == Some(true),
        self_owned_image_status,
        entrypoint_materialization_status,
        scheduler_metadata_payload_id: launcher_manifest
            .actual_scheduler_metadata_payload_id
            .clone(),
        scheduler_metadata_present: launcher_manifest.actual_scheduler_metadata_present,
        scheduler_metadata_hash: launcher_manifest.actual_scheduler_metadata_hash.clone(),
        required_stage_path_count: required_stage_paths.len(),
        required_stage_path_present_count: required_stage_paths.len()
            - missing_required_stage_paths.len(),
        missing_required_stage_paths,
        blocker_count: blockers.len(),
        blockers,
        issues,
    }
}

fn nsld_pipeline_self_owned_image_status(
    launcher_manifest_ready: bool,
    nsb_path: &str,
    nsb_present: bool,
    nsb_hash: Option<&str>,
    image_header_valid: bool,
) -> &'static str {
    if launcher_manifest_ready && nsb_present && image_header_valid {
        return "ready";
    }
    if nsb_path.is_empty() {
        return "path-missing";
    }
    if !nsb_present {
        return "missing";
    }
    if !image_header_valid {
        return "header-invalid";
    }
    if nsb_hash.is_none() {
        return "hash-missing";
    }
    "blocked"
}

fn nsld_pipeline_entrypoint_materialization_status(
    self_owned_image_status: &str,
    launcher_dry_run_ready: bool,
    would_enter_lifecycle_hook: bool,
) -> &'static str {
    if launcher_dry_run_ready && would_enter_lifecycle_hook {
        return "host-launcher-ready";
    }
    if self_owned_image_status == "ready" {
        return "image-ready-entrypoint-pending";
    }
    "blocked"
}

fn render_final_executable_pipeline(report: &NsldFinalExecutablePipelineEmitReport) -> String {
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
        report
            .missing_required_stage_paths
            .iter()
            .map(|value| format!("\"{}\"", toml::escape_toml_string(value)))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str(&format!("blocker_count = {}\n", report.blockers.len()));
    out.push_str(&format!(
        "blockers = [{}]\n",
        report
            .blockers
            .iter()
            .map(|value| format!("\"{}\"", toml::escape_toml_string(value)))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out
}

fn final_executable_pipeline_required_stage_paths(
    final_executable_emitted: bool,
    final_stage_plan_path: &str,
    final_output_path: &str,
    writer_input_path: &str,
    host_invoke_plan_path: &str,
    layout_plan_path: &str,
    image_dry_run_path: &str,
    final_executable_blocked_path: &str,
    launcher_manifest_path: &str,
    launcher_dry_run_path: &str,
) -> Vec<String> {
    let mut paths = vec![
        final_stage_plan_path.to_owned(),
        writer_input_path.to_owned(),
        host_invoke_plan_path.to_owned(),
        layout_plan_path.to_owned(),
        image_dry_run_path.to_owned(),
        final_executable_blocked_path.to_owned(),
        launcher_manifest_path.to_owned(),
        launcher_dry_run_path.to_owned(),
    ];
    if final_executable_emitted {
        paths.push(final_output_path.to_owned());
    }
    paths
}

fn missing_paths(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| !Path::new(path.as_str()).exists())
        .cloned()
        .collect()
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

fn non_empty_toml_string(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn optional_bool_text(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

fn push_usize_mismatch(
    issues: &mut Vec<String>,
    key: &str,
    expected: usize,
    actual: Option<usize>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{key} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_bool_mismatch(issues: &mut Vec<String>, key: &str, expected: bool, actual: Option<bool>) {
    if actual != Some(expected) {
        issues.push(format!(
            "{key} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_optional_string_mismatch(
    issues: &mut Vec<String>,
    key: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if actual != expected {
        issues.push(format!(
            "{key} mismatch: expected {}, found {}",
            expected.unwrap_or("missing"),
            actual.unwrap_or("missing")
        ));
    }
}

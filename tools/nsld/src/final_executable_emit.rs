use super::{
    final_executable_emit_actual::{
        nsld_final_executable_emit_actual_from_source, NsldFinalExecutableEmitActual,
    },
    final_executable_emit_output_verify::push_final_output_emit_verify_mismatches,
    final_executable_emit_shape::nsld_final_executable_emit_report_shape,
    final_executable_output_summary::populate_final_output_emit_summary,
    final_executable_paths::nsld_final_executable_blocked_path,
    final_executable_render::{
        optional_bool_toml, optional_usize_toml, render_final_executable_blocked,
    },
    reports::{NsldFinalExecutableEmitReport, NsldFinalExecutableEmitVerifyReport},
};
use std::{fs, path::Path, process::Command};

pub(crate) fn nsld_emit_final_executable_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableEmitReport, String> {
    let mut report = nsld_final_executable_emit_report_shape(manifest, plan);
    if report.can_emit_final_executable {
        if let Some(parent) = Path::new(&report.output_path).parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create nsld final executable output directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        if report.host_wrapper_required {
            emit_host_assisted_final_executable(&report)?;
        } else {
            fs::copy(&report.image_dry_run_bytes_path, &report.output_path).map_err(|error| {
                format!(
                    "failed to write nsld final executable output `{}` from `{}`: {error}",
                    report.output_path, report.image_dry_run_bytes_path
                )
            })?;
        }
        report.emitted = true;
        populate_final_output_emit_summary(&mut report);
    }
    let blocked_report_path = nsld_final_executable_blocked_path(plan);
    fs::write(
        &blocked_report_path,
        render_final_executable_blocked(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld final executable blocked report `{}`: {error}",
            blocked_report_path.display()
        )
    })?;
    Ok(report)
}

fn emit_host_assisted_final_executable(
    report: &NsldFinalExecutableEmitReport,
) -> Result<(), String> {
    let (program, args) = report
        .host_dry_run_command_args
        .split_first()
        .ok_or_else(|| "host finalizer command args are empty".to_owned())?;
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("failed to invoke host finalizer driver `{program}`: {error}"))?;
    if !status.success() {
        return Err(format!(
            "host finalizer driver `{program}` exited with status {status}"
        ));
    }
    if !Path::new(&report.output_path).is_file() {
        return Err(format!(
            "host finalizer driver `{program}` completed but did not create `{}`",
            report.output_path
        ));
    }
    Ok(())
}

pub(crate) fn nsld_verify_final_executable_emit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitVerifyReport {
    let mut expected = nsld_final_executable_emit_report_shape(manifest, plan);
    if expected.can_emit_final_executable {
        populate_final_output_emit_summary(&mut expected);
    }
    let input_path = nsld_final_executable_blocked_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_blocked `{}`: {error}",
            input_path.display()
        )
    });
    let NsldFinalExecutableEmitActual {
        final_stage_plan_hash: actual_plan_hash,
        emitted: actual_emitted,
        writer_input_valid: actual_writer_input_valid,
        writer_input_hash: actual_writer_input_hash,
        writer_input_issues: actual_writer_input_issues,
        host_dry_run_environment_ready: actual_host_environment_ready,
        host_dry_run_driver_available: actual_host_driver_available,
        host_dry_run_can_invoke: actual_host_can_invoke,
        host_dry_run_driver_resolved_path: actual_host_driver_resolved_path,
        host_dry_run_invocation_policy: actual_host_invocation_policy,
        host_dry_run_invocation_policy_reason: actual_host_invocation_policy_reason,
        host_dry_run_command_arg_count: actual_host_dry_run_command_arg_count,
        host_dry_run_command_args: actual_host_dry_run_command_args,
        host_dry_run_blocker_count: actual_host_dry_run_blocker_count,
        host_dry_run_blockers: actual_host_dry_run_blockers,
        host_invoke_plan_valid: actual_host_invoke_plan_valid,
        host_invoke_plan_would_invoke: actual_host_invoke_plan_would_invoke,
        host_invoke_plan_hash: actual_host_invoke_plan_hash,
        host_invoke_plan_invocation_policy: actual_host_invoke_plan_invocation_policy,
        host_invoke_plan_requires_explicit_allow: actual_host_invoke_plan_requires_explicit_allow,
        host_invoke_plan_explicit_allow_present: actual_host_invoke_plan_explicit_allow_present,
        host_invoke_plan_blocker_count: actual_host_invoke_plan_blocker_count,
        host_invoke_plan_issues: actual_host_invoke_plan_issues,
        host_finalizer_gate_status: actual_host_finalizer_gate_status,
        host_finalizer_gate_action: actual_host_finalizer_gate_action,
        layout_plan_valid: actual_layout_plan_valid,
        layout_plan_hash: actual_layout_plan_hash,
        layout_plan_issues: actual_layout_plan_issues,
        image_dry_run_valid: actual_image_dry_run_valid,
        image_dry_run_hash: actual_image_dry_run_hash,
        image_dry_run_size_bytes: actual_image_dry_run_size_bytes,
        image_dry_run_resolver_status: actual_image_dry_run_resolver_status,
        image_dry_run_patch_application_status: actual_image_dry_run_patch_application_status,
        image_dry_run_patch_byte_audit_status: actual_image_dry_run_patch_byte_audit_status,
        image_dry_run_patch_byte_audit_hash: actual_image_dry_run_patch_byte_audit_hash,
        image_dry_run_issues: actual_image_dry_run_issues,
        final_output_checked: actual_final_output_checked,
        final_output_present: actual_final_output_present,
        final_output_size_bytes: actual_final_output_size_bytes,
        final_output_hash: actual_final_output_hash,
        final_output_image_header_valid: actual_final_output_image_header_valid,
        final_output_runnable_candidate: actual_final_output_runnable_candidate,
        blocker_count: actual_blocker_count,
        blockers: actual_blockers,
    } = match actual.as_ref() {
        Ok(source) => nsld_final_executable_emit_actual_from_source(source),
        Err(error) => {
            issues.push(error.clone());
            NsldFinalExecutableEmitActual::default()
        }
    };
    if let Ok(actual) = actual {
        let expected_source = render_final_executable_blocked(&expected);
        if actual != expected_source {
            issues.push("final-executable-blocked-content-mismatch".to_owned());
        }
        if actual_plan_hash.as_deref() != Some(expected.final_stage_plan_hash.as_str()) {
            issues.push(format!(
                "final_stage_plan_hash mismatch: expected {}, found {}",
                expected.final_stage_plan_hash,
                actual_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_emitted != Some(expected.emitted) {
            issues.push(format!(
                "emitted mismatch: expected {}, found {}",
                expected.emitted,
                actual_emitted
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_writer_input_valid != expected.writer_input_valid {
            issues.push(format!(
                "writer_input_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.writer_input_valid),
                optional_bool_toml(actual_writer_input_valid)
            ));
        }
        if actual_writer_input_hash != expected.writer_input_hash {
            issues.push(format!(
                "writer_input_hash mismatch: expected {}, found {}",
                expected
                    .writer_input_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_writer_input_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_writer_input_issues != expected.writer_input_issues {
            issues.push(format!(
                "writer_input_issues mismatch: expected [{}], found [{}]",
                expected.writer_input_issues.join(", "),
                actual_writer_input_issues.join(", ")
            ));
        }
        if actual_host_environment_ready != expected.host_dry_run_environment_ready {
            issues.push(format!(
                "host_dry_run_environment_ready mismatch: expected {}, found {}",
                expected
                    .host_dry_run_environment_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_environment_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_driver_available != expected.host_dry_run_driver_available {
            issues.push(format!(
                "host_dry_run_driver_available mismatch: expected {}, found {}",
                expected
                    .host_dry_run_driver_available
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_driver_available
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_can_invoke != expected.host_dry_run_can_invoke {
            issues.push(format!(
                "host_dry_run_can_invoke mismatch: expected {}, found {}",
                expected
                    .host_dry_run_can_invoke
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_can_invoke
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_driver_resolved_path != expected.host_dry_run_driver_resolved_path {
            issues.push(format!(
                "host_dry_run_driver_resolved_path mismatch: expected {}, found {}",
                expected
                    .host_dry_run_driver_resolved_path
                    .as_deref()
                    .unwrap_or("missing"),
                actual_host_driver_resolved_path
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_host_invocation_policy != expected.host_dry_run_invocation_policy {
            issues.push(format!(
                "host_dry_run_invocation_policy mismatch: expected {}, found {}",
                expected
                    .host_dry_run_invocation_policy
                    .as_deref()
                    .unwrap_or("missing"),
                actual_host_invocation_policy
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_host_invocation_policy_reason != expected.host_dry_run_invocation_policy_reason {
            issues.push(format!(
                "host_dry_run_invocation_policy_reason mismatch: expected {}, found {}",
                expected
                    .host_dry_run_invocation_policy_reason
                    .as_deref()
                    .unwrap_or("missing"),
                actual_host_invocation_policy_reason
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_host_dry_run_command_arg_count != Some(expected.host_dry_run_command_args.len()) {
            issues.push(format!(
                "host_dry_run_command_arg_count mismatch: expected {}, found {}",
                expected.host_dry_run_command_args.len(),
                actual_host_dry_run_command_arg_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_dry_run_command_args != expected.host_dry_run_command_args {
            issues.push(format!(
                "host_dry_run_command_args mismatch: expected [{}], found [{}]",
                expected.host_dry_run_command_args.join(", "),
                actual_host_dry_run_command_args.join(", ")
            ));
        }
        if actual_host_dry_run_blockers != expected.host_dry_run_blockers {
            issues.push(format!(
                "host_dry_run_blockers mismatch: expected [{}], found [{}]",
                expected.host_dry_run_blockers.join(", "),
                actual_host_dry_run_blockers.join(", ")
            ));
        }
        if actual_host_dry_run_blocker_count != Some(expected.host_dry_run_blockers.len()) {
            issues.push(format!(
                "host_dry_run_blocker_count mismatch: expected {}, found {}",
                expected.host_dry_run_blockers.len(),
                actual_host_dry_run_blocker_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_invoke_plan_valid != expected.host_invoke_plan_valid {
            issues.push(format!(
                "host_invoke_plan_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.host_invoke_plan_valid),
                optional_bool_toml(actual_host_invoke_plan_valid)
            ));
        }
        if actual_host_invoke_plan_would_invoke != expected.host_invoke_plan_would_invoke {
            issues.push(format!(
                "host_invoke_plan_would_invoke mismatch: expected {}, found {}",
                optional_bool_toml(expected.host_invoke_plan_would_invoke),
                optional_bool_toml(actual_host_invoke_plan_would_invoke)
            ));
        }
        if actual_host_invoke_plan_hash != expected.host_invoke_plan_hash {
            issues.push(format!(
                "host_invoke_plan_hash mismatch: expected {}, found {}",
                expected
                    .host_invoke_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_invoke_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_invoke_plan_invocation_policy != expected.host_invoke_plan_invocation_policy
        {
            issues.push(format!(
                "host_invoke_plan_invocation_policy mismatch: expected {}, found {}",
                expected
                    .host_invoke_plan_invocation_policy
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_invoke_plan_invocation_policy
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_invoke_plan_requires_explicit_allow
            != expected.host_invoke_plan_requires_explicit_allow
        {
            issues.push(format!(
                "host_invoke_plan_requires_explicit_allow mismatch: expected {}, found {}",
                optional_bool_toml(expected.host_invoke_plan_requires_explicit_allow),
                optional_bool_toml(actual_host_invoke_plan_requires_explicit_allow)
            ));
        }
        if actual_host_invoke_plan_explicit_allow_present
            != expected.host_invoke_plan_explicit_allow_present
        {
            issues.push(format!(
                "host_invoke_plan_explicit_allow_present mismatch: expected {}, found {}",
                optional_bool_toml(expected.host_invoke_plan_explicit_allow_present),
                optional_bool_toml(actual_host_invoke_plan_explicit_allow_present)
            ));
        }
        if actual_host_invoke_plan_blocker_count != expected.host_invoke_plan_blocker_count {
            issues.push(format!(
                "host_invoke_plan_blocker_count mismatch: expected {}, found {}",
                optional_usize_toml(expected.host_invoke_plan_blocker_count),
                optional_usize_toml(actual_host_invoke_plan_blocker_count)
            ));
        }
        if actual_host_invoke_plan_issues != expected.host_invoke_plan_issues {
            issues.push(format!(
                "host_invoke_plan_issues mismatch: expected [{}], found [{}]",
                expected.host_invoke_plan_issues.join(", "),
                actual_host_invoke_plan_issues.join(", ")
            ));
        }
        if actual_layout_plan_valid != expected.layout_plan_valid {
            issues.push(format!(
                "layout_plan_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.layout_plan_valid),
                optional_bool_toml(actual_layout_plan_valid)
            ));
        }
        if actual_layout_plan_hash != expected.layout_plan_hash {
            issues.push(format!(
                "layout_plan_hash mismatch: expected {}, found {}",
                expected
                    .layout_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_layout_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_layout_plan_issues != expected.layout_plan_issues {
            issues.push(format!(
                "layout_plan_issues mismatch: expected [{}], found [{}]",
                expected.layout_plan_issues.join(", "),
                actual_layout_plan_issues.join(", ")
            ));
        }
        if actual_image_dry_run_valid != expected.image_dry_run_valid {
            issues.push(format!(
                "image_dry_run_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.image_dry_run_valid),
                optional_bool_toml(actual_image_dry_run_valid)
            ));
        }
        if actual_image_dry_run_hash != expected.image_dry_run_hash {
            issues.push(format!(
                "image_dry_run_hash mismatch: expected {}, found {}",
                expected
                    .image_dry_run_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_image_dry_run_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_dry_run_size_bytes != expected.image_dry_run_size_bytes {
            issues.push(format!(
                "image_dry_run_size_bytes mismatch: expected {}, found {}",
                optional_usize_toml(expected.image_dry_run_size_bytes),
                optional_usize_toml(actual_image_dry_run_size_bytes)
            ));
        }
        if actual_image_dry_run_resolver_status != expected.image_dry_run_resolver_status {
            issues.push(format!(
                "image_dry_run_resolver_status mismatch: expected {}, found {}",
                expected
                    .image_dry_run_resolver_status
                    .as_deref()
                    .unwrap_or("missing"),
                actual_image_dry_run_resolver_status
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_image_dry_run_patch_application_status
            != expected.image_dry_run_patch_application_status
        {
            issues.push(format!(
                "image_dry_run_patch_application_status mismatch: expected {}, found {}",
                expected
                    .image_dry_run_patch_application_status
                    .as_deref()
                    .unwrap_or("missing"),
                actual_image_dry_run_patch_application_status
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_image_dry_run_patch_byte_audit_status
            != expected.image_dry_run_patch_byte_audit_status
        {
            issues.push(format!(
                "image_dry_run_patch_byte_audit_status mismatch: expected {}, found {}",
                expected
                    .image_dry_run_patch_byte_audit_status
                    .as_deref()
                    .unwrap_or("missing"),
                actual_image_dry_run_patch_byte_audit_status
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_image_dry_run_patch_byte_audit_hash
            != expected.image_dry_run_patch_byte_audit_hash
        {
            issues.push(format!(
                "image_dry_run_patch_byte_audit_hash mismatch: expected {}, found {}",
                expected
                    .image_dry_run_patch_byte_audit_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_image_dry_run_patch_byte_audit_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_dry_run_issues != expected.image_dry_run_issues {
            issues.push(format!(
                "image_dry_run_issues mismatch: expected [{}], found [{}]",
                expected.image_dry_run_issues.join(", "),
                actual_image_dry_run_issues.join(", ")
            ));
        }
        push_final_output_emit_verify_mismatches(
            &mut issues,
            &expected,
            actual_final_output_checked,
            actual_final_output_present,
            actual_final_output_size_bytes,
            actual_final_output_hash.clone(),
            actual_final_output_image_header_valid,
            actual_final_output_runnable_candidate,
        );
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

    let expected_host_finalizer_gate_status =
        super::final_executable_render::host_finalizer_gate_status(&expected).to_owned();
    let expected_host_finalizer_gate_action =
        super::final_executable_render::host_finalizer_gate_action(&expected).to_owned();

    NsldFinalExecutableEmitVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_final_stage_plan_hash: expected.final_stage_plan_hash,
        actual_final_stage_plan_hash: actual_plan_hash,
        expected_emitted: expected.emitted,
        actual_emitted,
        expected_writer_input_valid: expected.writer_input_valid,
        actual_writer_input_valid,
        expected_writer_input_hash: expected.writer_input_hash,
        actual_writer_input_hash,
        expected_writer_input_issues: expected.writer_input_issues,
        actual_writer_input_issues,
        expected_host_dry_run_environment_ready: expected.host_dry_run_environment_ready,
        actual_host_dry_run_environment_ready: actual_host_environment_ready,
        expected_host_dry_run_driver_available: expected.host_dry_run_driver_available,
        actual_host_dry_run_driver_available: actual_host_driver_available,
        expected_host_dry_run_can_invoke: expected.host_dry_run_can_invoke,
        actual_host_dry_run_can_invoke: actual_host_can_invoke,
        expected_host_dry_run_driver_resolved_path: expected.host_dry_run_driver_resolved_path,
        actual_host_dry_run_driver_resolved_path: actual_host_driver_resolved_path,
        expected_host_dry_run_invocation_policy: expected.host_dry_run_invocation_policy,
        actual_host_dry_run_invocation_policy: actual_host_invocation_policy,
        expected_host_dry_run_invocation_policy_reason: expected
            .host_dry_run_invocation_policy_reason,
        actual_host_dry_run_invocation_policy_reason: actual_host_invocation_policy_reason,
        expected_host_dry_run_command_arg_count: expected.host_dry_run_command_args.len(),
        actual_host_dry_run_command_arg_count,
        expected_host_dry_run_command_args: expected.host_dry_run_command_args,
        actual_host_dry_run_command_args,
        expected_host_dry_run_blocker_count: expected.host_dry_run_blockers.len(),
        actual_host_dry_run_blocker_count,
        expected_host_dry_run_blockers: expected.host_dry_run_blockers,
        actual_host_dry_run_blockers,
        expected_host_invoke_plan_valid: expected.host_invoke_plan_valid,
        actual_host_invoke_plan_valid,
        expected_host_invoke_plan_would_invoke: expected.host_invoke_plan_would_invoke,
        actual_host_invoke_plan_would_invoke,
        expected_host_invoke_plan_hash: expected.host_invoke_plan_hash,
        actual_host_invoke_plan_hash,
        expected_host_invoke_plan_invocation_policy: expected.host_invoke_plan_invocation_policy,
        actual_host_invoke_plan_invocation_policy,
        expected_host_invoke_plan_requires_explicit_allow: expected
            .host_invoke_plan_requires_explicit_allow,
        actual_host_invoke_plan_requires_explicit_allow,
        expected_host_invoke_plan_explicit_allow_present: expected
            .host_invoke_plan_explicit_allow_present,
        actual_host_invoke_plan_explicit_allow_present,
        expected_host_invoke_plan_blocker_count: expected.host_invoke_plan_blocker_count,
        actual_host_invoke_plan_blocker_count,
        expected_host_invoke_plan_issues: expected.host_invoke_plan_issues,
        actual_host_invoke_plan_issues,
        expected_host_finalizer_gate_status,
        actual_host_finalizer_gate_status,
        expected_host_finalizer_gate_action,
        actual_host_finalizer_gate_action,
        expected_layout_plan_valid: expected.layout_plan_valid,
        actual_layout_plan_valid,
        expected_layout_plan_hash: expected.layout_plan_hash,
        actual_layout_plan_hash,
        expected_layout_plan_issues: expected.layout_plan_issues,
        actual_layout_plan_issues,
        expected_image_dry_run_valid: expected.image_dry_run_valid,
        actual_image_dry_run_valid,
        expected_image_dry_run_hash: expected.image_dry_run_hash,
        actual_image_dry_run_hash,
        expected_image_dry_run_size_bytes: expected.image_dry_run_size_bytes,
        actual_image_dry_run_size_bytes,
        expected_image_dry_run_resolver_status: expected.image_dry_run_resolver_status,
        actual_image_dry_run_resolver_status,
        expected_image_dry_run_patch_application_status: expected
            .image_dry_run_patch_application_status,
        actual_image_dry_run_patch_application_status,
        expected_image_dry_run_patch_byte_audit_status: expected
            .image_dry_run_patch_byte_audit_status,
        actual_image_dry_run_patch_byte_audit_status,
        expected_image_dry_run_patch_byte_audit_hash: expected.image_dry_run_patch_byte_audit_hash,
        actual_image_dry_run_patch_byte_audit_hash,
        expected_image_dry_run_issues: expected.image_dry_run_issues,
        actual_image_dry_run_issues,
        expected_final_output_checked: expected.final_output_checked,
        actual_final_output_checked,
        expected_final_output_present: expected.final_output_present,
        actual_final_output_present,
        expected_final_output_size_bytes: expected.final_output_size_bytes,
        actual_final_output_size_bytes,
        expected_final_output_hash: expected.final_output_hash,
        actual_final_output_hash,
        expected_final_output_image_header_valid: expected.final_output_image_header_valid,
        actual_final_output_image_header_valid,
        expected_final_output_runnable_candidate: expected.final_output_runnable_candidate,
        actual_final_output_runnable_candidate,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
    }
}

use super::{
    final_executable_emit_shape::nsld_final_executable_emit_report_shape,
    final_executable_paths::nsld_final_executable_blocked_path,
    final_executable_render::{
        optional_bool_toml, optional_usize_toml, render_final_executable_blocked,
    },
    final_executable_verify_helpers::{non_empty_toml_string, optional_usize_value},
    reports::{NsldFinalExecutableEmitReport, NsldFinalExecutableEmitVerifyReport},
    toml,
};
use std::{fs, path::Path};

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
        fs::copy(&report.image_dry_run_bytes_path, &report.output_path).map_err(|error| {
            format!(
                "failed to write nsld final executable output `{}` from `{}`: {error}",
                report.output_path, report.image_dry_run_bytes_path
            )
        })?;
        report.emitted = true;
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

pub(crate) fn nsld_verify_final_executable_emit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitVerifyReport {
    let expected = nsld_final_executable_emit_report_shape(manifest, plan);
    let input_path = nsld_final_executable_blocked_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_blocked `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_plan_hash,
        actual_emitted,
        actual_writer_input_valid,
        actual_writer_input_hash,
        actual_writer_input_issues,
        actual_host_environment_ready,
        actual_host_driver_available,
        actual_host_can_invoke,
        actual_host_driver_resolved_path,
        actual_host_invocation_policy,
        actual_host_invocation_policy_reason,
        actual_host_dry_run_command_arg_count,
        actual_host_dry_run_command_args,
        actual_host_dry_run_blocker_count,
        actual_host_dry_run_blockers,
        actual_host_invoke_plan_valid,
        actual_host_invoke_plan_would_invoke,
        actual_host_invoke_plan_hash,
        actual_host_invoke_plan_invocation_policy,
        actual_host_invoke_plan_requires_explicit_allow,
        actual_host_invoke_plan_explicit_allow_present,
        actual_host_invoke_plan_blocker_count,
        actual_host_invoke_plan_issues,
        actual_layout_plan_valid,
        actual_layout_plan_hash,
        actual_layout_plan_issues,
        actual_image_dry_run_valid,
        actual_image_dry_run_hash,
        actual_image_dry_run_size_bytes,
        actual_image_dry_run_issues,
        actual_blocker_count,
        actual_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "final_stage_plan_hash"),
            toml::bool_value(source, "emitted"),
            toml::bool_value(source, "writer_input_valid"),
            non_empty_toml_string(source, "writer_input_hash"),
            toml::string_array_value(source, "writer_input_issues"),
            toml::bool_value(source, "host_dry_run_environment_ready"),
            toml::bool_value(source, "host_dry_run_driver_available"),
            toml::bool_value(source, "host_dry_run_can_invoke"),
            non_empty_toml_string(source, "host_dry_run_driver_resolved_path"),
            non_empty_toml_string(source, "host_dry_run_invocation_policy"),
            non_empty_toml_string(source, "host_dry_run_invocation_policy_reason"),
            toml::usize_value(source, "host_dry_run_command_arg_count"),
            toml::string_array_value(source, "host_dry_run_command_args"),
            toml::usize_value(source, "host_dry_run_blocker_count"),
            toml::string_array_value(source, "host_dry_run_blockers"),
            toml::bool_value(source, "host_invoke_plan_valid"),
            toml::bool_value(source, "host_invoke_plan_would_invoke"),
            non_empty_toml_string(source, "host_invoke_plan_hash"),
            non_empty_toml_string(source, "host_invoke_plan_invocation_policy"),
            toml::bool_value(source, "host_invoke_plan_requires_explicit_allow"),
            toml::bool_value(source, "host_invoke_plan_explicit_allow_present"),
            toml::usize_value(source, "host_invoke_plan_blocker_count"),
            toml::string_array_value(source, "host_invoke_plan_issues"),
            toml::bool_value(source, "layout_plan_valid"),
            non_empty_toml_string(source, "layout_plan_hash"),
            toml::string_array_value(source, "layout_plan_issues"),
            toml::bool_value(source, "image_dry_run_valid"),
            non_empty_toml_string(source, "image_dry_run_hash"),
            optional_usize_value(source, "image_dry_run_size_bytes"),
            toml::string_array_value(source, "image_dry_run_issues"),
            toml::usize_value(source, "blocker_count"),
            toml::string_array_value(source, "blockers"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None,       // final_stage_plan_hash
                None,       // emitted
                None,       // writer_input_valid
                None,       // writer_input_hash
                Vec::new(), // writer_input_issues
                None,       // host_dry_run_environment_ready
                None,       // host_dry_run_driver_available
                None,       // host_dry_run_can_invoke
                None,       // host_dry_run_driver_resolved_path
                None,       // host_dry_run_invocation_policy
                None,       // host_dry_run_invocation_policy_reason
                None,       // host_dry_run_command_arg_count
                Vec::new(), // host_dry_run_command_args
                None,       // host_dry_run_blocker_count
                Vec::new(), // host_dry_run_blockers
                None,       // host_invoke_plan_valid
                None,       // host_invoke_plan_would_invoke
                None,       // host_invoke_plan_hash
                None,       // host_invoke_plan_invocation_policy
                None,       // host_invoke_plan_requires_explicit_allow
                None,       // host_invoke_plan_explicit_allow_present
                None,       // host_invoke_plan_blocker_count
                Vec::new(), // host_invoke_plan_issues
                None,       // layout_plan_valid
                None,       // layout_plan_hash
                Vec::new(), // layout_plan_issues
                None,       // image_dry_run_valid
                None,       // image_dry_run_hash
                None,       // image_dry_run_size_bytes
                Vec::new(), // image_dry_run_issues
                None,       // blocker_count
                Vec::new(), // blockers
            )
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
        if actual_image_dry_run_issues != expected.image_dry_run_issues {
            issues.push(format!(
                "image_dry_run_issues mismatch: expected [{}], found [{}]",
                expected.image_dry_run_issues.join(", "),
                actual_image_dry_run_issues.join(", ")
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
        expected_image_dry_run_issues: expected.image_dry_run_issues,
        actual_image_dry_run_issues,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
    }
}

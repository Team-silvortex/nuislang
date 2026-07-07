use super::{
    final_executable_paths::nsld_final_executable_host_invoke_plan_path,
    final_executable_summary::nsld_final_executable_writer_plan_report,
    final_executable_writer::{render_final_executable_host_invoke_plan, resolve_host_driver_path},
    final_executable_writer_input::nsld_verify_final_executable_writer_input_report,
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableHostDryRunReport, NsldFinalExecutableHostInvokePlanEmitReport,
        NsldFinalExecutableHostInvokePlanReport, NsldFinalExecutableHostInvokePlanVerifyReport,
    },
    toml,
};
use std::{env, fs, path::Path};

#[cfg(test)]
#[path = "final_executable_host_tests.rs"]
mod tests;

const HOST_FINALIZER_ALLOW_ENV: &str = "NUIS_NSLD_ALLOW_HOST_FINALIZER";
const HOST_FINALIZER_POLICY_ENV: &str = "NUIS_NSLD_HOST_FINALIZER_POLICY";

pub(crate) fn nsld_final_executable_host_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableHostDryRunReport {
    let writer_plan = nsld_final_executable_writer_plan_report(manifest, plan);
    let writer_input = nsld_verify_final_executable_writer_input_report(manifest, plan);
    let command_args = if writer_input.actual_command_args.is_empty() {
        writer_input.expected_command_args.clone()
    } else {
        writer_input.actual_command_args.clone()
    };
    let driver = command_args
        .first()
        .cloned()
        .unwrap_or_else(|| writer_plan.final_stage_driver.clone());
    let driver_resolved_path = resolve_host_driver_path(&driver);
    let driver_available = driver_resolved_path.is_some();
    let mut blockers = Vec::new();
    if !writer_input.valid {
        blockers.push("final-executable-writer-input:invalid".to_owned());
        blockers.extend(
            writer_input
                .issues
                .iter()
                .map(|issue| format!("final-executable-writer-input:{issue}")),
        );
    }
    if !driver_available {
        blockers.push(format!("host-finalizer-driver-unavailable:{driver}"));
    }
    blockers.extend(writer_plan.writer_blockers.iter().cloned());
    let (invocation_policy, invocation_policy_reason, policy_blocker) =
        host_finalizer_invocation_policy();
    if let Some(blocker) = policy_blocker {
        blockers.push(blocker);
    }
    let environment_ready = writer_input.valid && driver_available;
    let can_invoke_host_finalizer = environment_ready
        && writer_plan.writer_blockers.is_empty()
        && invocation_policy == "allow-host-invoke";
    let mut notes = writer_plan.notes.clone();
    notes.push("host-finalizer-dry-run-is-non-mutating".to_owned());
    notes.push("host-finalizer-is-not-invoked".to_owned());
    notes.push(format!(
        "host-finalizer-policy-env:{HOST_FINALIZER_POLICY_ENV}={}",
        invocation_policy
    ));

    NsldFinalExecutableHostDryRunReport {
        manifest: manifest.display().to_string(),
        writer_input_path: writer_input.input_path,
        writer_input_valid: writer_input.valid,
        writer_input_hash: writer_input.actual_writer_input_hash,
        driver,
        driver_available,
        driver_resolved_path,
        command_arg_count: command_args.len(),
        command_args,
        environment_ready,
        invocation_policy,
        invocation_policy_reason,
        can_invoke_host_finalizer,
        blockers,
        notes,
    }
}

pub(crate) fn nsld_final_executable_host_invoke_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableHostInvokePlanReport {
    let dry_run = nsld_final_executable_host_dry_run_report(manifest, plan);
    let requires_explicit_allow = true;
    let explicit_allow_present = host_finalizer_explicit_allow_present();
    let mut blockers = dry_run.blockers.clone();
    if requires_explicit_allow && !explicit_allow_present {
        blockers.push("host-finalizer-explicit-allow:missing".to_owned());
    }
    let would_invoke = dry_run.can_invoke_host_finalizer && explicit_allow_present;
    let invocation_kind = "host-finalizer-command".to_owned();
    let mut notes = dry_run.notes.clone();
    notes.push("host-invoke-plan-is-non-mutating".to_owned());
    notes.push("host-finalizer-process-is-not-spawned".to_owned());
    notes.push(format!(
        "host-finalizer-explicit-allow-env:{HOST_FINALIZER_ALLOW_ENV}={}",
        if explicit_allow_present {
            "present"
        } else {
            "missing"
        }
    ));

    NsldFinalExecutableHostInvokePlanReport {
        manifest: manifest.display().to_string(),
        output_path: plan.final_stage.output_path.clone(),
        writer_input_path: dry_run.writer_input_path,
        invocation_kind,
        invocation_policy: dry_run.invocation_policy,
        invocation_policy_reason: dry_run.invocation_policy_reason,
        requires_explicit_allow,
        explicit_allow_present,
        environment_ready: dry_run.environment_ready,
        driver_available: dry_run.driver_available,
        driver_resolved_path: dry_run.driver_resolved_path,
        can_invoke_host_finalizer: dry_run.can_invoke_host_finalizer,
        would_invoke,
        command_arg_count: dry_run.command_arg_count,
        command_args: dry_run.command_args,
        blockers,
        notes,
    }
}

pub(crate) fn nsld_emit_final_executable_host_invoke_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableHostInvokePlanEmitReport, String> {
    let report = nsld_final_executable_host_invoke_plan_report(manifest, plan);
    let source = render_final_executable_host_invoke_plan(&report);
    let output_path = nsld_final_executable_host_invoke_plan_path(plan);
    fs::write(&output_path, &source).map_err(|error| {
        format!(
            "failed to write nsld final executable host invoke plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldFinalExecutableHostInvokePlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        invoke_plan_hash: fnv1a64_hex(source.as_bytes()),
        invocation_policy: report.invocation_policy,
        requires_explicit_allow: report.requires_explicit_allow,
        explicit_allow_present: report.explicit_allow_present,
        would_invoke: report.would_invoke,
        blocker_count: report.blockers.len(),
    })
}

pub(crate) fn nsld_verify_final_executable_host_invoke_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableHostInvokePlanVerifyReport {
    let expected_report = nsld_final_executable_host_invoke_plan_report(manifest, plan);
    let expected_source = render_final_executable_host_invoke_plan(&expected_report);
    let expected_hash = fnv1a64_hex(expected_source.as_bytes());
    let input_path = nsld_final_executable_host_invoke_plan_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_host_invoke_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_hash,
        actual_invocation_policy,
        actual_requires_explicit_allow,
        actual_explicit_allow_present,
        actual_would_invoke,
        actual_command_arg_count,
        actual_command_args,
        actual_blocker_count,
        actual_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::string_value(source, "invocation_policy"),
            toml::bool_value(source, "requires_explicit_allow"),
            toml::bool_value(source, "explicit_allow_present"),
            toml::bool_value(source, "would_invoke"),
            toml::usize_value(source, "command_arg_count"),
            toml::string_array_value(source, "command_args"),
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
                Vec::new(),
                None,
                Vec::new(),
            )
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-host-invoke-plan-content-mismatch".to_owned());
        }
        if actual_invocation_policy.as_deref() != Some(expected_report.invocation_policy.as_str()) {
            issues.push(format!(
                "invocation_policy mismatch: expected {}, found {}",
                expected_report.invocation_policy,
                actual_invocation_policy
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_requires_explicit_allow != Some(expected_report.requires_explicit_allow) {
            issues.push(format!(
                "requires_explicit_allow mismatch: expected {}, found {}",
                expected_report.requires_explicit_allow,
                actual_requires_explicit_allow
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_explicit_allow_present != Some(expected_report.explicit_allow_present) {
            issues.push(format!(
                "explicit_allow_present mismatch: expected {}, found {}",
                expected_report.explicit_allow_present,
                actual_explicit_allow_present
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_would_invoke != Some(expected_report.would_invoke) {
            issues.push(format!(
                "would_invoke mismatch: expected {}, found {}",
                expected_report.would_invoke,
                actual_would_invoke
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_command_arg_count != Some(expected_report.command_args.len()) {
            issues.push(format!(
                "command_arg_count mismatch: expected {}, found {}",
                expected_report.command_args.len(),
                actual_command_arg_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_command_args != expected_report.command_args {
            issues.push(format!(
                "command_args mismatch: expected [{}], found [{}]",
                expected_report.command_args.join(", "),
                actual_command_args.join(", ")
            ));
        }
        if actual_blocker_count != Some(expected_report.blockers.len()) {
            issues.push(format!(
                "blocker_count mismatch: expected {}, found {}",
                expected_report.blockers.len(),
                actual_blocker_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_blockers != expected_report.blockers {
            issues.push(format!(
                "blockers mismatch: expected [{}], found [{}]",
                expected_report.blockers.join(", "),
                actual_blockers.join(", ")
            ));
        }
    }

    NsldFinalExecutableHostInvokePlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_invoke_plan_hash: expected_hash,
        actual_invoke_plan_hash: actual_hash,
        expected_invocation_policy: expected_report.invocation_policy,
        actual_invocation_policy,
        expected_requires_explicit_allow: expected_report.requires_explicit_allow,
        actual_requires_explicit_allow,
        expected_explicit_allow_present: expected_report.explicit_allow_present,
        actual_explicit_allow_present,
        expected_would_invoke: expected_report.would_invoke,
        actual_would_invoke,
        expected_command_arg_count: expected_report.command_args.len(),
        actual_command_arg_count,
        expected_command_args: expected_report.command_args,
        actual_command_args,
        expected_blocker_count: expected_report.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected_report.blockers,
        actual_blockers,
        issues,
    }
}

fn host_finalizer_explicit_allow_present() -> bool {
    env::var(HOST_FINALIZER_ALLOW_ENV)
        .map(|value| {
            let value = value.trim();
            value == "1"
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case("allow")
        })
        .unwrap_or(false)
}

fn host_finalizer_invocation_policy() -> (String, String, Option<String>) {
    match env::var(HOST_FINALIZER_POLICY_ENV) {
        Ok(value) => {
            let value = value.trim();
            if value == "allow-host-invoke" || value.eq_ignore_ascii_case("allow") {
                (
                    "allow-host-invoke".to_owned(),
                    format!("explicit-policy-env:{HOST_FINALIZER_POLICY_ENV}"),
                    None,
                )
            } else if value.is_empty() || value == "dry-run-only" {
                (
                    "dry-run-only".to_owned(),
                    "alpha-host-finalizer-execution-disabled".to_owned(),
                    Some("host-finalizer-policy:dry-run-only".to_owned()),
                )
            } else {
                (
                    "dry-run-only".to_owned(),
                    format!("invalid-policy-env:{HOST_FINALIZER_POLICY_ENV}={value}"),
                    Some(format!(
                        "host-finalizer-policy-env:invalid:{HOST_FINALIZER_POLICY_ENV}"
                    )),
                )
            }
        }
        Err(_) => (
            "dry-run-only".to_owned(),
            "alpha-host-finalizer-execution-disabled".to_owned(),
            Some("host-finalizer-policy:dry-run-only".to_owned()),
        ),
    }
}

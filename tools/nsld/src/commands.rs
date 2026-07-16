use crate::json_fields::{
    json_bool_field, json_optional_string_field, json_string_array_field, json_string_field,
};
use crate::{
    context::load_link_input_context, display, json, nsld_check_report, reports::NsldCheckReport,
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldCheckNextAction {
    pub(crate) available: bool,
    pub(crate) source: Option<String>,
    pub(crate) command_id: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) command_resolved: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) gate_action: Option<String>,
    pub(crate) gate_env_assignments: Vec<String>,
    pub(crate) crossing_env_assignments: Vec<String>,
    pub(crate) crossing_command_resolved: Option<String>,
}

pub(crate) fn run_status_command() {
    println!("Nsld linker front-door");
    println!("  tool: nsld");
    println!("  phase: alpha-0.10.0 executable-artifact closure");
    println!(
        "  current_role: link-plan inspection, artifact-chain diagnosis, and final executable readiness"
    );
    println!("  implementation: reuses nuisc::linker while linker ownership is split out");
    println!(
        "  final_link_status: self-contained Nsld image emission is available; host-native and OS-native finalization remain staged"
    );
}

pub(crate) fn run_plan_command(input: &Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    if json {
        println!("{}", nuisc::linker::render_link_plan_json(&ctx.plan));
    } else {
        println!("Nsld link plan");
        println!("  input: {}", ctx.input.display());
        println!("  manifest: {}", ctx.manifest.display());
        println!("  role: alpha-0.10.0 executable-artifact closure front-door");
        for line in nuisc::linker::render_link_plan_summary(&ctx.plan) {
            println!("  {line}");
        }
    }
    Ok(())
}

pub(crate) fn run_check_command(input: &Path, json_output: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_check_report(&ctx.manifest, &ctx.plan);
    if json_output {
        println!("{}", json::check_report_json(&report));
    } else {
        display::print_check_report(&report);
    }
    if report.valid {
        Ok(())
    } else {
        Err(nsld_check_failure_message(&report))
    }
}

pub(crate) fn run_check_next_action_command(input: &Path, json_output: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    let report = nsld_check_report(&ctx.manifest, &ctx.plan);
    let next_action = nsld_check_next_action(&report);
    if json_output {
        println!("{}", nsld_check_next_action_json(&next_action));
    } else if let Some(command) = nsld_check_next_action_dry_run(&report) {
        println!("{command}");
    } else {
        println!("no-next-action");
    }
    Ok(())
}

pub(crate) fn nsld_check_failure_message(report: &NsldCheckReport) -> String {
    let next_action = nsld_check_next_action(report);
    let command = next_action
        .crossing_command_resolved
        .as_deref()
        .or(next_action.command_resolved.as_deref());
    match (next_action.available, command) {
        (true, Some(command)) => format!("nsld check failed; next action: {command}"),
        _ => "nsld check failed".to_owned(),
    }
}

pub(crate) fn nsld_check_next_action_json(next_action: &NsldCheckNextAction) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_check_next_action"),
        json_bool_field("available", next_action.available),
        json_optional_string_field("source", next_action.source.as_deref()),
        json_optional_string_field("command_id", next_action.command_id.as_deref()),
        json_optional_string_field("command", next_action.command.as_deref()),
        json_optional_string_field("command_resolved", next_action.command_resolved.as_deref()),
        json_optional_string_field("reason", next_action.reason.as_deref()),
        json_optional_string_field("gate_action", next_action.gate_action.as_deref()),
        json_string_array_field("gate_env_assignments", &next_action.gate_env_assignments),
        json_string_array_field(
            "crossing_env_assignments",
            &next_action.crossing_env_assignments,
        ),
        json_optional_string_field(
            "crossing_command_resolved",
            next_action.crossing_command_resolved.as_deref(),
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_check_next_action(report: &NsldCheckReport) -> NsldCheckNextAction {
    if !report.next_action_available {
        if let Some(action) = final_output_materialization_next_action(report) {
            return action;
        }
    }
    let gate_action = next_action_gate_action(
        report,
        report.next_action_source.as_deref(),
        report.next_action_command_id.as_deref(),
    );
    let gate_env_assignments = gate_action_env_assignments(gate_action.as_deref());
    let crossing_env_assignments = next_action_crossing_env_assignments(
        report.next_action_source.as_deref(),
        report.next_action_command_id.as_deref(),
    );
    let crossing_command_resolved = crossing_command_resolved(
        &crossing_env_assignments,
        report.next_action_command_resolved.as_deref(),
    );
    NsldCheckNextAction {
        available: report.next_action_available,
        source: report.next_action_source.clone(),
        command_id: report.next_action_command_id.clone(),
        command: report.next_action_command.clone(),
        command_resolved: report.next_action_command_resolved.clone(),
        reason: report.next_action_command_reason.clone(),
        gate_action,
        gate_env_assignments,
        crossing_env_assignments,
        crossing_command_resolved,
    }
}

fn next_action_gate_action(
    report: &NsldCheckReport,
    source: Option<&str>,
    command_id: Option<&str>,
) -> Option<String> {
    if source == Some("final-output-boundary") && command_id == Some("final-executable-output") {
        return report.final_executable_host_finalizer_gate_action.clone();
    }
    None
}

fn gate_action_env_assignments(action: Option<&str>) -> Vec<String> {
    action
        .and_then(|action| action.strip_prefix("set-env:"))
        .map(|assignment| vec![assignment.to_owned()])
        .unwrap_or_default()
}

fn next_action_crossing_env_assignments(
    source: Option<&str>,
    command_id: Option<&str>,
) -> Vec<String> {
    if source == Some("final-output-boundary") && command_id == Some("final-executable-output") {
        return vec![
            "NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned(),
            "NUIS_NSLD_ALLOW_HOST_FINALIZER=1".to_owned(),
        ];
    }
    Vec::new()
}

fn crossing_command_resolved(
    env_assignments: &[String],
    command_resolved: Option<&str>,
) -> Option<String> {
    if env_assignments.is_empty() {
        return None;
    }
    command_resolved.map(|command| format!("env {} {command}", env_assignments.join(" ")))
}

fn final_output_materialization_next_action(
    report: &NsldCheckReport,
) -> Option<NsldCheckNextAction> {
    let command_id = match report
        .final_executable_output_recommended_next_action
        .as_str()
    {
        "emit-final-executable-launcher-manifest" => "emit-final-executable-launcher-manifest",
        "emit-final-executable-launcher-dry-run" => "emit-final-executable-launcher-dry-run",
        _ => return None,
    };
    let command = format!("nsld {command_id} <input>");
    Some(NsldCheckNextAction {
        available: true,
        source: Some("final-output-materialization".to_owned()),
        command_id: Some(command_id.to_owned()),
        command_resolved: Some(command.replace("<input>", &report.manifest)),
        command: Some(command),
        reason: Some(format!(
            "final executable output is ready; {}",
            report.final_executable_output_recommended_next_action
        )),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    })
}

pub(crate) fn nsld_check_next_action_dry_run(report: &NsldCheckReport) -> Option<String> {
    let next_action = nsld_check_next_action(report);
    next_action.available.then(|| {
        next_action
            .crossing_command_resolved
            .or(next_action.command_resolved)
    })?
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_check_failure_message, nsld_check_next_action, nsld_check_next_action_dry_run,
        nsld_check_next_action_json, run_check_command, run_plan_command,
    };
    use crate::{main_test_support::empty_link_plan, nsld_check_report};
    use std::path::Path;
    use std::{env, fs};

    #[test]
    fn plan_command_reports_missing_manifest_directory() {
        let dir = env::temp_dir().join(format!(
            "nsld-plan-command-missing-manifest-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let error = run_plan_command(&dir, false).unwrap_err();
        fs::remove_dir_all(dir).unwrap();

        assert!(error.contains("does not contain `nuis.build.manifest.toml`"));
    }

    #[test]
    fn check_command_reports_missing_manifest_directory() {
        let dir = env::temp_dir().join(format!(
            "nsld-check-command-missing-manifest-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let error = run_check_command(&dir, false).unwrap_err();
        fs::remove_dir_all(dir).unwrap();

        assert!(error.contains("does not contain `nuis.build.manifest.toml`"));
    }

    #[test]
    fn check_failure_message_includes_resolved_next_action() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = true;
        report.next_action_source = Some("required".to_owned());
        report.next_action_command_id = Some("emit-inputs".to_owned());
        report.next_action_command = Some("nsld emit-inputs <input>".to_owned());
        report.next_action_command_resolved = Some("nsld emit-inputs manifest.toml".to_owned());
        report.next_action_command_reason =
            Some("first missing required artifact stage `link-inputs`".to_owned());

        assert_eq!(
            nsld_check_failure_message(&report),
            "nsld check failed; next action: nsld emit-inputs manifest.toml"
        );
        assert_eq!(
            nsld_check_next_action(&report),
            super::NsldCheckNextAction {
                available: true,
                source: Some("required".to_owned()),
                command_id: Some("emit-inputs".to_owned()),
                command: Some("nsld emit-inputs <input>".to_owned()),
                command_resolved: Some("nsld emit-inputs manifest.toml".to_owned()),
                reason: Some("first missing required artifact stage `link-inputs`".to_owned()),
                gate_action: None,
                gate_env_assignments: Vec::new(),
                crossing_env_assignments: Vec::new(),
                crossing_command_resolved: None,
            }
        );
    }

    #[test]
    fn check_failure_message_omits_missing_next_action() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = false;
        report.next_action_command_resolved = None;

        assert_eq!(nsld_check_failure_message(&report), "nsld check failed");
    }

    #[test]
    fn check_failure_message_prefers_crossing_command_when_gate_is_required() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = true;
        report.next_action_source = Some("final-output-boundary".to_owned());
        report.next_action_command_id = Some("final-executable-output".to_owned());
        report.next_action_command_resolved =
            Some("nsld final-executable-output manifest.toml".to_owned());
        report.final_executable_host_finalizer_gate_action =
            Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned());

        assert_eq!(
            nsld_check_failure_message(&report),
            "nsld check failed; next action: env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml"
        );
    }

    #[test]
    fn check_next_action_dry_run_returns_resolved_command() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = true;
        report.next_action_command_resolved = Some("nsld emit-inputs manifest.toml".to_owned());

        assert_eq!(
            nsld_check_next_action_dry_run(&report).as_deref(),
            Some("nsld emit-inputs manifest.toml")
        );
    }

    #[test]
    fn check_next_action_dry_run_prefers_crossing_command_when_gate_is_required() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = true;
        report.next_action_source = Some("final-output-boundary".to_owned());
        report.next_action_command_id = Some("final-executable-output".to_owned());
        report.next_action_command_resolved =
            Some("nsld final-executable-output manifest.toml".to_owned());
        report.final_executable_host_finalizer_gate_action =
            Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned());

        assert_eq!(
            nsld_check_next_action_dry_run(&report).as_deref(),
            Some("env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml")
        );
    }

    #[test]
    fn check_next_action_exposes_final_output_gate_protocol() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = true;
        report.next_action_source = Some("final-output-boundary".to_owned());
        report.next_action_command_id = Some("final-executable-output".to_owned());
        report.next_action_command = Some("nsld final-executable-output <input>".to_owned());
        report.next_action_command_resolved =
            Some("nsld final-executable-output manifest.toml".to_owned());
        report.next_action_command_reason = Some(
            "final executable output boundary is blocked by `final-executable-output:not-nsld-owned`"
                .to_owned(),
        );
        report.final_executable_host_finalizer_gate_action =
            Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned());

        let next_action = nsld_check_next_action(&report);
        let json = nsld_check_next_action_json(&next_action);

        assert_eq!(
            next_action.gate_action.as_deref(),
            Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke")
        );
        assert_eq!(
            next_action.gate_env_assignments,
            vec!["NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned()]
        );
        assert!(json.contains(
            "\"gate_action\":\"set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\""
        ));
        assert!(json.contains(
            "\"gate_env_assignments\":[\"NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\"]"
        ));
        assert!(json.contains(
            "\"crossing_env_assignments\":[\"NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\",\"NUIS_NSLD_ALLOW_HOST_FINALIZER=1\"]"
        ));
        assert!(json.contains(
            "\"crossing_command_resolved\":\"env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml\""
        ));
    }

    #[test]
    fn check_next_action_dry_run_omits_unavailable_action() {
        let mut report = nsld_check_report(Path::new("manifest.toml"), &empty_link_plan());
        report.next_action_available = false;
        report.next_action_command_resolved = Some("nsld emit-inputs manifest.toml".to_owned());

        assert_eq!(nsld_check_next_action_dry_run(&report), None);
    }

    #[test]
    fn check_next_action_json_reports_dry_run_shape() {
        let next_action = super::NsldCheckNextAction {
            available: true,
            source: Some("required".to_owned()),
            command_id: Some("emit-inputs".to_owned()),
            command: Some("nsld emit-inputs <input>".to_owned()),
            command_resolved: Some("nsld emit-inputs manifest.toml".to_owned()),
            reason: Some("first missing required artifact stage `link-inputs`".to_owned()),
            gate_action: None,
            gate_env_assignments: Vec::new(),
            crossing_env_assignments: Vec::new(),
            crossing_command_resolved: None,
        };
        let json = nsld_check_next_action_json(&next_action);

        assert!(json.contains("\"kind\":\"nsld_check_next_action\""));
        assert!(json.contains("\"available\":true"));
        assert!(json.contains("\"source\":\"required\""));
        assert!(json.contains("\"command_resolved\":\"nsld emit-inputs manifest.toml\""));
        assert!(json.contains("\"gate_action\":null"));
        assert!(json.contains("\"gate_env_assignments\":[]"));
        assert!(json.contains("\"crossing_env_assignments\":[]"));
        assert!(json.contains("\"crossing_command_resolved\":null"));
    }
}

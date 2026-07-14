use crate::json_fields::{json_bool_field, json_optional_string_field, json_string_field};
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
    match (
        next_action.available,
        next_action.command_resolved.as_deref(),
    ) {
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
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_check_next_action(report: &NsldCheckReport) -> NsldCheckNextAction {
    if !report.next_action_available {
        if let Some(action) = final_output_materialization_next_action(report) {
            return action;
        }
    }
    NsldCheckNextAction {
        available: report.next_action_available,
        source: report.next_action_source.clone(),
        command_id: report.next_action_command_id.clone(),
        command: report.next_action_command.clone(),
        command_resolved: report.next_action_command_resolved.clone(),
        reason: report.next_action_command_reason.clone(),
    }
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
    })
}

pub(crate) fn nsld_check_next_action_dry_run(report: &NsldCheckReport) -> Option<String> {
    let next_action = nsld_check_next_action(report);
    next_action
        .available
        .then(|| next_action.command_resolved)
        .flatten()
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
        };
        let json = nsld_check_next_action_json(&next_action);

        assert!(json.contains("\"kind\":\"nsld_check_next_action\""));
        assert!(json.contains("\"available\":true"));
        assert!(json.contains("\"source\":\"required\""));
        assert!(json.contains("\"command_resolved\":\"nsld emit-inputs manifest.toml\""));
    }
}

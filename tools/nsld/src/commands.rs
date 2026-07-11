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
        "  final_link_status: final executable emission is still blocked before real binary linking"
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

pub(crate) fn nsld_check_next_action(report: &NsldCheckReport) -> NsldCheckNextAction {
    NsldCheckNextAction {
        available: report.next_action_available,
        source: report.next_action_source.clone(),
        command_id: report.next_action_command_id.clone(),
        command: report.next_action_command.clone(),
        command_resolved: report.next_action_command_resolved.clone(),
        reason: report.next_action_command_reason.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_check_failure_message, nsld_check_next_action, run_check_command, run_plan_command,
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
}

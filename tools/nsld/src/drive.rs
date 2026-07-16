use crate::commands::{
    nsld_check_next_action, nsld_check_next_action_dry_run, NsldCheckNextAction,
};
use crate::json_fields::{
    json_bool_field, json_optional_string_field, json_string_array_field, json_string_field,
    json_usize_field,
};
use crate::object_byte_layout::nsld_emit_object_byte_layout_report;
use crate::object_emit::nsld_emit_object_report;
use crate::object_file_layout::nsld_emit_object_file_layout_report;
use crate::object_image_dry_run::nsld_emit_object_image_dry_run_report;
use crate::object_plan::nsld_emit_object_plan_report;
use crate::object_writer_input::nsld_emit_object_writer_dry_run_report;
use crate::{
    context::load_link_input_context, nsld_check_report, nsld_emit_assemble_plan_report,
    nsld_emit_closure_report, nsld_emit_container_plan_report, nsld_emit_container_report,
    nsld_emit_final_executable_launcher_dry_run_report,
    nsld_emit_final_executable_launcher_manifest_report,
    nsld_emit_final_executable_pipeline_report, nsld_emit_final_executable_report,
    nsld_emit_final_stage_plan_report, nsld_emit_link_bundle_report, nsld_emit_link_inputs_report,
    nsld_emit_link_units_report, nsld_emit_section_manifest_report, nsld_prepare_report,
};
use std::{env, path::Path};

const HOST_FINALIZER_ALLOW_ENV: &str = "NUIS_NSLD_ALLOW_HOST_FINALIZER";
const HOST_FINALIZER_POLICY_ENV: &str = "NUIS_NSLD_HOST_FINALIZER_POLICY";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDriveApplyReport {
    pub(crate) applied: bool,
    pub(crate) command_id: Option<String>,
    pub(crate) command_resolved: Option<String>,
    pub(crate) gate_action: Option<String>,
    pub(crate) gate_env_assignments: Vec<String>,
    pub(crate) crossing_env_assignments: Vec<String>,
    pub(crate) crossing_command_resolved: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDriveUntilCleanReport {
    pub(crate) completed: bool,
    pub(crate) applied_steps: usize,
    pub(crate) capped: bool,
    pub(crate) stop_reason: String,
    pub(crate) stop_command_id: Option<String>,
    pub(crate) stop_source: Option<String>,
    pub(crate) stop_command_resolved: Option<String>,
    pub(crate) stop_action_reason: Option<String>,
    pub(crate) stop_gate_action: Option<String>,
    pub(crate) stop_gate_env_assignments: Vec<String>,
    pub(crate) stop_crossing_env_assignments: Vec<String>,
    pub(crate) stop_crossing_command_resolved: Option<String>,
    pub(crate) last_command_id: Option<String>,
    pub(crate) messages: Vec<String>,
}

pub(crate) fn run_drive_command(
    input: &Path,
    json_output: bool,
    apply: bool,
    until_clean: bool,
) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    if until_clean {
        let loop_report = nsld_drive_apply_until_clean(&ctx.manifest, &ctx.plan)?;
        if json_output {
            println!("{}", nsld_drive_until_clean_report_json(&loop_report));
        } else {
            println!(
                "drive apply-until-clean: applied_steps={}, completed={}, capped={}, stop_reason={}",
                loop_report.applied_steps,
                loop_report.completed,
                loop_report.capped,
                loop_report.stop_reason
            );
            if let Some(source) = loop_report.stop_source.as_deref() {
                println!("  stop_source: {source}");
            }
            if let Some(command) = loop_report.stop_command_resolved.as_deref() {
                println!("  stop_command_resolved: {command}");
            }
            if let Some(reason) = loop_report.stop_action_reason.as_deref() {
                println!("  stop_action_reason: {reason}");
            }
            for message in loop_report.messages {
                println!("  {message}");
            }
        }
        return Ok(());
    }
    let report = nsld_check_report(&ctx.manifest, &ctx.plan);
    let next_action = nsld_check_next_action(&report);
    if apply {
        let apply_report = nsld_drive_apply_next_action(&ctx.manifest, &ctx.plan, &next_action)?;
        if json_output {
            println!("{}", nsld_drive_apply_report_json(&apply_report));
        } else {
            println!("drive apply: {}", apply_report.message);
        }
        return Ok(());
    }
    if json_output {
        println!("{}", nsld_drive_dry_run_json(&next_action));
    } else if let Some(command) = nsld_check_next_action_dry_run(&report) {
        println!("drive dry-run: {command}");
    } else {
        println!("drive dry-run: no-next-action");
    }
    Ok(())
}

pub(crate) fn nsld_drive_dry_run_json(next_action: &NsldCheckNextAction) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_drive_dry_run"),
        json_bool_field("would_execute", next_action.available),
        json_bool_field("mutates_artifacts", false),
        json_string_field(
            "mutation_policy",
            drive_dry_run_mutation_policy(next_action),
        ),
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

pub(crate) fn nsld_drive_apply_report_json(report: &NsldDriveApplyReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_drive_apply"),
        json_bool_field("applied", report.applied),
        json_bool_field("mutates_artifacts", report.applied),
        json_string_field("mutation_policy", drive_apply_mutation_policy(report)),
        json_optional_string_field("command_id", report.command_id.as_deref()),
        json_optional_string_field("command_resolved", report.command_resolved.as_deref()),
        json_optional_string_field("gate_action", report.gate_action.as_deref()),
        json_string_array_field("gate_env_assignments", &report.gate_env_assignments),
        json_string_array_field("crossing_env_assignments", &report.crossing_env_assignments),
        json_optional_string_field(
            "crossing_command_resolved",
            report.crossing_command_resolved.as_deref(),
        ),
        json_string_field("safe_next_action", drive_apply_safe_next_action(report)),
        json_optional_string_field(
            "safe_next_command",
            drive_apply_safe_next_command(report).as_deref(),
        ),
        json_string_field("safe_next_reason", drive_apply_safe_next_reason(report)),
        json_string_field("message", &report.message),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_drive_until_clean_report_json(report: &NsldDriveUntilCleanReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_drive_until_clean"),
        json_bool_field("completed", report.completed),
        json_usize_field("applied_steps", report.applied_steps),
        json_bool_field("mutates_artifacts", report.applied_steps > 0),
        json_string_field("mutation_policy", drive_until_clean_mutation_policy(report)),
        json_bool_field("capped", report.capped),
        json_string_field("stop_reason", &report.stop_reason),
        json_optional_string_field("stop_command_id", report.stop_command_id.as_deref()),
        json_optional_string_field("stop_source", report.stop_source.as_deref()),
        json_optional_string_field(
            "stop_command_resolved",
            report.stop_command_resolved.as_deref(),
        ),
        json_optional_string_field("stop_action_reason", report.stop_action_reason.as_deref()),
        json_optional_string_field("stop_gate_action", report.stop_gate_action.as_deref()),
        json_string_array_field(
            "stop_gate_env_assignments",
            &report.stop_gate_env_assignments,
        ),
        json_string_array_field(
            "stop_crossing_env_assignments",
            &report.stop_crossing_env_assignments,
        ),
        json_optional_string_field(
            "stop_crossing_command_resolved",
            report.stop_crossing_command_resolved.as_deref(),
        ),
        json_string_field(
            "safe_next_action",
            drive_until_clean_safe_next_action(report),
        ),
        json_optional_string_field(
            "safe_next_command",
            drive_until_clean_safe_next_command(report).as_deref(),
        ),
        json_string_field(
            "safe_next_reason",
            drive_until_clean_safe_next_reason(report),
        ),
        json_optional_string_field("last_command_id", report.last_command_id.as_deref()),
        json_string_array_field("messages", &report.messages),
    ];
    format!("{{{}}}", fields.join(","))
}

fn drive_dry_run_mutation_policy(next_action: &NsldCheckNextAction) -> &'static str {
    if !next_action.available {
        return "read-only-observe";
    }
    match next_action.source.as_deref() {
        Some("final-output-boundary") => "read-only-boundary-observe",
        Some("final-output-materialization") => "read-only-materialization-observe",
        _ => "read-only-artifact-observe",
    }
}

fn drive_apply_mutation_policy(report: &NsldDriveApplyReport) -> &'static str {
    if report.applied {
        return match report.command_id.as_deref() {
            Some("materialize-provider-samples") => "whitelisted-boundary-materialization",
            Some("final-executable-output") => "gated-final-output-mutation",
            _ => "whitelisted-artifact-mutation",
        };
    }
    match report.message.as_str() {
        "no-next-action" => "read-only-observe",
        "next-action-command-id-missing" => "blocked-invalid-next-action",
        message if message.starts_with("read-only-boundary:") => "blocked-read-only-boundary",
        message if message.starts_with("next-action-not-whitelisted:") => {
            "blocked-unlisted-mutation"
        }
        message if message.starts_with("final-output-boundary-not-emitted:") => {
            "blocked-final-output-mutation"
        }
        _ => "blocked-not-applied",
    }
}

fn drive_until_clean_mutation_policy(report: &NsldDriveUntilCleanReport) -> &'static str {
    if report.applied_steps > 0 {
        return match report.stop_reason.as_str() {
            "clean" => "whitelisted-artifact-mutation-loop-clean",
            "provider-sample-materialization-required" => {
                "whitelisted-boundary-materialization-loop"
            }
            _ => "whitelisted-artifact-mutation-loop-stopped",
        };
    }
    match report.stop_reason.as_str() {
        "clean" => "read-only-observe",
        "host-finalizer-policy-required" => "blocked-read-only-boundary",
        "provider-sample-materialization-required" => "blocked-boundary-materialization",
        "not-applied" => "blocked-not-applied",
        "repeated-next-action" => "blocked-repeated-next-action",
        "max-steps" => "blocked-max-steps",
        _ => "blocked-boundary",
    }
}

fn drive_apply_safe_next_action(report: &NsldDriveApplyReport) -> &'static str {
    if report.applied {
        "rerun-drive-to-refresh-next-action"
    } else if report.crossing_command_resolved.is_some() {
        "explicit-boundary-crossing-command-available"
    } else if report.command_resolved.is_some() {
        "inspect-blocked-command"
    } else {
        "no-safe-next-command"
    }
}

fn drive_apply_safe_next_command(report: &NsldDriveApplyReport) -> Option<String> {
    if report.applied {
        return None;
    }
    report
        .crossing_command_resolved
        .clone()
        .or_else(|| report.command_resolved.clone())
}

fn drive_apply_safe_next_reason(report: &NsldDriveApplyReport) -> &str {
    if report.applied {
        "drive applied one mutation; rerun drive to observe the next deterministic action"
    } else if report.crossing_command_resolved.is_some() {
        "drive stopped at an explicit boundary; run the safe_next_command only if you accept the listed gate"
    } else if report.command_resolved.is_some() {
        "drive did not mutate artifacts; inspect the blocked command before retrying"
    } else {
        "drive has no safe follow-up command"
    }
}

fn drive_until_clean_safe_next_action(report: &NsldDriveUntilCleanReport) -> &'static str {
    if report.completed {
        "clean"
    } else if report.stop_crossing_command_resolved.is_some() {
        "explicit-boundary-crossing-command-available"
    } else if report.stop_command_resolved.is_some() {
        "inspect-stop-command"
    } else {
        "no-safe-next-command"
    }
}

fn drive_until_clean_safe_next_command(report: &NsldDriveUntilCleanReport) -> Option<String> {
    if report.completed {
        return None;
    }
    report
        .stop_crossing_command_resolved
        .clone()
        .or_else(|| report.stop_command_resolved.clone())
}

fn drive_until_clean_safe_next_reason(report: &NsldDriveUntilCleanReport) -> &str {
    if report.completed {
        "drive reached a clean artifact chain"
    } else if report.stop_crossing_command_resolved.is_some() {
        "drive stopped at an explicit boundary; run the safe_next_command only if you accept the listed gate"
    } else if report.stop_command_resolved.is_some() {
        "drive stopped before mutating the boundary; inspect the stop command before retrying"
    } else {
        "drive has no safe follow-up command"
    }
}

pub(crate) fn nsld_drive_apply_next_action(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    next_action: &NsldCheckNextAction,
) -> Result<NsldDriveApplyReport, String> {
    if !next_action.available {
        return Ok(NsldDriveApplyReport {
            applied: false,
            command_id: next_action.command_id.clone(),
            command_resolved: next_action.command_resolved.clone(),
            gate_action: next_action.gate_action.clone(),
            gate_env_assignments: next_action.gate_env_assignments.clone(),
            crossing_env_assignments: next_action.crossing_env_assignments.clone(),
            crossing_command_resolved: next_action.crossing_command_resolved.clone(),
            message: "no-next-action".to_owned(),
        });
    }
    let Some(command_id) = next_action.command_id.as_deref() else {
        return Ok(NsldDriveApplyReport {
            applied: false,
            command_id: None,
            command_resolved: next_action.command_resolved.clone(),
            gate_action: next_action.gate_action.clone(),
            gate_env_assignments: next_action.gate_env_assignments.clone(),
            crossing_env_assignments: next_action.crossing_env_assignments.clone(),
            crossing_command_resolved: next_action.crossing_command_resolved.clone(),
            message: "next-action-command-id-missing".to_owned(),
        });
    };
    if next_action.source.as_deref() == Some("final-output-boundary") {
        if command_id == "materialize-provider-samples" {
            let report = nsdb::materialize_provider_samples(Path::new(&plan.output_dir), None)?;
            return Ok(applied_report(
                next_action,
                &format!(
                    "applied materialize-provider-samples:{}:{}",
                    report.status, report.materialized_record_count
                ),
            ));
        }
        if command_id == "final-executable-output" && final_output_boundary_crossing_enabled() {
            let emit = nsld_emit_final_executable_report(manifest, plan)?;
            if emit.emitted {
                return Ok(applied_report(
                    next_action,
                    "applied final-executable-output",
                ));
            }
            let blocker = emit
                .blockers
                .first()
                .map(String::as_str)
                .unwrap_or("unknown");
            return Ok(NsldDriveApplyReport {
                applied: false,
                command_id: Some(command_id.to_owned()),
                command_resolved: next_action.command_resolved.clone(),
                gate_action: next_action.gate_action.clone(),
                gate_env_assignments: next_action.gate_env_assignments.clone(),
                crossing_env_assignments: next_action.crossing_env_assignments.clone(),
                crossing_command_resolved: next_action.crossing_command_resolved.clone(),
                message: format!("final-output-boundary-not-emitted:{blocker}"),
            });
        }
        return Ok(NsldDriveApplyReport {
            applied: false,
            command_id: Some(command_id.to_owned()),
            command_resolved: next_action.command_resolved.clone(),
            gate_action: next_action.gate_action.clone(),
            gate_env_assignments: next_action.gate_env_assignments.clone(),
            crossing_env_assignments: next_action.crossing_env_assignments.clone(),
            crossing_command_resolved: next_action.crossing_command_resolved.clone(),
            message: format!("read-only-boundary:{command_id}"),
        });
    }
    match command_id {
        "prepare" => {
            nsld_prepare_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied prepare"))
        }
        "emit-inputs" => {
            nsld_emit_link_inputs_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-inputs"))
        }
        "emit-units" => {
            nsld_emit_link_units_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-units"))
        }
        "emit-bundle" => {
            nsld_emit_link_bundle_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-bundle"))
        }
        "emit-assemble-plan" => {
            nsld_emit_assemble_plan_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-assemble-plan"))
        }
        "emit-section-manifest" => {
            nsld_emit_section_manifest_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-section-manifest"))
        }
        "emit-object-plan" => {
            nsld_emit_object_plan_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-object-plan"))
        }
        "emit-object-byte-layout" => {
            nsld_emit_object_byte_layout_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-object-byte-layout",
            ))
        }
        "emit-object-file-layout" => {
            nsld_emit_object_file_layout_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-object-file-layout",
            ))
        }
        "emit-object-image-dry-run" => {
            nsld_emit_object_image_dry_run_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-object-image-dry-run",
            ))
        }
        "emit-object" | "emit-native-object" => {
            nsld_emit_object_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                &format!("applied {command_id}"),
            ))
        }
        "emit-object-writer-dry-run" => {
            nsld_emit_object_writer_dry_run_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-object-writer-dry-run",
            ))
        }
        "emit-container-plan" => {
            nsld_emit_container_plan_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-container-plan"))
        }
        "emit-container" => {
            nsld_emit_container_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-container"))
        }
        "emit-closure" => {
            nsld_emit_closure_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-closure"))
        }
        "emit-final-stage-plan" => {
            nsld_emit_final_stage_plan_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-final-stage-plan"))
        }
        "emit-final-executable-pipeline" => {
            nsld_emit_final_executable_pipeline_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-final-executable-pipeline",
            ))
        }
        "emit-final-executable-launcher-manifest" => {
            nsld_emit_final_executable_launcher_manifest_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-final-executable-launcher-manifest",
            ))
        }
        "emit-final-executable-launcher-dry-run" => {
            nsld_emit_final_executable_launcher_dry_run_report(manifest, plan)?;
            Ok(applied_report(
                next_action,
                "applied emit-final-executable-launcher-dry-run",
            ))
        }
        other => Ok(NsldDriveApplyReport {
            applied: false,
            command_id: Some(other.to_owned()),
            command_resolved: next_action.command_resolved.clone(),
            gate_action: next_action.gate_action.clone(),
            gate_env_assignments: next_action.gate_env_assignments.clone(),
            crossing_env_assignments: next_action.crossing_env_assignments.clone(),
            crossing_command_resolved: next_action.crossing_command_resolved.clone(),
            message: format!("next-action-not-whitelisted:{other}"),
        }),
    }
}

pub(crate) fn nsld_drive_apply_until_clean(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldDriveUntilCleanReport, String> {
    const MAX_STEPS: usize = 64;
    let mut messages = Vec::new();
    let mut applied_command_ids = Vec::new();
    let mut applied_steps = 0;
    for _ in 0..MAX_STEPS {
        let report = nsld_check_report(manifest, plan);
        let next_action = nsld_check_next_action(&report);
        let command_id = next_action.command_id.clone();
        if let Some(command_id) = command_id.as_deref() {
            if applied_command_ids
                .iter()
                .any(|applied| applied == command_id)
            {
                messages.push(format!("repeated-next-action:{command_id}"));
                return Ok(NsldDriveUntilCleanReport {
                    completed: false,
                    applied_steps,
                    capped: false,
                    stop_reason: "repeated-next-action".to_owned(),
                    stop_command_id: Some(command_id.to_owned()),
                    stop_source: next_action.source.clone(),
                    stop_command_resolved: next_action.command_resolved.clone(),
                    stop_action_reason: next_action.reason.clone(),
                    stop_gate_action: next_action.gate_action.clone(),
                    stop_gate_env_assignments: next_action.gate_env_assignments.clone(),
                    stop_crossing_env_assignments: next_action.crossing_env_assignments.clone(),
                    stop_crossing_command_resolved: next_action.crossing_command_resolved.clone(),
                    last_command_id: applied_command_ids.last().cloned(),
                    messages,
                });
            }
        }
        let apply_report = nsld_drive_apply_next_action(manifest, plan, &next_action)?;
        let applied = apply_report.applied;
        messages.push(apply_report.message);
        if !applied {
            let stop_reason = if next_action.source.as_deref() == Some("final-output-boundary") {
                final_output_boundary_stop_reason(next_action.reason.as_deref())
            } else if next_action.available {
                "not-applied"
            } else {
                "clean"
            };
            let stop_action_reason =
                drive_stop_action_reason(&report, stop_reason, next_action.reason.as_deref());
            let stop_gate_action = next_action
                .gate_action
                .clone()
                .or_else(|| drive_stop_gate_action(&report, stop_reason));
            let stop_gate_env_assignments = if next_action.gate_env_assignments.is_empty() {
                stop_gate_action_env_assignments(stop_gate_action.as_deref())
            } else {
                next_action.gate_env_assignments.clone()
            };
            return Ok(NsldDriveUntilCleanReport {
                completed: !next_action.available,
                applied_steps,
                capped: false,
                stop_reason: stop_reason.to_owned(),
                stop_command_id: command_id,
                stop_source: next_action.source.clone(),
                stop_command_resolved: next_action.command_resolved.clone(),
                stop_action_reason,
                stop_gate_action,
                stop_gate_env_assignments,
                stop_crossing_env_assignments: next_action.crossing_env_assignments.clone(),
                stop_crossing_command_resolved: next_action.crossing_command_resolved.clone(),
                last_command_id: applied_command_ids.last().cloned(),
                messages,
            });
        }
        if let Some(command_id) = command_id {
            applied_command_ids.push(command_id);
        }
        applied_steps += 1;
    }
    Ok(NsldDriveUntilCleanReport {
        completed: false,
        applied_steps,
        capped: true,
        stop_reason: "max-steps".to_owned(),
        stop_command_id: None,
        stop_source: None,
        stop_command_resolved: None,
        stop_action_reason: None,
        stop_gate_action: None,
        stop_gate_env_assignments: Vec::new(),
        stop_crossing_env_assignments: Vec::new(),
        stop_crossing_command_resolved: None,
        last_command_id: applied_command_ids.last().cloned(),
        messages,
    })
}

fn drive_stop_gate_action(
    report: &crate::reports::NsldCheckReport,
    stop_reason: &str,
) -> Option<String> {
    if stop_reason == "host-finalizer-policy-required" {
        return report.final_executable_host_finalizer_gate_action.clone();
    }
    None
}

fn stop_gate_action_env_assignments(action: Option<&str>) -> Vec<String> {
    action
        .and_then(|action| action.strip_prefix("set-env:"))
        .map(|assignment| vec![assignment.to_owned()])
        .unwrap_or_default()
}

fn drive_stop_action_reason(
    report: &crate::reports::NsldCheckReport,
    stop_reason: &str,
    action_reason: Option<&str>,
) -> Option<String> {
    let reason = action_reason?;
    if stop_reason != "host-finalizer-policy-required" {
        return Some(reason.to_owned());
    }
    let Some(action) = report
        .final_executable_host_finalizer_gate_action
        .as_deref()
    else {
        return Some(reason.to_owned());
    };
    Some(format!("{reason}; next_gate_action:{action}"))
}

fn final_output_boundary_stop_reason(reason: Option<&str>) -> &'static str {
    match reason {
        Some(reason) if reason.contains("device-provider-sample:") => {
            "provider-sample-materialization-required"
        }
        Some(reason) if reason.contains("final-executable-output:not-nsld-owned") => {
            "host-finalizer-policy-required"
        }
        Some(reason) if reason.contains("final-executable-output:missing") => {
            "final-output-missing"
        }
        Some(reason) if reason.contains("final-executable-output:image-header-invalid") => {
            "final-output-invalid"
        }
        Some(reason) if reason.contains("final-executable-output:hash-mismatch") => {
            "final-output-invalid"
        }
        Some(reason) if reason.contains("final-executable-output:size-mismatch") => {
            "final-output-invalid"
        }
        _ => "blocked-boundary",
    }
}

fn final_output_boundary_crossing_enabled() -> bool {
    final_output_boundary_crossing_enabled_for(
        env::var(HOST_FINALIZER_POLICY_ENV).ok().as_deref(),
        env::var(HOST_FINALIZER_ALLOW_ENV).ok().as_deref(),
    )
}

fn final_output_boundary_crossing_enabled_for(policy: Option<&str>, allow: Option<&str>) -> bool {
    value_allows(policy, &["allow-host-invoke", "allow"])
        && value_allows(allow, &["1", "true", "yes", "allow"])
}

fn value_allows(value: Option<&str>, accepted: &[&str]) -> bool {
    value
        .map(|value| {
            let value = value.trim();
            accepted
                .iter()
                .any(|accepted| value.eq_ignore_ascii_case(accepted))
        })
        .unwrap_or(false)
}

fn applied_report(next_action: &NsldCheckNextAction, message: &str) -> NsldDriveApplyReport {
    NsldDriveApplyReport {
        applied: true,
        command_id: next_action.command_id.clone(),
        command_resolved: next_action.command_resolved.clone(),
        gate_action: next_action.gate_action.clone(),
        gate_env_assignments: next_action.gate_env_assignments.clone(),
        crossing_env_assignments: next_action.crossing_env_assignments.clone(),
        crossing_command_resolved: next_action.crossing_command_resolved.clone(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
#[path = "drive_gate_tests.rs"]
mod gate_tests;
#[cfg(test)]
#[path = "drive_tests.rs"]
mod tests;

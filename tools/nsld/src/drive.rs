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
    nsld_emit_final_executable_pipeline_report, nsld_emit_final_stage_plan_report,
    nsld_emit_link_bundle_report, nsld_emit_link_inputs_report, nsld_emit_link_units_report,
    nsld_emit_section_manifest_report, nsld_prepare_report,
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDriveApplyReport {
    pub(crate) applied: bool,
    pub(crate) command_id: Option<String>,
    pub(crate) command_resolved: Option<String>,
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
        json_optional_string_field("source", next_action.source.as_deref()),
        json_optional_string_field("command_id", next_action.command_id.as_deref()),
        json_optional_string_field("command", next_action.command.as_deref()),
        json_optional_string_field("command_resolved", next_action.command_resolved.as_deref()),
        json_optional_string_field("reason", next_action.reason.as_deref()),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_drive_apply_report_json(report: &NsldDriveApplyReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_drive_apply"),
        json_bool_field("applied", report.applied),
        json_optional_string_field("command_id", report.command_id.as_deref()),
        json_optional_string_field("command_resolved", report.command_resolved.as_deref()),
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
        json_bool_field("capped", report.capped),
        json_string_field("stop_reason", &report.stop_reason),
        json_optional_string_field("stop_command_id", report.stop_command_id.as_deref()),
        json_optional_string_field("stop_source", report.stop_source.as_deref()),
        json_optional_string_field(
            "stop_command_resolved",
            report.stop_command_resolved.as_deref(),
        ),
        json_optional_string_field("stop_action_reason", report.stop_action_reason.as_deref()),
        json_optional_string_field("last_command_id", report.last_command_id.as_deref()),
        json_string_array_field("messages", &report.messages),
    ];
    format!("{{{}}}", fields.join(","))
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
            message: "no-next-action".to_owned(),
        });
    }
    let Some(command_id) = next_action.command_id.as_deref() else {
        return Ok(NsldDriveApplyReport {
            applied: false,
            command_id: None,
            command_resolved: next_action.command_resolved.clone(),
            message: "next-action-command-id-missing".to_owned(),
        });
    };
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
        "emit-object" => {
            nsld_emit_object_report(manifest, plan)?;
            Ok(applied_report(next_action, "applied emit-object"))
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
        other => Ok(NsldDriveApplyReport {
            applied: false,
            command_id: Some(other.to_owned()),
            command_resolved: next_action.command_resolved.clone(),
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
                    last_command_id: applied_command_ids.last().cloned(),
                    messages,
                });
            }
        }
        let apply_report = nsld_drive_apply_next_action(manifest, plan, &next_action)?;
        let applied = apply_report.applied;
        messages.push(apply_report.message);
        if !applied {
            return Ok(NsldDriveUntilCleanReport {
                completed: !next_action.available,
                applied_steps,
                capped: false,
                stop_reason: if next_action.available {
                    "not-applied".to_owned()
                } else {
                    "clean".to_owned()
                },
                stop_command_id: command_id,
                stop_source: next_action.source.clone(),
                stop_command_resolved: next_action.command_resolved.clone(),
                stop_action_reason: next_action.reason.clone(),
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
        last_command_id: applied_command_ids.last().cloned(),
        messages,
    })
}

fn applied_report(next_action: &NsldCheckNextAction, message: &str) -> NsldDriveApplyReport {
    NsldDriveApplyReport {
        applied: true,
        command_id: next_action.command_id.clone(),
        command_resolved: next_action.command_resolved.clone(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
#[path = "drive_tests.rs"]
mod tests;

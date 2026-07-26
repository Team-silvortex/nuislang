use super::{
    cli::Command,
    context::load_link_input_context,
    final_executable_output::nsld_final_executable_output_report,
    final_executable_output_nsdb_handoff::{
        attach_final_output_nsdb_handoff_summary, persist_final_output_nsdb_handoff,
    },
    final_executable_pipeline::nsld_emit_final_executable_pipeline_report,
    final_executable_provider_sample::{
        nsld_device_provider_sample_evidence, NsldDeviceProviderSampleEvidence,
    },
    json_fields::*,
    prepare::nsld_prepare_report,
};
use nuisc::linker::LinkPlan;
use std::path::Path;

const SEAL_PROTOCOL: &str = "nsld-provider-neutral-seal-v1";
const SEAL_STAGE_COUNT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldSealReport {
    pub(crate) protocol: &'static str,
    pub(crate) manifest: String,
    pub(crate) output_dir: String,
    pub(crate) output_path: String,
    pub(crate) packaging_mode: String,
    pub(crate) final_link_mode: String,
    pub(crate) preflight_valid: bool,
    pub(crate) provider_manifest_available: bool,
    pub(crate) provider_manifest_status: String,
    pub(crate) provider_record_count: usize,
    pub(crate) provider_ready_record_count: usize,
    pub(crate) provider_pending_record_count: usize,
    pub(crate) provider_blocked_record_count: usize,
    pub(crate) selected_provider_bundle_count: Option<usize>,
    pub(crate) selected_provider_bundle_set_validation_status: String,
    pub(crate) bounded_stage_count: usize,
    pub(crate) completed_stage_count: usize,
    pub(crate) prepare_attempted: bool,
    pub(crate) prepare_valid: bool,
    pub(crate) pipeline_attempted: bool,
    pub(crate) pipeline_valid: bool,
    pub(crate) final_executable_emitted: bool,
    pub(crate) publish_attempted: bool,
    pub(crate) boundary_status: String,
    pub(crate) final_output_nsdb_handoff_persisted: bool,
    pub(crate) final_image_binding_proof_status: String,
    pub(crate) final_image_binding_proof_hash: Option<String>,
    pub(crate) replay_ready: bool,
    pub(crate) replay_status: String,
    pub(crate) loader_selected_provider_bundle_count: Option<usize>,
    pub(crate) loader_provider_dispatch_status: String,
    pub(crate) loader_provider_dispatch_count: usize,
    pub(crate) loader_provider_dispatch_table_hash: Option<String>,
    pub(crate) provider_completion_count: usize,
    pub(crate) completed: bool,
    pub(crate) blockers: Vec<String>,
}

impl NsldSealReport {
    fn new(manifest: &Path, plan: &LinkPlan, provider: &NsldDeviceProviderSampleEvidence) -> Self {
        Self {
            protocol: SEAL_PROTOCOL,
            manifest: manifest.display().to_string(),
            output_dir: plan.output_dir.clone(),
            output_path: plan.final_stage.output_path.clone(),
            packaging_mode: plan.packaging_mode.clone(),
            final_link_mode: plan.final_stage.link_mode.clone(),
            preflight_valid: false,
            provider_manifest_available: provider.available,
            provider_manifest_status: provider.status.clone(),
            provider_record_count: provider.record_count,
            provider_ready_record_count: provider.ready_record_count,
            provider_pending_record_count: provider.pending_record_count,
            provider_blocked_record_count: provider.blocked_record_count,
            selected_provider_bundle_count: provider.selected_provider_bundle_count,
            selected_provider_bundle_set_validation_status: provider
                .selected_provider_bundle_set_validation_status
                .clone(),
            bounded_stage_count: SEAL_STAGE_COUNT,
            completed_stage_count: 0,
            prepare_attempted: false,
            prepare_valid: false,
            pipeline_attempted: false,
            pipeline_valid: false,
            final_executable_emitted: false,
            publish_attempted: false,
            boundary_status: "not-attempted".to_owned(),
            final_output_nsdb_handoff_persisted: false,
            final_image_binding_proof_status: "not-attempted".to_owned(),
            final_image_binding_proof_hash: None,
            replay_ready: false,
            replay_status: "not-attempted".to_owned(),
            loader_selected_provider_bundle_count: None,
            loader_provider_dispatch_status: "not-attempted".to_owned(),
            loader_provider_dispatch_count: 0,
            loader_provider_dispatch_table_hash: None,
            provider_completion_count: 0,
            completed: false,
            blockers: Vec::new(),
        }
    }
}

pub(crate) fn run_seal_command(command: &Command) -> Result<bool, String> {
    let Command::Seal { input, json } = command else {
        return Ok(false);
    };
    let ctx = load_link_input_context(input)?;
    let report = nsld_seal_report(&ctx.manifest, &ctx.plan);
    if *json {
        println!("{}", nsld_seal_report_json(&report));
    } else {
        print_nsld_seal_report(&report);
    }
    if report.completed {
        Ok(true)
    } else {
        Err(format!(
            "nsld seal failed: {}",
            report
                .blockers
                .first()
                .map(String::as_str)
                .unwrap_or("unknown blocker")
        ))
    }
}

pub(crate) fn nsld_seal_report(manifest: &Path, plan: &LinkPlan) -> NsldSealReport {
    let provider = nsld_device_provider_sample_evidence(&plan.output_dir);
    let mut report = NsldSealReport::new(manifest, plan, &provider);
    validate_seal_preflight(plan, &provider, &mut report.blockers);
    report.preflight_valid = report.blockers.is_empty();
    if !report.preflight_valid {
        return report;
    }

    report.prepare_attempted = true;
    let prepare = match nsld_prepare_report(manifest, plan) {
        Ok(prepare) => prepare,
        Err(error) => {
            report.blockers.push(format!("prepare:error:{error}"));
            return report;
        }
    };
    report.prepare_valid = prepare.valid;
    if !prepare.valid {
        report.blockers.push("prepare:invalid".to_owned());
        return report;
    }
    report.completed_stage_count = 1;

    report.pipeline_attempted = true;
    let pipeline = match nsld_emit_final_executable_pipeline_report(manifest, plan) {
        Ok(pipeline) => pipeline,
        Err(error) => {
            report
                .blockers
                .push(format!("final-executable-pipeline:error:{error}"));
            return report;
        }
    };
    report.pipeline_valid = pipeline.valid;
    report.final_executable_emitted = pipeline.final_executable_emitted;
    if !pipeline.valid || !pipeline.final_executable_emitted {
        report
            .blockers
            .push("final-executable-pipeline:not-ready".to_owned());
        report.blockers.extend(pipeline.blockers);
        return report;
    }
    report.completed_stage_count = 2;

    report.publish_attempted = true;
    let mut output = nsld_final_executable_output_report(manifest, plan);
    let summary = persist_final_output_nsdb_handoff(Path::new(&plan.output_dir), &output);
    attach_final_output_nsdb_handoff_summary(&mut output, summary);
    report.boundary_status = output.boundary_status.clone();
    report.final_output_nsdb_handoff_persisted = output.final_output_nsdb_handoff_persisted;
    report.final_image_binding_proof_status = output
        .final_output_nsdb_final_image_binding_proof_status
        .clone();
    report.final_image_binding_proof_hash = output
        .final_output_nsdb_final_image_binding_proof_hash
        .clone();
    report.replay_ready = output.final_output_nsdb_replay_ready;
    report.replay_status = output.final_output_nsdb_replay_status.clone();
    report.loader_selected_provider_bundle_count =
        output.container_loader_selected_provider_bundle_count;
    report.loader_provider_dispatch_status =
        output.container_loader_provider_dispatch_status.clone();
    report.loader_provider_dispatch_count = output.container_loader_provider_dispatch_count;
    report.loader_provider_dispatch_table_hash =
        output.container_loader_provider_dispatch_table_hash.clone();
    report.provider_completion_count = output.final_output_nsdb_provider_completion_count;
    validate_sealed_output(&provider, &output, &mut report.blockers);
    if report.blockers.is_empty() {
        report.completed_stage_count = SEAL_STAGE_COUNT;
        report.completed = true;
    }
    report
}

fn validate_seal_preflight(
    plan: &LinkPlan,
    provider: &NsldDeviceProviderSampleEvidence,
    blockers: &mut Vec<String>,
) {
    if plan.packaging_mode != "nuis-self-contained-image" {
        blockers.push(format!(
            "packaging-mode:not-self-contained:{}",
            plan.packaging_mode
        ));
    }
    if plan.final_stage.link_mode != "self-contained" {
        blockers.push(format!(
            "final-link-mode:not-self-contained:{}",
            plan.final_stage.link_mode
        ));
    }
    if !provider.available {
        return;
    }
    if provider.record_count == 0 {
        if provider.status != "empty" {
            blockers.push(format!(
                "provider-manifest:not-sealable:{}",
                provider.status
            ));
        }
        return;
    }
    if provider.status != "ready"
        || provider.ready_record_count != provider.record_count
        || provider.pending_record_count != 0
        || provider.blocked_record_count != 0
    {
        blockers.push(format!("provider-manifest:not-ready:{}", provider.status));
    }
    if provider.selected_provider_bundle_set_validation_status != "verified" {
        blockers.push(format!(
            "selected-provider-set:not-verified:{}",
            provider.selected_provider_bundle_set_validation_status
        ));
    }
}

fn validate_sealed_output(
    provider: &NsldDeviceProviderSampleEvidence,
    output: &super::reports::NsldFinalExecutableOutputReport,
    blockers: &mut Vec<String>,
) {
    if output.boundary_status != "ready" {
        blockers.push(format!(
            "final-output-boundary:not-ready:{}",
            output.boundary_status
        ));
    }
    if !output.final_output_nsdb_handoff_persisted {
        blockers.push(
            output
                .final_output_nsdb_handoff_error
                .as_deref()
                .map(|error| format!("final-output-handoff:not-persisted:{error}"))
                .unwrap_or_else(|| "final-output-handoff:not-persisted".to_owned()),
        );
    }
    if !matches!(
        output
            .final_output_nsdb_final_image_binding_proof_status
            .as_str(),
        "verified" | "verified-empty"
    ) {
        blockers.push(format!(
            "final-image-binding-proof:not-verified:{}",
            output.final_output_nsdb_final_image_binding_proof_status
        ));
    }
    if !output.final_output_nsdb_replay_ready
        || output.final_output_nsdb_replay_status != "replay-evidence-ready"
    {
        blockers.push(format!(
            "final-output-replay:not-ready:{}",
            output.final_output_nsdb_replay_status
        ));
    }
    if provider.record_count == 0 {
        return;
    }
    if output.device_provider_sample_manifest_status != "ready" {
        blockers.push(format!(
            "sealed-provider-manifest:not-ready:{}",
            output.device_provider_sample_manifest_status
        ));
    }
    if output.container_loader_selected_provider_bundle_count
        != provider.selected_provider_bundle_count
    {
        blockers.push("sealed-provider-set:loader-count-mismatch".to_owned());
    }
    if output.container_loader_provider_dispatch_status != "verified"
        || output.container_loader_provider_dispatch_count
            != provider.selected_provider_bundle_count.unwrap_or_default()
        || output
            .container_loader_provider_dispatch_table_hash
            .is_none()
    {
        blockers.push("sealed-provider-dispatch:not-verified".to_owned());
    }
    if output.final_output_nsdb_provider_completion_count != provider.record_count {
        blockers.push("sealed-provider-completions:count-mismatch".to_owned());
    }
}

pub(crate) fn nsld_seal_report_json(report: &NsldSealReport) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_seal"),
        json_string_field("protocol", report.protocol),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("output_path", &report.output_path),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("final_link_mode", &report.final_link_mode),
        json_bool_field("preflight_valid", report.preflight_valid),
        json_bool_field(
            "provider_manifest_available",
            report.provider_manifest_available,
        ),
        json_string_field("provider_manifest_status", &report.provider_manifest_status),
        json_usize_field("provider_record_count", report.provider_record_count),
        json_usize_field(
            "provider_ready_record_count",
            report.provider_ready_record_count,
        ),
        json_usize_field(
            "provider_pending_record_count",
            report.provider_pending_record_count,
        ),
        json_usize_field(
            "provider_blocked_record_count",
            report.provider_blocked_record_count,
        ),
        json_optional_usize_field(
            "selected_provider_bundle_count",
            report.selected_provider_bundle_count,
        ),
        json_string_field(
            "selected_provider_bundle_set_validation_status",
            &report.selected_provider_bundle_set_validation_status,
        ),
        json_usize_field("bounded_stage_count", report.bounded_stage_count),
        json_usize_field("completed_stage_count", report.completed_stage_count),
        json_bool_field("prepare_attempted", report.prepare_attempted),
        json_bool_field("prepare_valid", report.prepare_valid),
        json_bool_field("pipeline_attempted", report.pipeline_attempted),
        json_bool_field("pipeline_valid", report.pipeline_valid),
        json_bool_field("final_executable_emitted", report.final_executable_emitted),
        json_bool_field("publish_attempted", report.publish_attempted),
        json_string_field("boundary_status", &report.boundary_status),
        json_bool_field(
            "final_output_nsdb_handoff_persisted",
            report.final_output_nsdb_handoff_persisted,
        ),
        json_string_field(
            "final_image_binding_proof_status",
            &report.final_image_binding_proof_status,
        ),
        json_optional_string_field(
            "final_image_binding_proof_hash",
            report.final_image_binding_proof_hash.as_deref(),
        ),
        json_bool_field("replay_ready", report.replay_ready),
        json_string_field("replay_status", &report.replay_status),
        json_optional_usize_field(
            "loader_selected_provider_bundle_count",
            report.loader_selected_provider_bundle_count,
        ),
        json_string_field(
            "loader_provider_dispatch_status",
            &report.loader_provider_dispatch_status,
        ),
        json_usize_field(
            "loader_provider_dispatch_count",
            report.loader_provider_dispatch_count,
        ),
        json_optional_string_field(
            "loader_provider_dispatch_table_hash",
            report.loader_provider_dispatch_table_hash.as_deref(),
        ),
        json_usize_field(
            "provider_completion_count",
            report.provider_completion_count,
        ),
        json_bool_field("completed", report.completed),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn print_nsld_seal_report(report: &NsldSealReport) {
    println!("Nsld seal");
    println!("  protocol: {}", report.protocol);
    println!("  manifest: {}", report.manifest);
    println!("  output_dir: {}", report.output_dir);
    println!("  packaging_mode: {}", report.packaging_mode);
    println!("  final_link_mode: {}", report.final_link_mode);
    println!("  preflight_valid: {}", report.preflight_valid);
    println!(
        "  provider_manifest: status={} records={} ready={} pending={} blocked={}",
        report.provider_manifest_status,
        report.provider_record_count,
        report.provider_ready_record_count,
        report.provider_pending_record_count,
        report.provider_blocked_record_count
    );
    println!(
        "  bounded_stages: completed={}/{} prepare={} pipeline={} publish={}",
        report.completed_stage_count,
        report.bounded_stage_count,
        report.prepare_valid,
        report.pipeline_valid,
        report.publish_attempted
    );
    println!("  boundary_status: {}", report.boundary_status);
    println!(
        "  final_image_binding_proof_status: {}",
        report.final_image_binding_proof_status
    );
    println!("  replay_status: {}", report.replay_status);
    println!("  completed: {}", report.completed);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

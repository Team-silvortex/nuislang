use super::{
    artifact_chain::{
        nsld_artifact_stage_kind_path, nsld_artifact_stage_kind_path_for_plan,
        NsldArtifactStageKind,
    },
    object_byte_layout::nsld_verify_object_byte_layout_report,
    object_emit::nsld_verify_object_emit_report,
    object_file_layout::nsld_verify_object_file_layout_report,
    object_image_dry_run::nsld_verify_object_image_dry_run_report,
    object_output::nsld_verify_object_output_report,
    object_plan::nsld_verify_object_plan_report,
    object_writer_input::{
        nsld_verify_object_writer_dry_run_report, nsld_verify_object_writer_input_report,
    },
    reports::{NsldObjectImageRelocationRecordDiagnostic, NsldRelocationLoweringRuleDiagnostic},
};
use std::path::Path;

pub(crate) struct NsldCheckObjectSnapshot {
    pub(crate) object_plan_present: bool,
    pub(crate) object_plan_valid: Option<bool>,
    pub(crate) object_plan_issues: Vec<String>,
    pub(crate) object_writer_input_present: bool,
    pub(crate) object_writer_input_valid: Option<bool>,
    pub(crate) object_writer_input_issues: Vec<String>,
    pub(crate) object_byte_layout_present: bool,
    pub(crate) object_byte_layout_valid: Option<bool>,
    pub(crate) object_byte_layout_issues: Vec<String>,
    pub(crate) object_file_layout_present: bool,
    pub(crate) object_file_layout_valid: Option<bool>,
    pub(crate) object_file_layout_issues: Vec<String>,
    pub(crate) object_image_dry_run_present: bool,
    pub(crate) object_image_dry_run_valid: Option<bool>,
    pub(crate) object_image_dry_run_issues: Vec<String>,
    pub(crate) object_image_relocation_lowering_valid: Option<bool>,
    pub(crate) object_image_relocation_lowering_rule_count: Option<usize>,
    pub(crate) object_image_relocation_lowering_rules: Vec<NsldRelocationLoweringRuleDiagnostic>,
    pub(crate) object_image_relocation_lowering_issues: Vec<String>,
    pub(crate) object_image_relocation_record_count: Option<usize>,
    pub(crate) object_image_relocation_record_table_hash: Option<String>,
    pub(crate) object_image_relocation_records: Vec<NsldObjectImageRelocationRecordDiagnostic>,
    pub(crate) object_image_dry_run_bytes_present: bool,
    pub(crate) object_emit_blocked_present: bool,
    pub(crate) object_emit_blocked_valid: Option<bool>,
    pub(crate) object_emit_blocked_issues: Vec<String>,
    pub(crate) object_output_present: bool,
    pub(crate) object_output_valid: Option<bool>,
    pub(crate) object_output_expected_size_bytes: Option<usize>,
    pub(crate) object_output_actual_size_bytes: Option<usize>,
    pub(crate) object_output_expected_hash: Option<String>,
    pub(crate) object_output_actual_hash: Option<String>,
    pub(crate) object_output_issues: Vec<String>,
    pub(crate) object_writer_dry_run_present: bool,
    pub(crate) object_writer_dry_run_valid: Option<bool>,
    pub(crate) object_writer_dry_run_issues: Vec<String>,
}

pub(crate) fn nsld_check_object_snapshot(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldCheckObjectSnapshot {
    let object_plan_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectPlan);
    let object_plan_present = object_plan_path.exists();
    let object_plan_verify_report =
        object_plan_present.then(|| nsld_verify_object_plan_report(manifest, plan));
    let object_plan_valid = object_plan_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_plan_issues = object_plan_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_writer_input_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectWriterInput);
    let object_writer_input_present = object_writer_input_path.exists();
    let object_writer_input_verify_report =
        object_writer_input_present.then(|| nsld_verify_object_writer_input_report(manifest, plan));
    let object_writer_input_valid = object_writer_input_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_writer_input_issues = object_writer_input_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_byte_layout_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectByteLayout);
    let object_byte_layout_present = object_byte_layout_path.exists();
    let object_byte_layout_verify_report =
        object_byte_layout_present.then(|| nsld_verify_object_byte_layout_report(manifest, plan));
    let object_byte_layout_valid = object_byte_layout_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_byte_layout_issues = object_byte_layout_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_file_layout_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectFileLayout);
    let object_file_layout_present = object_file_layout_path.exists();
    let object_file_layout_verify_report =
        object_file_layout_present.then(|| nsld_verify_object_file_layout_report(manifest, plan));
    let object_file_layout_valid = object_file_layout_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_file_layout_issues = object_file_layout_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_image_dry_run_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectImageDryRun);
    let object_image_dry_run_present = object_image_dry_run_path.exists();
    let object_image_dry_run_verify_report = object_image_dry_run_present
        .then(|| nsld_verify_object_image_dry_run_report(manifest, plan));
    let object_image_dry_run_valid = object_image_dry_run_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_image_dry_run_issues = object_image_dry_run_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_image_relocation_lowering_valid = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_valid);
    let object_image_relocation_lowering_rule_count = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_rule_count);
    let object_image_relocation_lowering_rules = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_rules.clone())
        .unwrap_or_default();
    let object_image_relocation_lowering_issues = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_lowering_issues.clone())
        .unwrap_or_default();
    let object_image_relocation_record_count = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_record_count);
    let object_image_relocation_record_table_hash = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_record_table_hash.clone());
    let object_image_relocation_records = object_image_dry_run_verify_report
        .as_ref()
        .and_then(|report| report.actual_relocation_records.clone())
        .unwrap_or_default();
    let object_image_dry_run_bytes_present = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::ObjectImageDryRunBytes,
    )
    .exists();
    let object_emit_blocked_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectEmitBlocked);
    let object_emit_blocked_present = object_emit_blocked_path.exists();
    let object_emit_blocked_verify_report =
        object_emit_blocked_present.then(|| nsld_verify_object_emit_report(manifest, plan));
    let object_emit_blocked_valid = object_emit_blocked_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_emit_blocked_issues = object_emit_blocked_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_output_path =
        nsld_artifact_stage_kind_path_for_plan(plan, NsldArtifactStageKind::ObjectOutput);
    let object_output_present = object_output_path.exists();
    let object_output_verify_report =
        object_output_present.then(|| nsld_verify_object_output_report(manifest, plan));
    let object_output_valid = object_output_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_output_expected_size_bytes = object_output_verify_report
        .as_ref()
        .and_then(|report| report.expected_size_bytes);
    let object_output_actual_size_bytes = object_output_verify_report
        .as_ref()
        .and_then(|report| report.actual_size_bytes);
    let object_output_expected_hash = object_output_verify_report
        .as_ref()
        .and_then(|report| report.expected_hash.clone());
    let object_output_actual_hash = object_output_verify_report
        .as_ref()
        .and_then(|report| report.actual_hash.clone());
    let object_output_issues = object_output_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();
    let object_writer_dry_run_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectWriterDryRun);
    let object_writer_dry_run_present = object_writer_dry_run_path.exists();
    let object_writer_dry_run_verify_report = object_writer_dry_run_present
        .then(|| nsld_verify_object_writer_dry_run_report(manifest, plan));
    let object_writer_dry_run_valid = object_writer_dry_run_verify_report
        .as_ref()
        .map(|report| report.valid);
    let object_writer_dry_run_issues = object_writer_dry_run_verify_report
        .as_ref()
        .map(|report| report.issues.clone())
        .unwrap_or_default();

    NsldCheckObjectSnapshot {
        object_plan_present,
        object_plan_valid,
        object_plan_issues,
        object_writer_input_present,
        object_writer_input_valid,
        object_writer_input_issues,
        object_byte_layout_present,
        object_byte_layout_valid,
        object_byte_layout_issues,
        object_file_layout_present,
        object_file_layout_valid,
        object_file_layout_issues,
        object_image_dry_run_present,
        object_image_dry_run_valid,
        object_image_dry_run_issues,
        object_image_relocation_lowering_valid,
        object_image_relocation_lowering_rule_count,
        object_image_relocation_lowering_rules,
        object_image_relocation_lowering_issues,
        object_image_relocation_record_count,
        object_image_relocation_record_table_hash,
        object_image_relocation_records,
        object_image_dry_run_bytes_present,
        object_emit_blocked_present,
        object_emit_blocked_valid,
        object_emit_blocked_issues,
        object_output_present,
        object_output_valid,
        object_output_expected_size_bytes,
        object_output_actual_size_bytes,
        object_output_expected_hash,
        object_output_actual_hash,
        object_output_issues,
        object_writer_dry_run_present,
        object_writer_dry_run_valid,
        object_writer_dry_run_issues,
    }
}

pub(crate) fn push_object_snapshot_issues(
    issues: &mut Vec<String>,
    snapshot: &NsldCheckObjectSnapshot,
) {
    push_object_failure(
        issues,
        snapshot.object_plan_valid,
        "object plan verification failed",
        &snapshot.object_plan_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_writer_input_valid,
        "object writer input verification failed",
        &snapshot.object_writer_input_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_byte_layout_valid,
        "object byte layout verification failed",
        &snapshot.object_byte_layout_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_file_layout_valid,
        "object file layout verification failed",
        &snapshot.object_file_layout_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_image_dry_run_valid,
        "object image dry-run verification failed",
        &snapshot.object_image_dry_run_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_emit_blocked_valid,
        "object emit blocked report verification failed",
        &snapshot.object_emit_blocked_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_output_valid,
        "object output verification failed",
        &snapshot.object_output_issues,
    );
    push_object_failure(
        issues,
        snapshot.object_writer_dry_run_valid,
        "object writer dry-run verification failed",
        &snapshot.object_writer_dry_run_issues,
    );
}

fn push_object_failure(
    issues: &mut Vec<String>,
    valid: Option<bool>,
    headline: &str,
    details: &[String],
) {
    if valid == Some(false) {
        issues.push(headline.to_owned());
        issues.extend(details.iter().cloned());
    }
}

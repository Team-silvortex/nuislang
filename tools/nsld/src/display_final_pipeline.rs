use super::{
    display_text::{optional_bool_text, optional_string_text, optional_usize_text},
    reports::{NsldFinalExecutablePipelineEmitReport, NsldFinalExecutablePipelineVerifyReport},
};

pub(crate) fn print_nsld_final_executable_pipeline_emit_report(
    report: &NsldFinalExecutablePipelineEmitReport,
) {
    println!("Nsld final executable pipeline emit");
    println!("  manifest: {}", report.manifest);
    println!("  valid: {}", report.valid);
    println!("  final_stage_plan_path: {}", report.final_stage_plan_path);
    println!("  final_output_path: {}", report.final_output_path);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  host_invoke_plan_path: {}", report.host_invoke_plan_path);
    println!("  layout_plan_path: {}", report.layout_plan_path);
    println!("  image_dry_run_path: {}", report.image_dry_run_path);
    println!(
        "  final_executable_blocked_path: {}",
        report.final_executable_blocked_path
    );
    println!(
        "  launcher_manifest_path: {}",
        report.launcher_manifest_path
    );
    println!("  launcher_dry_run_path: {}", report.launcher_dry_run_path);
    println!(
        "  final_executable_emitted: {}",
        report.final_executable_emitted
    );
    println!(
        "  launcher_manifest_ready: {}",
        report.launcher_manifest_ready
    );
    println!(
        "  launcher_dry_run_ready: {}",
        report.launcher_dry_run_ready
    );
    println!(
        "  would_enter_lifecycle_hook: {}",
        report.would_enter_lifecycle_hook
    );
    println!(
        "  self_owned_image_status: {}",
        report.self_owned_image_status
    );
    println!(
        "  entrypoint_materialization_status: {}",
        report.entrypoint_materialization_status
    );
    println!(
        "  entrypoint_materialization_kind: {}",
        report.entrypoint_materialization_kind
    );
    println!(
        "  entrypoint_materialization_path: {}",
        optional_string_text(report.entrypoint_materialization_path.as_deref())
    );
    println!(
        "  entrypoint_materialization_ready: {}",
        report.entrypoint_materialization_ready
    );
    println!(
        "  entrypoint_materialization_first_blocker: {}",
        optional_string_text(report.entrypoint_materialization_first_blocker.as_deref())
    );
    println!(
        "  entrypoint_materialization_present: {}",
        optional_bool_text(report.entrypoint_materialization_present)
    );
    println!(
        "  entrypoint_materialization_hash: {}",
        optional_string_text(report.entrypoint_materialization_hash.as_deref())
    );
    println!(
        "  entrypoint_materialization_runner_command: {}",
        optional_string_text(report.entrypoint_materialization_runner_command.as_deref())
    );
    println!(
        "  execution_handoff_contract: {}",
        report.execution_handoff_contract
    );
    println!(
        "  execution_handoff_ready: {}",
        report.execution_handoff_ready
    );
    println!(
        "  execution_handoff_status: {}",
        report.execution_handoff_status
    );
    println!(
        "  execution_handoff_target: {}",
        report.execution_handoff_target
    );
    println!(
        "  execution_handoff_evidence_status: {}",
        report.execution_handoff_evidence_status
    );
    println!(
        "  execution_handoff_first_blocker: {}",
        optional_string_text(report.execution_handoff_first_blocker.as_deref())
    );
    println!(
        "  execution_handoff_decision_code: {}",
        report.execution_handoff_decision_code
    );
    println!(
        "  required_stage_path_count: {}",
        report.required_stage_path_count
    );
    println!(
        "  required_stage_path_present_count: {}",
        report.required_stage_path_present_count
    );
    for path in &report.missing_required_stage_paths {
        println!("  missing_required_stage_path: {path}");
    }
    println!("  blocker_count: {}", report.blocker_count);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_pipeline_verify_report(
    report: &NsldFinalExecutablePipelineVerifyReport,
) {
    println!("Nsld final executable pipeline verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_pipeline_hash: {}",
        report.expected_pipeline_hash
    );
    println!(
        "  actual_pipeline_hash: {}",
        optional_string_text(report.actual_pipeline_hash.as_deref())
    );
    println!("  expected_valid: {}", report.expected_valid);
    println!(
        "  actual_valid: {}",
        optional_bool_text(report.actual_valid)
    );
    println!(
        "  expected_final_executable_emitted: {}",
        report.expected_final_executable_emitted
    );
    println!(
        "  actual_final_executable_emitted: {}",
        optional_bool_text(report.actual_final_executable_emitted)
    );
    println!(
        "  expected_launcher_manifest_ready: {}",
        report.expected_launcher_manifest_ready
    );
    println!(
        "  actual_launcher_manifest_ready: {}",
        optional_bool_text(report.actual_launcher_manifest_ready)
    );
    println!(
        "  expected_launcher_dry_run_ready: {}",
        report.expected_launcher_dry_run_ready
    );
    println!(
        "  actual_launcher_dry_run_ready: {}",
        optional_bool_text(report.actual_launcher_dry_run_ready)
    );
    println!(
        "  expected_would_enter_lifecycle_hook: {}",
        report.expected_would_enter_lifecycle_hook
    );
    println!(
        "  actual_would_enter_lifecycle_hook: {}",
        optional_bool_text(report.actual_would_enter_lifecycle_hook)
    );
    println!(
        "  expected_self_owned_image_status: {}",
        report.expected_self_owned_image_status
    );
    println!(
        "  actual_self_owned_image_status: {}",
        optional_string_text(report.actual_self_owned_image_status.as_deref())
    );
    println!(
        "  expected_entrypoint_materialization_status: {}",
        report.expected_entrypoint_materialization_status
    );
    println!(
        "  actual_entrypoint_materialization_status: {}",
        optional_string_text(report.actual_entrypoint_materialization_status.as_deref())
    );
    println!(
        "  expected_entrypoint_materialization_kind: {}",
        report.expected_entrypoint_materialization_kind
    );
    println!(
        "  actual_entrypoint_materialization_kind: {}",
        optional_string_text(report.actual_entrypoint_materialization_kind.as_deref())
    );
    println!(
        "  expected_entrypoint_materialization_path: {}",
        optional_string_text(report.expected_entrypoint_materialization_path.as_deref())
    );
    println!(
        "  actual_entrypoint_materialization_path: {}",
        optional_string_text(report.actual_entrypoint_materialization_path.as_deref())
    );
    println!(
        "  expected_entrypoint_materialization_ready: {}",
        report.expected_entrypoint_materialization_ready
    );
    println!(
        "  actual_entrypoint_materialization_ready: {}",
        optional_bool_text(report.actual_entrypoint_materialization_ready)
    );
    println!(
        "  expected_entrypoint_materialization_first_blocker: {}",
        optional_string_text(
            report
                .expected_entrypoint_materialization_first_blocker
                .as_deref()
        )
    );
    println!(
        "  actual_entrypoint_materialization_first_blocker: {}",
        optional_string_text(
            report
                .actual_entrypoint_materialization_first_blocker
                .as_deref()
        )
    );
    println!(
        "  expected_entrypoint_materialization_present: {}",
        optional_bool_text(report.expected_entrypoint_materialization_present)
    );
    println!(
        "  actual_entrypoint_materialization_present: {}",
        optional_bool_text(report.actual_entrypoint_materialization_present)
    );
    println!(
        "  expected_entrypoint_materialization_hash: {}",
        optional_string_text(report.expected_entrypoint_materialization_hash.as_deref())
    );
    println!(
        "  actual_entrypoint_materialization_hash: {}",
        optional_string_text(report.actual_entrypoint_materialization_hash.as_deref())
    );
    println!(
        "  expected_entrypoint_materialization_runner_command: {}",
        optional_string_text(
            report
                .expected_entrypoint_materialization_runner_command
                .as_deref()
        )
    );
    println!(
        "  actual_entrypoint_materialization_runner_command: {}",
        optional_string_text(
            report
                .actual_entrypoint_materialization_runner_command
                .as_deref()
        )
    );
    println!(
        "  expected_execution_handoff_contract: {}",
        report.expected_execution_handoff_contract
    );
    println!(
        "  actual_execution_handoff_contract: {}",
        optional_string_text(report.actual_execution_handoff_contract.as_deref())
    );
    println!(
        "  expected_execution_handoff_ready: {}",
        report.expected_execution_handoff_ready
    );
    println!(
        "  actual_execution_handoff_ready: {}",
        optional_bool_text(report.actual_execution_handoff_ready)
    );
    println!(
        "  expected_execution_handoff_status: {}",
        report.expected_execution_handoff_status
    );
    println!(
        "  actual_execution_handoff_status: {}",
        optional_string_text(report.actual_execution_handoff_status.as_deref())
    );
    println!(
        "  expected_execution_handoff_target: {}",
        report.expected_execution_handoff_target
    );
    println!(
        "  actual_execution_handoff_target: {}",
        optional_string_text(report.actual_execution_handoff_target.as_deref())
    );
    println!(
        "  expected_execution_handoff_evidence_status: {}",
        report.expected_execution_handoff_evidence_status
    );
    println!(
        "  actual_execution_handoff_evidence_status: {}",
        optional_string_text(report.actual_execution_handoff_evidence_status.as_deref())
    );
    println!(
        "  expected_execution_handoff_first_blocker: {}",
        optional_string_text(report.expected_execution_handoff_first_blocker.as_deref())
    );
    println!(
        "  actual_execution_handoff_first_blocker: {}",
        optional_string_text(report.actual_execution_handoff_first_blocker.as_deref())
    );
    println!(
        "  expected_execution_handoff_decision_code: {}",
        report.expected_execution_handoff_decision_code
    );
    println!(
        "  actual_execution_handoff_decision_code: {}",
        optional_string_text(report.actual_execution_handoff_decision_code.as_deref())
    );
    println!(
        "  expected_required_stage_path_count: {}",
        report.expected_required_stage_path_count
    );
    println!(
        "  actual_required_stage_path_count: {}",
        optional_usize_text(report.actual_required_stage_path_count)
    );
    println!(
        "  expected_required_stage_path_present_count: {}",
        report.expected_required_stage_path_present_count
    );
    println!(
        "  actual_required_stage_path_present_count: {}",
        optional_usize_text(report.actual_required_stage_path_present_count)
    );
    for path in &report.expected_missing_required_stage_paths {
        println!("  expected_missing_required_stage_path: {path}");
    }
    for path in &report.actual_missing_required_stage_paths {
        println!("  actual_missing_required_stage_path: {path}");
    }
    println!(
        "  expected_blocker_count: {}",
        report.expected_blocker_count
    );
    println!(
        "  actual_blocker_count: {}",
        optional_usize_text(report.actual_blocker_count)
    );
    for blocker in &report.expected_blockers {
        println!("  expected_blocker: {blocker}");
    }
    for blocker in &report.actual_blockers {
        println!("  actual_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

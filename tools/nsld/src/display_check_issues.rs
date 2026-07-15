use super::reports::NsldCheckReport;

pub(crate) fn print_check_report_issue_lines(report: &NsldCheckReport) {
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
    for issue in &report.link_input_table_issues {
        println!("  link_input_table_issue: {issue}");
    }
    for issue in &report.link_unit_table_issues {
        println!("  link_unit_table_issue: {issue}");
    }
    for issue in &report.link_bundle_issues {
        println!("  link_bundle_issue: {issue}");
    }
    for issue in &report.assemble_plan_issues {
        println!("  assemble_plan_issue: {issue}");
    }
    for issue in &report.section_manifest_issues {
        println!("  section_manifest_issue: {issue}");
    }
    for issue in &report.object_plan_issues {
        println!("  object_plan_issue: {issue}");
    }
    for issue in &report.object_writer_input_issues {
        println!("  object_writer_input_issue: {issue}");
    }
    for issue in &report.object_byte_layout_issues {
        println!("  object_byte_layout_issue: {issue}");
    }
    for issue in &report.object_file_layout_issues {
        println!("  object_file_layout_issue: {issue}");
    }
    for issue in &report.object_image_dry_run_issues {
        println!("  object_image_dry_run_issue: {issue}");
    }
    for issue in &report.object_image_relocation_lowering_issues {
        println!("  object_image_relocation_lowering_issue: {issue}");
    }
    for issue in &report.object_emit_blocked_issues {
        println!("  object_emit_blocked_issue: {issue}");
    }
    for issue in &report.object_output_issues {
        println!("  object_output_issue: {issue}");
    }
    for issue in &report.object_writer_dry_run_issues {
        println!("  object_writer_dry_run_issue: {issue}");
    }
    for issue in &report.container_plan_issues {
        println!("  container_plan_issue: {issue}");
    }
    for issue in &report.container_issues {
        println!("  container_issue: {issue}");
    }
    for issue in &report.container_section_issues {
        println!("  container_section_issue: {issue}");
    }
    for issue in &report.container_loader_symbol_issues {
        println!("  container_loader_symbol_issue: {issue}");
    }
    for issue in &report.container_relocation_issues {
        println!("  container_relocation_issue: {issue}");
    }
    for issue in &report.container_compatibility_domain_issues {
        println!("  container_compatibility_domain_issue: {issue}");
    }
    for issue in &report.container_external_import_issues {
        println!("  container_external_import_issue: {issue}");
    }
    for issue in &report.container_payload_issues {
        println!("  container_payload_issue: {issue}");
    }
    for issue in &report.closure_snapshot_issues {
        println!("  closure_snapshot_issue: {issue}");
    }
    for issue in &report.final_stage_plan_issues {
        println!("  final_stage_plan_issue: {issue}");
    }
    for issue in &report.final_executable_writer_input_issues {
        println!("  final_executable_writer_input_issue: {issue}");
    }
    for issue in &report.final_executable_host_invoke_plan_issues {
        println!("  final_executable_host_invoke_plan_issue: {issue}");
    }
    for issue in &report.final_executable_layout_plan_issues {
        println!("  final_executable_layout_plan_issue: {issue}");
    }
    for issue in &report.final_executable_image_dry_run_issues {
        println!("  final_executable_image_dry_run_issue: {issue}");
    }
    for issue in &report.final_executable_blocked_issues {
        println!("  final_executable_blocked_issue: {issue}");
    }
    for blocker in &report.final_executable_output_blockers {
        println!("  final_executable_output_blocker: {blocker}");
    }
    for issue in &report.final_executable_output_object_issues {
        println!("  final_executable_output_object_issue: {issue}");
    }
    for issue in &report.final_executable_output_issues {
        println!("  final_executable_output_issue: {issue}");
    }
    for issue in &report.final_executable_launcher_manifest_issues {
        println!("  final_executable_launcher_manifest_issue: {issue}");
    }
    for issue in &report.final_executable_launcher_dry_run_issues {
        println!("  final_executable_launcher_dry_run_issue: {issue}");
    }
    for issue in &report.final_executable_pipeline_issues {
        println!("  final_executable_pipeline_issue: {issue}");
    }
    for path in &report.final_executable_pipeline_missing_required_stage_paths {
        println!("  final_executable_pipeline_missing_required_stage_path: {path}");
    }
    for blocker in &report.container_loader_blockers {
        println!("  container_loader_blocker: {blocker}");
    }
    for advisory in &report.artifact_chain_advisories {
        println!("  artifact_chain_advisory: {advisory}");
    }
    if let Some(reason) = report.artifact_chain_advisory_command_reason.as_deref() {
        println!("  artifact_chain_advisory_command_reason: {reason}");
    }
    if let Some(reason) = report.artifact_chain_next_action_command_reason.as_deref() {
        println!("  artifact_chain_next_action_command_reason: {reason}");
    }
    if let Some(reason) = report.next_action_command_reason.as_deref() {
        println!("  next_action_command_reason: {reason}");
    }
    for issue in &report.artifact_chain_issues {
        println!("  artifact_chain_issue: {issue}");
    }
}

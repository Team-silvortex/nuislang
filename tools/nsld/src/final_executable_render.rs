use super::{
    reports::{
        NsldFinalExecutableEmitReport, NsldFinalExecutableImageDryRunReport,
        NsldFinalExecutableLayoutPlanReport, NsldFinalStagePlanReport,
    },
    toml,
};

pub(crate) fn render_final_stage_plan(report: &NsldFinalStagePlanReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-stage-plan-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("plan_kind = \"deterministic-final-stage-plan\"\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "plan_hash = \"{}\"\n",
        toml::escape_toml_string(&report.plan_hash)
    ));
    out.push_str(&format!(
        "final_stage_kind = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_kind)
    ));
    out.push_str(&format!(
        "final_stage_driver = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_driver)
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "final_output_path = \"{}\"\n",
        toml::escape_toml_string(&report.final_output_path)
    ));
    out.push_str(&format!(
        "host_wrapper_required = {}\n",
        report.host_wrapper_required
    ));
    out.push_str(&format!(
        "compatibility_mode = \"{}\"\n",
        toml::escape_toml_string(&report.compatibility_mode)
    ));
    out.push_str(&format!("input_count = {}\n", report.input_count));
    out.push_str(&format!(
        "container_hash = \"{}\"\n",
        toml::escape_toml_string(&report.container_hash)
    ));
    out.push_str(&format!(
        "payload_hash = \"{}\"\n",
        toml::escape_toml_string(&report.payload_hash)
    ));
    out.push_str(&format!(
        "linker_contract_hash = \"{}\"\n",
        toml::escape_toml_string(&report.linker_contract_hash)
    ));
    out.push_str(&format!(
        "native_object_required = {}\n",
        report.native_object_required
    ));
    out.push_str(&format!(
        "native_object_present = {}\n",
        report.native_object_present
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml::toml_string_array_literal(&report.blockers)
    ));
    out.push_str(&format!(
        "notes = [{}]\n",
        toml::toml_string_array_literal(&report.notes)
    ));
    for input in &report.inputs {
        out.push_str("\n[[final_stage_input]]\n");
        out.push_str(&format!("order_index = {}\n", input.order_index));
        out.push_str(&format!(
            "input_id = \"{}\"\n",
            toml::escape_toml_string(&input.input_id)
        ));
        out.push_str(&format!(
            "input_kind = \"{}\"\n",
            toml::escape_toml_string(&input.input_kind)
        ));
        out.push_str(&format!(
            "path = \"{}\"\n",
            toml::escape_toml_string(&input.path)
        ));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            toml::escape_toml_string(&input.content_hash)
        ));
        out.push_str(&format!("required = {}\n", input.required));
        out.push_str(&format!("present = {}\n", input.present));
    }
    out
}

pub(crate) fn render_final_executable_blocked(report: &NsldFinalExecutableEmitReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-blocked-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        toml::escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "blocked_report_path = \"{}\"\n",
        toml::escape_toml_string(&report.blocked_report_path)
    ));
    out.push_str(&format!("emitted = {}\n", report.emitted));
    out.push_str(&format!(
        "can_emit_final_executable = {}\n",
        report.can_emit_final_executable
    ));
    out.push_str(&format!(
        "final_stage_ready = {}\n",
        report.final_stage_ready
    ));
    out.push_str(&format!(
        "final_stage_plan_hash = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_plan_hash)
    ));
    out.push_str(&format!(
        "final_stage_driver = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_driver)
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "host_wrapper_required = {}\n",
        report.host_wrapper_required
    ));
    out.push_str(&format!(
        "writer_kind = \"{}\"\n",
        toml::escape_toml_string(&report.writer_kind)
    ));
    out.push_str(&format!(
        "writer_status = \"{}\"\n",
        toml::escape_toml_string(&report.writer_status)
    ));
    out.push_str(&format!(
        "writer_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.writer_blockers)
    ));
    out.push_str(&format!(
        "writer_input_path = \"{}\"\n",
        toml::escape_toml_string(&report.writer_input_path)
    ));
    out.push_str(&format!(
        "writer_input_valid = {}\n",
        optional_bool_toml(report.writer_input_valid)
    ));
    out.push_str(&format!(
        "writer_input_hash = \"{}\"\n",
        toml::escape_toml_string(report.writer_input_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "writer_input_issues = [{}]\n",
        toml::toml_string_array_literal(&report.writer_input_issues)
    ));
    out.push_str(&format!(
        "host_dry_run_environment_ready = {}\n",
        optional_bool_toml(report.host_dry_run_environment_ready)
    ));
    out.push_str(&format!(
        "host_dry_run_driver_available = {}\n",
        optional_bool_toml(report.host_dry_run_driver_available)
    ));
    out.push_str(&format!(
        "host_dry_run_driver_resolved_path = \"{}\"\n",
        toml::escape_toml_string(
            report
                .host_dry_run_driver_resolved_path
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "host_dry_run_can_invoke = {}\n",
        optional_bool_toml(report.host_dry_run_can_invoke)
    ));
    out.push_str(&format!(
        "host_dry_run_invocation_policy = \"{}\"\n",
        toml::escape_toml_string(
            report
                .host_dry_run_invocation_policy
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "host_dry_run_invocation_policy_reason = \"{}\"\n",
        toml::escape_toml_string(
            report
                .host_dry_run_invocation_policy_reason
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "host_finalizer_gate_status = \"{}\"\n",
        host_finalizer_gate_status(report)
    ));
    out.push_str(&format!(
        "host_finalizer_gate_action = \"{}\"\n",
        host_finalizer_gate_action(report)
    ));
    out.push_str(&format!(
        "host_dry_run_command_arg_count = {}\n",
        report.host_dry_run_command_args.len()
    ));
    out.push_str(&format!(
        "host_dry_run_command_args = [{}]\n",
        toml::toml_string_array_literal(&report.host_dry_run_command_args)
    ));
    out.push_str(&format!(
        "host_dry_run_blocker_count = {}\n",
        report.host_dry_run_blockers.len()
    ));
    out.push_str(&format!(
        "host_dry_run_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.host_dry_run_blockers)
    ));
    out.push_str(&format!(
        "host_invoke_plan_path = \"{}\"\n",
        toml::escape_toml_string(&report.host_invoke_plan_path)
    ));
    out.push_str(&format!(
        "host_invoke_plan_valid = {}\n",
        optional_bool_toml(report.host_invoke_plan_valid)
    ));
    out.push_str(&format!(
        "host_invoke_plan_hash = \"{}\"\n",
        toml::escape_toml_string(report.host_invoke_plan_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "host_invoke_plan_invocation_policy = \"{}\"\n",
        toml::escape_toml_string(
            report
                .host_invoke_plan_invocation_policy
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "host_invoke_plan_requires_explicit_allow = {}\n",
        optional_bool_toml(report.host_invoke_plan_requires_explicit_allow)
    ));
    out.push_str(&format!(
        "host_invoke_plan_explicit_allow_present = {}\n",
        optional_bool_toml(report.host_invoke_plan_explicit_allow_present)
    ));
    out.push_str(&format!(
        "host_invoke_plan_would_invoke = {}\n",
        optional_bool_toml(report.host_invoke_plan_would_invoke)
    ));
    out.push_str(&format!(
        "host_invoke_plan_blocker_count = {}\n",
        optional_usize_toml(report.host_invoke_plan_blocker_count)
    ));
    out.push_str(&format!(
        "host_invoke_plan_issues = [{}]\n",
        toml::toml_string_array_literal(&report.host_invoke_plan_issues)
    ));
    out.push_str(&format!(
        "layout_plan_path = \"{}\"\n",
        toml::escape_toml_string(&report.layout_plan_path)
    ));
    out.push_str(&format!(
        "layout_plan_valid = {}\n",
        optional_bool_toml(report.layout_plan_valid)
    ));
    out.push_str(&format!(
        "layout_plan_hash = \"{}\"\n",
        toml::escape_toml_string(report.layout_plan_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "layout_plan_issues = [{}]\n",
        toml::toml_string_array_literal(&report.layout_plan_issues)
    ));
    out.push_str(&format!(
        "image_dry_run_path = \"{}\"\n",
        toml::escape_toml_string(&report.image_dry_run_path)
    ));
    out.push_str(&format!(
        "image_dry_run_bytes_path = \"{}\"\n",
        toml::escape_toml_string(&report.image_dry_run_bytes_path)
    ));
    out.push_str(&format!(
        "image_dry_run_valid = {}\n",
        optional_bool_toml(report.image_dry_run_valid)
    ));
    out.push_str(&format!(
        "image_dry_run_hash = \"{}\"\n",
        toml::escape_toml_string(report.image_dry_run_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "image_dry_run_size_bytes = {}\n",
        optional_usize_toml(report.image_dry_run_size_bytes)
    ));
    out.push_str(&format!(
        "image_dry_run_resolver_status = \"{}\"\n",
        toml::escape_toml_string(
            report
                .image_dry_run_resolver_status
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "image_dry_run_patch_application_status = \"{}\"\n",
        toml::escape_toml_string(
            report
                .image_dry_run_patch_application_status
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "image_dry_run_patch_byte_audit_status = \"{}\"\n",
        toml::escape_toml_string(
            report
                .image_dry_run_patch_byte_audit_status
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "image_dry_run_patch_byte_audit_hash = \"{}\"\n",
        toml::escape_toml_string(
            report
                .image_dry_run_patch_byte_audit_hash
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "image_dry_run_issues = [{}]\n",
        toml::toml_string_array_literal(&report.image_dry_run_issues)
    ));
    out.push_str(&format!(
        "final_output_checked = {}\n",
        report.final_output_checked
    ));
    out.push_str(&format!(
        "final_output_present = {}\n",
        report.final_output_present
    ));
    out.push_str(&format!(
        "final_output_size_bytes = {}\n",
        optional_usize_toml(report.final_output_size_bytes)
    ));
    out.push_str(&format!(
        "final_output_hash = \"{}\"\n",
        toml::escape_toml_string(report.final_output_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "final_output_image_header_valid = {}\n",
        optional_bool_toml(report.final_output_image_header_valid)
    ));
    out.push_str(&format!(
        "final_output_runnable_candidate = {}\n",
        optional_bool_toml(report.final_output_runnable_candidate)
    ));
    out.push_str(&format!("input_count = {}\n", report.input_count));
    out.push_str(&format!("blocker_count = {}\n", report.blockers.len()));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml::toml_string_array_literal(&report.blockers)
    ));
    out.push_str(&format!(
        "notes = [{}]\n",
        toml::toml_string_array_literal(&report.notes)
    ));
    out
}

pub(crate) fn host_finalizer_gate_status(report: &NsldFinalExecutableEmitReport) -> &'static str {
    if !report.host_wrapper_required {
        return "not-required";
    }
    if report.host_invoke_plan_would_invoke == Some(true) {
        return "open";
    }
    if report.host_dry_run_environment_ready == Some(false) {
        return "environment-blocked";
    }
    if report.host_invoke_plan_valid == Some(false) {
        return "invoke-plan-invalid";
    }
    if report.host_dry_run_invocation_policy.as_deref() != Some("allow-host-invoke") {
        return "policy-blocked";
    }
    if report.host_invoke_plan_explicit_allow_present == Some(false) {
        return "explicit-allow-missing";
    }
    "blocked"
}

pub(crate) fn host_finalizer_gate_action(report: &NsldFinalExecutableEmitReport) -> &'static str {
    match host_finalizer_gate_status(report) {
        "not-required" => "none",
        "open" => "emit-final-executable",
        "environment-blocked" => "fix-host-finalizer-environment",
        "invoke-plan-invalid" => "emit-final-executable-host-invoke-plan",
        "policy-blocked" => "set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke",
        "explicit-allow-missing" => "set-env:NUIS_NSLD_ALLOW_HOST_FINALIZER=1",
        _ => "inspect-host-finalizer-blockers",
    }
}

pub(crate) fn render_final_executable_layout_plan(
    report: &NsldFinalExecutableLayoutPlanReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-layout-plan-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        toml::escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "layout_hash = \"{}\"\n",
        toml::escape_toml_string(&report.layout_hash)
    ));
    out.push_str(&format!(
        "final_stage_plan_hash = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_plan_hash)
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "platform_envelope_family = \"{}\"\n",
        toml::escape_toml_string(&report.platform_envelope_family)
    ));
    out.push_str(&format!(
        "platform_envelope_policy = \"{}\"\n",
        toml::escape_toml_string(&report.platform_envelope_policy)
    ));
    out.push_str(&format!(
        "internal_binary_format = \"{}\"\n",
        toml::escape_toml_string(&report.internal_binary_format)
    ));
    out.push_str(&format!(
        "lifecycle_entry_hook = \"{}\"\n",
        toml::escape_toml_string(&report.lifecycle_entry_hook)
    ));
    out.push_str(&format!(
        "scheduler_contract = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_contract)
    ));
    out.push_str(&format!(
        "scheduler_metadata_payload = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_metadata_payload)
    ));
    out.push_str(&format!(
        "scheduler_metadata_lifecycle_hook = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_metadata_lifecycle_hook)
    ));
    out.push_str(&format!(
        "scheduler_hetero_node_count = {}\n",
        report.scheduler_hetero_node_count
    ));
    out.push_str(&format!(
        "scheduler_wait_event_count = {}\n",
        report.scheduler_wait_event_count
    ));
    out.push_str(&format!(
        "scheduler_emit_event_count = {}\n",
        report.scheduler_emit_event_count
    ));
    out.push_str(&format!(
        "data_segment_ordering = \"{}\"\n",
        toml::escape_toml_string(&report.data_segment_ordering)
    ));
    out.push_str(&format!(
        "relocation_application_strategy = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_strategy)
    ));
    out.push_str(&format!(
        "relocation_application_table_source = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_table_source)
    ));
    out.push_str(&format!(
        "relocation_application_count = {}\n",
        report.relocation_application_count
    ));
    out.push_str(&format!(
        "relocation_application_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_table_hash)
    ));
    out.push_str(&format!(
        "native_object_path = \"{}\"\n",
        toml::escape_toml_string(&report.native_object_path)
    ));
    out.push_str(&format!(
        "native_object_required = {}\n",
        report.native_object_required
    ));
    out.push_str(&format!(
        "native_object_present = {}\n",
        report.native_object_present
    ));
    out.push_str(&format!(
        "compatibility_domain = \"{}\"\n",
        toml::escape_toml_string(&report.compatibility_domain)
    ));
    out.push_str(&format!(
        "compatibility_lifecycle_hook = \"{}\"\n",
        toml::escape_toml_string(&report.compatibility_lifecycle_hook)
    ));
    out.push_str(&format!("payload_count = {}\n", report.payload_count));
    out.push_str(&format!(
        "payloads = [{}]\n",
        toml::toml_string_array_literal(&report.payload_names)
    ));
    out.push_str(&format!("byte_alignment = {}\n", report.byte_alignment));
    out.push_str(&format!("byte_span = {}\n", report.byte_span));
    out.push_str(&format!(
        "byte_map_hash = \"{}\"\n",
        toml::escape_toml_string(&report.byte_map_hash)
    ));
    out.push_str(&format!(
        "notes = [{}]\n",
        toml::toml_string_array_literal(&report.notes)
    ));
    for payload in &report.payloads {
        out.push_str("\n[[payload]]\n");
        out.push_str(&format!("order_index = {}\n", payload.order_index));
        out.push_str(&format!(
            "payload_id = \"{}\"\n",
            toml::escape_toml_string(&payload.payload_id)
        ));
        out.push_str(&format!(
            "payload_kind = \"{}\"\n",
            toml::escape_toml_string(&payload.payload_kind)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            toml::escape_toml_string(&payload.lifecycle_hook)
        ));
        out.push_str(&format!(
            "path = \"{}\"\n",
            toml::escape_toml_string(&payload.path)
        ));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            toml::escape_toml_string(&payload.content_hash)
        ));
        out.push_str(&format!("required = {}\n", payload.required));
        out.push_str(&format!("present = {}\n", payload.present));
    }
    for entry in &report.byte_map_entries {
        out.push_str("\n[[byte_map_entry]]\n");
        out.push_str(&format!("order_index = {}\n", entry.order_index));
        out.push_str(&format!(
            "payload_id = \"{}\"\n",
            toml::escape_toml_string(&entry.payload_id)
        ));
        out.push_str(&format!(
            "payload_kind = \"{}\"\n",
            toml::escape_toml_string(&entry.payload_kind)
        ));
        out.push_str(&format!("offset = {}\n", entry.offset));
        out.push_str(&format!("size_bytes = {}\n", entry.size_bytes));
        out.push_str(&format!("alignment = {}\n", entry.alignment));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            toml::escape_toml_string(&entry.content_hash)
        ));
    }
    for record in &report.relocation_applications {
        out.push_str("\n[[relocation_application]]\n");
        out.push_str(&format!("order_index = {}\n", record.order_index));
        out.push_str(&format!(
            "relocation_id = \"{}\"\n",
            toml::escape_toml_string(&record.relocation_id)
        ));
        out.push_str(&format!(
            "relocation_kind = \"{}\"\n",
            toml::escape_toml_string(&record.relocation_kind)
        ));
        out.push_str(&format!(
            "source_payload_id = \"{}\"\n",
            toml::escape_toml_string(&record.source_payload_id)
        ));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            toml::escape_toml_string(&record.source_section_id)
        ));
        out.push_str(&format!("source_offset = {}\n", record.source_offset));
        out.push_str(&format!("image_offset = {}\n", record.image_offset));
        out.push_str(&format!(
            "target_symbol_id = \"{}\"\n",
            toml::escape_toml_string(&record.target_symbol_id)
        ));
        out.push_str(&format!("addend = {}\n", record.addend));
        out.push_str(&format!(
            "application_status = \"{}\"\n",
            toml::escape_toml_string(&record.application_status)
        ));
    }
    out
}

pub(crate) fn render_final_executable_image_dry_run(
    report: &NsldFinalExecutableImageDryRunReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-image-dry-run-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        toml::escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "image_path = \"{}\"\n",
        toml::escape_toml_string(&report.image_path)
    ));
    out.push_str(&format!(
        "image_format = \"{}\"\n",
        toml::escape_toml_string(&report.image_format)
    ));
    out.push_str(&format!(
        "image_magic = \"{}\"\n",
        toml::escape_toml_string(&report.image_magic)
    ));
    out.push_str(&format!(
        "image_header_size = {}\n",
        report.image_header_size
    ));
    out.push_str(&format!(
        "payload_byte_offset = {}\n",
        report.payload_byte_offset
    ));
    out.push_str(&format!(
        "payload_byte_span = {}\n",
        report.payload_byte_span
    ));
    out.push_str(&format!(
        "layout_hash = \"{}\"\n",
        toml::escape_toml_string(&report.layout_hash)
    ));
    out.push_str(&format!(
        "byte_map_hash = \"{}\"\n",
        toml::escape_toml_string(&report.byte_map_hash)
    ));
    out.push_str(&format!("payload_count = {}\n", report.payload_count));
    out.push_str(&format!("byte_span = {}\n", report.byte_span));
    out.push_str(&format!(
        "scheduler_metadata_payload_id = \"{}\"\n",
        toml::escape_toml_string(&report.scheduler_metadata_payload_id)
    ));
    out.push_str(&format!(
        "scheduler_metadata_present = {}\n",
        report.scheduler_metadata_present
    ));
    out.push_str(&format!(
        "scheduler_metadata_offset = {}\n",
        optional_usize_toml(report.scheduler_metadata_offset)
    ));
    out.push_str(&format!(
        "scheduler_metadata_hash = \"{}\"\n",
        toml::escape_toml_string(report.scheduler_metadata_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "relocation_application_strategy = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_strategy)
    ));
    out.push_str(&format!(
        "relocation_application_count = {}\n",
        report.relocation_application_count
    ));
    out.push_str(&format!(
        "relocation_application_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_table_hash)
    ));
    out.push_str(&format!(
        "relocation_application_audit_status = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_audit_status)
    ));
    out.push_str(&format!(
        "relocation_application_audit_count = {}\n",
        report.relocation_application_audit_count
    ));
    out.push_str(&format!(
        "relocation_application_audit_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_application_audit_table_hash)
    ));
    out.push_str(&format!(
        "relocation_application_audit_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.relocation_application_audit_blockers)
    ));
    out.push_str(&format!(
        "relocation_patch_preview_status = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_preview_status)
    ));
    out.push_str(&format!(
        "relocation_patch_preview_count = {}\n",
        report.relocation_patch_preview_count
    ));
    out.push_str(&format!(
        "relocation_patch_preview_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_preview_table_hash)
    ));
    out.push_str(&format!(
        "relocation_patch_application_status = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_application_status)
    ));
    out.push_str(&format!(
        "relocation_patch_application_count = {}\n",
        report.relocation_patch_application_count
    ));
    out.push_str(&format!(
        "relocation_patch_application_table_hash = \"{}\"\n",
        toml::escape_toml_string(&report.relocation_patch_application_table_hash)
    ));
    out.push_str(&format!(
        "relocation_patch_application_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.relocation_patch_application_blockers)
    ));
    out.push_str(&format!(
        "image_constructed = {}\n",
        report.image_constructed
    ));
    out.push_str(&format!("image_ready = {}\n", report.image_ready));
    out.push_str(&format!(
        "image_size_bytes = {}\n",
        optional_usize_toml(report.image_size_bytes)
    ));
    out.push_str(&format!(
        "image_hash = \"{}\"\n",
        toml::escape_toml_string(report.image_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml::toml_string_array_literal(&report.blockers)
    ));
    for record in &report.relocation_patch_previews {
        out.push_str("\n[[relocation_patch_preview]]\n");
        out.push_str(&format!("order_index = {}\n", record.order_index));
        out.push_str(&format!(
            "relocation_id = \"{}\"\n",
            toml::escape_toml_string(&record.relocation_id)
        ));
        out.push_str(&format!(
            "patch_kind = \"{}\"\n",
            toml::escape_toml_string(&record.patch_kind)
        ));
        out.push_str(&format!("patch_offset = {}\n", record.patch_offset));
        out.push_str(&format!(
            "patch_width_bytes = {}\n",
            record.patch_width_bytes
        ));
        out.push_str(&format!(
            "resolved_patch_value = {}\n",
            optional_usize_toml(record.resolved_patch_value)
        ));
        out.push_str(&format!(
            "patch_value_hash = \"{}\"\n",
            toml::escape_toml_string(&record.patch_value_hash)
        ));
        out.push_str(&format!(
            "target_symbol_id = \"{}\"\n",
            toml::escape_toml_string(&record.target_symbol_id)
        ));
        out.push_str(&format!(
            "target_symbol_image_offset = {}\n",
            optional_usize_toml(record.target_symbol_image_offset)
        ));
        out.push_str(&format!(
            "preview_status = \"{}\"\n",
            toml::escape_toml_string(&record.preview_status)
        ));
        out.push_str(&format!(
            "resolver_status = \"{}\"\n",
            toml::escape_toml_string(&record.resolver_status)
        ));
    }
    out
}

pub(crate) fn optional_bool_toml(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "false".to_owned())
}

pub(crate) fn optional_usize_toml(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_owned())
}

use crate::native_entry::NativeEntryHandoffEvidence;

pub(super) fn print_native_entry_evidence(evidence: &NativeEntryHandoffEvidence) {
    println!(
        "  native_entry_handoff: protocol={} status={} ready={} section={} section_hash={} code_hash={}",
        evidence.protocol,
        evidence.status,
        evidence.ready,
        evidence.section_id.as_deref().unwrap_or("<none>"),
        evidence.section_hash_status,
        evidence.code_hash_status
    );
    println!(
        "  native_entry_payload: offset={} size={} hash={}",
        optional_usize(evidence.container_payload_offset),
        optional_usize(evidence.container_payload_size_bytes),
        evidence
            .container_payload_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  native_entry_code: offset={} size={}",
        optional_usize(evidence.code_offset),
        optional_usize(evidence.code_size_bytes)
    );
    println!(
        "  native_entry_preparation: protocol={} status={} ready={} target_arch={} host_arch={} arch_status={} mapping_size={} protection={}",
        evidence.preparation_protocol.as_deref().unwrap_or("<none>"),
        evidence.preparation_status,
        evidence.preparation_ready,
        evidence.target_machine_arch.as_deref().unwrap_or("<none>"),
        evidence.host_machine_arch.as_deref().unwrap_or("<none>"),
        evidence.machine_arch_status,
        evidence.mapping_size_bytes,
        evidence.protection_status
    );
    println!(
        "  native_entry_context: protocol={} status={} version={} size={} identity={} plan={} execution={} clock={} glm={} scheduler={} lifecycle={}",
        evidence.context_protocol.as_deref().unwrap_or("<none>"),
        evidence.context_status,
        optional_u32(evidence.context_version),
        optional_u32(evidence.context_size_bytes),
        evidence.context_identity_hash.as_deref().unwrap_or("<none>"),
        optional_u64_hex(evidence.context_plan_identity),
        optional_u64_hex(evidence.context_execution_identity),
        optional_u64_hex(evidence.context_clock_root_handle),
        optional_u64_hex(evidence.context_glm_root_handle),
        optional_u64_hex(evidence.context_scheduler_handle),
        optional_u64_hex(evidence.context_lifecycle_hook_handle)
    );
    println!(
        "  native_entry_dispatch: protocol={} status={} declared={} table={} capabilities={} slot={} code={} acknowledged={}",
        evidence
            .dispatch_resolution_protocol
            .as_deref()
            .unwrap_or("<none>"),
        evidence.dispatch_resolution_status,
        evidence.dispatch_import_declared,
        optional_u64_hex(evidence.dispatch_table_identity),
        optional_u64_hex(evidence.dispatch_capability_mask),
        optional_u32(evidence.dispatch_slot),
        optional_i32(evidence.dispatch_status_code),
        evidence.dispatch_acknowledged
    );
    println!(
        "  native_entry_invocation: requested={} permit_protocol={} protocol={} status={} invoked={} return={} return_status={}",
        evidence.invocation_requested,
        evidence
            .invocation_permit_protocol
            .as_deref()
            .unwrap_or("<none>"),
        evidence.invocation_protocol.as_deref().unwrap_or("<none>"),
        evidence.invocation_status,
        evidence.invoked,
        optional_i64(evidence.invocation_return_value),
        evidence.invocation_return_status
    );
    println!(
        "  native_entry_blockers: {}",
        if evidence.blockers.is_empty() {
            "<none>".to_owned()
        } else {
            evidence.blockers.join(", ")
        }
    );
}

pub(super) fn native_entry_evidence_json(evidence: &NativeEntryHandoffEvidence) -> String {
    format!(
        "{{\"protocol\":\"{}\",\"status\":\"{}\",\"ready\":{},\"container_payload_offset\":{},\"container_payload_size_bytes\":{},\"container_payload_hash\":{},\"section_id\":{},\"section_hash_status\":\"{}\",\"code_offset\":{},\"code_size_bytes\":{},\"code_hash_status\":\"{}\",\"target_machine_arch\":{},\"host_machine_arch\":{},\"machine_arch_status\":\"{}\",\"preparation_protocol\":{},\"preparation_status\":\"{}\",\"preparation_ready\":{},\"mapping_size_bytes\":{},\"protection_status\":\"{}\",\"context_protocol\":{},\"context_status\":\"{}\",\"context_version\":{},\"context_size_bytes\":{},\"context_identity_hash\":{},\"context_plan_identity\":{},\"context_execution_identity\":{},\"context_clock_root_handle\":{},\"context_glm_root_handle\":{},\"context_scheduler_handle\":{},\"context_lifecycle_hook_handle\":{},\"dispatch_resolution_protocol\":{},\"dispatch_resolution_status\":\"{}\",\"dispatch_import_declared\":{},\"dispatch_table_identity\":{},\"dispatch_capability_mask\":{},\"dispatch_slot\":{},\"dispatch_status_code\":{},\"dispatch_acknowledged\":{},\"invocation_requested\":{},\"invocation_permit_protocol\":{},\"invocation_protocol\":{},\"invocation_status\":\"{}\",\"invoked\":{},\"invocation_return_value\":{},\"invocation_return_status\":\"{}\",\"blockers\":[{}]}}",
        json_escape(evidence.protocol),
        json_escape(&evidence.status),
        evidence.ready,
        json_optional_usize(evidence.container_payload_offset),
        json_optional_usize(evidence.container_payload_size_bytes),
        json_optional_string(evidence.container_payload_hash.as_deref()),
        json_optional_string(evidence.section_id.as_deref()),
        json_escape(&evidence.section_hash_status),
        json_optional_usize(evidence.code_offset),
        json_optional_usize(evidence.code_size_bytes),
        json_escape(&evidence.code_hash_status),
        json_optional_string(evidence.target_machine_arch.as_deref()),
        json_optional_string(evidence.host_machine_arch.as_deref()),
        json_escape(&evidence.machine_arch_status),
        json_optional_string(evidence.preparation_protocol.as_deref()),
        json_escape(&evidence.preparation_status),
        evidence.preparation_ready,
        evidence.mapping_size_bytes,
        json_escape(&evidence.protection_status),
        json_optional_string(evidence.context_protocol.as_deref()),
        json_escape(&evidence.context_status),
        json_optional_u32(evidence.context_version),
        json_optional_u32(evidence.context_size_bytes),
        json_optional_string(evidence.context_identity_hash.as_deref()),
        json_optional_u64(evidence.context_plan_identity),
        json_optional_u64(evidence.context_execution_identity),
        json_optional_u64(evidence.context_clock_root_handle),
        json_optional_u64(evidence.context_glm_root_handle),
        json_optional_u64(evidence.context_scheduler_handle),
        json_optional_u64(evidence.context_lifecycle_hook_handle),
        json_optional_string(evidence.dispatch_resolution_protocol.as_deref()),
        json_escape(&evidence.dispatch_resolution_status),
        evidence.dispatch_import_declared,
        json_optional_u64(evidence.dispatch_table_identity),
        json_optional_u64(evidence.dispatch_capability_mask),
        json_optional_u32(evidence.dispatch_slot),
        json_optional_i32(evidence.dispatch_status_code),
        evidence.dispatch_acknowledged,
        evidence.invocation_requested,
        json_optional_string(evidence.invocation_permit_protocol.as_deref()),
        json_optional_string(evidence.invocation_protocol.as_deref()),
        json_escape(&evidence.invocation_status),
        evidence.invoked,
        json_optional_i64(evidence.invocation_return_value),
        json_escape(&evidence.invocation_return_status),
        evidence
            .blockers
            .iter()
            .map(|blocker| format!("\"{}\"", json_escape(blocker)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_owned())
}

fn optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_owned())
}

fn optional_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_owned())
}

fn optional_u32(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_owned())
}

fn optional_u64_hex(value: Option<u64>) -> String {
    value
        .map(|value| format!("0x{value:016x}"))
        .unwrap_or_else(|| "<none>".to_owned())
}

fn json_optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_u32(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

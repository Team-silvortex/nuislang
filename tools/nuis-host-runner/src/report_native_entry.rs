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
        "  native_entry_preparation: protocol={} status={} ready={} target_arch={} host_arch={} arch_status={} mapping_size={} protection={} invocation={}",
        evidence.preparation_protocol.as_deref().unwrap_or("<none>"),
        evidence.preparation_status,
        evidence.preparation_ready,
        evidence.target_machine_arch.as_deref().unwrap_or("<none>"),
        evidence.host_machine_arch.as_deref().unwrap_or("<none>"),
        evidence.machine_arch_status,
        evidence.mapping_size_bytes,
        evidence.protection_status,
        evidence.invocation_status
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
        "{{\"protocol\":\"{}\",\"status\":\"{}\",\"ready\":{},\"container_payload_offset\":{},\"container_payload_size_bytes\":{},\"container_payload_hash\":{},\"section_id\":{},\"section_hash_status\":\"{}\",\"code_offset\":{},\"code_size_bytes\":{},\"code_hash_status\":\"{}\",\"target_machine_arch\":{},\"host_machine_arch\":{},\"machine_arch_status\":\"{}\",\"preparation_protocol\":{},\"preparation_status\":\"{}\",\"preparation_ready\":{},\"mapping_size_bytes\":{},\"protection_status\":\"{}\",\"invocation_status\":\"{}\",\"blockers\":[{}]}}",
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
        json_escape(evidence.invocation_status),
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

fn json_optional_usize(value: Option<usize>) -> String {
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

use crate::{json_bool_field, json_field, json_usize_field};

pub(crate) const CONTRACT: &str = "nuis-runtime-dispatch-receipt-v1";
const RESOLUTION_PROTOCOL: &str = "nuis-runtime-dispatch-import-resolution-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeDispatchReceipt {
    pub(crate) resolution_protocol: String,
    pub(crate) resolution_status: String,
    pub(crate) execution_identity_hash: String,
    pub(crate) import_identity_hash: String,
    pub(crate) table_identity: String,
    pub(crate) capability_mask: String,
    pub(crate) slot: u32,
    pub(crate) status_code: i32,
    pub(crate) acknowledged: bool,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistedRuntimeDispatchReceipt {
    pub(crate) contract: Option<String>,
    pub(crate) verification_status: String,
    pub(crate) resolution_protocol: Option<String>,
    pub(crate) resolution_status: Option<String>,
    pub(crate) execution_identity_hash: Option<String>,
    pub(crate) import_identity_hash: Option<String>,
    pub(crate) table_identity: Option<String>,
    pub(crate) capability_mask: Option<String>,
    pub(crate) slot: Option<u32>,
    pub(crate) status_code: Option<i32>,
    pub(crate) acknowledged: Option<bool>,
    pub(crate) receipt_hash: Option<String>,
}

pub(crate) fn from_host_runner_json(source: &str) -> Option<RuntimeDispatchReceipt> {
    if json_bool_value(source, "dispatch_import_declared") != Some(true) {
        return None;
    }
    let mut receipt = RuntimeDispatchReceipt {
        resolution_protocol: json_string_value(source, "dispatch_resolution_protocol")
            .unwrap_or_else(|| "none".to_owned()),
        resolution_status: json_string_value(source, "dispatch_resolution_status")
            .unwrap_or_else(|| "none".to_owned()),
        execution_identity_hash: json_string_value(
            source,
            "runtime_bootstrap_execution_identity_hash",
        )
        .unwrap_or_else(|| "none".to_owned()),
        import_identity_hash: json_string_value(source, "dispatch_import_identity_hash")
            .unwrap_or_else(|| "none".to_owned()),
        table_identity: json_u64_value(source, "dispatch_table_identity")
            .map(canonical_u64)
            .unwrap_or_else(|| "none".to_owned()),
        capability_mask: json_u64_value(source, "dispatch_capability_mask")
            .map(canonical_u64)
            .unwrap_or_else(|| "none".to_owned()),
        slot: json_u64_value(source, "dispatch_slot")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0),
        status_code: json_i64_value(source, "dispatch_status_code")
            .and_then(|value| i32::try_from(value).ok())
            .unwrap_or(i32::MIN),
        acknowledged: json_bool_value(source, "dispatch_acknowledged").unwrap_or(false),
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    Some(receipt)
}

pub(crate) fn independently_verify(source: &str) -> PersistedRuntimeDispatchReceipt {
    let any_claim = source
        .lines()
        .any(|line| line.trim_start().starts_with("runtime_dispatch_receipt_"));
    if !any_claim {
        return absent_receipt();
    }
    let contract = toml_string(source, "runtime_dispatch_receipt_contract");
    let resolution_protocol = toml_string(source, "runtime_dispatch_receipt_resolution_protocol");
    let resolution_status = toml_string(source, "runtime_dispatch_receipt_resolution_status");
    let execution_identity_hash =
        toml_string(source, "runtime_dispatch_receipt_execution_identity_hash");
    let import_identity_hash = toml_string(source, "runtime_dispatch_receipt_import_identity_hash");
    let table_identity = toml_string(source, "runtime_dispatch_receipt_table_identity");
    let capability_mask = toml_string(source, "runtime_dispatch_receipt_capability_mask");
    let slot = toml_u64(source, "runtime_dispatch_receipt_slot")
        .and_then(|value| u32::try_from(value).ok());
    let status_code = toml_i64(source, "runtime_dispatch_receipt_status_code")
        .and_then(|value| i32::try_from(value).ok());
    let acknowledged = toml_bool(source, "runtime_dispatch_receipt_acknowledged");
    let claimed_hash = toml_string(source, "runtime_dispatch_receipt_hash");
    let mut persisted = PersistedRuntimeDispatchReceipt {
        contract,
        verification_status: String::new(),
        resolution_protocol,
        resolution_status,
        execution_identity_hash,
        import_identity_hash,
        table_identity,
        capability_mask,
        slot,
        status_code,
        acknowledged,
        receipt_hash: claimed_hash,
    };
    let semantic_status = verify_semantics(&persisted);
    persisted.verification_status = if semantic_status != "verified" {
        semantic_status
    } else {
        let claim = RuntimeDispatchReceipt {
            resolution_protocol: persisted.resolution_protocol.clone().unwrap_or_default(),
            resolution_status: persisted.resolution_status.clone().unwrap_or_default(),
            execution_identity_hash: persisted
                .execution_identity_hash
                .clone()
                .unwrap_or_default(),
            import_identity_hash: persisted.import_identity_hash.clone().unwrap_or_default(),
            table_identity: persisted.table_identity.clone().unwrap_or_default(),
            capability_mask: persisted.capability_mask.clone().unwrap_or_default(),
            slot: persisted.slot.unwrap_or_default(),
            status_code: persisted.status_code.unwrap_or_default(),
            acknowledged: persisted.acknowledged.unwrap_or(false),
            receipt_hash: String::new(),
        };
        match persisted.receipt_hash.as_deref() {
            None | Some("") => "hash-missing".to_owned(),
            Some(value) if value == receipt_hash(&claim) => "verified".to_owned(),
            Some(_) => "hash-mismatch".to_owned(),
        }
    };
    persisted
}

pub(crate) fn render_claim_fields(receipt: &RuntimeDispatchReceipt) -> String {
    let mut out = String::new();
    push_string(&mut out, "runtime_dispatch_receipt_contract", CONTRACT);
    push_string(
        &mut out,
        "runtime_dispatch_receipt_resolution_protocol",
        &receipt.resolution_protocol,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_resolution_status",
        &receipt.resolution_status,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_execution_identity_hash",
        &receipt.execution_identity_hash,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_import_identity_hash",
        &receipt.import_identity_hash,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_table_identity",
        &receipt.table_identity,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_capability_mask",
        &receipt.capability_mask,
    );
    out.push_str(&format!(
        "runtime_dispatch_receipt_slot = {}\n",
        receipt.slot
    ));
    out.push_str(&format!(
        "runtime_dispatch_receipt_status_code = {}\n",
        receipt.status_code
    ));
    out.push_str(&format!(
        "runtime_dispatch_receipt_acknowledged = {}\n",
        receipt.acknowledged
    ));
    push_string(
        &mut out,
        "runtime_dispatch_receipt_hash",
        &receipt.receipt_hash,
    );
    out
}

pub(crate) fn upsert_claim(source: &str, receipt: &RuntimeDispatchReceipt) -> String {
    let boundary = source.find("[[records]]").unwrap_or(source.len());
    let (head, tail) = source.split_at(boundary);
    let mut out = head
        .lines()
        .filter(|line| !line.trim_start().starts_with("runtime_dispatch_receipt_"))
        .collect::<Vec<_>>()
        .join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&render_claim_fields(receipt));
    if !tail.is_empty() {
        out.push('\n');
        out.push_str(tail);
    }
    out
}

impl PersistedRuntimeDispatchReceipt {
    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![
            json_field(
                &format!("{prefix}_runtime_dispatch_receipt_status"),
                &self.verification_status,
            ),
            json_field(
                &format!("{prefix}_runtime_dispatch_receipt_contract"),
                self.contract.as_deref().unwrap_or("none"),
            ),
            json_field(
                &format!("{prefix}_runtime_dispatch_receipt_execution_identity_hash"),
                self.execution_identity_hash.as_deref().unwrap_or("none"),
            ),
            json_field(
                &format!("{prefix}_runtime_dispatch_receipt_import_identity_hash"),
                self.import_identity_hash.as_deref().unwrap_or("none"),
            ),
            json_field(
                &format!("{prefix}_runtime_dispatch_receipt_hash"),
                self.receipt_hash.as_deref().unwrap_or("none"),
            ),
            json_usize_field(
                &format!("{prefix}_runtime_dispatch_receipt_slot"),
                self.slot.unwrap_or(0) as usize,
            ),
            json_bool_field(
                &format!("{prefix}_runtime_dispatch_receipt_acknowledged"),
                self.acknowledged.unwrap_or(false),
            ),
        ]
    }
}

fn verify_semantics(receipt: &PersistedRuntimeDispatchReceipt) -> String {
    if receipt.contract.as_deref() != Some(CONTRACT) {
        return "invalid-contract".to_owned();
    }
    if receipt.resolution_protocol.as_deref() != Some(RESOLUTION_PROTOCOL) {
        return "invalid-resolution-protocol".to_owned();
    }
    if receipt.resolution_status.as_deref() != Some("resolved") {
        return "unresolved".to_owned();
    }
    if !receipt
        .execution_identity_hash
        .as_deref()
        .is_some_and(valid_u64_hex)
    {
        return "invalid-execution-identity".to_owned();
    }
    if !receipt
        .import_identity_hash
        .as_deref()
        .is_some_and(valid_u64_hex)
    {
        return "invalid-import-identity".to_owned();
    }
    let Some(table_identity) = receipt
        .table_identity
        .as_deref()
        .and_then(parse_u64_hex)
        .filter(|value| *value != 0)
    else {
        return "invalid-table-identity".to_owned();
    };
    let _ = table_identity;
    let Some(capability_mask) = receipt.capability_mask.as_deref().and_then(parse_u64_hex) else {
        return "invalid-capability-mask".to_owned();
    };
    let Some(slot) = receipt.slot.filter(|value| (1..=64).contains(value)) else {
        return "invalid-slot".to_owned();
    };
    if capability_mask & (1_u64 << (slot - 1)) == 0 {
        return "invalid-capability-mask".to_owned();
    }
    if receipt.status_code != Some(0) {
        return "invalid-status-code".to_owned();
    }
    if receipt.acknowledged != Some(true) {
        return "not-acknowledged".to_owned();
    }
    "verified".to_owned()
}

fn receipt_hash(receipt: &RuntimeDispatchReceipt) -> String {
    let canonical = format!(
        "{CONTRACT}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        receipt.resolution_protocol,
        receipt.resolution_status,
        receipt.execution_identity_hash,
        receipt.import_identity_hash,
        receipt.table_identity,
        receipt.capability_mask,
        receipt.slot,
        receipt.status_code,
        receipt.acknowledged,
    );
    format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes()))
}

fn absent_receipt() -> PersistedRuntimeDispatchReceipt {
    PersistedRuntimeDispatchReceipt {
        contract: None,
        verification_status: "legacy-absent".to_owned(),
        resolution_protocol: None,
        resolution_status: None,
        execution_identity_hash: None,
        import_identity_hash: None,
        table_identity: None,
        capability_mask: None,
        slot: None,
        status_code: None,
        acknowledged: None,
        receipt_hash: None,
    }
}

fn canonical_u64(value: u64) -> String {
    format!("0x{value:016x}")
}

fn valid_u64_hex(value: &str) -> bool {
    parse_u64_hex(value).is_some()
}

fn parse_u64_hex(value: &str) -> Option<u64> {
    let hex = value.strip_prefix("0x")?;
    (hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| u64::from_str_radix(hex, 16).ok())?
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn json_string_value(source: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let tail = &source[source.find(&needle)? + needle.len()..];
    Some(tail[..tail.find('"')?].to_owned())
}

fn json_bool_value(source: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\":");
    let tail = source[source.find(&needle)? + needle.len()..].trim_start();
    tail.starts_with("true")
        .then_some(true)
        .or_else(|| tail.starts_with("false").then_some(false))
}

fn json_u64_value(source: &str, key: &str) -> Option<u64> {
    json_integer_text(source, key)?.parse().ok()
}

fn json_i64_value(source: &str, key: &str) -> Option<i64> {
    json_integer_text(source, key)?.parse().ok()
}

fn json_integer_text<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\":");
    let tail = source[source.find(&needle)? + needle.len()..].trim_start();
    let end = tail
        .find(|ch: char| ch != '-' && !ch.is_ascii_digit())
        .unwrap_or(tail.len());
    (end > 0).then_some(&tail[..end])
}

fn toml_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    let line = source.lines().find(|line| line.starts_with(&prefix))?;
    line[prefix.len()..].strip_suffix('"').map(str::to_owned)
}

fn toml_u64(source: &str, key: &str) -> Option<u64> {
    toml_scalar(source, key)?.parse().ok()
}

fn toml_i64(source: &str, key: &str) -> Option<i64> {
    toml_scalar(source, key)?.parse().ok()
}

fn toml_bool(source: &str, key: &str) -> Option<bool> {
    match toml_scalar(source, key)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn toml_scalar<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).map(str::trim))
}

fn push_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host_json() -> &'static str {
        r#"{"runtime_bootstrap_execution_identity_hash":"0x1111111111111111","native_entry_handoff":{"dispatch_resolution_protocol":"nuis-runtime-dispatch-import-resolution-v1","dispatch_resolution_status":"resolved","dispatch_import_declared":true,"dispatch_import_identity_hash":"0x2222222222222222","dispatch_table_identity":3689348814741910323,"dispatch_capability_mask":3,"dispatch_slot":2,"dispatch_status_code":0,"dispatch_acknowledged":true}}"#
    }

    #[test]
    fn host_receipt_round_trips_through_independent_verifier() {
        let receipt = from_host_runner_json(host_json()).expect("dispatch receipt");
        let source = render_claim_fields(&receipt);
        let verified = independently_verify(&source);

        assert_eq!(verified.verification_status, "verified");
        assert_eq!(
            verified.table_identity.as_deref(),
            Some("0x3333333333333333")
        );
        assert_eq!(verified.slot, Some(2));
    }

    #[test]
    fn independent_verifier_rejects_identity_drift() {
        let receipt = from_host_runner_json(host_json()).expect("dispatch receipt");
        let source =
            render_claim_fields(&receipt).replace("0x2222222222222222", "0x4444444444444444");

        assert_eq!(
            independently_verify(&source).verification_status,
            "hash-mismatch"
        );
        assert_eq!(
            independently_verify("").verification_status,
            "legacy-absent"
        );
    }
}

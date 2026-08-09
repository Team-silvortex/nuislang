pub(crate) const CONTRACT: &str = "nuis-runtime-dispatch-receipt-v1";
const RESOLUTION_PROTOCOL: &str = "nuis-runtime-dispatch-import-resolution-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeDispatchReceiptInfo {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) resolution_protocol: String,
    pub(crate) resolution_status: String,
    pub(crate) execution_identity_hash: String,
    pub(crate) import_identity_hash: String,
    pub(crate) table_identity: String,
    pub(crate) capability_mask: String,
    pub(crate) slot: Option<u32>,
    pub(crate) status_code: Option<i32>,
    pub(crate) acknowledged: Option<bool>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct RuntimeDispatchReceiptSummary {
    pub contract: Option<String>,
    pub status: String,
    pub receipt_hash: Option<String>,
    pub execution_identity_hash: Option<String>,
    pub import_identity_hash: Option<String>,
    pub table_identity: Option<String>,
    pub capability_mask: Option<String>,
    pub slot: Option<u32>,
    pub status_code: Option<i32>,
    pub acknowledged: Option<bool>,
}

#[allow(dead_code)]
pub(crate) fn public_summary(info: &RuntimeDispatchReceiptInfo) -> RuntimeDispatchReceiptSummary {
    RuntimeDispatchReceiptSummary {
        contract: optional(&info.contract),
        status: info.status.clone(),
        receipt_hash: optional(&info.receipt_hash),
        execution_identity_hash: optional(&info.execution_identity_hash),
        import_identity_hash: optional(&info.import_identity_hash),
        table_identity: optional(&info.table_identity),
        capability_mask: optional(&info.capability_mask),
        slot: info.slot,
        status_code: info.status_code,
        acknowledged: info.acknowledged,
    }
}

pub(crate) fn parse_and_verify(source: &str) -> RuntimeDispatchReceiptInfo {
    if !source
        .lines()
        .any(|line| line.trim_start().starts_with("runtime_dispatch_receipt_"))
    {
        return absent();
    }
    let mut info = RuntimeDispatchReceiptInfo {
        contract: string_field(source, "runtime_dispatch_receipt_contract")
            .unwrap_or_else(|| "none".to_owned()),
        status: String::new(),
        resolution_protocol: string_field(source, "runtime_dispatch_receipt_resolution_protocol")
            .unwrap_or_else(|| "none".to_owned()),
        resolution_status: string_field(source, "runtime_dispatch_receipt_resolution_status")
            .unwrap_or_else(|| "none".to_owned()),
        execution_identity_hash: string_field(
            source,
            "runtime_dispatch_receipt_execution_identity_hash",
        )
        .unwrap_or_else(|| "none".to_owned()),
        import_identity_hash: string_field(source, "runtime_dispatch_receipt_import_identity_hash")
            .unwrap_or_else(|| "none".to_owned()),
        table_identity: string_field(source, "runtime_dispatch_receipt_table_identity")
            .unwrap_or_else(|| "none".to_owned()),
        capability_mask: string_field(source, "runtime_dispatch_receipt_capability_mask")
            .unwrap_or_else(|| "none".to_owned()),
        slot: unsigned_field(source, "runtime_dispatch_receipt_slot")
            .and_then(|value| u32::try_from(value).ok()),
        status_code: signed_field(source, "runtime_dispatch_receipt_status_code")
            .and_then(|value| i32::try_from(value).ok()),
        acknowledged: bool_field(source, "runtime_dispatch_receipt_acknowledged"),
        receipt_hash: string_field(source, "runtime_dispatch_receipt_hash")
            .unwrap_or_else(|| "none".to_owned()),
    };
    info.status = semantic_status(&info).to_owned();
    if info.status == "verified" {
        info.status = if info.receipt_hash == digest(&info) {
            "verified"
        } else if info.receipt_hash == "none" || info.receipt_hash.is_empty() {
            "hash-missing"
        } else {
            "hash-mismatch"
        }
        .to_owned();
    }
    info
}

pub(crate) fn render_fields(info: &RuntimeDispatchReceiptInfo) -> String {
    if info.status == "legacy-absent" {
        return String::new();
    }
    let mut out = String::new();
    push_string(
        &mut out,
        "runtime_dispatch_receipt_contract",
        &info.contract,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_resolution_protocol",
        &info.resolution_protocol,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_resolution_status",
        &info.resolution_status,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_execution_identity_hash",
        &info.execution_identity_hash,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_import_identity_hash",
        &info.import_identity_hash,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_table_identity",
        &info.table_identity,
    );
    push_string(
        &mut out,
        "runtime_dispatch_receipt_capability_mask",
        &info.capability_mask,
    );
    out.push_str(&format!(
        "runtime_dispatch_receipt_slot = {}\n",
        info.slot.unwrap_or(0)
    ));
    out.push_str(&format!(
        "runtime_dispatch_receipt_status_code = {}\n",
        info.status_code.unwrap_or(i32::MIN)
    ));
    out.push_str(&format!(
        "runtime_dispatch_receipt_acknowledged = {}\n",
        info.acknowledged.unwrap_or(false)
    ));
    push_string(
        &mut out,
        "runtime_dispatch_receipt_hash",
        &info.receipt_hash,
    );
    out
}

fn semantic_status(info: &RuntimeDispatchReceiptInfo) -> &'static str {
    if info.contract != CONTRACT {
        return "invalid-contract";
    }
    if info.resolution_protocol != RESOLUTION_PROTOCOL {
        return "invalid-resolution-protocol";
    }
    if info.resolution_status != "resolved" {
        return "unresolved";
    }
    if !valid_hex_identity(&info.execution_identity_hash) {
        return "invalid-execution-identity";
    }
    if !valid_hex_identity(&info.import_identity_hash) {
        return "invalid-import-identity";
    }
    if parse_hex_identity(&info.table_identity).is_none_or(|value| value == 0) {
        return "invalid-table-identity";
    }
    let Some(mask) = parse_hex_identity(&info.capability_mask) else {
        return "invalid-capability-mask";
    };
    let Some(slot) = info.slot.filter(|slot| (1..=64).contains(slot)) else {
        return "invalid-slot";
    };
    if mask & (1_u64 << (slot - 1)) == 0 {
        return "invalid-capability-mask";
    }
    if info.status_code != Some(0) {
        return "invalid-status-code";
    }
    if info.acknowledged != Some(true) {
        return "not-acknowledged";
    }
    "verified"
}

fn digest(info: &RuntimeDispatchReceiptInfo) -> String {
    let canonical = format!(
        "{CONTRACT}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        info.resolution_protocol,
        info.resolution_status,
        info.execution_identity_hash,
        info.import_identity_hash,
        info.table_identity,
        info.capability_mask,
        info.slot.unwrap_or(0),
        info.status_code.unwrap_or(i32::MIN),
        info.acknowledged.unwrap_or(false),
    );
    let hash = canonical
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    format!("fnv1a64:{hash:016x}")
}

fn absent() -> RuntimeDispatchReceiptInfo {
    RuntimeDispatchReceiptInfo {
        contract: "none".to_owned(),
        status: "legacy-absent".to_owned(),
        resolution_protocol: "none".to_owned(),
        resolution_status: "none".to_owned(),
        execution_identity_hash: "none".to_owned(),
        import_identity_hash: "none".to_owned(),
        table_identity: "none".to_owned(),
        capability_mask: "none".to_owned(),
        slot: None,
        status_code: None,
        acknowledged: None,
        receipt_hash: "none".to_owned(),
    }
}

fn valid_hex_identity(value: &str) -> bool {
    parse_hex_identity(value).is_some()
}

#[allow(dead_code)]
fn optional(value: &str) -> Option<String> {
    (value != "none" && !value.is_empty()).then(|| value.to_owned())
}

fn parse_hex_identity(value: &str) -> Option<u64> {
    let hex = value.strip_prefix("0x")?;
    if hex.len() != 16 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u64::from_str_radix(hex, 16).ok()
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    let line = source.lines().find(|line| line.starts_with(&prefix))?;
    line[prefix.len()..].strip_suffix('"').map(str::to_owned)
}

fn unsigned_field(source: &str, key: &str) -> Option<u64> {
    scalar_field(source, key)?.parse().ok()
}

fn signed_field(source: &str, key: &str) -> Option<i64> {
    scalar_field(source, key)?.parse().ok()
}

fn bool_field(source: &str, key: &str) -> Option<bool> {
    match scalar_field(source, key)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn scalar_field<'a>(source: &'a str, key: &str) -> Option<&'a str> {
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

    fn claim() -> RuntimeDispatchReceiptInfo {
        let mut info = RuntimeDispatchReceiptInfo {
            contract: CONTRACT.to_owned(),
            status: "verified".to_owned(),
            resolution_protocol: RESOLUTION_PROTOCOL.to_owned(),
            resolution_status: "resolved".to_owned(),
            execution_identity_hash: "0x1111111111111111".to_owned(),
            import_identity_hash: "0x2222222222222222".to_owned(),
            table_identity: "0x3333333333333333".to_owned(),
            capability_mask: "0x0000000000000003".to_owned(),
            slot: Some(2),
            status_code: Some(0),
            acknowledged: Some(true),
            receipt_hash: String::new(),
        };
        info.receipt_hash = digest(&info);
        info
    }

    #[test]
    fn independently_verifies_persisted_receipt() {
        let source = render_fields(&claim());
        assert_eq!(parse_and_verify(&source).status, "verified");
    }

    #[test]
    fn rejects_receipt_hash_drift_and_accepts_legacy_absence() {
        let source = render_fields(&claim()).replace("0x3333333333333333", "0x4444444444444444");
        assert_eq!(parse_and_verify(&source).status, "hash-mismatch");
        assert_eq!(parse_and_verify("").status, "legacy-absent");
    }
}

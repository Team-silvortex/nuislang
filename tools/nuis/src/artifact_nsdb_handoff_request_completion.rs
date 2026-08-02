use std::collections::BTreeSet;

const COLLECTION_CONTRACT: &str = "nuis-provider-request-completion-receipt-collection-v1";
const RECEIPT_CONTRACT: &str = "nuis-provider-request-completion-receipt-v1";

#[derive(Clone)]
pub(crate) struct PersistedRequestCompletionAudit {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) root_hash: String,
    pub(crate) receipts: Vec<PersistedRequestCompletionReceipt>,
    pub(crate) validation_status: String,
}

#[derive(Clone)]
pub(crate) struct PersistedRequestCompletionReceipt {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) request_id: String,
    pub(crate) provider_family: String,
    pub(crate) dispatch_id: String,
    pub(crate) completion_clock: String,
    pub(crate) output_hash: String,
    pub(crate) completion_token: String,
    pub(crate) selected_set_hash: String,
}

pub(crate) fn parse_and_append(
    material: &mut String,
    record: &str,
) -> PersistedRequestCompletionAudit {
    if optional_field(record, "request_completion_contract").is_none() {
        return audit(
            "none",
            "not-applicable",
            0,
            "none",
            Vec::new(),
            "legacy-unavailable",
        );
    }
    let contract = field(record, "request_completion_contract");
    let status = field(record, "request_completion_status");
    let count = field(record, "request_completion_count")
        .parse::<usize>()
        .unwrap_or(0);
    let root_hash = field(record, "request_completion_root_hash");
    material.push_str(&format!("\0{contract}\0{status}\0{count}\0{root_hash}"));
    let receipts = (0..count)
        .map(|index| parse_receipt_and_append(material, record, index))
        .collect::<Vec<_>>();
    let validation_status = if status == "verified"
        && contract == COLLECTION_CONTRACT
        && count > 0
        && root_hash == receipt_root_hash(&receipts)
        && receipts_verified(&receipts)
    {
        "verified"
    } else if status == "pre-seal-acquisition" && contract == COLLECTION_CONTRACT {
        "pre-seal-acquisition"
    } else if status == "not-applicable" && count == 0 && root_hash == "none" {
        "not-applicable"
    } else {
        "mismatch"
    };
    audit(
        &contract,
        &status,
        count,
        &root_hash,
        receipts,
        validation_status,
    )
}

fn parse_receipt_and_append(
    material: &mut String,
    record: &str,
    index: usize,
) -> PersistedRequestCompletionReceipt {
    let prefix = format!("request_completion_{index}_");
    let value = |suffix| field(record, &format!("{prefix}{suffix}"));
    let receipt = PersistedRequestCompletionReceipt {
        contract: value("contract"),
        status: value("status"),
        request_id: value("request_id"),
        provider_family: value("provider_family"),
        dispatch_id: value("dispatch_id"),
        completion_clock: value("completion_clock"),
        output_hash: value("output_hash"),
        completion_token: value("completion_token"),
        selected_set_hash: value("selected_set_hash"),
    };
    material.push_str(&format!(
        "\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        receipt.contract,
        receipt.status,
        receipt.request_id,
        receipt.provider_family,
        receipt.dispatch_id,
        receipt.completion_clock,
        receipt.output_hash,
        receipt.completion_token,
        receipt.selected_set_hash
    ));
    receipt
}

fn receipts_verified(receipts: &[PersistedRequestCompletionReceipt]) -> bool {
    let mut request_ids = BTreeSet::new();
    receipts.iter().all(|receipt| {
        receipt.contract == RECEIPT_CONTRACT
            && receipt.status == "verified"
            && request_ids.insert(receipt.request_id.as_str())
            && [
                receipt.provider_family.as_str(),
                receipt.dispatch_id.as_str(),
                receipt.completion_clock.as_str(),
                receipt.output_hash.as_str(),
                receipt.completion_token.as_str(),
                receipt.selected_set_hash.as_str(),
            ]
            .iter()
            .all(|value| !matches!(*value, "" | "none"))
    })
}

fn receipt_root_hash(receipts: &[PersistedRequestCompletionReceipt]) -> String {
    let mut material = format!("{COLLECTION_CONTRACT}\n");
    for (index, receipt) in receipts.iter().enumerate() {
        material.push_str(&format!(
            "{index}|{}|{}|{}|{}|{}|{}|{}\n",
            receipt.request_id,
            receipt.provider_family,
            receipt.dispatch_id,
            receipt.completion_clock,
            receipt.output_hash,
            receipt.completion_token,
            receipt.selected_set_hash
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

fn audit(
    contract: &str,
    status: &str,
    count: usize,
    root_hash: &str,
    receipts: Vec<PersistedRequestCompletionReceipt>,
    validation_status: &str,
) -> PersistedRequestCompletionAudit {
    PersistedRequestCompletionAudit {
        contract: contract.to_owned(),
        status: status.to_owned(),
        count,
        root_hash: root_hash.to_owned(),
        receipts,
        validation_status: validation_status.to_owned(),
    }
}

fn optional_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .and_then(|value| value.trim().strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape)
}

fn field(source: &str, key: &str) -> String {
    optional_field(source, key).unwrap_or_else(|| "none".to_owned())
}

fn unescape(value: &str) -> String {
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    output
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_receipt_root_and_rejects_tampering() {
        let receipt = PersistedRequestCompletionReceipt {
            contract: RECEIPT_CONTRACT.to_owned(),
            status: "verified".to_owned(),
            request_id: "request.cuda".to_owned(),
            provider_family: "cuda:nvidia-gpu".to_owned(),
            dispatch_id: "dispatch0001".to_owned(),
            completion_clock: "clock:1".to_owned(),
            output_hash: "0x1111111111111111".to_owned(),
            completion_token: "completion:1".to_owned(),
            selected_set_hash: "fnv1a64:2222222222222222".to_owned(),
        };
        let root = receipt_root_hash(std::slice::from_ref(&receipt));
        let source = format!(
            r#"request_completion_contract = "{COLLECTION_CONTRACT}"
request_completion_status = "verified"
request_completion_count = "1"
request_completion_root_hash = "{root}"
request_completion_0_contract = "{}"
request_completion_0_status = "{}"
request_completion_0_request_id = "{}"
request_completion_0_provider_family = "{}"
request_completion_0_dispatch_id = "{}"
request_completion_0_completion_clock = "{}"
request_completion_0_output_hash = "{}"
request_completion_0_completion_token = "{}"
request_completion_0_selected_set_hash = "{}"
"#,
            receipt.contract,
            receipt.status,
            receipt.request_id,
            receipt.provider_family,
            receipt.dispatch_id,
            receipt.completion_clock,
            receipt.output_hash,
            receipt.completion_token,
            receipt.selected_set_hash
        );
        let mut material = "base".to_owned();
        let audit = parse_and_append(&mut material, &source);
        assert_eq!(audit.validation_status, "verified");
        assert!(material.contains(&receipt.request_id));

        let mut tampered_material = "base".to_owned();
        let tampered = parse_and_append(
            &mut tampered_material,
            &source.replace(&root, "0x0000000000000000"),
        );
        assert_eq!(tampered.validation_status, "mismatch");
    }

    #[test]
    fn legacy_record_does_not_change_hash_material() {
        let mut material = "base".to_owned();
        let audit = parse_and_append(&mut material, "completion_evidence_status = \"verified\"");

        assert_eq!(material, "base");
        assert_eq!(audit.validation_status, "legacy-unavailable");
    }
}

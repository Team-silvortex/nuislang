use crate::final_image_provider_dispatch::FinalImageProviderDispatchAuthority;
use std::collections::BTreeSet;

pub(crate) const REQUEST_COMPLETION_COLLECTION_CONTRACT: &str =
    "nuis-provider-request-completion-receipt-collection-v1";
pub(crate) const REQUEST_COMPLETION_RECEIPT_CONTRACT: &str =
    "nuis-provider-request-completion-receipt-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequestCompletionReceipt {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequestCompletionEvidence {
    present: bool,
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) root_hash: String,
    pub(crate) receipts: Vec<ProviderRequestCompletionReceipt>,
}

impl Default for ProviderRequestCompletionEvidence {
    fn default() -> Self {
        Self {
            present: false,
            contract: REQUEST_COMPLETION_COLLECTION_CONTRACT.to_owned(),
            status: "not-applicable".to_owned(),
            count: 0,
            root_hash: "none".to_owned(),
            receipts: Vec::new(),
        }
    }
}

pub(crate) fn from_output_payload(
    source: &str,
    count: usize,
) -> Result<ProviderRequestCompletionEvidence, String> {
    let request_order = list_field(source, "provider_request_order");
    let adapter_order = list_field(source, "provider_request_adapter_order");
    if !request_order.is_empty() && request_order.len() != count {
        return Err("request completion request-order count mismatch".to_owned());
    }
    if !adapter_order.is_empty() && adapter_order.len() != count {
        return Err("request completion adapter-order count mismatch".to_owned());
    }
    let mut seen_request_ids = BTreeSet::new();
    let mut receipts = Vec::with_capacity(count);
    for index in 0..count {
        let value = |name| required(source, &format!("native_output_{index}_{name}"));
        let request_id = value("request_id")?;
        if request_order
            .get(index)
            .is_some_and(|expected| expected != &request_id)
        {
            return Err(format!(
                "request completion output {index} request-order mismatch"
            ));
        }
        if !seen_request_ids.insert(request_id.clone()) {
            return Err(format!(
                "request completion request id is duplicated: {request_id}"
            ));
        }
        receipts.push(ProviderRequestCompletionReceipt {
            contract: REQUEST_COMPLETION_RECEIPT_CONTRACT.to_owned(),
            status: "pre-seal-acquisition".to_owned(),
            request_id,
            provider_family: adapter_order
                .get(index)
                .cloned()
                .unwrap_or_else(|| "record-provider".to_owned()),
            dispatch_id: "none".to_owned(),
            completion_clock: value("completion_clock_evidence")?,
            output_hash: value("hash")?,
            completion_token: value("completion_token")?,
            selected_set_hash: "none".to_owned(),
        });
    }
    Ok(ProviderRequestCompletionEvidence {
        present: true,
        contract: REQUEST_COMPLETION_COLLECTION_CONTRACT.to_owned(),
        status: "pre-seal-acquisition".to_owned(),
        count: receipts.len(),
        root_hash: "none".to_owned(),
        receipts,
    })
}

pub(crate) fn bind_final_image_dispatch(
    evidence: &mut ProviderRequestCompletionEvidence,
    final_image: &FinalImageProviderDispatchAuthority,
    record_provider_family: &str,
) -> Result<(), String> {
    if evidence.receipts.is_empty() {
        return Ok(());
    }
    for receipt in &mut evidence.receipts {
        if receipt.provider_family == "record-provider" {
            receipt.provider_family = record_provider_family.to_owned();
        }
    }
    if !final_image.available {
        evidence.status = "pre-seal-acquisition".to_owned();
        return Ok(());
    }
    if final_image.status != "verified" || !final_image.blockers.is_empty() {
        return Err(final_image.blockers.join(", "));
    }
    let selected_set_hash = final_image
        .selected_set_hash
        .as_deref()
        .filter(|value| *value != "none")
        .ok_or_else(|| "request completion selected-set root is missing".to_owned())?;
    for receipt in &mut evidence.receipts {
        let entry = final_image
            .entries
            .iter()
            .find(|entry| entry.provider_family == receipt.provider_family)
            .ok_or_else(|| {
                format!(
                    "request-completion-dispatch:entry-missing:{}:{}",
                    receipt.request_id, receipt.provider_family
                )
            })?;
        receipt.dispatch_id = entry.dispatch_id.clone();
        receipt.selected_set_hash = selected_set_hash.to_owned();
        receipt.status = "verified".to_owned();
    }
    evidence.status = "verified".to_owned();
    evidence.count = evidence.receipts.len();
    evidence.root_hash = receipt_root_hash(&evidence.receipts);
    Ok(())
}

pub(crate) fn render_fields(out: &mut String, evidence: &ProviderRequestCompletionEvidence) {
    if !evidence.present {
        return;
    }
    push(out, "request_completion_contract", &evidence.contract);
    push(out, "request_completion_status", &evidence.status);
    push(out, "request_completion_count", &evidence.count.to_string());
    push(out, "request_completion_root_hash", &evidence.root_hash);
    for (index, receipt) in evidence.receipts.iter().enumerate() {
        let prefix = format!("request_completion_{index}_");
        for (name, value) in [
            ("contract", receipt.contract.as_str()),
            ("status", receipt.status.as_str()),
            ("request_id", receipt.request_id.as_str()),
            ("provider_family", receipt.provider_family.as_str()),
            ("dispatch_id", receipt.dispatch_id.as_str()),
            ("completion_clock", receipt.completion_clock.as_str()),
            ("output_hash", receipt.output_hash.as_str()),
            ("completion_token", receipt.completion_token.as_str()),
            ("selected_set_hash", receipt.selected_set_hash.as_str()),
        ] {
            push(out, &format!("{prefix}{name}"), value);
        }
    }
}

pub(crate) fn parse_fields(source: &str) -> ProviderRequestCompletionEvidence {
    let present = string_field(source, "request_completion_contract").is_some();
    if !present {
        return ProviderRequestCompletionEvidence::default();
    }
    let count = usize_field(source, "request_completion_count").unwrap_or(0);
    let receipts = (0..count)
        .map(|index| {
            let prefix = format!("request_completion_{index}_");
            ProviderRequestCompletionReceipt {
                contract: field_or(source, &format!("{prefix}contract"), "none"),
                status: field_or(source, &format!("{prefix}status"), "not-applicable"),
                request_id: field_or(source, &format!("{prefix}request_id"), "none"),
                provider_family: field_or(source, &format!("{prefix}provider_family"), "none"),
                dispatch_id: field_or(source, &format!("{prefix}dispatch_id"), "none"),
                completion_clock: field_or(source, &format!("{prefix}completion_clock"), "none"),
                output_hash: field_or(source, &format!("{prefix}output_hash"), "none"),
                completion_token: field_or(source, &format!("{prefix}completion_token"), "none"),
                selected_set_hash: field_or(source, &format!("{prefix}selected_set_hash"), "none"),
            }
        })
        .collect::<Vec<_>>();
    let mut evidence = ProviderRequestCompletionEvidence {
        present,
        contract: field_or(
            source,
            "request_completion_contract",
            REQUEST_COMPLETION_COLLECTION_CONTRACT,
        ),
        status: field_or(source, "request_completion_status", "not-applicable"),
        count,
        root_hash: field_or(source, "request_completion_root_hash", "none"),
        receipts,
    };
    if evidence.status == "verified" && !verified_shape(&evidence) {
        evidence.status = "mismatch".to_owned();
    }
    evidence
}

pub(crate) fn append_hash_material(
    material: &mut String,
    evidence: &ProviderRequestCompletionEvidence,
) {
    if !evidence.present {
        return;
    }
    material.push_str(&format!(
        "\0{}\0{}\0{}\0{}",
        evidence.contract, evidence.status, evidence.count, evidence.root_hash
    ));
    for receipt in &evidence.receipts {
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
    }
}

fn verified_shape(evidence: &ProviderRequestCompletionEvidence) -> bool {
    let mut request_ids = BTreeSet::new();
    evidence.contract == REQUEST_COMPLETION_COLLECTION_CONTRACT
        && evidence.count == evidence.receipts.len()
        && evidence.root_hash == receipt_root_hash(&evidence.receipts)
        && evidence.receipts.iter().all(|receipt| {
            receipt.contract == REQUEST_COMPLETION_RECEIPT_CONTRACT
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

fn receipt_root_hash(receipts: &[ProviderRequestCompletionReceipt]) -> String {
    let mut material = format!("{REQUEST_COMPLETION_COLLECTION_CONTRACT}\n");
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

fn list_field(source: &str, key: &str) -> Vec<String> {
    string_field(source, key)
        .filter(|value| !value.is_empty())
        .map(|value| value.split(',').map(str::to_owned).collect())
        .unwrap_or_default()
}

fn required(source: &str, key: &str) -> Result<String, String> {
    string_field(source, key)
        .filter(|value| !matches!(value.as_str(), "" | "none" | "pending"))
        .ok_or_else(|| format!("request completion output field `{key}` is missing"))
}

fn field_or(source: &str, key: &str, fallback: &str) -> String {
    string_field(source, key).unwrap_or_else(|| fallback.to_owned())
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    string_field(source, key)?.parse().ok()
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&prefix)?.trim();
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .or(Some(value))
            .map(str::to_owned)
    })
}

fn push(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
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
    use crate::final_image_provider_dispatch::FinalImageProviderDispatch;

    #[test]
    fn binds_each_request_to_its_family_dispatch() {
        let source = r#"
provider_request_order = "request.vulkan,request.cuda,request.vulkan.final"
provider_request_adapter_order = "spirv:vulkan-gpu,cuda:nvidia-gpu,spirv:vulkan-gpu"
native_output_0_request_id = "request.vulkan"
native_output_0_completion_clock_evidence = "clock:0"
native_output_0_hash = "0x0000000000000001"
native_output_0_completion_token = "completion:0"
native_output_1_request_id = "request.cuda"
native_output_1_completion_clock_evidence = "clock:1"
native_output_1_hash = "0x0000000000000002"
native_output_1_completion_token = "completion:1"
native_output_2_request_id = "request.vulkan.final"
native_output_2_completion_clock_evidence = "clock:2"
native_output_2_hash = "0x0000000000000003"
native_output_2_completion_token = "completion:2"
"#;
        let mut evidence = from_output_payload(source, 3).unwrap();
        let dispatch = |dispatch_id: &str, provider_family: &str| FinalImageProviderDispatch {
            dispatch_id: dispatch_id.to_owned(),
            package_id: format!("package.{dispatch_id}"),
            bundle_id: format!("bundle.{dispatch_id}"),
            provider_family: provider_family.to_owned(),
            runner_contract: "runner-contract".to_owned(),
            runner_adapter_contract: "runner-adapter-contract".to_owned(),
            runner_adapter_id: format!("runner.{dispatch_id}"),
        };
        let final_image = FinalImageProviderDispatchAuthority {
            available: true,
            status: "verified".to_owned(),
            image_path: Some("nuis.final-image.nsb".to_owned()),
            table_hash: Some("0x1111111111111111".to_owned()),
            selected_set_hash: Some("fnv1a64:2222222222222222".to_owned()),
            entries: vec![
                dispatch("dispatch0000", "spirv:vulkan-gpu"),
                dispatch("dispatch0001", "cuda:nvidia-gpu"),
            ],
            blockers: Vec::new(),
        };

        bind_final_image_dispatch(&mut evidence, &final_image, "spirv:vulkan-gpu").unwrap();

        assert_eq!(evidence.status, "verified");
        assert_eq!(evidence.count, 3);
        assert!(evidence.root_hash.starts_with("0x"));
        assert_eq!(evidence.receipts[0].dispatch_id, "dispatch0000");
        assert_eq!(evidence.receipts[1].dispatch_id, "dispatch0001");
        assert_eq!(evidence.receipts[2].dispatch_id, "dispatch0000");
        assert!(verified_shape(&evidence));
    }

    #[test]
    fn parsed_verified_receipts_reject_duplicate_request_ids() {
        let mut receipts = vec![ProviderRequestCompletionReceipt {
            contract: REQUEST_COMPLETION_RECEIPT_CONTRACT.to_owned(),
            status: "verified".to_owned(),
            request_id: "request.alpha".to_owned(),
            provider_family: "provider:alpha".to_owned(),
            dispatch_id: "dispatch0000".to_owned(),
            completion_clock: "clock:1".to_owned(),
            output_hash: "0x1111111111111111".to_owned(),
            completion_token: "provider-completion:0x2222222222222222".to_owned(),
            selected_set_hash: "fnv1a64:3333333333333333".to_owned(),
        }];
        receipts.push(receipts[0].clone());
        let evidence = ProviderRequestCompletionEvidence {
            present: true,
            contract: REQUEST_COMPLETION_COLLECTION_CONTRACT.to_owned(),
            status: "verified".to_owned(),
            count: 2,
            root_hash: receipt_root_hash(&receipts),
            receipts,
        };

        assert!(!verified_shape(&evidence));
    }
}

use crate::artifact_nsdb_handoff_integrity::record_hash;
use std::collections::BTreeSet;

const AUTHORITY_CONTRACT: &str = "nuis-provider-completion-dispatch-authority-v1";
const FINAL_IMAGE_DISPATCH_CONTRACT: &str = "nuis-final-image-provider-dispatch-v1";
const COMPLETION_EVIDENCE_COLLECTION_CONTRACT: &str =
    "nuis-provider-completion-evidence-collection-v1";

#[derive(Clone)]
pub(crate) struct PersistedProviderCompletion {
    pub(crate) trace_id: String,
    pub(crate) provider_family: String,
    pub(crate) output_contract: String,
    pub(crate) output_evidence: String,
    pub(crate) dispatch_authority_contract: String,
    pub(crate) dispatch_authority_status: String,
    pub(crate) dispatch_table_hash: String,
    pub(crate) dispatch_selected_set_hash: String,
    pub(crate) dispatch_id: String,
    pub(crate) dispatch_package_id: String,
    pub(crate) dispatch_bundle_id: String,
    pub(crate) dispatch_provider_family: String,
    pub(crate) dispatch_runner_contract: String,
    pub(crate) dispatch_runner_adapter_contract: String,
    pub(crate) dispatch_runner_adapter_id: String,
    pub(crate) record_hash: String,
}

#[derive(Clone)]
pub(crate) struct PersistedProviderDispatchIdentity {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) table_hash: String,
    pub(crate) selected_set_hash: String,
    pub(crate) identity_hash: String,
}

pub(crate) fn parse_provider_completions(
    source: &str,
    digest_contract: &str,
) -> Vec<PersistedProviderCompletion> {
    source
        .split("[[records]]")
        .skip(1)
        .filter(|record| field(record, "execution_phase") == "provider-device-completion")
        .map(|record| parse_completion(record, digest_contract))
        .collect()
}

pub(crate) fn dispatch_identity(
    completions: &[PersistedProviderCompletion],
    final_image_status: &str,
) -> PersistedProviderDispatchIdentity {
    if completions.is_empty() {
        return identity("not-applicable", "none", "none");
    }
    let first = &completions[0];
    if completions.iter().all(|completion| {
        completion.dispatch_authority_contract == "none"
            && completion.dispatch_authority_status == "none"
    }) {
        let status = if matches!(final_image_status, "verified" | "verified-empty") {
            "final-image-authority-missing"
        } else {
            "pre-seal-acquisition"
        };
        return identity(status, "none", "none");
    }
    if completions.iter().any(|completion| {
        completion.dispatch_authority_contract != AUTHORITY_CONTRACT
            || completion.dispatch_authority_contract != first.dispatch_authority_contract
            || completion.dispatch_table_hash != first.dispatch_table_hash
            || completion.dispatch_selected_set_hash != first.dispatch_selected_set_hash
            || (completion.dispatch_authority_status == "verified"
                && completion.provider_family != completion.dispatch_provider_family)
    }) {
        return identity(
            "mismatch",
            &first.dispatch_table_hash,
            &first.dispatch_selected_set_hash,
        );
    }
    if completions
        .iter()
        .all(|completion| completion.dispatch_authority_status == "verified")
    {
        let mut dispatch_ids = BTreeSet::new();
        if completions.iter().any(|completion| {
            [
                completion.dispatch_id.as_str(),
                completion.dispatch_package_id.as_str(),
                completion.dispatch_bundle_id.as_str(),
                completion.dispatch_runner_contract.as_str(),
                completion.dispatch_runner_adapter_contract.as_str(),
                completion.dispatch_runner_adapter_id.as_str(),
            ]
            .iter()
            .any(|value| matches!(*value, "" | "none"))
                || !dispatch_ids.insert(completion.dispatch_id.as_str())
        }) {
            return identity(
                "mismatch",
                &first.dispatch_table_hash,
                &first.dispatch_selected_set_hash,
            );
        }
        return identity(
            "verified",
            &first.dispatch_table_hash,
            &first.dispatch_selected_set_hash,
        );
    }
    let status = if matches!(final_image_status, "verified" | "verified-empty") {
        "final-image-authority-missing"
    } else {
        "pre-seal-acquisition"
    };
    identity(
        status,
        &first.dispatch_table_hash,
        &first.dispatch_selected_set_hash,
    )
}

fn parse_completion(record: &str, digest_contract: &str) -> PersistedProviderCompletion {
    let trace_id = field(record, "trace_id");
    let provider_family = field(record, "provider_family");
    let output_contract = field(record, "output_contract");
    let output_evidence = field(record, "output_evidence");
    let dispatch_authority_contract = field(record, "dispatch_authority_contract");
    let dispatch_authority_status = field(record, "dispatch_authority_status");
    let dispatch_table_hash = field(record, "dispatch_table_hash");
    let dispatch_selected_set_hash = field(record, "dispatch_selected_set_hash");
    let dispatch_id = field(record, "dispatch_id");
    let dispatch_package_id = field(record, "dispatch_package_id");
    let dispatch_bundle_id = field(record, "dispatch_bundle_id");
    let dispatch_provider_family = field(record, "dispatch_provider_family");
    let dispatch_runner_contract = field(record, "dispatch_runner_contract");
    let dispatch_runner_adapter_contract = field(record, "dispatch_runner_adapter_contract");
    let dispatch_runner_adapter_id = field(record, "dispatch_runner_adapter_id");
    let mut material =
        format!("{trace_id}\0{provider_family}\0{output_contract}\0{output_evidence}");
    append_completion_evidence_hash_material(&mut material, record);
    if dispatch_authority_contract == AUTHORITY_CONTRACT
        && dispatch_authority_status != "not-applicable"
    {
        material.push_str(&format!(
            "\0{dispatch_authority_contract}\0{dispatch_authority_status}\0{dispatch_table_hash}\0{dispatch_selected_set_hash}\0{dispatch_id}\0{dispatch_package_id}\0{dispatch_bundle_id}\0{dispatch_provider_family}\0{dispatch_runner_contract}\0{dispatch_runner_adapter_contract}\0{dispatch_runner_adapter_id}"
        ));
    }
    PersistedProviderCompletion {
        trace_id,
        provider_family,
        output_contract,
        output_evidence,
        dispatch_authority_contract,
        dispatch_authority_status,
        dispatch_table_hash,
        dispatch_selected_set_hash,
        dispatch_id,
        dispatch_package_id,
        dispatch_bundle_id,
        dispatch_provider_family,
        dispatch_runner_contract,
        dispatch_runner_adapter_contract,
        dispatch_runner_adapter_id,
        record_hash: record_hash(digest_contract, material.as_bytes())
            .unwrap_or_else(|| "none".to_owned()),
    }
}

fn identity(
    status: &str,
    table_hash: &str,
    selected_set_hash: &str,
) -> PersistedProviderDispatchIdentity {
    let identity_hash = if status == "verified"
        && table_hash != "none"
        && selected_set_hash != "none"
    {
        fnv1a64_hex(
            format!(
                "{AUTHORITY_CONTRACT}\0{FINAL_IMAGE_DISPATCH_CONTRACT}\0{table_hash}\0{selected_set_hash}"
            )
            .as_bytes(),
        )
    } else {
        "none".to_owned()
    };
    PersistedProviderDispatchIdentity {
        contract: AUTHORITY_CONTRACT.to_owned(),
        status: status.to_owned(),
        table_hash: table_hash.to_owned(),
        selected_set_hash: selected_set_hash.to_owned(),
        identity_hash,
    }
}

fn append_completion_evidence_hash_material(material: &mut String, record: &str) {
    if field(record, "completion_evidence_contract") != COMPLETION_EVIDENCE_COLLECTION_CONTRACT
        || field(record, "completion_evidence_status") != "verified"
    {
        return;
    }
    for key in [
        "completion_evidence_contract",
        "completion_evidence_status",
        "completion_evidence_count",
        "completion_clock_evidence",
        "completion_tokens",
        "glm_release_contract",
        "glm_release_tokens",
        "glm_release_status",
        "code_asset_identity_contract",
        "code_asset_identity_status",
        "code_asset_identity_asset_id",
        "code_asset_identity_hash",
        "code_asset_identity_set_contract",
        "code_asset_identity_set_status",
        "code_asset_identity_set_count",
        "code_asset_identity_set_root_hash",
    ] {
        material.push('\0');
        material.push_str(&field(record, key));
    }
    append_compiled_code_asset_selection_hash_material(material, record);
}

fn append_compiled_code_asset_selection_hash_material(material: &mut String, record: &str) {
    if field(record, "compiled_code_asset_selection_status") != "verified" {
        return;
    }
    for key in [
        "compiled_code_asset_selection_contract",
        "compiled_code_asset_selection_status",
        "compiled_code_asset_table_contract",
        "compiled_code_asset_table_hash",
        "compiled_code_asset_contribution_count",
        "compiled_code_asset_identity_set_root_hash",
        "compiled_code_asset_contribution_index",
        "compiled_code_asset_asset_id",
        "compiled_code_asset_identity_hash",
        "compiled_code_asset_selection_count",
    ] {
        append_rendered_toml_hash_line(material, key, &field(record, key));
    }
    let selection_count = field(record, "compiled_code_asset_selection_count")
        .parse::<usize>()
        .unwrap_or(0);
    for index in 0..selection_count {
        for suffix in ["contribution_index", "asset_id", "identity_hash"] {
            let key = format!("compiled_code_asset_selection_{index}_{suffix}");
            append_rendered_toml_hash_line(material, &key, &field(record, &key));
        }
    }
}

fn append_rendered_toml_hash_line(material: &mut String, key: &str, value: &str) {
    material.push('\0');
    material.push_str(key);
    material.push_str(" = \"");
    material.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    material.push('"');
}

fn field(source: &str, key: &str) -> String {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .and_then(|value| value.trim().strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape)
        .unwrap_or_else(|| "none".to_owned())
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
    use super::{dispatch_identity, parse_provider_completions};

    const RECORD: &str = r#"
[[records]]
execution_phase = "provider-device-completion"
trace_id = "trace:metal"
provider_family = "metal:apple-silicon-gpu"
output_contract = "provider-output-v1"
output_evidence = "metal.toml"
dispatch_authority_contract = "nuis-provider-completion-dispatch-authority-v1"
dispatch_authority_status = "verified"
dispatch_table_hash = "0x1111111111111111"
dispatch_selected_set_hash = "fnv1a64:2222222222222222"
dispatch_id = "dispatch:metal"
dispatch_package_id = "pixelmagic"
dispatch_bundle_id = "pixelmagic.metal"
dispatch_provider_family = "metal:apple-silicon-gpu"
dispatch_runner_contract = "nuis-provider-runner-v1"
dispatch_runner_adapter_contract = "nuis-provider-runner-adapter-v1"
dispatch_runner_adapter_id = "metal-real-device-v1"
"#;

    #[test]
    fn independently_binds_verified_completion_dispatch_identity() {
        let completions =
            parse_provider_completions(RECORD, "nuis-provider-completion-digest-sha256-v1");
        let identity = dispatch_identity(&completions, "verified");

        assert_eq!(identity.status, "verified");
        assert_eq!(identity.table_hash, "0x1111111111111111");
        assert_eq!(identity.selected_set_hash, "fnv1a64:2222222222222222");
        assert!(identity.identity_hash.starts_with("0x"));
    }

    #[test]
    fn rejects_provider_family_and_runner_identity_drift() {
        for drifted in [
            RECORD.replace(
                "dispatch_provider_family = \"metal:apple-silicon-gpu\"",
                "dispatch_provider_family = \"coreml:apple-ane\"",
            ),
            RECORD.replace(
                "dispatch_runner_adapter_id = \"metal-real-device-v1\"",
                "dispatch_runner_adapter_id = \"none\"",
            ),
        ] {
            let completions =
                parse_provider_completions(&drifted, "nuis-provider-completion-digest-sha256-v1");
            assert_eq!(
                dispatch_identity(&completions, "verified").status,
                "mismatch"
            );
        }
    }
}

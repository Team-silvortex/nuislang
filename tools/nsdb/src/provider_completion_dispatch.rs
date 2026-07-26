use crate::{
    final_image_provider_dispatch::{FinalImageProviderDispatchAuthority, DISPATCH_CONTRACT},
    model::{
        NsdbDeviceProviderSampleRecordInfo, NsdbPayloadExecutionEvent,
        NsdbProviderCompletionDispatchAuthority, NsdbProviderCompletionDispatchIdentity,
    },
    provider_sample_payload::fnv1a64_hex,
};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) const COMPLETION_AUTHORITY_CONTRACT: &str =
    "nuis-provider-completion-dispatch-authority-v1";

pub(crate) fn authority_for_record(
    final_image: &FinalImageProviderDispatchAuthority,
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> Result<NsdbProviderCompletionDispatchAuthority, String> {
    if !final_image.available {
        return Ok(NsdbProviderCompletionDispatchAuthority {
            contract: COMPLETION_AUTHORITY_CONTRACT.to_owned(),
            status: final_image.status.clone(),
            ..authority_defaults()
        });
    }
    if final_image.status != "verified" || !final_image.blockers.is_empty() {
        return Err(final_image.blockers.join(", "));
    }
    let entry = final_image
        .entries
        .iter()
        .find(|entry| {
            entry.package_id == record.provider_bundle_package_id
                && entry.bundle_id == record.provider_bundle_id
                && entry.provider_family == record.provider_family
        })
        .ok_or_else(|| {
            format!(
                "provider-completion-dispatch:entry-missing:{}:{}:{}",
                record.provider_bundle_package_id,
                record.provider_bundle_id,
                record.provider_family
            )
        })?;
    if entry.runner_contract != record.provider_runner_contract
        || entry.runner_adapter_contract != record.provider_runner_adapter_contract
        || entry.runner_adapter_id != record.provider_runner_adapter_id
    {
        return Err(format!(
            "provider-completion-dispatch:runner-drift:{}",
            entry.dispatch_id
        ));
    }
    Ok(NsdbProviderCompletionDispatchAuthority {
        contract: COMPLETION_AUTHORITY_CONTRACT.to_owned(),
        status: "verified".to_owned(),
        table_hash: final_image
            .table_hash
            .clone()
            .unwrap_or_else(|| "none".to_owned()),
        selected_set_hash: final_image
            .selected_set_hash
            .clone()
            .unwrap_or_else(|| "none".to_owned()),
        dispatch_id: entry.dispatch_id.clone(),
        package_id: entry.package_id.clone(),
        bundle_id: entry.bundle_id.clone(),
        provider_family: entry.provider_family.clone(),
        runner_contract: entry.runner_contract.clone(),
        runner_adapter_contract: entry.runner_adapter_contract.clone(),
        runner_adapter_id: entry.runner_adapter_id.clone(),
    })
}

pub(crate) fn bind_events_from_final_image(
    output_dir: &Path,
    events: &mut [NsdbPayloadExecutionEvent],
) -> Result<usize, String> {
    let completion_count = events
        .iter()
        .filter(|event| event.execution_phase == "provider-device-completion")
        .count();
    if completion_count == 0 {
        return Ok(0);
    }
    let final_image =
        crate::final_image_provider_dispatch::final_image_provider_dispatch_authority(output_dir);
    if !final_image.available {
        return Ok(0);
    }
    if final_image.status != "verified" || !final_image.blockers.is_empty() {
        return Err(final_image.blockers.join(", "));
    }
    let manifest = crate::provider_sample::read_device_provider_sample_manifest_info(output_dir);
    if !manifest.available {
        return Err("provider-completion-dispatch:sample-manifest-missing".to_owned());
    }
    let mut bound = 0usize;
    for event in events
        .iter_mut()
        .filter(|event| event.execution_phase == "provider-device-completion")
    {
        let record = manifest
            .records
            .iter()
            .find(|record| record.trace_id == event.trace_id)
            .ok_or_else(|| {
                format!(
                    "provider-completion-dispatch:sample-record-missing:{}",
                    event.trace_id
                )
            })?;
        event.provider_completion_dispatch = authority_for_record(&final_image, record)?;
        bound += 1;
    }
    Ok(bound)
}

pub(crate) fn completion_identity(
    events: &[NsdbPayloadExecutionEvent],
    final_image_proof_status: &str,
) -> NsdbProviderCompletionDispatchIdentity {
    let completions = events
        .iter()
        .filter(|event| event.execution_phase == "provider-device-completion")
        .collect::<Vec<_>>();
    if completions.is_empty() {
        return identity("not-applicable", "none", "none");
    }
    let first = &completions[0].provider_completion_dispatch;
    if completions.iter().all(|event| {
        let authority = &event.provider_completion_dispatch;
        authority.contract == "none" && authority.status == "none"
    }) {
        let status = if matches!(final_image_proof_status, "verified" | "verified-empty") {
            "final-image-authority-missing"
        } else {
            "pre-seal-acquisition"
        };
        return identity(status, "none", "none");
    }
    if completions.iter().any(|event| {
        let authority = &event.provider_completion_dispatch;
        authority.contract != COMPLETION_AUTHORITY_CONTRACT
            || authority.contract != first.contract
            || authority.table_hash != first.table_hash
            || authority.selected_set_hash != first.selected_set_hash
            || (authority.status == "verified"
                && authority.provider_family != event.provider_family)
    }) {
        return identity("mismatch", &first.table_hash, &first.selected_set_hash);
    }
    if completions
        .iter()
        .all(|event| event.provider_completion_dispatch.status == "verified")
    {
        let mut dispatch_ids = BTreeSet::new();
        if completions.iter().any(|event| {
            let authority = &event.provider_completion_dispatch;
            [
                authority.dispatch_id.as_str(),
                authority.package_id.as_str(),
                authority.bundle_id.as_str(),
                authority.runner_contract.as_str(),
                authority.runner_adapter_contract.as_str(),
                authority.runner_adapter_id.as_str(),
            ]
            .iter()
            .any(|value| matches!(*value, "" | "none"))
                || !dispatch_ids.insert(authority.dispatch_id.as_str())
        }) {
            return identity("mismatch", &first.table_hash, &first.selected_set_hash);
        }
        return identity("verified", &first.table_hash, &first.selected_set_hash);
    }
    if matches!(final_image_proof_status, "verified" | "verified-empty") {
        return identity(
            "final-image-authority-missing",
            &first.table_hash,
            &first.selected_set_hash,
        );
    }
    identity(
        "pre-seal-acquisition",
        &first.table_hash,
        &first.selected_set_hash,
    )
}

pub(crate) fn verified_identity_hash(
    contract: &str,
    table_hash: &str,
    selected_set_hash: &str,
) -> Option<String> {
    (contract == COMPLETION_AUTHORITY_CONTRACT
        && table_hash != "none"
        && selected_set_hash != "none")
        .then(|| {
            fnv1a64_hex(
                format!("{contract}\0{DISPATCH_CONTRACT}\0{table_hash}\0{selected_set_hash}")
                    .as_bytes(),
            )
        })
}

pub(crate) fn render_event_fields(
    out: &mut String,
    authority: &NsdbProviderCompletionDispatchAuthority,
) {
    for (key, value) in [
        ("dispatch_authority_contract", authority.contract.as_str()),
        ("dispatch_authority_status", authority.status.as_str()),
        ("dispatch_table_hash", authority.table_hash.as_str()),
        (
            "dispatch_selected_set_hash",
            authority.selected_set_hash.as_str(),
        ),
        ("dispatch_id", authority.dispatch_id.as_str()),
        ("dispatch_package_id", authority.package_id.as_str()),
        ("dispatch_bundle_id", authority.bundle_id.as_str()),
        (
            "dispatch_provider_family",
            authority.provider_family.as_str(),
        ),
        (
            "dispatch_runner_contract",
            authority.runner_contract.as_str(),
        ),
        (
            "dispatch_runner_adapter_contract",
            authority.runner_adapter_contract.as_str(),
        ),
        (
            "dispatch_runner_adapter_id",
            authority.runner_adapter_id.as_str(),
        ),
    ] {
        push_toml_string(out, key, value);
    }
}

pub(crate) fn parse_event_fields(source: &str) -> NsdbProviderCompletionDispatchAuthority {
    NsdbProviderCompletionDispatchAuthority {
        contract: field(source, "dispatch_authority_contract"),
        status: field(source, "dispatch_authority_status"),
        table_hash: field(source, "dispatch_table_hash"),
        selected_set_hash: field(source, "dispatch_selected_set_hash"),
        dispatch_id: field(source, "dispatch_id"),
        package_id: field(source, "dispatch_package_id"),
        bundle_id: field(source, "dispatch_bundle_id"),
        provider_family: field(source, "dispatch_provider_family"),
        runner_contract: field(source, "dispatch_runner_contract"),
        runner_adapter_contract: field(source, "dispatch_runner_adapter_contract"),
        runner_adapter_id: field(source, "dispatch_runner_adapter_id"),
    }
}

fn identity(
    status: &str,
    table_hash: &str,
    selected_set_hash: &str,
) -> NsdbProviderCompletionDispatchIdentity {
    let identity_hash = if status == "verified" {
        verified_identity_hash(COMPLETION_AUTHORITY_CONTRACT, table_hash, selected_set_hash)
            .unwrap_or_else(|| "none".to_owned())
    } else {
        "none".to_owned()
    };
    NsdbProviderCompletionDispatchIdentity {
        contract: COMPLETION_AUTHORITY_CONTRACT.to_owned(),
        status: status.to_owned(),
        table_hash: table_hash.to_owned(),
        selected_set_hash: selected_set_hash.to_owned(),
        identity_hash,
    }
}

fn authority_defaults() -> NsdbProviderCompletionDispatchAuthority {
    NsdbProviderCompletionDispatchAuthority {
        table_hash: "none".to_owned(),
        selected_set_hash: "none".to_owned(),
        dispatch_id: "none".to_owned(),
        package_id: "none".to_owned(),
        bundle_id: "none".to_owned(),
        provider_family: "none".to_owned(),
        runner_contract: "none".to_owned(),
        runner_adapter_contract: "none".to_owned(),
        runner_adapter_id: "none".to_owned(),
        ..NsdbProviderCompletionDispatchAuthority::default()
    }
}

fn field(source: &str, key: &str) -> String {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .and_then(|value| value.trim().strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_toml)
        .unwrap_or_else(|| "none".to_owned())
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!("{key} = \"{}\"\n", escape_toml(value)));
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unescape_toml(value: &str) -> String {
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

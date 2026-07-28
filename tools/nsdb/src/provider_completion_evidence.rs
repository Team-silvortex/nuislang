use crate::model::NsdbPayloadExecutionEvent;
use std::{fs, path::Path};

pub(crate) const COMPLETION_EVIDENCE_COLLECTION_CONTRACT: &str =
    "nuis-provider-completion-evidence-collection-v1";
const OUTPUT_COLLECTION_CONTRACT: &str = "nuis-provider-output-collection-v1";
const COMPLETION_CONTRACT: &str = "nuis-provider-completion-evidence-v1";
const COMPLETION_CLOCK_CONTRACT: &str = "nuis-provider-completion-clock-v1";
const GLM_RELEASE_CONTRACT: &str = "nuis-provider-glm-release-evidence-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCompletionEvidence {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) clock_evidence: String,
    pub(crate) completion_tokens: String,
    pub(crate) glm_release_contract: String,
    pub(crate) glm_release_tokens: String,
    pub(crate) glm_release_status: String,
}

impl Default for ProviderCompletionEvidence {
    fn default() -> Self {
        Self {
            contract: "none".to_owned(),
            status: "not-applicable".to_owned(),
            count: 0,
            clock_evidence: "none".to_owned(),
            completion_tokens: "none".to_owned(),
            glm_release_contract: "none".to_owned(),
            glm_release_tokens: "none".to_owned(),
            glm_release_status: "not-applicable".to_owned(),
        }
    }
}

pub(crate) fn event_from_record(
    output_dir: &Path,
    record: &crate::model::NsdbDeviceProviderSampleRecordInfo,
    final_image: &crate::final_image_provider_dispatch::FinalImageProviderDispatchAuthority,
) -> Result<NsdbPayloadExecutionEvent, String> {
    let output_evidence = if !matches!(
        record.provider_output_payload_evidence.as_str(),
        "none" | "not-materialized"
    ) {
        record.provider_output_payload_evidence.clone()
    } else {
        record.output_evidence.clone()
    };
    let output_contract = if record.provider_output_payload_contract == "none" {
        "nsdb-yir-provider-sample-artifact-v1".to_owned()
    } else {
        record.provider_output_payload_contract.clone()
    };
    let provider_completion_evidence = from_output_payload(output_dir, &output_evidence)?;
    Ok(NsdbPayloadExecutionEvent {
        index: 0,
        trace_id: record.trace_id.clone(),
        status: "ready".to_owned(),
        execution_phase: "provider-device-completion".to_owned(),
        target: record.provider_family.clone(),
        entry_symbol: record.provider.clone(),
        entry_kind: output_contract.clone(),
        entry_section_id: output_evidence.clone(),
        provider_family: record.provider_family.clone(),
        output_contract,
        output_evidence,
        provider_completion_evidence,
        provider_completion_dispatch: crate::provider_completion_dispatch::authority_for_record(
            final_image,
            record,
        )?,
        first_blocker: "none".to_owned(),
        next_action: "replay-provider-completion".to_owned(),
    })
}

pub(crate) fn from_output_payload(
    output_dir: &Path,
    output_evidence: &str,
) -> Result<ProviderCompletionEvidence, String> {
    let (file_name, hash_claim) = output_reference(output_evidence)?;
    let relative = Path::new(file_name);
    if relative.is_absolute()
        || relative.components().count() != 1
        || !matches!(
            relative.components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err("provider completion output path is not package-relative".to_owned());
    }
    let bytes = fs::read(output_dir.join(relative))
        .map_err(|error| format!("provider completion output is unreadable: {error}"))?;
    if fnv1a64_hex(&bytes) != hash_claim {
        return Err("provider completion output evidence hash mismatch".to_owned());
    }
    let source = std::str::from_utf8(&bytes)
        .map_err(|_| "provider completion output is not UTF-8".to_owned())?;
    if string_field(source, "native_output_collection_contract").is_none() {
        return Ok(ProviderCompletionEvidence::default());
    }
    if string_field(source, "native_output_collection_contract").as_deref()
        != Some(OUTPUT_COLLECTION_CONTRACT)
    {
        return Err("provider completion output collection contract mismatch".to_owned());
    }
    let count = usize_field(source, "native_output_count")
        .filter(|count| *count > 0)
        .ok_or_else(|| "provider completion output collection is empty".to_owned())?;
    let collection_hash = required(source, "native_output_collection_hash")?;
    let mut clocks = Vec::with_capacity(count);
    let mut completion_tokens = Vec::with_capacity(count);
    let mut release_tokens = Vec::with_capacity(count);
    let mut collection_material = String::new();
    for index in 0..count {
        let output = validate_output(source, index)?;
        clocks.push(output.clock);
        completion_tokens.push(output.completion_token);
        release_tokens.push(output.release_token);
        collection_material.push_str(&output.collection_material);
    }
    if fnv1a64_hex(collection_material.as_bytes()) != collection_hash {
        return Err("provider completion native-output collection hash mismatch".to_owned());
    }
    Ok(ProviderCompletionEvidence {
        contract: COMPLETION_EVIDENCE_COLLECTION_CONTRACT.to_owned(),
        status: "verified".to_owned(),
        count,
        clock_evidence: clocks.join("|"),
        completion_tokens: completion_tokens.join(","),
        glm_release_contract: GLM_RELEASE_CONTRACT.to_owned(),
        glm_release_tokens: release_tokens.join(","),
        glm_release_status: "released-at-graph-close".to_owned(),
    })
}

struct ValidatedOutput {
    clock: String,
    completion_token: String,
    release_token: String,
    collection_material: String,
}

fn validate_output(source: &str, index: usize) -> Result<ValidatedOutput, String> {
    let value = |name| required(source, &format!("native_output_{index}_{name}"));
    let clock = value("completion_clock_evidence")?;
    validate_clock(&clock)?;
    let completion_token = value("completion_token")?;
    if value("completion_evidence_contract")? != COMPLETION_CONTRACT
        || value("completion_status")? != "worker-output-verified"
    {
        return Err(format!(
            "provider completion output {index} completion contract is invalid"
        ));
    }
    let completion_material = format!(
        "{clock}:{}:{}:{}:{}:{}",
        value("worker_operation_token")?,
        value("worker_execution_capsule_token")?,
        value("worker_output_descriptor_roles")?,
        value("worker_output_descriptor_hash")?,
        present(
            source,
            &format!("native_output_{index}_worker_additional_output_hashes")
        )?
    );
    if completion_token
        != format!(
            "provider-completion:{}",
            fnv1a64_hex(completion_material.as_bytes())
        )
    {
        return Err(format!(
            "provider completion output {index} completion token mismatch"
        ));
    }
    let release_token = value("glm_release_token")?;
    if value("glm_release_contract")? != GLM_RELEASE_CONTRACT
        || value("glm_release_status")? != "released-at-graph-close"
    {
        return Err(format!(
            "provider completion output {index} GLM release contract is invalid"
        ));
    }
    let release_manifest = release_manifest(
        &value("output_binding_roles")?,
        &value("output_binding_buffers")?,
    )?;
    let release_material = format!(
        "{completion_token}:{}:{}:{release_manifest}",
        value("graph_output_ownership_contract")?,
        value("output_handle_ownership_tokens")?
    );
    if release_token != format!("glm-release:{}", fnv1a64_hex(release_material.as_bytes())) {
        return Err(format!(
            "provider completion output {index} GLM release token mismatch"
        ));
    }
    let collection_material = format!(
        "{index}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{};",
        value("request_id")?,
        value("hash")?,
        value("output_carrier_adapter_id")?,
        value("output_residency_kind")?,
        value("output_transfer_scope")?,
        value("output_observation_mode")?,
        value("output_device_retention_status")?,
        value("session_lease_id")?,
        value("output_handle_id")?,
        value("output_handle_ownership_token")?,
        completion_token,
        release_token,
        value("comparison_contract")?,
        value("comparison_status")?
    );
    Ok(ValidatedOutput {
        clock,
        completion_token,
        release_token,
        collection_material,
    })
}

fn validate_clock(clock: &str) -> Result<(), String> {
    let clock = clock
        .strip_prefix(&format!("{COMPLETION_CLOCK_CONTRACT}:domain="))
        .ok_or_else(|| "provider completion clock contract mismatch".to_owned())?;
    let (_, sequence) = clock
        .rsplit_once(":session=")
        .ok_or_else(|| "provider completion clock has no session tick".to_owned())?;
    let (session, worker) = sequence
        .split_once(":worker=")
        .ok_or_else(|| "provider completion clock has no worker tick".to_owned())?;
    if session.parse::<usize>().ok() != worker.parse::<usize>().ok()
        || session.parse::<usize>().is_err()
    {
        return Err("provider completion clock session and worker ticks diverged".to_owned());
    }
    Ok(())
}

fn release_manifest(roles: &str, buffers: &str) -> Result<String, String> {
    let roles = roles.split(',').collect::<Vec<_>>();
    let buffers = buffers.split(',').collect::<Vec<_>>();
    if roles.len() != buffers.len() || roles.is_empty() {
        return Err("provider completion release manifest shape mismatch".to_owned());
    }
    let mut pairs = roles.into_iter().zip(buffers).collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.1.cmp(right.1));
    Ok(pairs
        .into_iter()
        .map(|(role, buffer)| format!("{role}={buffer}"))
        .collect::<Vec<_>>()
        .join(","))
}

pub(crate) fn render_event_fields(out: &mut String, event: &ProviderCompletionEvidence) {
    for (key, value) in [
        ("completion_evidence_contract", event.contract.as_str()),
        ("completion_evidence_status", event.status.as_str()),
        ("completion_evidence_count", &event.count.to_string()),
        ("completion_clock_evidence", event.clock_evidence.as_str()),
        ("completion_tokens", event.completion_tokens.as_str()),
        ("glm_release_contract", event.glm_release_contract.as_str()),
        ("glm_release_tokens", event.glm_release_tokens.as_str()),
        ("glm_release_status", event.glm_release_status.as_str()),
    ] {
        push_toml_string(out, key, value);
    }
}

pub(crate) fn parse_event_fields(source: &str) -> ProviderCompletionEvidence {
    ProviderCompletionEvidence {
        contract: field_or(source, "completion_evidence_contract", "none"),
        status: field_or(source, "completion_evidence_status", "not-applicable"),
        count: usize_field(source, "completion_evidence_count").unwrap_or(0),
        clock_evidence: field_or(source, "completion_clock_evidence", "none"),
        completion_tokens: field_or(source, "completion_tokens", "none"),
        glm_release_contract: field_or(source, "glm_release_contract", "none"),
        glm_release_tokens: field_or(source, "glm_release_tokens", "none"),
        glm_release_status: field_or(source, "glm_release_status", "not-applicable"),
    }
}

pub(crate) fn append_hash_material(material: &mut String, event: &NsdbPayloadExecutionEvent) {
    let evidence = &event.provider_completion_evidence;
    if evidence.contract == COMPLETION_EVIDENCE_COLLECTION_CONTRACT && evidence.status == "verified"
    {
        material.push_str(&format!(
            "\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
            evidence.contract,
            evidence.status,
            evidence.count,
            evidence.clock_evidence,
            evidence.completion_tokens,
            evidence.glm_release_contract,
            evidence.glm_release_tokens,
            evidence.glm_release_status
        ));
    }
}

fn output_reference(evidence: &str) -> Result<(&str, String), String> {
    let mut parts = evidence.split(':');
    let file_name = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "provider completion output evidence has no file".to_owned())?;
    let hash = parts
        .find_map(|part| part.strip_prefix("hash="))
        .filter(|value| valid_hash(value))
        .ok_or_else(|| "provider completion output evidence has no valid hash".to_owned())?;
    Ok((file_name, hash.to_owned()))
}

fn required(source: &str, key: &str) -> Result<String, String> {
    string_field(source, key)
        .filter(|value| !value.is_empty() && value != "none" && value != "pending")
        .ok_or_else(|| format!("provider completion output field `{key}` is missing"))
}

fn present(source: &str, key: &str) -> Result<String, String> {
    string_field(source, key)
        .ok_or_else(|| format!("provider completion output field `{key}` is missing"))
}

fn field_or(source: &str, key: &str, fallback: &str) -> String {
    string_field(source, key).unwrap_or_else(|| fallback.to_owned())
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)?
            .trim()
            .strip_prefix('"')?
            .strip_suffix('"')
            .map(str::to_owned)
    })
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)?
            .trim()
            .trim_matches('"')
            .parse()
            .ok()
    })
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!(
        "{key} = \"{}\"\n",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    ));
}

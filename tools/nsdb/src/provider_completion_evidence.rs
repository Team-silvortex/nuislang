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
    pub(crate) code_asset_identity_contract: String,
    pub(crate) code_asset_identity_status: String,
    pub(crate) code_asset_identity_asset_id: String,
    pub(crate) code_asset_identity_hash: String,
    pub(crate) code_asset_identity_set_contract: String,
    pub(crate) code_asset_identity_set_status: String,
    pub(crate) code_asset_identity_set_count: usize,
    pub(crate) code_asset_identity_set_root_hash: String,
    pub(crate) compiled_code_asset_selection: crate::model::CompiledCodeAssetSelectionEvidence,
    pub(crate) request_completions:
        crate::provider_request_completion::ProviderRequestCompletionEvidence,
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
            code_asset_identity_contract:
                crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT.to_owned(),
            code_asset_identity_status: "not-applicable".to_owned(),
            code_asset_identity_asset_id: "none".to_owned(),
            code_asset_identity_hash: "none".to_owned(),
            code_asset_identity_set_contract:
                crate::provider_code_asset_identity::CODE_ASSET_IDENTITY_SET_CONTRACT.to_owned(),
            code_asset_identity_set_status: "not-applicable".to_owned(),
            code_asset_identity_set_count: 0,
            code_asset_identity_set_root_hash: "none".to_owned(),
            compiled_code_asset_selection:
                crate::model::CompiledCodeAssetSelectionEvidence::default(),
            request_completions: Default::default(),
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
    let mut provider_completion_evidence = from_output_payload(output_dir, &output_evidence)?;
    crate::provider_request_completion::bind_final_image_dispatch(
        &mut provider_completion_evidence.request_completions,
        final_image,
        &record.provider_family,
    )?;
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
    let code_asset_identity = validate_code_asset_identity(source)?;
    let code_asset_identity_set = validate_code_asset_identity_set(source, &code_asset_identity.1)?;
    let compiled_code_asset_selection =
        crate::provider_code_asset::contribution::validate_provider_output_selection(source)?;
    let request_completions =
        crate::provider_request_completion::from_output_payload(source, count)?;
    Ok(ProviderCompletionEvidence {
        contract: COMPLETION_EVIDENCE_COLLECTION_CONTRACT.to_owned(),
        status: "verified".to_owned(),
        count,
        clock_evidence: clocks.join("|"),
        completion_tokens: completion_tokens.join(","),
        glm_release_contract: GLM_RELEASE_CONTRACT.to_owned(),
        glm_release_tokens: release_tokens.join(","),
        glm_release_status: "released-at-graph-close".to_owned(),
        code_asset_identity_contract: code_asset_identity.0,
        code_asset_identity_status: code_asset_identity.1,
        code_asset_identity_asset_id: code_asset_identity.2,
        code_asset_identity_hash: code_asset_identity.3,
        code_asset_identity_set_contract: code_asset_identity_set.0,
        code_asset_identity_set_status: code_asset_identity_set.1,
        code_asset_identity_set_count: code_asset_identity_set.2,
        code_asset_identity_set_root_hash: code_asset_identity_set.3,
        compiled_code_asset_selection,
        request_completions,
    })
}

fn validate_code_asset_identity(source: &str) -> Result<(String, String, String, String), String> {
    let default = ProviderCompletionEvidence::default();
    let Some(status) = string_field(source, "provider_code_asset_identity_status") else {
        return Ok((
            default.code_asset_identity_contract,
            default.code_asset_identity_status,
            default.code_asset_identity_asset_id,
            default.code_asset_identity_hash,
        ));
    };
    let contract = string_field(source, "provider_code_asset_identity_contract")
        .ok_or_else(|| "provider completion code-asset identity contract is missing".to_owned())?;
    let asset_id = string_field(source, "provider_code_asset_identity_asset_id")
        .ok_or_else(|| "provider completion code-asset identity asset id is missing".to_owned())?;
    let identity_hash = string_field(source, "provider_code_asset_identity_hash")
        .ok_or_else(|| "provider completion code-asset identity hash is missing".to_owned())?;
    if status == "not-applicable"
        && contract == crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT
        && asset_id == "none"
        && identity_hash == "none"
    {
        return Ok((contract, status, asset_id, identity_hash));
    }
    if status != "verified" || !valid_hash(&identity_hash) {
        return Err("provider completion code-asset identity is invalid".to_owned());
    }
    let contract_valid =
        if contract == crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT {
            asset_id.starts_with("kernel.cuda.project.")
                && asset_id == format!("kernel.cuda.project.{}", &identity_hash[2..])
        } else if contract
            == crate::provider_code_asset_identity::DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT
        {
            valid_identity_token(&asset_id)
        } else {
            false
        };
    if !contract_valid {
        return Err("provider completion code-asset identity is invalid".to_owned());
    }
    Ok((contract, status, asset_id, identity_hash))
}

fn validate_code_asset_identity_set(
    source: &str,
    identity_status: &str,
) -> Result<(String, String, usize, String), String> {
    let default = ProviderCompletionEvidence::default();
    let Some(status) = string_field(source, "provider_code_asset_identity_set_status") else {
        if identity_status == "verified" {
            return Err("provider completion code-asset identity set is missing".to_owned());
        }
        return Ok((
            default.code_asset_identity_set_contract,
            default.code_asset_identity_set_status,
            default.code_asset_identity_set_count,
            default.code_asset_identity_set_root_hash,
        ));
    };
    let contract =
        string_field(source, "provider_code_asset_identity_set_contract").ok_or_else(|| {
            "provider completion code-asset identity set contract is missing".to_owned()
        })?;
    let count = usize_field(source, "provider_code_asset_identity_set_count")
        .ok_or_else(|| "provider completion code-asset identity set count is missing".to_owned())?;
    let root_hash = string_field(source, "provider_code_asset_identity_set_root_hash")
        .ok_or_else(|| "provider completion code-asset identity set root is missing".to_owned())?;
    if status == "not-applicable"
        && identity_status == "not-applicable"
        && contract == crate::provider_code_asset_identity::CODE_ASSET_IDENTITY_SET_CONTRACT
        && count == 0
        && root_hash == "none"
    {
        return Ok((contract, status, count, root_hash));
    }
    if status != "verified"
        || identity_status != "verified"
        || contract != crate::provider_code_asset_identity::CODE_ASSET_IDENTITY_SET_CONTRACT
        || count == 0
        || !valid_hash(&root_hash)
    {
        return Err("provider completion code-asset identity set is invalid".to_owned());
    }
    Ok((contract, status, count, root_hash))
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
        (
            "code_asset_identity_contract",
            event.code_asset_identity_contract.as_str(),
        ),
        (
            "code_asset_identity_status",
            event.code_asset_identity_status.as_str(),
        ),
        (
            "code_asset_identity_asset_id",
            event.code_asset_identity_asset_id.as_str(),
        ),
        (
            "code_asset_identity_hash",
            event.code_asset_identity_hash.as_str(),
        ),
        (
            "code_asset_identity_set_contract",
            event.code_asset_identity_set_contract.as_str(),
        ),
        (
            "code_asset_identity_set_status",
            event.code_asset_identity_set_status.as_str(),
        ),
        (
            "code_asset_identity_set_count",
            &event.code_asset_identity_set_count.to_string(),
        ),
        (
            "code_asset_identity_set_root_hash",
            event.code_asset_identity_set_root_hash.as_str(),
        ),
    ] {
        push_toml_string(out, key, value);
    }
    crate::provider_code_asset::contribution::render_completion_event_fields(
        out,
        &event.compiled_code_asset_selection,
    );
    crate::provider_request_completion::render_fields(out, &event.request_completions);
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
        code_asset_identity_contract: field_or(
            source,
            "code_asset_identity_contract",
            crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        ),
        code_asset_identity_status: field_or(
            source,
            "code_asset_identity_status",
            "not-applicable",
        ),
        code_asset_identity_asset_id: field_or(source, "code_asset_identity_asset_id", "none"),
        code_asset_identity_hash: field_or(source, "code_asset_identity_hash", "none"),
        code_asset_identity_set_contract: field_or(
            source,
            "code_asset_identity_set_contract",
            crate::provider_code_asset_identity::CODE_ASSET_IDENTITY_SET_CONTRACT,
        ),
        code_asset_identity_set_status: field_or(
            source,
            "code_asset_identity_set_status",
            "not-applicable",
        ),
        code_asset_identity_set_count: usize_field(source, "code_asset_identity_set_count")
            .unwrap_or(0),
        code_asset_identity_set_root_hash: field_or(
            source,
            "code_asset_identity_set_root_hash",
            "none",
        ),
        compiled_code_asset_selection:
            crate::provider_code_asset::contribution::parse_completion_event_fields(source),
        request_completions: crate::provider_request_completion::parse_fields(source),
    }
}

pub(crate) fn append_hash_material(material: &mut String, event: &NsdbPayloadExecutionEvent) {
    let evidence = &event.provider_completion_evidence;
    if evidence.contract == COMPLETION_EVIDENCE_COLLECTION_CONTRACT && evidence.status == "verified"
    {
        material.push_str(&format!(
            "\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
            evidence.contract,
            evidence.status,
            evidence.count,
            evidence.clock_evidence,
            evidence.completion_tokens,
            evidence.glm_release_contract,
            evidence.glm_release_tokens,
            evidence.glm_release_status,
            evidence.code_asset_identity_contract,
            evidence.code_asset_identity_status,
            evidence.code_asset_identity_asset_id,
            evidence.code_asset_identity_hash,
            evidence.code_asset_identity_set_contract,
            evidence.code_asset_identity_set_status,
            evidence.code_asset_identity_set_count,
            evidence.code_asset_identity_set_root_hash
        ));
        crate::provider_code_asset::contribution::append_selection_hash_material(
            material,
            &evidence.compiled_code_asset_selection,
        );
        crate::provider_request_completion::append_hash_material(
            material,
            &evidence.request_completions,
        );
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

fn valid_identity_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
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

#[cfg(test)]
mod tests {
    use super::{validate_code_asset_identity, validate_code_asset_identity_set};

    #[test]
    fn validates_project_identity_and_explicit_not_applicable_state() {
        let hash = "0x0123456789abcdef";
        let verified = format!(
            "provider_code_asset_identity_contract = \"nuis-kernel-project-code-asset-identity-v1\"\nprovider_code_asset_identity_status = \"verified\"\nprovider_code_asset_identity_asset_id = \"kernel.cuda.project.{}\"\nprovider_code_asset_identity_hash = \"{hash}\"\n",
            &hash[2..]
        );
        assert!(validate_code_asset_identity(&verified).is_ok());
        assert!(validate_code_asset_identity(
            "provider_code_asset_identity_contract = \"nuis-kernel-project-code-asset-identity-v1\"\nprovider_code_asset_identity_status = \"not-applicable\"\nprovider_code_asset_identity_asset_id = \"none\"\nprovider_code_asset_identity_hash = \"none\"\n"
        )
        .is_ok());
        assert!(validate_code_asset_identity(
            &verified.replace("kernel.cuda.project.0123", "kernel.cuda.project.ffff")
        )
        .is_err());
        let descriptor = "provider_code_asset_identity_contract = \"nuis-provider-code-asset-descriptor-identity-v1\"\nprovider_code_asset_identity_status = \"verified\"\nprovider_code_asset_identity_asset_id = \"shader.witsage.vector-bias.metal\"\nprovider_code_asset_identity_hash = \"0x0123456789abcdef\"\n";
        assert!(validate_code_asset_identity(descriptor).is_ok());
        assert!(validate_code_asset_identity(
            &descriptor.replace("shader.witsage.vector-bias.metal", "../bad")
        )
        .is_err());
        let verified_set = "provider_code_asset_identity_set_contract = \"nuis-provider-code-asset-identity-set-v1\"\nprovider_code_asset_identity_set_status = \"verified\"\nprovider_code_asset_identity_set_count = \"1\"\nprovider_code_asset_identity_set_root_hash = \"0x0123456789abcdef\"\n";
        assert!(validate_code_asset_identity_set(verified_set, "verified").is_ok());
        assert!(validate_code_asset_identity_set("", "verified").is_err());
        assert!(validate_code_asset_identity_set(
            "provider_code_asset_identity_set_contract = \"nuis-provider-code-asset-identity-set-v1\"\nprovider_code_asset_identity_set_status = \"not-applicable\"\nprovider_code_asset_identity_set_count = \"0\"\nprovider_code_asset_identity_set_root_hash = \"none\"\n",
            "not-applicable"
        )
        .is_ok());
    }
}

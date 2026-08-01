#[cfg(test)]
use crate::model::NsdbDeviceProviderSampleRecordInfo;
use std::{collections::BTreeSet, fs, path::Path};

const LAUNCHER_FILE_NAME: &str = "nuis.nsld.final-executable-launcher.toml";
const IMAGE_MAGIC: &[u8; 8] = b"NUIFIMG\0";
const IMAGE_HEADER_SIZE: usize = 64;
const CONTAINER_SCHEMA: &str = "nuis-nsld-container-v1";
const CONTAINER_END_MARKER: &str = "\n# nuis-nsld-container-end-v1\n";
pub(crate) const DISPATCH_CONTRACT: &str = "nuis-final-image-provider-dispatch-v1";
const DISPATCH_BINDING_ID: &str = "runtime.provider-dispatch-table";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";
const SELECTED_SET_BINDING_ID: &str = "identity.selected-provider-bundle-set";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalImageProviderDispatch {
    pub(crate) dispatch_id: String,
    pub(crate) package_id: String,
    pub(crate) bundle_id: String,
    pub(crate) provider_family: String,
    pub(crate) runner_contract: String,
    pub(crate) runner_adapter_contract: String,
    pub(crate) runner_adapter_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalImageProviderDispatchAuthority {
    pub(crate) available: bool,
    pub(crate) status: String,
    pub(crate) image_path: Option<String>,
    pub(crate) table_hash: Option<String>,
    pub(crate) selected_set_hash: Option<String>,
    pub(crate) entries: Vec<FinalImageProviderDispatch>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MetadataBinding {
    binding_id: String,
    contract: String,
    value_count: usize,
    value_hash: String,
    validation_status: String,
    required: bool,
}

pub(crate) fn final_image_provider_dispatch_authority(
    output_dir: &Path,
) -> FinalImageProviderDispatchAuthority {
    let launcher_path = output_dir.join(LAUNCHER_FILE_NAME);
    if !launcher_path.exists() {
        return pre_seal_authority();
    }
    let launcher = match fs::read_to_string(&launcher_path) {
        Ok(source) => source,
        Err(_) => return blocked_authority(None, "final-image-dispatch:launcher-unreadable"),
    };
    if bool_value(&launcher, "ready") != Some(true) {
        return pre_seal_authority();
    }
    let mut blockers = Vec::new();
    let Some(nsb_path_text) = string_value(&launcher, "nsb_path") else {
        return blocked_authority(None, "final-image-dispatch:nsb-path-missing");
    };
    let nsb_path = if Path::new(&nsb_path_text).is_absolute() {
        Path::new(&nsb_path_text).to_path_buf()
    } else {
        launcher_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&nsb_path_text)
    };
    let image_path = Some(nsb_path.display().to_string());
    let bytes = match fs::read(&nsb_path) {
        Ok(bytes) => bytes,
        Err(_) => return blocked_authority(image_path, "final-image-dispatch:nsb-unreadable"),
    };
    if string_value(&launcher, "nsb_hash").as_deref() != Some(fnv1a64_hex(&bytes).as_str()) {
        blockers.push("final-image-dispatch:nsb-hash-mismatch".to_owned());
    }
    let source = match final_image_container_source(&bytes) {
        Ok(source) => source,
        Err(blocker) => return blocked_authority(image_path, blocker),
    };
    if string_value(source, "schema").as_deref() != Some(CONTAINER_SCHEMA) {
        blockers.push("final-image-dispatch:container-schema-mismatch".to_owned());
    }

    let entries = parse_dispatch_entries(source);
    let actual_table_hash = dispatch_table_hash(&entries);
    let actual_selected_set_hash = selected_set_hash(&entries);
    if string_value(source, "provider_dispatch_contract").as_deref() != Some(DISPATCH_CONTRACT) {
        blockers.push("final-image-dispatch:contract-mismatch".to_owned());
    }
    if string_value(source, "provider_dispatch_validation_status").as_deref() != Some("verified") {
        blockers.push("final-image-dispatch:status-unverified".to_owned());
    }
    if usize_value(source, "provider_dispatch_count") != Some(entries.len()) {
        blockers.push("final-image-dispatch:count-mismatch".to_owned());
    }
    if string_value(source, "provider_dispatch_table_hash").as_deref()
        != Some(actual_table_hash.as_str())
    {
        blockers.push("final-image-dispatch:table-hash-mismatch".to_owned());
    }
    validate_bindings(
        source,
        entries.len(),
        &actual_table_hash,
        &actual_selected_set_hash,
        &mut blockers,
    );
    let mut bundle_ids = BTreeSet::new();
    for entry in &entries {
        if entry_incomplete(entry) || !bundle_ids.insert(entry.bundle_id.as_str()) {
            blockers.push(format!(
                "final-image-dispatch:entry-invalid:{}",
                entry.dispatch_id
            ));
        }
    }
    FinalImageProviderDispatchAuthority {
        available: true,
        status: if blockers.is_empty() {
            "verified"
        } else {
            "mismatch"
        }
        .to_owned(),
        image_path,
        table_hash: Some(actual_table_hash),
        selected_set_hash: Some(actual_selected_set_hash),
        entries,
        blockers,
    }
}

#[cfg(test)]
pub(crate) fn validate_provider_records_against_final_image(
    authority: &FinalImageProviderDispatchAuthority,
    records: &[&NsdbDeviceProviderSampleRecordInfo],
) -> Result<usize, String> {
    if !authority.available {
        return Ok(0);
    }
    if !authority.blockers.is_empty() {
        return Err(authority.blockers.join(", "));
    }
    let mut matched = 0usize;
    for record in records {
        let entry = authority.entries.iter().find(|entry| {
            entry.package_id == record.provider_bundle_package_id
                && entry.bundle_id == record.provider_bundle_id
                && entry.provider_family == record.provider_family
        });
        let Some(entry) = entry else {
            return Err(format!(
                "final-image-dispatch:sidecar-entry-missing:{}:{}:{}",
                record.provider_bundle_package_id,
                record.provider_bundle_id,
                record.provider_family
            ));
        };
        if entry.runner_contract != record.provider_runner_contract
            || entry.runner_adapter_contract != record.provider_runner_adapter_contract
            || entry.runner_adapter_id != record.provider_runner_adapter_id
        {
            return Err(format!(
                "final-image-dispatch:sidecar-runner-drift:{}",
                entry.dispatch_id
            ));
        }
        matched += 1;
    }
    Ok(matched)
}

pub(crate) fn validate_provider_families_against_final_image(
    authority: &FinalImageProviderDispatchAuthority,
    provider_families: &[String],
) -> Result<usize, String> {
    if !authority.available {
        return Ok(0);
    }
    if !authority.blockers.is_empty() {
        return Err(authority.blockers.join(", "));
    }
    let mut matched = 0usize;
    for family in provider_families {
        let bundle = crate::provider_bundle_registry::provider_bundle_evidence(family)
            .ok_or_else(|| format!("final-image-dispatch:bundle-unregistered:{family}"))?;
        let adapter = crate::provider_runner_registry::select_provider_runner_adapter(family);
        let entry = authority.entries.iter().find(|entry| {
            entry.package_id == bundle.package_id
                && entry.bundle_id == bundle.bundle_id
                && entry.provider_family == *family
        });
        let Some(entry) = entry else {
            return Err(format!(
                "final-image-dispatch:request-family-entry-missing:{}:{}:{family}",
                bundle.package_id, bundle.bundle_id
            ));
        };
        if entry.runner_contract != "nuis-provider-runner-v1"
            || entry.runner_adapter_contract != "nuis-provider-runner-adapter-v1"
            || entry.runner_adapter_id != adapter.adapter_id
        {
            return Err(format!(
                "final-image-dispatch:request-family-runner-drift:{}",
                entry.dispatch_id
            ));
        }
        matched += 1;
    }
    Ok(matched)
}

fn pre_seal_authority() -> FinalImageProviderDispatchAuthority {
    FinalImageProviderDispatchAuthority {
        available: false,
        status: "pre-seal-acquisition".to_owned(),
        image_path: None,
        table_hash: None,
        selected_set_hash: None,
        entries: Vec::new(),
        blockers: Vec::new(),
    }
}

fn blocked_authority(
    image_path: Option<String>,
    blocker: &str,
) -> FinalImageProviderDispatchAuthority {
    FinalImageProviderDispatchAuthority {
        available: true,
        status: "mismatch".to_owned(),
        image_path,
        table_hash: None,
        selected_set_hash: None,
        entries: Vec::new(),
        blockers: vec![blocker.to_owned()],
    }
}

fn final_image_container_source(bytes: &[u8]) -> Result<&str, &'static str> {
    if bytes.len() < IMAGE_HEADER_SIZE || bytes.get(..8) != Some(IMAGE_MAGIC) {
        return Err("final-image-dispatch:image-header-invalid");
    }
    let payload_span = read_u64(bytes, 24).ok_or("final-image-dispatch:image-header-invalid")?;
    let payload_offset = read_u64(bytes, 32).ok_or("final-image-dispatch:image-header-invalid")?;
    let payload_end = payload_offset
        .checked_add(payload_span)
        .ok_or("final-image-dispatch:payload-range-invalid")?;
    let payload = bytes
        .get(payload_offset..payload_end)
        .ok_or("final-image-dispatch:payload-range-invalid")?;
    let schema_marker = b"schema = \"nuis-nsld-container-v1\"";
    let schema_offset =
        find_bytes(payload, schema_marker).ok_or("final-image-dispatch:container-missing")?;
    let capsule = &payload[schema_offset..];
    let end = find_bytes(capsule, CONTAINER_END_MARKER.as_bytes())
        .map(|offset| offset + CONTAINER_END_MARKER.len())
        .ok_or("final-image-dispatch:container-end-missing")?;
    std::str::from_utf8(&capsule[..end]).map_err(|_| "final-image-dispatch:container-invalid-utf8")
}

fn validate_bindings(
    source: &str,
    count: usize,
    table_hash: &str,
    selected_set_hash: &str,
    blockers: &mut Vec<String>,
) {
    let bindings = parse_metadata_bindings(source);
    if usize_value(source, "metadata_binding_count") != Some(bindings.len()) {
        blockers.push("final-image-dispatch:binding-count-mismatch".to_owned());
    }
    if string_value(source, "metadata_binding_table_hash").as_deref()
        != Some(metadata_binding_table_hash(&bindings).as_str())
    {
        blockers.push("final-image-dispatch:binding-table-hash-mismatch".to_owned());
    }
    validate_binding(
        &bindings,
        SELECTED_SET_BINDING_ID,
        SELECTED_SET_CONTRACT,
        count,
        selected_set_hash,
        blockers,
    );
    validate_binding(
        &bindings,
        DISPATCH_BINDING_ID,
        DISPATCH_CONTRACT,
        count,
        table_hash,
        blockers,
    );
}

fn validate_binding(
    bindings: &[MetadataBinding],
    binding_id: &str,
    contract: &str,
    count: usize,
    hash: &str,
    blockers: &mut Vec<String>,
) {
    let valid = bindings.iter().any(|binding| {
        binding.binding_id == binding_id
            && binding.contract == contract
            && binding.value_count == count
            && binding.value_hash == hash
            && binding.validation_status == "verified"
            && binding.required
    });
    if !valid {
        blockers.push(format!(
            "final-image-dispatch:binding-mismatch:{binding_id}"
        ));
    }
}

fn parse_dispatch_entries(source: &str) -> Vec<FinalImageProviderDispatch> {
    table_blocks(source, "provider_dispatch")
        .into_iter()
        .filter_map(|block| {
            Some(FinalImageProviderDispatch {
                dispatch_id: string_value(block, "dispatch_id")?,
                package_id: string_value(block, "provider_bundle_package_id")?,
                bundle_id: string_value(block, "provider_bundle_id")?,
                provider_family: string_value(block, "provider_family")?,
                runner_contract: string_value(block, "runner_contract")?,
                runner_adapter_contract: string_value(block, "runner_adapter_contract")?,
                runner_adapter_id: string_value(block, "runner_adapter_id")?,
            })
        })
        .collect()
}

fn parse_metadata_bindings(source: &str) -> Vec<MetadataBinding> {
    table_blocks(source, "metadata_binding")
        .into_iter()
        .filter_map(|block| {
            Some(MetadataBinding {
                binding_id: string_value(block, "binding_id")?,
                contract: string_value(block, "contract")?,
                value_count: usize_value(block, "value_count")?,
                value_hash: string_value(block, "value_hash")?,
                validation_status: string_value(block, "validation_status")?,
                required: bool_value(block, "required")?,
            })
        })
        .collect()
}

fn table_blocks<'a>(source: &'a str, table: &str) -> Vec<&'a str> {
    let header = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut start = None;
    for (offset, _) in source.match_indices('\n') {
        let line_start = offset + 1;
        let tail = &source[line_start..];
        let line_end = tail.find('\n').unwrap_or(tail.len());
        let current = tail[..line_end].trim();
        if current.starts_with("[[") && current.ends_with("]]") {
            if let Some(block_start) = start.take() {
                blocks.push(&source[block_start..offset]);
            }
            if current == header {
                start = Some(line_start + line_end + 1);
            }
        }
    }
    if let Some(block_start) = start {
        blocks.push(&source[block_start..]);
    }
    blocks
}

fn dispatch_table_hash(entries: &[FinalImageProviderDispatch]) -> String {
    let mut canonical = format!("{DISPATCH_CONTRACT}\n");
    for entry in entries {
        canonical.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}\n",
            entry.dispatch_id,
            entry.package_id,
            entry.bundle_id,
            entry.provider_family,
            entry.runner_contract,
            entry.runner_adapter_contract,
            entry.runner_adapter_id
        ));
    }
    fnv1a64_hex(canonical.as_bytes())
}

fn selected_set_hash(entries: &[FinalImageProviderDispatch]) -> String {
    let mut canonical = format!("{SELECTED_SET_CONTRACT}\n");
    for (index, entry) in entries.iter().enumerate() {
        canonical.push_str(&format!(
            "{index}|{}|{}|{}\n",
            entry.package_id, entry.bundle_id, entry.provider_family
        ));
    }
    format!(
        "fnv1a64:{}",
        fnv1a64_hex(canonical.as_bytes()).trim_start_matches("0x")
    )
}

fn metadata_binding_table_hash(bindings: &[MetadataBinding]) -> String {
    let mut canonical = String::new();
    for binding in bindings {
        canonical.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            binding.binding_id,
            binding.contract,
            binding.value_count,
            binding.value_hash,
            binding.validation_status,
            binding.required
        ));
    }
    fnv1a64_hex(canonical.as_bytes())
}

fn entry_incomplete(entry: &FinalImageProviderDispatch) -> bool {
    [
        entry.dispatch_id.as_str(),
        entry.package_id.as_str(),
        entry.bundle_id.as_str(),
        entry.provider_family.as_str(),
        entry.runner_contract.as_str(),
        entry.runner_adapter_contract.as_str(),
        entry.runner_adapter_id.as_str(),
    ]
    .into_iter()
    .any(|value| value.is_empty() || value == "none")
}

fn string_value(source: &str, key: &str) -> Option<String> {
    let value = field_value(source, key)?;
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| value.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn usize_value(source: &str, key: &str) -> Option<usize> {
    field_value(source, key)?.parse().ok()
}

fn bool_value(source: &str, key: &str) -> Option<bool> {
    match field_value(source, key)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn field_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<usize> {
    let chunk: [u8; 8] = bytes.get(offset..offset + 8)?.try_into().ok()?;
    usize::try_from(u64::from_le_bytes(chunk)).ok()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
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
    use std::env;

    fn entry(adapter_id: &str) -> FinalImageProviderDispatch {
        FinalImageProviderDispatch {
            dispatch_id: "dispatch0000".to_owned(),
            package_id: "official.data".to_owned(),
            bundle_id: "data.host.bundle.v1".to_owned(),
            provider_family: "data:host".to_owned(),
            runner_contract: "nuis-provider-runner-v1".to_owned(),
            runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
            runner_adapter_id: adapter_id.to_owned(),
        }
    }

    fn container_source(adapter_id: &str, declared_adapter_id: &str) -> String {
        let declared_entries = vec![entry(declared_adapter_id)];
        let table_hash = dispatch_table_hash(&declared_entries);
        let selected_hash = selected_set_hash(&declared_entries);
        let bindings = vec![
            MetadataBinding {
                binding_id: SELECTED_SET_BINDING_ID.to_owned(),
                contract: SELECTED_SET_CONTRACT.to_owned(),
                value_count: 1,
                value_hash: selected_hash.clone(),
                validation_status: "verified".to_owned(),
                required: true,
            },
            MetadataBinding {
                binding_id: DISPATCH_BINDING_ID.to_owned(),
                contract: DISPATCH_CONTRACT.to_owned(),
                value_count: 1,
                value_hash: table_hash.clone(),
                validation_status: "verified".to_owned(),
                required: true,
            },
        ];
        let binding_hash = metadata_binding_table_hash(&bindings);
        format!(
            "schema = \"{CONTAINER_SCHEMA}\"\n\
             metadata_binding_count = 2\n\
             metadata_binding_table_hash = \"{binding_hash}\"\n\
             provider_dispatch_contract = \"{DISPATCH_CONTRACT}\"\n\
             provider_dispatch_validation_status = \"verified\"\n\
             provider_dispatch_count = 1\n\
             provider_dispatch_table_hash = \"{table_hash}\"\n\
             [[provider_dispatch]]\n\
             dispatch_id = \"dispatch0000\"\n\
             provider_bundle_package_id = \"official.data\"\n\
             provider_bundle_id = \"data.host.bundle.v1\"\n\
             provider_family = \"data:host\"\n\
             runner_contract = \"nuis-provider-runner-v1\"\n\
             runner_adapter_contract = \"nuis-provider-runner-adapter-v1\"\n\
             runner_adapter_id = \"{adapter_id}\"\n\
             [[metadata_binding]]\n\
             binding_id = \"{SELECTED_SET_BINDING_ID}\"\n\
             contract = \"{SELECTED_SET_CONTRACT}\"\n\
             value_count = 1\n\
             value_hash = \"{selected_hash}\"\n\
             validation_status = \"verified\"\n\
             required = true\n\
             [[metadata_binding]]\n\
             binding_id = \"{DISPATCH_BINDING_ID}\"\n\
             contract = \"{DISPATCH_CONTRACT}\"\n\
             value_count = 1\n\
             value_hash = \"{table_hash}\"\n\
             validation_status = \"verified\"\n\
             required = true\n\
             # nuis-nsld-container-end-v1\n"
        )
    }

    fn write_image(dir: &Path, adapter_id: &str, declared_adapter_id: &str) {
        let payload = container_source(adapter_id, declared_adapter_id).into_bytes();
        let mut bytes = vec![0u8; IMAGE_HEADER_SIZE + payload.len()];
        bytes[..8].copy_from_slice(IMAGE_MAGIC);
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&(IMAGE_HEADER_SIZE as u32).to_le_bytes());
        bytes[24..32].copy_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes[32..40].copy_from_slice(&(IMAGE_HEADER_SIZE as u64).to_le_bytes());
        bytes[IMAGE_HEADER_SIZE..].copy_from_slice(&payload);
        fs::write(dir.join("nuis-app.nsb"), &bytes).unwrap();
        fs::write(
            dir.join(LAUNCHER_FILE_NAME),
            format!(
                "ready = true\nnsb_path = \"nuis-app.nsb\"\nnsb_hash = \"{}\"\n",
                fnv1a64_hex(&bytes)
            ),
        )
        .unwrap();
    }

    fn record(adapter_id: &str) -> NsdbDeviceProviderSampleRecordInfo {
        NsdbDeviceProviderSampleRecordInfo {
            index: 0,
            valid: true,
            trace_id: "trace0".to_owned(),
            provider: "registered".to_owned(),
            provider_family: "data:host".to_owned(),
            provider_bundle_package_id: "official.data".to_owned(),
            provider_bundle_id: "data.host.bundle.v1".to_owned(),
            requested_runner_contract: "nuis-provider-runner-v1".to_owned(),
            requested_runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
            requested_runner_adapter_id: adapter_id.to_owned(),
            requested_runner_adapter_capability_status: "registered-real-device".to_owned(),
            provider_runner_contract: "nuis-provider-runner-v1".to_owned(),
            provider_runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
            provider_runner_adapter_id: adapter_id.to_owned(),
            handoff_target: "device-provider-sample".to_owned(),
            sample_status: "ready".to_owned(),
            validation_status: "verified".to_owned(),
            input_evidence: "input".to_owned(),
            output_evidence: "output".to_owned(),
            provider_output_payload_contract: "none".to_owned(),
            provider_output_payload_status: "none".to_owned(),
            provider_output_payload_evidence_status: "none".to_owned(),
            provider_output_payload_evidence: "none".to_owned(),
            provider_output_payload_detail: "none".to_owned(),
            provider_output_payload_next_action: "none".to_owned(),
            materialization_status: "provider-sample-materialized".to_owned(),
            materialization_detail: "ready".to_owned(),
            next_action: "execute".to_owned(),
            diagnostic: "loaded".to_owned(),
        }
    }

    fn temp_dir(label: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "nsdb-final-image-dispatch-{label}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn independently_loads_verified_dispatch_from_final_image() {
        let dir = temp_dir("verified");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_image(
            &dir,
            "data.host.provider-worker-native",
            "data.host.provider-worker-native",
        );

        let authority = final_image_provider_dispatch_authority(&dir);
        let sample = record("data.host.provider-worker-native");
        let matched =
            validate_provider_records_against_final_image(&authority, &[&sample]).unwrap();

        assert_eq!(authority.status, "verified");
        assert_eq!(authority.entries.len(), 1);
        assert_eq!(matched, 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn unready_launcher_remains_pre_seal_without_reading_missing_image() {
        let dir = temp_dir("unready-launcher");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(LAUNCHER_FILE_NAME),
            "ready = false\nnsb_path = \"missing.nsb\"\nnsb_hash = \"\"\n",
        )
        .unwrap();

        let authority = final_image_provider_dispatch_authority(&dir);

        assert!(!authority.available);
        assert_eq!(authority.status, "pre-seal-acquisition");
        assert!(authority.blockers.is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_sidecar_runner_drift_before_execution() {
        let dir = temp_dir("sidecar-drift");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_image(
            &dir,
            "data.host.provider-worker-native",
            "data.host.provider-worker-native",
        );

        let authority = final_image_provider_dispatch_authority(&dir);
        let sample = record("data.host.provider-worker-driftt");
        let error =
            validate_provider_records_against_final_image(&authority, &[&sample]).unwrap_err();

        assert!(error.contains("sidecar-runner-drift"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_final_image_adapter_hash_drift() {
        let dir = temp_dir("image-drift");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_image(
            &dir,
            "data.host.provider-worker-driftt",
            "data.host.provider-worker-native",
        );

        let authority = final_image_provider_dispatch_authority(&dir);

        assert_eq!(authority.status, "mismatch");
        assert!(authority
            .blockers
            .contains(&"final-image-dispatch:table-hash-mismatch".to_owned()));
        fs::remove_dir_all(dir).unwrap();
    }
}

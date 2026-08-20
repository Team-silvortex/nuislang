use std::fs;

use crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY;
use crate::registry::{
    HostFfiMemoryCapability, HostFfiMemoryDestructor, HostFfiMemoryKind, HostFfiMemorySlot,
};
use crate::registry_host_ffi::parse_memory_capability;
use yir_core::ffi::{
    ffi_symbol_signature_hash, OWNED_BUFFER_DESTRUCTOR_SIGNATURE,
    OWNED_OBJECT_DESTRUCTOR_SIGNATURE, OWNED_UTF8_DESTRUCTOR_SIGNATURE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostFfiIndexFootprint {
    pub(crate) index_path: Option<String>,
    pub(crate) symbol_count: usize,
    pub(crate) policy_count: usize,
    pub(crate) memory_capability_count: usize,
    pub(crate) policy: String,
    pub(crate) entries: Vec<HostFfiIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostFfiIndexEntry {
    pub(crate) abi: String,
    pub(crate) symbol: String,
    pub(crate) signature_pattern: String,
    pub(crate) signature_hash: String,
    pub(crate) policy: String,
    pub(crate) memory_capabilities: Vec<HostFfiMemoryCapability>,
}

pub(crate) fn host_ffi_index_footprint(index_path: Option<&str>) -> HostFfiIndexFootprint {
    let entries = host_ffi_entries_from_index(index_path);
    HostFfiIndexFootprint {
        index_path: index_path.map(str::to_owned),
        symbol_count: entries.len(),
        policy_count: entries
            .iter()
            .filter(|entry| !entry.policy.is_empty())
            .count(),
        memory_capability_count: entries
            .iter()
            .map(|entry| entry.memory_capabilities.len())
            .sum(),
        policy: SIGNATURE_WHITELIST_POLICY.to_owned(),
        entries,
    }
}

pub(crate) fn host_ffi_symbol_count_from_index(index_path: Option<&str>) -> usize {
    host_ffi_entries_from_index(index_path).len()
}

pub(crate) fn host_ffi_policy_count_from_index(index_path: Option<&str>) -> usize {
    host_ffi_entries_from_index(index_path)
        .iter()
        .filter(|entry| !entry.policy.is_empty())
        .count()
}

pub(crate) fn verify_host_ffi_index_source(index_path: &str, source: &str) -> Result<(), String> {
    parse_host_ffi_index_source(index_path, source)?;
    Ok(())
}

pub(crate) fn parse_host_ffi_index_source(
    index_path: &str,
    source: &str,
) -> Result<Vec<HostFfiIndexEntry>, String> {
    let mut entries = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        entries.push(parse_host_ffi_index_line(index_path, line_index + 1, line)?);
    }
    validate_host_ffi_memory_links(index_path, &entries)?;
    Ok(entries)
}

fn parse_host_ffi_index_line(
    index_path: &str,
    line_number: usize,
    line: &str,
) -> Result<HostFfiIndexEntry, String> {
    let abi = required_tab_field(index_path, line_number, line, "abi")?;
    let symbol = required_tab_field(index_path, line_number, line, "symbol")?;
    let signature_pattern = required_tab_field(index_path, line_number, line, "signature_pattern")?;
    let signature_hash = required_tab_field(index_path, line_number, line, "signature_hash")?;
    let policy = required_tab_field(index_path, line_number, line, "policy")?;
    let memory_capability_count = required_tab_field(
        index_path,
        line_number,
        line,
        "memory_capability_count",
    )?
    .parse::<usize>()
    .map_err(|_| {
        format!(
            "project host_ffi index `{index_path}` line {line_number} has invalid `memory_capability_count`"
        )
    })?;
    let memory_capabilities_raw =
        required_tab_field(index_path, line_number, line, "memory_capabilities")?;
    if policy != SIGNATURE_WHITELIST_POLICY {
        return Err(format!(
            "project host_ffi index `{index_path}` line {line_number} has unsupported policy `{policy}`; expected `{SIGNATURE_WHITELIST_POLICY}`"
        ));
    }
    let expected_hash = ffi_symbol_signature_hash(abi, symbol, signature_pattern);
    if signature_hash != expected_hash {
        return Err(format!(
            "project host_ffi index `{index_path}` line {line_number} signature hash mismatch for `{symbol}` ABI `{abi}` signature `{signature_pattern}`: expected `{expected_hash}`, found `{signature_hash}`"
        ));
    }
    let memory_capabilities = parse_index_memory_capabilities(
        index_path,
        line_number,
        abi,
        symbol,
        signature_pattern,
        signature_hash,
        memory_capability_count,
        memory_capabilities_raw,
    )?;
    Ok(HostFfiIndexEntry {
        abi: abi.to_owned(),
        symbol: symbol.to_owned(),
        signature_pattern: signature_pattern.to_owned(),
        signature_hash: signature_hash.to_owned(),
        policy: policy.to_owned(),
        memory_capabilities,
    })
}

#[allow(clippy::too_many_arguments)]
fn parse_index_memory_capabilities(
    index_path: &str,
    line_number: usize,
    abi: &str,
    symbol: &str,
    signature_pattern: &str,
    signature_hash: &str,
    expected_count: usize,
    raw: &str,
) -> Result<Vec<HostFfiMemoryCapability>, String> {
    let capabilities = if raw == "-" {
        Vec::new()
    } else {
        raw.split(';')
            .map(|entry| parse_memory_capability(entry, index_path))
            .collect::<Result<Vec<_>, _>>()?
    };
    if capabilities.len() != expected_count {
        return Err(format!(
            "project host_ffi index `{index_path}` line {line_number} memory capability count mismatch: expected {expected_count}, found {}",
            capabilities.len()
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for capability in &capabilities {
        if capability.abi != abi
            || capability.symbol != symbol
            || capability.signature_hash != signature_hash
        {
            return Err(format!(
                "project host_ffi index `{index_path}` line {line_number} memory capability identity does not match ABI `{abi}` symbol `{symbol}` signature hash `{signature_hash}`"
            ));
        }
        if !seen.insert((capability.kind, capability.slot.clone())) {
            return Err(format!(
                "project host_ffi index `{index_path}` line {line_number} repeats memory capability `{}` slot `{}`",
                capability.kind.as_str(),
                capability.slot.render()
            ));
        }
        validate_index_memory_shape(index_path, line_number, capability, signature_pattern)?;
    }
    Ok(capabilities)
}

fn validate_index_memory_shape(
    index_path: &str,
    line_number: usize,
    capability: &HostFfiMemoryCapability,
    signature: &str,
) -> Result<(), String> {
    match (&capability.kind, &capability.slot) {
        (HostFfiMemoryKind::BorrowedUtf8, HostFfiMemorySlot::Arg(index)) => {
            let args = signature
                .split_once('(')
                .and_then(|(_, args)| args.strip_suffix(')'))
                .map(|args| {
                    if args.is_empty() {
                        Vec::new()
                    } else {
                        args.split(',').collect::<Vec<_>>()
                    }
                })
                .unwrap_or_default();
            if args.get(*index).copied() != Some("String") {
                return Err(format!(
                    "project host_ffi index `{index_path}` line {line_number} borrowed UTF-8 slot `arg:{index}` does not reference `String` in `{signature}`"
                ));
            }
        }
        (HostFfiMemoryKind::OwnedReturnBuffer, HostFfiMemorySlot::Return) => {
            if signature.split_once('(').map(|(ret, _)| ret) != Some("ref_Buffer") {
                return Err(format!(
                    "project host_ffi index `{index_path}` line {line_number} owned return-buffer capability requires `ref_Buffer` return signature"
                ));
            }
        }
        (HostFfiMemoryKind::OwnedReturnUtf8, HostFfiMemorySlot::Return) => {
            if signature.split_once('(').map(|(ret, _)| ret) != Some("ref_String") {
                return Err(format!(
                    "project host_ffi index `{index_path}` line {line_number} owned UTF-8 capability requires `ref_String` return signature"
                ));
            }
        }
        (HostFfiMemoryKind::OwnedReturnObject, HostFfiMemorySlot::Return) => {
            if signature.split_once('(').map(|(ret, _)| ret) != Some("ref_FfiObject") {
                return Err(format!(
                    "project host_ffi index `{index_path}` line {line_number} owned object capability requires `ref_FfiObject` return signature"
                ));
            }
        }
        _ => {
            return Err(format!(
                "project host_ffi index `{index_path}` line {line_number} memory capability kind/slot mismatch"
            ))
        }
    }
    Ok(())
}

fn validate_host_ffi_memory_links(
    index_path: &str,
    entries: &[HostFfiIndexEntry],
) -> Result<(), String> {
    for entry in entries {
        for capability in &entry.memory_capabilities {
            let HostFfiMemoryDestructor::Registered {
                symbol,
                signature_hash,
            } = &capability.destructor
            else {
                continue;
            };
            let expected_signature = match capability.kind {
                HostFfiMemoryKind::OwnedReturnBuffer => OWNED_BUFFER_DESTRUCTOR_SIGNATURE,
                HostFfiMemoryKind::OwnedReturnUtf8 => OWNED_UTF8_DESTRUCTOR_SIGNATURE,
                HostFfiMemoryKind::OwnedReturnObject => OWNED_OBJECT_DESTRUCTOR_SIGNATURE,
                HostFfiMemoryKind::BorrowedUtf8 => {
                    return Err(format!(
                        "project host_ffi index `{index_path}` borrowed UTF-8 capability for `{}` cannot own destructor authority",
                        capability.symbol
                    ));
                }
            };
            let valid = entries.iter().any(|candidate| {
                candidate.abi == capability.abi
                    && candidate.symbol == *symbol
                    && candidate.signature_hash == *signature_hash
                    && candidate.signature_pattern == expected_signature
            });
            if !valid {
                return Err(format!(
                    "project host_ffi index `{index_path}` owned capability `{}` for `{}` references missing or drifted destructor `{symbol}` signature `{expected_signature}` hash `{signature_hash}`",
                    capability.kind.as_str(), capability.symbol
                ));
            }
        }
    }
    Ok(())
}

fn required_tab_field<'a>(
    index_path: &str,
    line_number: usize,
    line: &'a str,
    key: &str,
) -> Result<&'a str, String> {
    let prefix = format!("{key}=");
    line.split('\t')
        .find_map(|field| field.strip_prefix(&prefix))
        .ok_or_else(|| {
            format!(
                "project host_ffi index `{index_path}` line {line_number} is missing `{key}` field"
            )
        })
}

fn host_ffi_index_source(index_path: Option<&str>) -> Option<String> {
    index_path.and_then(|path| fs::read_to_string(path).ok())
}

fn host_ffi_entries_from_index(index_path: Option<&str>) -> Vec<HostFfiIndexEntry> {
    host_ffi_index_source(index_path)
        .and_then(|source| parse_host_ffi_index_source("<host_ffi_index>", &source).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_line() -> String {
        let hash = ffi_symbol_signature_hash("c", "host_sleep_ns", "i64(i64)");
        format!(
            "abi=c\tsymbol=host_sleep_ns\tsignature_pattern=i64(i64)\tsignature_hash={hash}\tpolicy={SIGNATURE_WHITELIST_POLICY}\tmemory_capability_count=0\tmemory_capabilities=-"
        )
    }

    #[test]
    fn verifies_valid_host_ffi_index_source() {
        let source = format!("{}\n\n", valid_line());

        verify_host_ffi_index_source("nuis.project.host_ffi.txt", &source).unwrap();
        let entries = parse_host_ffi_index_source("nuis.project.host_ffi.txt", &source).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].abi, "c");
        assert_eq!(entries[0].symbol, "host_sleep_ns");
        assert_eq!(entries[0].signature_pattern, "i64(i64)");
        assert_eq!(entries[0].policy, SIGNATURE_WHITELIST_POLICY);
        assert!(entries[0].memory_capabilities.is_empty());
    }

    #[test]
    fn counts_host_ffi_index_footprint_from_parsed_entries() {
        let dir = std::env::temp_dir().join(format!(
            "nuisc_host_ffi_index_footprint_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("nuis.project.host_ffi.txt");
        fs::write(&path, format!("{}\n", valid_line())).unwrap();

        let path_text = path.display().to_string();
        let footprint = host_ffi_index_footprint(Some(&path_text));

        assert_eq!(footprint.index_path.as_deref(), Some(path_text.as_str()));
        assert_eq!(footprint.symbol_count, 1);
        assert_eq!(footprint.policy_count, 1);
        assert_eq!(footprint.policy, SIGNATURE_WHITELIST_POLICY);
    }

    #[test]
    fn rejects_host_ffi_index_missing_required_field() {
        let source = valid_line().replace("\tpolicy=signature-whitelist-required", "");

        let error = verify_host_ffi_index_source("nuis.project.host_ffi.txt", &source).unwrap_err();

        assert!(error.contains("missing `policy` field"));
    }

    #[test]
    fn rejects_host_ffi_index_unsupported_policy() {
        let source = valid_line().replace(SIGNATURE_WHITELIST_POLICY, "unchecked");

        let error = verify_host_ffi_index_source("nuis.project.host_ffi.txt", &source).unwrap_err();

        assert!(error.contains("unsupported policy `unchecked`"));
    }

    #[test]
    fn rejects_host_ffi_index_signature_hash_mismatch() {
        let source = valid_line().replace("signature_hash=fnv1a64:", "signature_hash=fnv1a64:0");

        let error = verify_host_ffi_index_source("nuis.project.host_ffi.txt", &source).unwrap_err();

        assert!(error.contains("signature hash mismatch"));
    }
}

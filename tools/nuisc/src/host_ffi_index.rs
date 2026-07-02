use std::fs;

use crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY;
use yir_core::ffi::ffi_symbol_signature_hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostFfiIndexFootprint {
    pub(crate) index_path: Option<String>,
    pub(crate) symbol_count: usize,
    pub(crate) policy_count: usize,
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
    Ok(HostFfiIndexEntry {
        abi: abi.to_owned(),
        symbol: symbol.to_owned(),
        signature_pattern: signature_pattern.to_owned(),
        signature_hash: signature_hash.to_owned(),
        policy: policy.to_owned(),
    })
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
            "abi=c\tsymbol=host_sleep_ns\tsignature_pattern=i64(i64)\tsignature_hash={hash}\tpolicy={SIGNATURE_WHITELIST_POLICY}"
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

use std::collections::{BTreeMap, BTreeSet};

use crate::host_ffi_index::host_ffi_index_footprint;

use super::{
    LinkPlanHostFfiAbiEntry, LinkPlanHostFfiAbiGroup, LinkPlanHostFfiEntry,
    LinkPlanHostFfiFootprint, LinkPlanHostFfiValidationSummary,
};

pub(super) fn build_host_ffi_footprint(index_path: Option<&str>) -> LinkPlanHostFfiFootprint {
    let footprint = host_ffi_index_footprint(index_path);
    let entries = footprint
        .entries
        .into_iter()
        .map(|entry| LinkPlanHostFfiEntry {
            abi: entry.abi,
            symbol: entry.symbol,
            signature_pattern: entry.signature_pattern,
            signature_hash: entry.signature_hash,
            policy: entry.policy,
        })
        .collect::<Vec<_>>();
    let validation = validate_host_ffi_footprint(
        footprint.symbol_count,
        footprint.policy_count,
        &footprint.policy,
        &entries,
    );
    let abi_groups = derive_host_ffi_abi_groups(&entries);
    LinkPlanHostFfiFootprint {
        index_path: footprint.index_path,
        symbol_count: footprint.symbol_count,
        policy_count: footprint.policy_count,
        policy: footprint.policy,
        abi_groups,
        validation,
        entries,
    }
}

pub(super) fn validate_host_ffi_footprint(
    symbol_count: usize,
    policy_count: usize,
    policy: &str,
    entries: &[LinkPlanHostFfiEntry],
) -> LinkPlanHostFfiValidationSummary {
    let mut issues = Vec::new();
    let mut notes = Vec::new();
    if symbol_count != entries.len() {
        issues.push(format!(
            "host_ffi symbol_count {symbol_count} does not match parsed entries {}",
            entries.len()
        ));
    }
    if policy_count != entries.len() {
        issues.push(format!(
            "host_ffi policy_count {policy_count} does not match parsed entries {}",
            entries.len()
        ));
    }
    let mut seen_signatures = BTreeSet::new();
    let mut signatures_by_symbol: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    for entry in entries {
        let key = (
            entry.abi.as_str(),
            entry.symbol.as_str(),
            entry.signature_pattern.as_str(),
        );
        signatures_by_symbol
            .entry((entry.abi.as_str(), entry.symbol.as_str()))
            .or_default()
            .insert(entry.signature_pattern.as_str());
        if !seen_signatures.insert(key) {
            issues.push(format!(
                "host_ffi duplicate whitelist entry for ABI `{}` symbol `{}` signature `{}`",
                entry.abi, entry.symbol, entry.signature_pattern
            ));
        }
        if entry.policy != policy {
            issues.push(format!(
                "host_ffi entry `{}` uses policy `{}` but link plan policy is `{policy}`",
                entry.symbol, entry.policy
            ));
        }
    }
    for ((abi, symbol), signatures) in signatures_by_symbol {
        if signatures.len() > 1 {
            notes.push(format!(
                "host_ffi ABI `{abi}` symbol `{symbol}` has {} whitelisted signatures",
                signatures.len()
            ));
        }
    }
    let valid = issues.is_empty();
    LinkPlanHostFfiValidationSummary {
        checked: entries.len(),
        valid,
        link_allowed: valid,
        issues,
        notes,
    }
}

pub(super) fn derive_host_ffi_abi_groups(
    entries: &[LinkPlanHostFfiEntry],
) -> Vec<LinkPlanHostFfiAbiGroup> {
    let mut groups: BTreeMap<&str, Vec<&LinkPlanHostFfiEntry>> = BTreeMap::new();
    for entry in entries {
        groups.entry(entry.abi.as_str()).or_default().push(entry);
    }
    groups
        .into_iter()
        .map(|(abi, entries)| {
            let abi_entries = entries
                .iter()
                .map(|entry| LinkPlanHostFfiAbiEntry {
                    symbol: entry.symbol.clone(),
                    signature_pattern: entry.signature_pattern.clone(),
                    signature_hash: entry.signature_hash.clone(),
                    policy: entry.policy.clone(),
                })
                .collect::<Vec<_>>();
            LinkPlanHostFfiAbiGroup {
                abi: abi.to_owned(),
                symbol_count: entries.len(),
                policy_count: entries
                    .iter()
                    .filter(|entry| !entry.policy.is_empty())
                    .count(),
                symbols: entries
                    .iter()
                    .map(|entry| format!("{}:{}", entry.symbol, entry.signature_pattern))
                    .collect(),
                validation: validate_host_ffi_abi_group(abi, &abi_entries),
                entries: abi_entries,
            }
        })
        .collect()
}

fn validate_host_ffi_abi_group(
    abi: &str,
    entries: &[LinkPlanHostFfiAbiEntry],
) -> LinkPlanHostFfiValidationSummary {
    let mut issues = Vec::new();
    let mut notes = Vec::new();
    let mut seen_signatures = BTreeSet::new();
    let mut signatures_by_symbol: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for entry in entries {
        let key = (entry.symbol.as_str(), entry.signature_pattern.as_str());
        signatures_by_symbol
            .entry(entry.symbol.as_str())
            .or_default()
            .insert(entry.signature_pattern.as_str());
        if !seen_signatures.insert(key) {
            issues.push(format!(
                "host_ffi ABI `{abi}` duplicate symbol `{}` signature `{}`",
                entry.symbol, entry.signature_pattern
            ));
        }
        if entry.policy != crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY {
            issues.push(format!(
                "host_ffi ABI `{abi}` symbol `{}` uses unsupported policy `{}`",
                entry.symbol, entry.policy
            ));
        }
    }
    for (symbol, signatures) in signatures_by_symbol {
        if signatures.len() > 1 {
            notes.push(format!(
                "host_ffi ABI `{abi}` symbol `{symbol}` has {} whitelisted signatures",
                signatures.len()
            ));
        }
    }
    let valid = issues.is_empty();
    LinkPlanHostFfiValidationSummary {
        checked: entries.len(),
        valid,
        link_allowed: valid,
        issues,
        notes,
    }
}

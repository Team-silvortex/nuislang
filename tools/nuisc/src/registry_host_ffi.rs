use std::collections::{BTreeMap, BTreeSet};

use crate::registry::NustarPackageManifest;
use yir_core::ffi::is_ffi_symbol_hash_token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostFfiSymbolRegistration {
    Signature(String),
    Hash(String),
}

impl HostFfiSymbolRegistration {
    pub fn render(&self) -> String {
        match self {
            Self::Signature(signature) => format!("signature:{signature}"),
            Self::Hash(hash) => format!("hash:{hash}"),
        }
    }

    pub fn matches(&self, signature_matches: impl FnOnce(&str) -> bool, actual_hash: &str) -> bool {
        match self {
            Self::Signature(pattern) => signature_matches(pattern),
            Self::Hash(expected) => expected == actual_hash,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostFfiRegistryView {
    signature_families: BTreeMap<String, Vec<String>>,
    symbol_registrations: BTreeMap<(String, String), Vec<HostFfiSymbolRegistration>>,
}

impl HostFfiRegistryView {
    pub fn from_manifest(manifest: &NustarPackageManifest) -> Self {
        let mut view = Self::default();
        for raw in &manifest.abi_capabilities {
            let Some((abi, caps)) = raw.split_once(':') else {
                continue;
            };
            let abi = abi.trim();
            if abi.is_empty() {
                continue;
            }
            for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
                if let Some(pattern) = cap.strip_prefix("ffi:") {
                    view.signature_families
                        .entry(abi.to_owned())
                        .or_default()
                        .push(pattern.trim().to_owned());
                } else if let Some(entry) = cap.strip_prefix("ffi_symbol:") {
                    let Some((symbol, signature)) = entry.split_once('=') else {
                        continue;
                    };
                    view.symbol_registrations
                        .entry((abi.to_owned(), symbol.trim().to_owned()))
                        .or_default()
                        .push(HostFfiSymbolRegistration::Signature(
                            signature.trim().to_owned(),
                        ));
                } else if let Some(entry) = cap.strip_prefix("ffi_symbol_hash:") {
                    let Some((symbol, hash)) = entry.split_once('=') else {
                        continue;
                    };
                    view.symbol_registrations
                        .entry((abi.to_owned(), symbol.trim().to_owned()))
                        .or_default()
                        .push(HostFfiSymbolRegistration::Hash(hash.trim().to_owned()));
                }
            }
        }
        for values in view.signature_families.values_mut() {
            values.sort();
            values.dedup();
        }
        for values in view.symbol_registrations.values_mut() {
            values.sort_by_key(HostFfiSymbolRegistration::render);
            values.dedup();
        }
        view
    }

    pub fn has_abi(&self, abi: &str) -> bool {
        self.signature_families.contains_key(abi)
            || self
                .symbol_registrations
                .keys()
                .any(|(entry_abi, _)| entry_abi == abi)
    }

    pub fn signature_families(&self, abi: &str) -> &[String] {
        self.signature_families
            .get(abi)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn symbol_registrations(&self, abi: &str, symbol: &str) -> &[HostFfiSymbolRegistration] {
        self.symbol_registrations
            .get(&(abi.to_owned(), symbol.to_owned()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

pub fn validate_abi_capabilities(
    manifest: &NustarPackageManifest,
    required_abi: &str,
    used_surfaces: &[String],
    used_ops: &[String],
) -> Result<(), String> {
    if manifest.abi_capabilities.is_empty() {
        return Ok(());
    }

    let mut surface_allowed = BTreeSet::new();
    let mut op_allowed = BTreeSet::new();
    let mut saw_required_abi = false;
    for raw in &manifest.abi_capabilities {
        let Some((abi, caps)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; expected `abi:kind:value[|kind:value...]`",
                manifest.package_id, raw
            ));
        };
        if abi.trim().is_empty() {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; ABI id must not be empty",
                manifest.package_id, raw
            ));
        }
        let abi_matches = abi.trim() == required_abi;
        if !abi_matches {
            continue;
        }
        saw_required_abi = true;
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(value) = cap.strip_prefix("surface:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `surface:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                surface_allowed.insert(value.to_owned());
            } else if let Some(value) = cap.strip_prefix("op:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `op:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                op_allowed.insert(value.to_owned());
            } else if let Some(value) = cap.strip_prefix("ffi:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `ffi:` capability must include a signature pattern",
                        manifest.package_id, raw
                    ));
                }
            } else if let Some(value) = cap.strip_prefix("ffi_symbol:") {
                let Some((symbol, signature)) = value.split_once('=') else {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `ffi_symbol:` capability must use `symbol=signature`",
                        manifest.package_id, raw
                    ));
                };
                if symbol.trim().is_empty() || signature.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `ffi_symbol:` capability must include a symbol and signature",
                        manifest.package_id, raw
                    ));
                }
            } else if let Some(value) = cap.strip_prefix("ffi_symbol_hash:") {
                let Some((symbol, hash)) = value.split_once('=') else {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `ffi_symbol_hash:` capability must use `symbol=fnv1a64:<hex>`",
                        manifest.package_id, raw
                    ));
                };
                if symbol.trim().is_empty() || !is_ffi_symbol_hash_token(hash.trim()) {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `ffi_symbol_hash:` capability must include a symbol and `fnv1a64:<hex>` hash",
                        manifest.package_id, raw
                    ));
                }
            } else {
                return Err(format!(
                    "nustar package `{}` has invalid abi_capabilities capability `{}` in `{}`; expected `surface:<pattern>`, `op:<pattern>`, `ffi:<signature>`, `ffi_symbol:<symbol>=<signature>`, or `ffi_symbol_hash:<symbol>=fnv1a64:<hex>`",
                    manifest.package_id, cap, raw
                ));
            }
        }
    }

    if !saw_required_abi {
        return Err(format!(
            "ABI `{}` of nustar package `{}` has no abi_capabilities mapping; add `{}:...` in manifest",
            required_abi, manifest.package_id, required_abi
        ));
    }

    if !surface_allowed.is_empty() && !surface_allowed.contains("*") {
        for surface in used_surfaces {
            if !surface_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, surface))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow support surface `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    surface,
                    surface_allowed
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    if !op_allowed.is_empty() && !op_allowed.contains("*") {
        for op in used_ops {
            if !op_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, op))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow op `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    op,
                    op_allowed.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    Ok(())
}

fn capability_matches(pattern: &str, actual: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return actual.starts_with(prefix);
    }
    pattern == actual
}

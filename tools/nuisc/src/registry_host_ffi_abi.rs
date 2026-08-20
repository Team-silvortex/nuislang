use std::collections::BTreeSet;

use crate::registry::{HostFfiRegistryView, NustarPackageManifest};
use yir_core::ffi::is_ffi_symbol_hash_token;

pub fn validate_abi_capabilities(
    manifest: &NustarPackageManifest,
    required_abi: &str,
    used_surfaces: &[String],
    used_ops: &[String],
) -> Result<(), String> {
    HostFfiRegistryView::try_from_manifest(manifest)?;
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
        if abi.trim() != required_abi {
            continue;
        }
        saw_required_abi = true;
        validate_capability_entries(manifest, raw, caps, &mut surface_allowed, &mut op_allowed)?;
    }

    if !saw_required_abi {
        return Err(format!(
            "ABI `{required_abi}` of nustar package `{}` has no abi_capabilities mapping; add `{required_abi}:...` in manifest",
            manifest.package_id
        ));
    }
    validate_used_capabilities(
        manifest,
        required_abi,
        "support surface",
        used_surfaces,
        &surface_allowed,
    )?;
    validate_used_capabilities(manifest, required_abi, "op", used_ops, &op_allowed)
}

fn validate_capability_entries(
    manifest: &NustarPackageManifest,
    raw: &str,
    caps: &str,
    surface_allowed: &mut BTreeSet<String>,
    op_allowed: &mut BTreeSet<String>,
) -> Result<(), String> {
    for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
        if let Some(value) = cap.strip_prefix("surface:") {
            insert_pattern(manifest, raw, "surface", value, surface_allowed)?;
        } else if let Some(value) = cap.strip_prefix("op:") {
            insert_pattern(manifest, raw, "op", value, op_allowed)?;
        } else if let Some(value) = cap.strip_prefix("ffi:") {
            require_nonempty(manifest, raw, "ffi", value)?;
        } else if let Some(value) = cap.strip_prefix("ffi_symbol:") {
            let Some((symbol, signature)) = value.split_once('=') else {
                return invalid_entry(
                    manifest,
                    raw,
                    "`ffi_symbol:` capability must use `symbol=signature`",
                );
            };
            if symbol.trim().is_empty() || signature.trim().is_empty() {
                return invalid_entry(
                    manifest,
                    raw,
                    "`ffi_symbol:` capability must include a symbol and signature",
                );
            }
        } else if let Some(value) = cap.strip_prefix("ffi_symbol_hash:") {
            let Some((symbol, hash)) = value.split_once('=') else {
                return invalid_entry(
                    manifest,
                    raw,
                    "`ffi_symbol_hash:` capability must use `symbol=fnv1a64:<hex>`",
                );
            };
            if symbol.trim().is_empty() || !is_ffi_symbol_hash_token(hash.trim()) {
                return invalid_entry(
                    manifest,
                    raw,
                    "`ffi_symbol_hash:` capability must include a symbol and `fnv1a64:<hex>` hash",
                );
            }
        } else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities capability `{cap}` in `{raw}`; expected `surface:<pattern>`, `op:<pattern>`, `ffi:<signature>`, `ffi_symbol:<symbol>=<signature>`, or `ffi_symbol_hash:<symbol>=fnv1a64:<hex>`",
                manifest.package_id
            ));
        }
    }
    Ok(())
}

fn insert_pattern(
    manifest: &NustarPackageManifest,
    raw: &str,
    kind: &str,
    value: &str,
    target: &mut BTreeSet<String>,
) -> Result<(), String> {
    require_nonempty(manifest, raw, kind, value)?;
    target.insert(value.to_owned());
    Ok(())
}

fn require_nonempty(
    manifest: &NustarPackageManifest,
    raw: &str,
    kind: &str,
    value: &str,
) -> Result<(), String> {
    if value.trim().is_empty() {
        return invalid_entry(
            manifest,
            raw,
            &format!("`{kind}:` capability must include a pattern"),
        );
    }
    Ok(())
}

fn invalid_entry<T>(
    manifest: &NustarPackageManifest,
    raw: &str,
    detail: &str,
) -> Result<T, String> {
    Err(format!(
        "nustar package `{}` has invalid abi_capabilities entry `{raw}`; {detail}",
        manifest.package_id
    ))
}

fn validate_used_capabilities(
    manifest: &NustarPackageManifest,
    abi: &str,
    kind: &str,
    used: &[String],
    allowed: &BTreeSet<String>,
) -> Result<(), String> {
    if allowed.is_empty() || allowed.contains("*") {
        return Ok(());
    }
    for actual in used {
        if !allowed
            .iter()
            .any(|pattern| capability_matches(pattern, actual))
        {
            return Err(format!(
                "ABI `{abi}` of nustar package `{}` does not allow {kind} `{actual}` (allowed: {})",
                manifest.package_id,
                allowed.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    Ok(())
}

fn capability_matches(pattern: &str, actual: &str) -> bool {
    pattern == "*"
        || pattern
            .strip_suffix('*')
            .is_some_and(|prefix| actual.starts_with(prefix))
        || pattern == actual
}

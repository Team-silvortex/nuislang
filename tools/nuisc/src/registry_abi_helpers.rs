use std::collections::BTreeSet;

use crate::registry::NustarPackageManifest;
use crate::registry_abi_target::{parse_registered_abi_target, RegisteredAbiTarget};
use yir_core::YirModule;

pub fn validate_unit_binding(
    manifests: &[NustarPackageManifest],
    domain: &str,
    unit: &str,
) -> Result<(), String> {
    let manifest = manifests
        .iter()
        .find(|manifest| manifest.domain_family == domain)
        .ok_or_else(|| format!("no nustar manifest loaded for mod domain `{domain}`"))?;

    if manifest.unit_types.is_empty() {
        return Ok(());
    }

    if manifest
        .unit_types
        .iter()
        .any(|candidate| candidate == unit)
    {
        return Ok(());
    }

    Err(format!(
        "unit `{unit}` is not registered by nustar package `{}` for mod domain `{domain}`",
        manifest.package_id
    ))
}

pub fn validate_manifest_abi(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<(), String> {
    if manifest
        .abi_profiles
        .iter()
        .any(|profile| profile == required_abi)
    {
        return Ok(());
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare required ABI `{}`; declared ABI profiles: {}",
        manifest.package_id,
        manifest.domain_family,
        required_abi,
        if manifest.abi_profiles.is_empty() {
            "<none>".to_owned()
        } else {
            manifest.abi_profiles.join(", ")
        }
    ))
}

pub fn registered_abi_target(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        if abi.trim() != required_abi {
            continue;
        }
        return parse_registered_abi_target(required_abi, fields, manifest, raw);
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare abi target metadata for `{}`",
        manifest.package_id, manifest.domain_family, required_abi
    ))
}

pub fn registered_abi_target_for_clang(
    manifest: &NustarPackageManifest,
    clang_target: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    let mut matches = Vec::new();
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        let target = parse_registered_abi_target(abi.trim(), fields, manifest, raw)?;
        if target.clang_target == clang_target {
            matches.push(target);
        }
    }
    matches.into_iter().next().ok_or_else(|| {
        format!(
            "nustar package `{}` for domain `{}` does not register clang target `{}` in `abi_targets`",
            manifest.package_id, manifest.domain_family, clang_target
        )
    })
}

pub fn used_ops_for_domain(module: &YirModule, domain_family: &str) -> Vec<String> {
    let mut ops = module
        .nodes
        .iter()
        .filter(|node| node.op.module == domain_family)
        .map(|node| node.op.full_name())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ops.sort();
    ops
}

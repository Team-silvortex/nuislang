use std::collections::{BTreeMap, BTreeSet};

use crate::registry::{HostFfiRegistryView, HostFfiSymbolRegistration, NustarPackageManifest};

pub const NUSTAR_LINKER_RESOLVER_PROVIDER_CONTRACT: &str =
    "nuis-nustar-linker-resolver-provider-v1";
pub const NUSTAR_LINKER_SYMBOL_VERSION_CONTRACT: &str = "nuis-nustar-linker-symbol-version-v1";
pub const NUSTAR_LINKER_RESOLVER_REGISTRY_CONTRACT: &str =
    "nuis-nustar-linker-resolver-registry-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarLinkerResolverProviderRegistration {
    pub package_id: String,
    pub provider_id: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub host_ffi_abi: String,
    pub interpreter_identity: String,
    pub interpreter_path: String,
    pub dependency_identity: String,
    pub needed_name: String,
    pub symbol_version_policy: String,
    pub resolver_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarLinkerSymbolVersionRegistration {
    pub package_id: String,
    pub provider_id: String,
    pub target_symbol: String,
    pub version_identity: String,
    pub version_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NustarLinkerResolverRegistrations {
    pub providers: Vec<NustarLinkerResolverProviderRegistration>,
    pub symbol_versions: Vec<NustarLinkerSymbolVersionRegistration>,
}

pub fn linker_resolver_registrations(
    manifests: &[NustarPackageManifest],
) -> Result<NustarLinkerResolverRegistrations, String> {
    let mut registrations = NustarLinkerResolverRegistrations::default();
    let mut manifests_by_package = BTreeMap::new();
    let mut ffi_views = BTreeMap::new();

    for manifest in manifests {
        let declares_resolvers = !manifest.linker_resolver_providers.is_empty()
            || !manifest.linker_symbol_versions.is_empty();
        if !declares_resolvers {
            continue;
        }
        if manifest.domain_family != "cffi" {
            return Err(format!(
                "nustar package `{}` cannot register linker resolvers outside the `cffi` domain",
                manifest.package_id
            ));
        }
        let view = HostFfiRegistryView::try_from_manifest(manifest)?;
        manifests_by_package.insert(manifest.package_id.clone(), manifest);
        ffi_views.insert(manifest.package_id.clone(), view);
        for entry in &manifest.linker_resolver_providers {
            registrations
                .providers
                .push(parse_provider_registration(manifest, entry)?);
        }
        for entry in &manifest.linker_symbol_versions {
            registrations
                .symbol_versions
                .push(parse_symbol_version_registration(manifest, entry)?);
        }
    }

    validate_registrations(&registrations, &manifests_by_package, &ffi_views)?;
    Ok(registrations)
}

fn parse_provider_registration(
    manifest: &NustarPackageManifest,
    entry: &str,
) -> Result<NustarLinkerResolverProviderRegistration, String> {
    let fields = entry.split('|').collect::<Vec<_>>();
    if fields.len() != 14 || fields[0] != NUSTAR_LINKER_RESOLVER_PROVIDER_CONTRACT {
        return Err(format!(
            "nustar package `{}` linker resolver provider entry `{entry}` has an invalid registration contract",
            manifest.package_id
        ));
    }
    for (label, value) in [
        ("provider id", fields[1]),
        ("machine architecture", fields[2]),
        ("machine OS", fields[3]),
        ("object format", fields[4]),
        ("calling ABI", fields[5]),
        ("clang target", fields[6]),
        ("host FFI ABI", fields[7]),
        ("interpreter identity", fields[8]),
        ("dependency identity", fields[10]),
        ("needed name", fields[11]),
        ("symbol-version policy", fields[12]),
        ("resolver identity", fields[13]),
    ] {
        if !is_identity(value) {
            return Err(format!(
                "nustar package `{}` linker resolver {label} `{value}` is invalid",
                manifest.package_id
            ));
        }
    }
    if !is_absolute_target_path(fields[9]) {
        return Err(format!(
            "nustar package `{}` linker resolver interpreter path `{}` is invalid",
            manifest.package_id, fields[9]
        ));
    }
    if !manifest.host_ffi_abis.iter().any(|abi| abi == fields[7]) {
        return Err(format!(
            "nustar package `{}` linker resolver provider `{}` references undeclared host FFI ABI `{}`",
            manifest.package_id, fields[1], fields[7]
        ));
    }
    let target = format!(
        "arch={}|os={}|object={}|calling={}|clang={}",
        fields[2], fields[3], fields[4], fields[5], fields[6]
    );
    if !manifest
        .abi_targets
        .iter()
        .filter_map(|entry| entry.split_once(':').map(|(_, fields)| fields))
        .any(|fields| fields == target)
    {
        return Err(format!(
            "nustar package `{}` linker resolver provider `{}` references undeclared ABI target `{target}`",
            manifest.package_id, fields[1]
        ));
    }

    Ok(NustarLinkerResolverProviderRegistration {
        package_id: manifest.package_id.clone(),
        provider_id: fields[1].to_owned(),
        machine_arch: fields[2].to_owned(),
        machine_os: fields[3].to_owned(),
        object_format: fields[4].to_owned(),
        calling_abi: fields[5].to_owned(),
        clang_target: fields[6].to_owned(),
        host_ffi_abi: fields[7].to_owned(),
        interpreter_identity: fields[8].to_owned(),
        interpreter_path: fields[9].to_owned(),
        dependency_identity: fields[10].to_owned(),
        needed_name: fields[11].to_owned(),
        symbol_version_policy: fields[12].to_owned(),
        resolver_identity: fields[13].to_owned(),
    })
}

fn parse_symbol_version_registration(
    manifest: &NustarPackageManifest,
    entry: &str,
) -> Result<NustarLinkerSymbolVersionRegistration, String> {
    let fields = entry.split('|').collect::<Vec<_>>();
    if fields.len() != 5 || fields[0] != NUSTAR_LINKER_SYMBOL_VERSION_CONTRACT {
        return Err(format!(
            "nustar package `{}` linker symbol-version entry `{entry}` has an invalid registration contract",
            manifest.package_id
        ));
    }
    for (label, value) in [
        ("provider id", fields[1]),
        ("target symbol", fields[2]),
        ("version identity", fields[3]),
        ("version name", fields[4]),
    ] {
        if !is_identity(value) {
            return Err(format!(
                "nustar package `{}` linker symbol-version {label} `{value}` is invalid",
                manifest.package_id
            ));
        }
    }
    Ok(NustarLinkerSymbolVersionRegistration {
        package_id: manifest.package_id.clone(),
        provider_id: fields[1].to_owned(),
        target_symbol: fields[2].to_owned(),
        version_identity: fields[3].to_owned(),
        version_name: fields[4].to_owned(),
    })
}

fn validate_registrations(
    registrations: &NustarLinkerResolverRegistrations,
    manifests: &BTreeMap<String, &NustarPackageManifest>,
    ffi_views: &BTreeMap<String, HostFfiRegistryView>,
) -> Result<(), String> {
    let mut provider_ids = BTreeMap::new();
    let mut target_abis = BTreeSet::new();
    for provider in &registrations.providers {
        if provider_ids
            .insert(provider.provider_id.as_str(), provider)
            .is_some()
        {
            return Err(format!(
                "duplicate Nustar linker resolver provider `{}`",
                provider.provider_id
            ));
        }
        if !target_abis.insert((
            provider.machine_arch.as_str(),
            provider.machine_os.as_str(),
            provider.object_format.as_str(),
            provider.calling_abi.as_str(),
            provider.clang_target.as_str(),
            provider.host_ffi_abi.as_str(),
        )) {
            return Err(format!(
                "ambiguous Nustar linker resolver provider target for ABI `{}`",
                provider.host_ffi_abi
            ));
        }
    }

    let mut symbols = BTreeSet::new();
    let mut version_identities = BTreeMap::new();
    let mut version_names = BTreeMap::new();
    let mut provider_symbol_counts = BTreeMap::<&str, usize>::new();
    for version in &registrations.symbol_versions {
        let provider = provider_ids
            .get(version.provider_id.as_str())
            .ok_or_else(|| {
                format!(
                    "Nustar linker symbol `{}` references missing provider `{}`",
                    version.target_symbol, version.provider_id
                )
            })?;
        if provider.package_id != version.package_id {
            return Err(format!(
                "Nustar package `{}` cannot attach symbol `{}` to provider `{}` owned by `{}`",
                version.package_id, version.target_symbol, version.provider_id, provider.package_id
            ));
        }
        if !symbols.insert((version.provider_id.as_str(), version.target_symbol.as_str())) {
            return Err(format!(
                "duplicate Nustar linker symbol-version registration `{}:{}`",
                version.provider_id, version.target_symbol
            ));
        }
        validate_version_identity_maps(version, &mut version_identities, &mut version_names)?;
        validate_symbol_whitelist(version, provider, manifests, ffi_views)?;
        *provider_symbol_counts
            .entry(version.provider_id.as_str())
            .or_default() += 1;
    }
    for provider in &registrations.providers {
        if provider_symbol_counts
            .get(provider.provider_id.as_str())
            .copied()
            .unwrap_or_default()
            == 0
        {
            return Err(format!(
                "Nustar linker resolver provider `{}` has no symbol-version registrations",
                provider.provider_id
            ));
        }
    }
    Ok(())
}

fn validate_version_identity_maps<'a>(
    version: &'a NustarLinkerSymbolVersionRegistration,
    identities: &mut BTreeMap<(&'a str, &'a str), &'a str>,
    names: &mut BTreeMap<(&'a str, &'a str), &'a str>,
) -> Result<(), String> {
    let identity_consistent = identities
        .insert(
            (
                version.provider_id.as_str(),
                version.version_identity.as_str(),
            ),
            version.version_name.as_str(),
        )
        .is_none_or(|previous| previous == version.version_name);
    let name_consistent = names
        .insert(
            (version.provider_id.as_str(), version.version_name.as_str()),
            version.version_identity.as_str(),
        )
        .is_none_or(|previous| previous == version.version_identity);
    if identity_consistent && name_consistent {
        Ok(())
    } else {
        Err(format!(
            "Nustar linker provider `{}` has inconsistent symbol-version identity `{}` / `{}`",
            version.provider_id, version.version_identity, version.version_name
        ))
    }
}

fn validate_symbol_whitelist(
    version: &NustarLinkerSymbolVersionRegistration,
    provider: &NustarLinkerResolverProviderRegistration,
    manifests: &BTreeMap<String, &NustarPackageManifest>,
    ffi_views: &BTreeMap<String, HostFfiRegistryView>,
) -> Result<(), String> {
    let manifest = manifests.get(&version.package_id).ok_or_else(|| {
        format!(
            "missing Nustar manifest for linker symbol `{}`",
            version.target_symbol
        )
    })?;
    let view = ffi_views.get(&version.package_id).ok_or_else(|| {
        format!(
            "missing CFFI whitelist for linker symbol `{}`",
            version.target_symbol
        )
    })?;
    let entries = view.symbol_registrations(&provider.host_ffi_abi, &version.target_symbol);
    let exact = matches!(
        entries,
        [HostFfiSymbolRegistration::Signature(signature)] if !signature.contains('*')
    ) || matches!(entries, [HostFfiSymbolRegistration::Hash(hash)] if !hash.is_empty());
    if exact
        && manifest
            .host_ffi_abis
            .iter()
            .any(|abi| abi == &provider.host_ffi_abi)
    {
        Ok(())
    } else {
        Err(format!(
            "nustar package `{}` linker symbol `{}:{}` requires one exact `ffi_symbol:` or `ffi_symbol_hash:` whitelist registration",
            version.package_id, provider.host_ffi_abi, version.target_symbol
        ))
    }
}

fn is_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
}

fn is_absolute_target_path(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 4096
        && !value.contains("//")
        && value.bytes().all(|byte| byte.is_ascii_graphic())
        && value
            .split('/')
            .skip(1)
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn official_cffi_manifest() -> NustarPackageManifest {
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), "cffi")
            .expect("official CFFI manifest")
    }

    #[test]
    fn official_cffi_owns_the_static_gnu_resolver_rows() {
        let registrations = linker_resolver_registrations(&[official_cffi_manifest()])
            .expect("official CFFI linker resolver registrations");

        assert_eq!(registrations.providers.len(), 2);
        assert_eq!(registrations.symbol_versions.len(), 4);
        assert_eq!(
            registrations.providers[0].provider_id,
            "nsld.elf.amd64.linux-gnu.libc-v1"
        );
        assert_eq!(registrations.symbol_versions[3].target_symbol, "cos");
    }

    #[test]
    fn rejects_symbol_rows_without_exact_cffi_whitelist_authority() {
        let mut manifest = official_cffi_manifest();
        manifest
            .abi_capabilities
            .retain(|entry| !entry.starts_with("libm:"));

        let error = linker_resolver_registrations(&[manifest])
            .expect_err("unwhitelisted resolver symbols must fail closed");

        assert!(error.contains("libm:cos"));
        assert!(error.contains("exact `ffi_symbol:`"));
    }
}

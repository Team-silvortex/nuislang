use crate::registry::NustarPackageManifest;

pub const NUSTAR_PROVIDER_BUNDLE_ENTRY_CONTRACT: &str = "nuis-provider-bundle-manifest-entry-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarProviderBundleRegistration {
    pub package_id: String,
    pub bundle_id: String,
    pub provider_family: String,
    pub runner_adapter_id: String,
    pub adapter_kind: String,
    pub rust_const: String,
}

pub fn provider_bundle_registrations(
    manifest: &NustarPackageManifest,
) -> Result<Vec<NustarProviderBundleRegistration>, String> {
    let mut registrations = manifest
        .provider_bundles
        .iter()
        .map(|entry| parse_provider_bundle_registration(&manifest.package_id, entry))
        .collect::<Result<Vec<_>, _>>()?;
    registrations.sort_by(|lhs, rhs| lhs.bundle_id.cmp(&rhs.bundle_id));
    Ok(registrations)
}

fn parse_provider_bundle_registration(
    package_id: &str,
    entry: &str,
) -> Result<NustarProviderBundleRegistration, String> {
    let fields = entry.split('|').collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(format!(
            "nustar package `{package_id}` provider bundle entry `{entry}` must contain six ordered fields"
        ));
    }
    if fields[0] != NUSTAR_PROVIDER_BUNDLE_ENTRY_CONTRACT {
        return Err(format!(
            "nustar package `{package_id}` provider bundle entry `{entry}` has unsupported contract `{}`",
            fields[0]
        ));
    }
    for (label, value) in [
        ("bundle id", fields[1]),
        ("runner adapter id", fields[3]),
        ("adapter kind", fields[4]),
    ] {
        if !is_identity(value) {
            return Err(format!(
                "nustar package `{package_id}` provider bundle {label} `{value}` is invalid"
            ));
        }
    }
    if fields[2].split_once(':').is_none() || !is_identity(fields[2]) {
        return Err(format!(
            "nustar package `{package_id}` provider family `{}` is invalid",
            fields[2]
        ));
    }
    if !is_rust_const(fields[5]) {
        return Err(format!(
            "nustar package `{package_id}` provider bundle Rust const `{}` is invalid",
            fields[5]
        ));
    }
    Ok(NustarProviderBundleRegistration {
        package_id: package_id.to_owned(),
        bundle_id: fields[1].to_owned(),
        provider_family: fields[2].to_owned(),
        runner_adapter_id: fields[3].to_owned(),
        adapter_kind: fields[4].to_owned(),
        rust_const: fields[5].to_owned(),
    })
}

fn is_identity(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
}

fn is_rust_const(value: &str) -> bool {
    value.ends_with("::PROVIDER_BUNDLE")
        && value.split("::").all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unversioned_or_unsafe_static_provider_bundle_bindings() {
        assert!(parse_provider_bundle_registration(
            "test.data",
            "provider-bundle|bundle.v1|data:host|runner.v1|runner-kind|../bad::PROVIDER_BUNDLE"
        )
        .is_err());
    }
}

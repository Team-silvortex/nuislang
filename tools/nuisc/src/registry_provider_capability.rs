use crate::registry::NustarPackageManifest;

pub const NUSTAR_PROVIDER_CAPABILITY_ENTRY_CONTRACT: &str = "nuis-provider-capability-record-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarProviderCapabilityRegistration {
    pub package_id: String,
    pub provider_id: String,
    pub bundle_id: String,
    pub provider_family: String,
    pub priority: u16,
    pub capabilities: Vec<String>,
}

pub fn provider_capability_registrations(
    manifest: &NustarPackageManifest,
) -> Result<Vec<NustarProviderCapabilityRegistration>, String> {
    let mut registrations = manifest
        .provider_capabilities
        .iter()
        .map(|entry| parse_provider_capability_registration(&manifest.package_id, entry))
        .collect::<Result<Vec<_>, _>>()?;
    registrations.sort_by(|lhs, rhs| lhs.provider_id.cmp(&rhs.provider_id));
    Ok(registrations)
}

fn parse_provider_capability_registration(
    package_id: &str,
    entry: &str,
) -> Result<NustarProviderCapabilityRegistration, String> {
    let fields = entry.split('|').collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(format!(
            "nustar package `{package_id}` provider capability entry `{entry}` must contain six ordered fields"
        ));
    }
    if fields[0] != NUSTAR_PROVIDER_CAPABILITY_ENTRY_CONTRACT {
        return Err(format!(
            "nustar package `{package_id}` provider capability entry `{entry}` has unsupported contract `{}`",
            fields[0]
        ));
    }
    for (label, value) in [
        ("provider id", fields[1]),
        ("bundle id", fields[2]),
        ("provider family", fields[3]),
    ] {
        if !is_identity(value) {
            return Err(format!(
                "nustar package `{package_id}` provider capability {label} `{value}` is invalid"
            ));
        }
    }
    if fields[3].split_once(':').is_none() {
        return Err(format!(
            "nustar package `{package_id}` provider capability family `{}` must contain a domain separator",
            fields[3]
        ));
    }
    let priority = fields[4].parse::<u16>().map_err(|_| {
        format!(
            "nustar package `{package_id}` provider capability priority `{}` is invalid",
            fields[4]
        )
    })?;
    if priority == 0 {
        return Err(format!(
            "nustar package `{package_id}` provider capability priority must be positive"
        ));
    }
    let capabilities = fields[5].split(',').map(str::to_owned).collect::<Vec<_>>();
    if capabilities.is_empty() || capabilities.iter().any(|value| !is_identity(value)) {
        return Err(format!(
            "nustar package `{package_id}` provider capability set `{}` is invalid",
            fields[5]
        ));
    }
    if capabilities.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(format!(
            "nustar package `{package_id}` provider capabilities must be strictly sorted and unique"
        ));
    }
    Ok(NustarProviderCapabilityRegistration {
        package_id: package_id.to_owned(),
        provider_id: fields[1].to_owned(),
        bundle_id: fields[2].to_owned(),
        provider_family: fields[3].to_owned(),
        priority,
        capabilities,
    })
}

fn is_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn official_data_manifest_registers_cpu_memory_reference_provider() {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), "data")
                .expect("official Data manifest");
        let registrations = provider_capability_registrations(&manifest).unwrap();

        assert_eq!(registrations.len(), 1);
        let registration = &registrations[0];
        assert_eq!(registration.provider_id, "data.cpu-memory.reference.v1");
        assert_eq!(registration.bundle_id, "data.host.bundle.v1");
        assert_eq!(registration.provider_family, "data:host");
        assert_eq!(registration.priority, 100);
        assert_eq!(
            registration.capabilities,
            [
                "clock.fabric-monotonic",
                "completion.verified",
                "execution.reference",
                "glm.owned-transfer",
                "memory.cpu",
                "movement.copy",
                "residency.host",
            ]
        );
    }

    #[test]
    fn rejects_duplicate_or_unordered_capabilities() {
        for capabilities in ["memory.cpu,memory.cpu", "movement.copy,memory.cpu"] {
            let entry = format!(
                "{NUSTAR_PROVIDER_CAPABILITY_ENTRY_CONTRACT}|data.test.v1|data.host.bundle.v1|data:host|1|{capabilities}"
            );
            assert!(parse_provider_capability_registration("test.data", &entry).is_err());
        }
    }
}

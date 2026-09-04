use std::{env, fs, path::PathBuf};

use nuisc::registry::{
    ensure_registered_domains_valid, load_all_manifests, provider_bundle_registrations,
    provider_capability_registrations, NustarProviderBundleRegistration,
    NustarProviderCapabilityRegistration, NUSTAR_PROVIDER_BUNDLE_ENTRY_CONTRACT,
    NUSTAR_PROVIDER_CAPABILITY_ENTRY_CONTRACT,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let registry_root = manifest_dir.join("../..").join("nustar-packages");
    println!("cargo:rerun-if-changed={}", registry_root.display());
    println!(
        "cargo:rerun-if-changed={}",
        registry_root.join("index.toml").display()
    );
    ensure_registered_domains_valid(&registry_root)
        .unwrap_or_else(|error| panic!("invalid Nustar registry: {error}"));

    let manifests = load_all_manifests(&registry_root)
        .unwrap_or_else(|error| panic!("failed to load Nustar manifests: {error}"));
    let mut registrations = Vec::new();
    let mut capability_registrations = Vec::new();
    for manifest in manifests {
        println!(
            "cargo:rerun-if-changed={}",
            registry_root
                .join(format!("{}.toml", manifest.domain_family))
                .display()
        );
        registrations.extend(
            provider_bundle_registrations(&manifest)
                .unwrap_or_else(|error| panic!("invalid provider bundle registration: {error}")),
        );
        capability_registrations.extend(
            provider_capability_registrations(&manifest)
                .unwrap_or_else(|error| panic!("invalid provider capability: {error}")),
        );
    }
    registrations.sort_by(|lhs, rhs| lhs.bundle_id.cmp(&rhs.bundle_id));
    capability_registrations.sort_by(|lhs, rhs| lhs.provider_id.cmp(&rhs.provider_id));
    if registrations.is_empty() {
        panic!("Nustar registry contains no provider bundle registrations");
    }

    let canonical = canonical_manifest(&registrations);
    let hash = format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes()));
    let generated = render_generated_registry(&registrations, &hash);
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("provider_bundle_registry_generated.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("failed to write `{}`: {error}", output.display()));

    let capability_canonical = canonical_capability_manifest(&capability_registrations);
    let capability_hash = format!("fnv1a64:{:016x}", fnv1a64(capability_canonical.as_bytes()));
    let capability_generated =
        render_generated_capability_registry(&capability_registrations, &capability_hash);
    let capability_output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("provider_capability_registry_generated.rs");
    fs::write(&capability_output, capability_generated).unwrap_or_else(|error| {
        panic!("failed to write `{}`: {error}", capability_output.display())
    });
}

fn canonical_manifest(registrations: &[NustarProviderBundleRegistration]) -> String {
    let mut canonical = "nuis-provider-bundle-manifest-v1\n".to_owned();
    for registration in registrations {
        canonical.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}\n",
            NUSTAR_PROVIDER_BUNDLE_ENTRY_CONTRACT,
            registration.package_id,
            registration.bundle_id,
            registration.provider_family,
            registration.runner_adapter_id,
            registration.adapter_kind,
            registration.rust_const,
        ));
    }
    canonical
}

fn render_generated_registry(
    registrations: &[NustarProviderBundleRegistration],
    hash: &str,
) -> String {
    let mut source = format!(
        "pub(crate) const PROVIDER_BUNDLE_MANIFEST_HASH: &str = \"{hash}\";\n\
         pub(crate) const PROVIDER_BUNDLE_MANIFEST_ENTRY_COUNT: usize = {};\n\
         pub(crate) const PROVIDER_BUNDLE_REGISTRATIONS: &[ProviderBundleRegistration] = &[\n",
        registrations.len()
    );
    for registration in registrations {
        source.push_str(&format!("    crate::{},\n", registration.rust_const));
    }
    source.push_str("];\n");
    source.push_str(
        "pub(crate) const PROVIDER_BUNDLE_MANIFEST_ENTRIES: &[ProviderBundleManifestEntry] = &[\n",
    );
    for registration in registrations {
        source.push_str(&format!(
            "    ProviderBundleManifestEntry {{ package_id: {:?}, bundle_id: {:?}, provider_family: {:?}, runner_adapter_id: {:?}, adapter_kind: {:?}, rust_const: {:?} }},\n",
            registration.package_id,
            registration.bundle_id,
            registration.provider_family,
            registration.runner_adapter_id,
            registration.adapter_kind,
            registration.rust_const,
        ));
    }
    source.push_str("];\n");
    source
}

fn canonical_capability_manifest(registrations: &[NustarProviderCapabilityRegistration]) -> String {
    let mut canonical = "nuis-provider-capability-manifest-v1\n".to_owned();
    for registration in registrations {
        canonical.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}\n",
            NUSTAR_PROVIDER_CAPABILITY_ENTRY_CONTRACT,
            registration.package_id,
            registration.provider_id,
            registration.bundle_id,
            registration.provider_family,
            registration.priority,
            registration.capabilities.join(","),
        ));
    }
    canonical
}

fn render_generated_capability_registry(
    registrations: &[NustarProviderCapabilityRegistration],
    hash: &str,
) -> String {
    let mut source = format!(
        "pub(crate) const PROVIDER_CAPABILITY_MANIFEST_HASH: &str = \"{hash}\";\n\
         pub(crate) const PROVIDER_CAPABILITY_MANIFEST_ENTRY_COUNT: usize = {};\n\
         pub(crate) const PROVIDER_CAPABILITY_MANIFEST_ENTRIES: &[ProviderCapabilityManifestEntry] = &[\n",
        registrations.len()
    );
    for registration in registrations {
        source.push_str(&format!(
            "    ProviderCapabilityManifestEntry {{ package_id: {:?}, provider_id: {:?}, bundle_id: {:?}, provider_family: {:?}, priority: {}, capabilities: &{:?} }},\n",
            registration.package_id,
            registration.provider_id,
            registration.bundle_id,
            registration.provider_family,
            registration.priority,
            registration.capabilities,
        ));
    }
    source.push_str("];\n");
    source
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

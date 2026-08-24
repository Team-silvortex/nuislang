use std::{env, fmt::Write as _, fs, path::PathBuf};

use nuisc::registry::{
    ensure_registered_domains_valid, linker_resolver_registrations, load_all_manifests, load_index,
    manifest_path, NustarLinkerResolverProviderRegistration, NustarLinkerResolverRegistrations,
    NustarLinkerSymbolVersionRegistration, NUSTAR_LINKER_RESOLVER_PROVIDER_CONTRACT,
    NUSTAR_LINKER_RESOLVER_REGISTRY_CONTRACT, NUSTAR_LINKER_SYMBOL_VERSION_CONTRACT,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let registry_root = manifest_dir.join("../..").join("nustar-packages");
    println!("cargo:rerun-if-changed={}", registry_root.display());
    println!(
        "cargo:rerun-if-changed={}",
        registry_root.join("index.toml").display()
    );
    for entry in load_index(&registry_root)
        .unwrap_or_else(|error| panic!("failed to load Nustar index: {error}"))
    {
        println!(
            "cargo:rerun-if-changed={}",
            manifest_path(&registry_root, &entry).display()
        );
    }

    ensure_registered_domains_valid(&registry_root)
        .unwrap_or_else(|error| panic!("invalid Nustar registry: {error}"));
    let manifests = load_all_manifests(&registry_root)
        .unwrap_or_else(|error| panic!("failed to load Nustar manifests: {error}"));
    let registrations = linker_resolver_registrations(&manifests)
        .unwrap_or_else(|error| panic!("invalid linker resolver registration: {error}"));
    if registrations.providers.is_empty() || registrations.symbol_versions.is_empty() {
        panic!("Nustar registry contains no linker resolver registrations");
    }

    let canonical = canonical_manifest(&registrations);
    let hash = format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes()));
    let generated = render_generated_registry(&registrations, &hash);
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("linker_resolver_registry_generated.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("failed to write `{}`: {error}", output.display()));
}

fn canonical_manifest(registrations: &NustarLinkerResolverRegistrations) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, NUSTAR_LINKER_RESOLVER_REGISTRY_CONTRACT);
    for provider in &registrations.providers {
        append_provider(&mut canonical, provider);
    }
    for version in &registrations.symbol_versions {
        append_version(&mut canonical, version);
    }
    canonical
}

fn append_provider(canonical: &mut String, provider: &NustarLinkerResolverProviderRegistration) {
    for value in [
        NUSTAR_LINKER_RESOLVER_PROVIDER_CONTRACT,
        provider.package_id.as_str(),
        provider.provider_id.as_str(),
        provider.machine_arch.as_str(),
        provider.machine_os.as_str(),
        provider.object_format.as_str(),
        provider.calling_abi.as_str(),
        provider.clang_target.as_str(),
        provider.host_ffi_abi.as_str(),
        provider.interpreter_identity.as_str(),
        provider.interpreter_path.as_str(),
        provider.dependency_identity.as_str(),
        provider.needed_name.as_str(),
        provider.symbol_version_policy.as_str(),
        provider.resolver_identity.as_str(),
    ] {
        append_text(canonical, value);
    }
}

fn append_version(canonical: &mut String, version: &NustarLinkerSymbolVersionRegistration) {
    for value in [
        NUSTAR_LINKER_SYMBOL_VERSION_CONTRACT,
        version.package_id.as_str(),
        version.provider_id.as_str(),
        version.target_symbol.as_str(),
        version.version_identity.as_str(),
        version.version_name.as_str(),
    ] {
        append_text(canonical, value);
    }
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).expect("write canonical manifest");
}

fn render_generated_registry(
    registrations: &NustarLinkerResolverRegistrations,
    hash: &str,
) -> String {
    let mut source = format!(
        "pub(crate) const NUSTAR_LINKER_RESOLVER_MANIFEST_HASH: &str = {hash:?};\n\
         pub(crate) const NUSTAR_LINKER_RESOLVER_PROVIDER_COUNT: usize = {};\n\
         pub(crate) const NUSTAR_LINKER_SYMBOL_VERSION_COUNT: usize = {};\n\
         const DYNAMIC_RESOLVER_PROVIDERS: &[DynamicResolverProvider] = &[\n",
        registrations.providers.len(),
        registrations.symbol_versions.len()
    );
    for provider in &registrations.providers {
        writeln!(
            source,
            "    DynamicResolverProvider {{ provider_id: {:?}, machine_arch: {:?}, machine_os: {:?}, object_format: {:?}, calling_abi: {:?}, clang_target: {:?}, host_ffi_abi: {:?}, interpreter_identity: {:?}, interpreter_path: {:?}, dependency_identity: {:?}, needed_name: {:?}, symbol_version_policy: {:?}, resolver_identity: {:?} }},",
            provider.provider_id,
            provider.machine_arch,
            provider.machine_os,
            provider.object_format,
            provider.calling_abi,
            provider.clang_target,
            provider.host_ffi_abi,
            provider.interpreter_identity,
            provider.interpreter_path,
            provider.dependency_identity,
            provider.needed_name,
            provider.symbol_version_policy,
            provider.resolver_identity,
        )
        .expect("write generated provider");
    }
    source.push_str("];\n");
    source.push_str("const DYNAMIC_SYMBOL_VERSIONS: &[DynamicSymbolVersionRegistration] = &[\n");
    for version in &registrations.symbol_versions {
        writeln!(
            source,
            "    DynamicSymbolVersionRegistration {{ provider_id: {:?}, target_symbol: {:?}, version_identity: {:?}, version_name: {:?} }},",
            version.provider_id,
            version.target_symbol,
            version.version_identity,
            version.version_name,
        )
        .expect("write generated symbol version");
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

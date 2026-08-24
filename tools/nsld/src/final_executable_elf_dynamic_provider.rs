use crate::final_executable_elf_dynamic_plan::ElfAmd64DynamicDependencyPlan;
use nuisc::linker::LinkPlan;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

pub(crate) const ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT: &str =
    "nuis-nsld-elf-dynamic-resolver-provider-registry-v1";

#[derive(Clone, Copy)]
pub(crate) struct DynamicResolverProvider {
    pub(crate) provider_id: &'static str,
    pub(crate) machine_arch: &'static str,
    pub(crate) machine_os: &'static str,
    pub(crate) object_format: &'static str,
    pub(crate) calling_abi: &'static str,
    pub(crate) clang_target: &'static str,
    pub(crate) host_ffi_abi: &'static str,
    pub(crate) interpreter_identity: &'static str,
    pub(crate) interpreter_path: &'static str,
    pub(crate) dependency_identity: &'static str,
    pub(crate) needed_name: &'static str,
    pub(crate) symbol_version_policy: &'static str,
    pub(crate) resolver_identity: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct DynamicSymbolVersionRegistration {
    pub(crate) provider_id: &'static str,
    pub(crate) target_symbol: &'static str,
    pub(crate) version_identity: &'static str,
    pub(crate) version_name: &'static str,
    pub(crate) version_index: u16,
}

const DYNAMIC_RESOLVER_PROVIDERS: &[DynamicResolverProvider] = &[DynamicResolverProvider {
    provider_id: "nsld.elf.amd64.linux-gnu.libc-v1",
    machine_arch: "x86_64",
    machine_os: "linux",
    object_format: "elf",
    calling_abi: "sysv64",
    clang_target: "x86_64-unknown-linux-gnu",
    host_ffi_abi: "libc",
    interpreter_identity: "linux.gnu.ld-so.x86-64-v1",
    interpreter_path: "/lib64/ld-linux-x86-64.so.2",
    dependency_identity: "linux.gnu.libc.so.6-v1",
    needed_name: "libc.so.6",
    symbol_version_policy: "elf-registered-symbol-version-whitelist-v1",
    resolver_identity: "elf.sysv.amd64.bind-now-plt-v1",
}];

const DYNAMIC_SYMBOL_VERSIONS: &[DynamicSymbolVersionRegistration] = &[
    DynamicSymbolVersionRegistration {
        provider_id: "nsld.elf.amd64.linux-gnu.libc-v1",
        target_symbol: "puts",
        version_identity: "linux.gnu.glibc.2.2.5-v1",
        version_name: "GLIBC_2.2.5",
        version_index: 2,
    },
    DynamicSymbolVersionRegistration {
        provider_id: "nsld.elf.amd64.linux-gnu.libc-v1",
        target_symbol: "sched_yield",
        version_identity: "linux.gnu.glibc.2.2.5-v1",
        version_name: "GLIBC_2.2.5",
        version_index: 2,
    },
];

pub(crate) fn matching_dynamic_resolver_providers(
    plan: &LinkPlan,
    host_ffi_abi: &str,
) -> Vec<DynamicResolverProvider> {
    DYNAMIC_RESOLVER_PROVIDERS
        .iter()
        .filter(|provider| {
            provider.machine_arch == plan.cpu_target.machine_arch
                && provider.machine_os == plan.cpu_target.machine_os
                && provider.object_format == plan.cpu_target.object_format
                && provider.calling_abi == plan.cpu_target.calling_abi
                && provider.clang_target == plan.cpu_target.clang_target
                && provider.host_ffi_abi == host_ffi_abi
        })
        .copied()
        .collect()
}

pub(crate) fn registered_dynamic_symbol_version(
    provider_id: &str,
    target_symbol: &str,
) -> Option<DynamicSymbolVersionRegistration> {
    DYNAMIC_SYMBOL_VERSIONS
        .iter()
        .find(|version| {
            version.provider_id == provider_id && version.target_symbol == target_symbol
        })
        .copied()
}

pub(crate) fn validate_dynamic_resolver_provider_registry() -> Result<String, String> {
    let mut provider_ids = BTreeSet::new();
    let mut target_abis = BTreeSet::new();
    let mut canonical = String::new();
    append_text(
        &mut canonical,
        ELF_DYNAMIC_RESOLVER_PROVIDER_REGISTRY_CONTRACT,
    );
    for provider in DYNAMIC_RESOLVER_PROVIDERS {
        let values = provider_values(*provider);
        if values.iter().any(|value| value.is_empty())
            || !provider_ids.insert(provider.provider_id)
            || !target_abis.insert((
                provider.machine_arch,
                provider.machine_os,
                provider.object_format,
                provider.calling_abi,
                provider.clang_target,
                provider.host_ffi_abi,
            ))
            || !provider.interpreter_path.starts_with('/')
            || provider.needed_name.contains('/')
        {
            return Err("invalid ELF dynamic resolver provider registry".to_owned());
        }
        for value in values {
            append_text(&mut canonical, value);
        }
    }
    let mut symbols = BTreeSet::new();
    let mut provider_versions = BTreeMap::new();
    for version in DYNAMIC_SYMBOL_VERSIONS {
        let provider_exists = provider_ids.contains(version.provider_id);
        let version_identity = (version.version_name, version.version_identity);
        let version_index_consistent = provider_versions
            .insert(
                (version.provider_id, version.version_index),
                version_identity,
            )
            .is_none_or(|previous| previous == version_identity);
        if !provider_exists
            || version.target_symbol.is_empty()
            || version.version_identity.is_empty()
            || version.version_name.is_empty()
            || version.version_index < 2
            || !symbols.insert((version.provider_id, version.target_symbol))
            || !version_index_consistent
        {
            return Err("invalid ELF dynamic symbol-version registry".to_owned());
        }
        for value in [
            version.provider_id,
            version.target_symbol,
            version.version_identity,
            version.version_name,
        ] {
            append_text(&mut canonical, value);
        }
        writeln!(
            canonical,
            "version={}|{}",
            version.version_index,
            elf_version_name_hash(version.version_name)
        )
        .unwrap();
    }
    Ok(crate::fnv1a64_hex(canonical.as_bytes()))
}

pub(crate) fn dependency_matches_registered_provider(
    dependency: &ElfAmd64DynamicDependencyPlan,
) -> bool {
    DYNAMIC_RESOLVER_PROVIDERS.iter().any(|provider| {
        dependency.provider_id == provider.provider_id
            && dependency.provider_target_key == provider_target_key(*provider)
            && dependency.host_ffi_abi == provider.host_ffi_abi
            && dependency.interpreter_identity == provider.interpreter_identity
            && dependency.interpreter_path == provider.interpreter_path
            && dependency.dependency_identity == provider.dependency_identity
            && dependency.needed_name == provider.needed_name
            && dependency.symbol_version_policy == provider.symbol_version_policy
            && dependency.resolver_identity == provider.resolver_identity
    })
}

pub(crate) fn provider_target_key(provider: DynamicResolverProvider) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        provider.machine_arch,
        provider.machine_os,
        provider.object_format,
        provider.calling_abi,
        provider.clang_target
    )
}

pub(crate) fn elf_version_name_hash(name: &str) -> u32 {
    let mut hash = 0u32;
    for byte in name.bytes() {
        hash = hash.wrapping_shl(4).wrapping_add(u32::from(byte));
        let high = hash & 0xf000_0000;
        if high != 0 {
            hash ^= high >> 24;
        }
        hash &= !high;
    }
    hash
}

fn provider_values(provider: DynamicResolverProvider) -> [&'static str; 13] {
    [
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
    ]
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_glibc_versions_use_the_elf_hash_and_shared_index() {
        assert_eq!(elf_version_name_hash("GLIBC_2.2.5"), 0x0969_1a75);
        let puts =
            registered_dynamic_symbol_version("nsld.elf.amd64.linux-gnu.libc-v1", "puts").unwrap();
        let sched =
            registered_dynamic_symbol_version("nsld.elf.amd64.linux-gnu.libc-v1", "sched_yield")
                .unwrap();
        assert_eq!(puts.version_index, sched.version_index);
        assert_eq!(puts.version_name, sched.version_name);
        assert!(validate_dynamic_resolver_provider_registry().is_ok());
    }
}

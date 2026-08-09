use crate::{
    CompiledEntryTransferResult, NativeLifecycleEntryContextV1, NativeRuntimeDispatchTableV1,
    NATIVE_RUNTIME_DISPATCH_IMPORT_KIND, NATIVE_RUNTIME_DISPATCH_IMPORT_NAME,
    NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER, NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES,
    NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1, NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2,
};

pub const RUNTIME_DISPATCH_IMPORT_RESOLUTION_PROTOCOL: &str =
    "nuis-runtime-dispatch-import-resolution-v1";
pub const RUNTIME_DISPATCH_IMPORT_BINDING_PROTOCOL: &str =
    "nuis-runtime-dispatch-import-binding-v1";
pub const RUNTIME_DISPATCH_IMPORT_IDENTITY_CONTRACT: &str =
    "nuis-runtime-dispatch-import-identity-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDispatchImportDeclaration {
    pub import_kind: String,
    pub import_name: String,
    pub provider: String,
    pub required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeDispatchImportFacts {
    pub declarations: Vec<RuntimeDispatchImportDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRuntimeDispatchImport {
    pub protocol: &'static str,
    pub import_identity_hash: String,
    pub import_kind: String,
    pub import_name: String,
    pub provider: String,
    pub required: bool,
    pub capability_mask: u64,
    pub table_identity: Option<u64>,
}

impl ResolvedRuntimeDispatchImport {
    fn registered() -> Self {
        let mut binding = Self {
            protocol: RUNTIME_DISPATCH_IMPORT_BINDING_PROTOCOL,
            import_identity_hash: String::new(),
            import_kind: NATIVE_RUNTIME_DISPATCH_IMPORT_KIND.to_owned(),
            import_name: NATIVE_RUNTIME_DISPATCH_IMPORT_NAME.to_owned(),
            provider: NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER.to_owned(),
            required: true,
            capability_mask: NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES,
            table_identity: None,
        };
        binding.import_identity_hash = binding.derived_identity_hash();
        binding
    }

    pub(crate) fn validate_static(&self) -> Result<(), String> {
        if self.protocol != RUNTIME_DISPATCH_IMPORT_BINDING_PROTOCOL
            || self.import_kind != NATIVE_RUNTIME_DISPATCH_IMPORT_KIND
            || self.import_name != NATIVE_RUNTIME_DISPATCH_IMPORT_NAME
            || self.provider != NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER
            || !self.required
            || self.capability_mask != NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES
        {
            return Err("runtime-dispatch-binding:contract-mismatch".to_owned());
        }
        if self.import_identity_hash != self.derived_identity_hash() {
            return Err("runtime-dispatch-binding:identity-mismatch".to_owned());
        }
        Ok(())
    }

    pub(crate) fn materialize(
        &mut self,
        context: &NativeLifecycleEntryContextV1,
    ) -> Result<(), String> {
        self.validate_static()?;
        let table = NativeRuntimeDispatchTableV1::from_context(context);
        if table.capability_mask() != self.capability_mask {
            return Err("runtime-dispatch-binding:capability-mask-mismatch".to_owned());
        }
        self.table_identity = Some(table.identity());
        Ok(())
    }

    pub(crate) fn validate_materialized(
        &self,
        context: &NativeLifecycleEntryContextV1,
    ) -> Result<(), String> {
        self.validate_static()?;
        let table = NativeRuntimeDispatchTableV1::from_context(context);
        if self.table_identity != Some(table.identity()) {
            return Err("runtime-dispatch-binding:table-identity-mismatch".to_owned());
        }
        if self.capability_mask != table.capability_mask() {
            return Err("runtime-dispatch-binding:capability-mask-mismatch".to_owned());
        }
        Ok(())
    }

    fn derived_identity_hash(&self) -> String {
        fnv1a64_hex(
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                RUNTIME_DISPATCH_IMPORT_IDENTITY_CONTRACT,
                self.import_kind,
                self.import_name,
                self.provider,
                self.required,
                self.capability_mask
            )
            .as_bytes(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDispatchImportResolution {
    pub protocol: &'static str,
    pub status: &'static str,
    pub ready: bool,
    pub declared: bool,
    pub binding: Option<ResolvedRuntimeDispatchImport>,
    pub blockers: Vec<String>,
}

pub fn resolve_runtime_dispatch_import(
    entry_abi_contract: &str,
    facts: &RuntimeDispatchImportFacts,
) -> RuntimeDispatchImportResolution {
    let declared = !facts.declarations.is_empty();
    if entry_abi_contract == NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1 {
        return if declared {
            blocked(
                true,
                vec!["runtime-dispatch-import:legacy-abi-declaration-forbidden".to_owned()],
            )
        } else {
            RuntimeDispatchImportResolution {
                protocol: RUNTIME_DISPATCH_IMPORT_RESOLUTION_PROTOCOL,
                status: "legacy-absent",
                ready: true,
                declared: false,
                binding: None,
                blockers: Vec::new(),
            }
        };
    }
    if entry_abi_contract != NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2 {
        return blocked(
            declared,
            vec!["runtime-dispatch-import:entry-abi-unsupported".to_owned()],
        );
    }

    let mut blockers = Vec::new();
    if facts.declarations.len() != 1 {
        blockers.push("runtime-dispatch-import:declaration-count-invalid".to_owned());
    }
    if let Some(declaration) = facts.declarations.first() {
        if declaration.import_kind != NATIVE_RUNTIME_DISPATCH_IMPORT_KIND {
            blockers.push("runtime-dispatch-import:kind-unsupported".to_owned());
        }
        if declaration.import_name != NATIVE_RUNTIME_DISPATCH_IMPORT_NAME {
            blockers.push("runtime-dispatch-import:name-unsupported".to_owned());
        }
        if declaration.provider != NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER {
            blockers.push("runtime-dispatch-import:provider-unsupported".to_owned());
        }
        if !declaration.required {
            blockers.push("runtime-dispatch-import:must-be-required".to_owned());
        }
    }
    blockers.sort();
    blockers.dedup();
    if !blockers.is_empty() {
        return blocked(declared, blockers);
    }

    RuntimeDispatchImportResolution {
        protocol: RUNTIME_DISPATCH_IMPORT_RESOLUTION_PROTOCOL,
        status: "resolved-static",
        ready: true,
        declared: true,
        binding: Some(ResolvedRuntimeDispatchImport::registered()),
        blockers: Vec::new(),
    }
}

pub(crate) fn materialize_transfer_runtime_dispatch(
    transfer: &mut CompiledEntryTransferResult,
    context: &NativeLifecycleEntryContextV1,
) -> Result<(), String> {
    match transfer.entry_abi_contract.as_deref() {
        Some(NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1) => {
            if transfer.runtime_dispatch_import.is_some() {
                return Err("runtime-dispatch-binding:legacy-abi-binding-forbidden".to_owned());
            }
        }
        Some(NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2) => {
            transfer
                .runtime_dispatch_import
                .as_mut()
                .ok_or_else(|| "runtime-dispatch-binding:required-binding-missing".to_owned())?
                .materialize(context)?;
        }
        _ => return Err("runtime-dispatch-binding:entry-abi-unsupported".to_owned()),
    }
    Ok(())
}

pub(crate) fn validate_transfer_runtime_dispatch(
    transfer: &CompiledEntryTransferResult,
    context: &NativeLifecycleEntryContextV1,
) -> Result<(), String> {
    match transfer.entry_abi_contract.as_deref() {
        Some(NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1) => {
            if transfer.runtime_dispatch_import.is_some() {
                return Err("runtime-dispatch-binding:legacy-abi-binding-forbidden".to_owned());
            }
        }
        Some(NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2) => transfer
            .runtime_dispatch_import
            .as_ref()
            .ok_or_else(|| "runtime-dispatch-binding:required-binding-missing".to_owned())?
            .validate_materialized(context)?,
        _ => return Err("runtime-dispatch-binding:entry-abi-unsupported".to_owned()),
    }
    Ok(())
}

fn blocked(declared: bool, blockers: Vec<String>) -> RuntimeDispatchImportResolution {
    RuntimeDispatchImportResolution {
        protocol: RUNTIME_DISPATCH_IMPORT_RESOLUTION_PROTOCOL,
        status: "blocked",
        ready: false,
        declared,
        binding: None,
        blockers,
    }
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registered_facts(provider: &str) -> RuntimeDispatchImportFacts {
        RuntimeDispatchImportFacts {
            declarations: vec![RuntimeDispatchImportDeclaration {
                import_kind: NATIVE_RUNTIME_DISPATCH_IMPORT_KIND.to_owned(),
                import_name: NATIVE_RUNTIME_DISPATCH_IMPORT_NAME.to_owned(),
                provider: provider.to_owned(),
                required: true,
            }],
        }
    }

    #[test]
    fn dispatch_aware_abi_requires_one_registered_import() {
        let missing = resolve_runtime_dispatch_import(
            NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2,
            &RuntimeDispatchImportFacts::default(),
        );
        assert!(!missing.ready);
        assert!(missing
            .blockers
            .contains(&"runtime-dispatch-import:declaration-count-invalid".to_owned()));

        let forged = resolve_runtime_dispatch_import(
            NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2,
            &registered_facts("host-special-case"),
        );
        assert!(!forged.ready);
        assert!(forged
            .blockers
            .contains(&"runtime-dispatch-import:provider-unsupported".to_owned()));
    }

    #[test]
    fn legacy_abi_is_explicitly_import_free() {
        let legacy = resolve_runtime_dispatch_import(
            NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1,
            &RuntimeDispatchImportFacts::default(),
        );
        assert!(legacy.ready);
        assert_eq!(legacy.status, "legacy-absent");
        assert!(legacy.binding.is_none());

        let ambiguous = resolve_runtime_dispatch_import(
            NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1,
            &registered_facts(NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER),
        );
        assert!(!ambiguous.ready);
    }

    #[test]
    fn registered_import_has_a_stable_provider_neutral_identity() {
        let resolution = resolve_runtime_dispatch_import(
            NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2,
            &registered_facts(NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER),
        );
        let binding = resolution.binding.expect("registered import resolves");
        binding.validate_static().unwrap();
        assert!(binding.import_identity_hash.starts_with("0x"));
        assert_eq!(binding.table_identity, None);
    }
}

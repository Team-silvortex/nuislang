use crate::container::ExternalImportSummary;
use nuis_runtime::{NativeLifecycleEntryContextV1, NativeRuntimeDispatchTableV1};

pub(super) const RUNTIME_DISPATCH_RESOLUTION_PROTOCOL: &str =
    "nuis-host-runtime-dispatch-resolution-v1";

pub(super) struct RuntimeDispatchResolution {
    pub(super) status: &'static str,
    pub(super) ready: bool,
    pub(super) declared: bool,
    pub(super) table_identity: Option<u64>,
    pub(super) capability_mask: Option<u64>,
    pub(super) blockers: Vec<String>,
}

pub(super) fn resolve_runtime_dispatch(
    imports: &ExternalImportSummary,
    context: &NativeLifecycleEntryContextV1,
) -> RuntimeDispatchResolution {
    let candidates = imports
        .entries
        .iter()
        .filter(|entry| entry.import_kind == nuis_runtime::NATIVE_RUNTIME_DISPATCH_IMPORT_KIND)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return RuntimeDispatchResolution {
            status: "not-declared",
            ready: true,
            declared: false,
            table_identity: None,
            capability_mask: None,
            blockers: Vec::new(),
        };
    }

    let mut blockers = Vec::new();
    if candidates.len() != 1 {
        blockers.push("runtime-dispatch-import:declaration-count-invalid".to_owned());
    }
    if let Some(import) = candidates.first() {
        if import.import_name != nuis_runtime::NATIVE_RUNTIME_DISPATCH_IMPORT_NAME {
            blockers.push("runtime-dispatch-import:name-unsupported".to_owned());
        }
        if import.provider != nuis_runtime::NATIVE_RUNTIME_DISPATCH_IMPORT_PROVIDER {
            blockers.push("runtime-dispatch-import:provider-unsupported".to_owned());
        }
        if !import.required {
            blockers.push("runtime-dispatch-import:must-be-required".to_owned());
        }
    }
    blockers.sort();
    blockers.dedup();
    if !blockers.is_empty() {
        return RuntimeDispatchResolution {
            status: "blocked",
            ready: false,
            declared: true,
            table_identity: None,
            capability_mask: None,
            blockers,
        };
    }

    let table = NativeRuntimeDispatchTableV1::from_context(context);
    RuntimeDispatchResolution {
        status: "resolved",
        ready: true,
        declared: true,
        table_identity: Some(table.identity()),
        capability_mask: Some(table.capability_mask()),
        blockers: Vec::new(),
    }
}

use crate::container::ExternalImportSummary;
use nuis_runtime::{RuntimeDispatchImportDeclaration, RuntimeDispatchImportFacts};

pub(super) fn runtime_dispatch_import_facts(
    imports: &ExternalImportSummary,
) -> RuntimeDispatchImportFacts {
    RuntimeDispatchImportFacts {
        declarations: imports
            .entries
            .iter()
            .filter(|entry| entry.import_kind == nuis_runtime::NATIVE_RUNTIME_DISPATCH_IMPORT_KIND)
            .map(|entry| RuntimeDispatchImportDeclaration {
                import_kind: entry.import_kind.clone(),
                import_name: entry.import_name.clone(),
                provider: entry.provider.clone(),
                required: entry.required,
            })
            .collect(),
    }
}

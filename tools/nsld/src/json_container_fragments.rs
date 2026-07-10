use super::{
    container::{
        NsldContainerCompatibilityDomain, NsldContainerExternalImport, NsldContainerLoaderSymbol,
        NsldContainerRelocationEntry,
    },
    json_fields::*,
};

pub(crate) fn nsld_container_loader_symbols_json(symbols: &[NsldContainerLoaderSymbol]) -> String {
    symbols
        .iter()
        .map(|symbol| {
            let fields = [
                json_string_field("symbol_id", &symbol.symbol_id),
                json_string_field("symbol_kind", &symbol.symbol_kind),
                json_string_field("symbol_name", &symbol.symbol_name),
                json_string_field("lifecycle_hook", &symbol.lifecycle_hook),
                json_string_field("section_id", &symbol.section_id),
                json_usize_field("offset", symbol.offset),
                json_usize_field("size_bytes", symbol.size_bytes),
                json_string_field("payload_hash", &symbol.payload_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_container_relocations_json(
    relocations: &[NsldContainerRelocationEntry],
) -> String {
    relocations
        .iter()
        .map(|relocation| {
            let fields = [
                json_string_field("relocation_id", &relocation.relocation_id),
                json_string_field("relocation_kind", &relocation.relocation_kind),
                json_string_field("source_section_id", &relocation.source_section_id),
                json_usize_field("source_offset", relocation.source_offset),
                json_string_field("target_symbol_id", &relocation.target_symbol_id),
                json_isize_field("addend", relocation.addend),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_container_external_imports_json(
    imports: &[NsldContainerExternalImport],
) -> String {
    imports
        .iter()
        .map(|external_import| {
            let fields = [
                json_string_field("import_id", &external_import.import_id),
                json_string_field("import_kind", &external_import.import_kind),
                json_string_field("import_name", &external_import.import_name),
                json_string_field("provider", &external_import.provider),
                json_bool_field("required", external_import.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_container_compatibility_domains_json(
    domains: &[NsldContainerCompatibilityDomain],
) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = [
                json_string_field("domain_id", &domain.domain_id),
                json_string_field("domain_kind", &domain.domain_kind),
                json_string_field("paradigm", &domain.paradigm),
                json_string_field("lifecycle_hook", &domain.lifecycle_hook),
                json_string_field("abi_family", &domain.abi_family),
                json_string_field("wrapper_policy", &domain.wrapper_policy),
                json_bool_field("required", domain.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

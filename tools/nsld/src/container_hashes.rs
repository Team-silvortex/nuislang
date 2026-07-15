use super::container_model::*;

pub(crate) fn loader_symbol_table_hash(
    symbols: &[NsldContainerLoaderSymbol],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    for symbol in symbols {
        material.push_str(&symbol.symbol_id);
        material.push('\t');
        material.push_str(&symbol.symbol_kind);
        material.push('\t');
        material.push_str(&symbol.symbol_name);
        material.push('\t');
        material.push_str(&symbol.lifecycle_hook);
        material.push('\t');
        material.push_str(&symbol.section_id);
        material.push('\t');
        material.push_str(&symbol.offset.to_string());
        material.push('\t');
        material.push_str(&symbol.size_bytes.to_string());
        material.push('\t');
        material.push_str(&symbol.payload_hash);
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn external_import_table_hash(
    imports: &[NsldContainerExternalImport],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    for import in imports {
        material.push_str(&import.import_id);
        material.push('\t');
        material.push_str(&import.import_kind);
        material.push('\t');
        material.push_str(&import.import_name);
        material.push('\t');
        material.push_str(&import.provider);
        material.push('\t');
        material.push_str(if import.required { "true" } else { "false" });
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn backend_artifact_payload_table_hash(
    payloads: &[super::container_model::NsldContainerBackendArtifactPayload],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    for payload in payloads {
        material.push_str(&payload.payload_id);
        material.push('\t');
        material.push_str(&payload.domain_family);
        material.push('\t');
        material.push_str(&payload.backend_family);
        material.push('\t');
        material.push_str(&payload.target_device);
        material.push('\t');
        material.push_str(&payload.payload_format);
        material.push('\t');
        material.push_str(&payload.payload_path);
        material.push('\t');
        material.push_str(&payload.role_status);
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn compatibility_domain_table_hash(
    domains: &[NsldContainerCompatibilityDomain],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    for domain in domains {
        material.push_str(&domain.domain_id);
        material.push('\t');
        material.push_str(&domain.domain_kind);
        material.push('\t');
        material.push_str(&domain.paradigm);
        material.push('\t');
        material.push_str(&domain.lifecycle_hook);
        material.push('\t');
        material.push_str(&domain.abi_family);
        material.push('\t');
        material.push_str(&domain.wrapper_policy);
        material.push('\t');
        material.push_str(if domain.required { "true" } else { "false" });
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn relocation_table_hash(
    relocations: &[NsldContainerRelocationEntry],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    for relocation in relocations {
        material.push_str(&relocation.relocation_id);
        material.push('\t');
        material.push_str(&relocation.relocation_kind);
        material.push('\t');
        material.push_str(&relocation.source_section_id);
        material.push('\t');
        material.push_str(&relocation.source_offset.to_string());
        material.push('\t');
        material.push_str(&relocation.target_symbol_id);
        material.push('\t');
        material.push_str(&relocation.addend.to_string());
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn container_section_table_hash(
    sections: &[NsldContainerSectionEntry],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_path);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\t');
        material.push_str(&section.payload_hash);
        material.push('\t');
        material.push_str(if section.required { "true" } else { "false" });
        material.push('\t');
        material.push_str(&section.offset.to_string());
        material.push('\t');
        material.push_str(&section.size_bytes.to_string());
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn metadata_table_hash(
    container_section_table_hash: &str,
    loader_symbol_table_hash: &str,
    relocation_table_hash: &str,
    compatibility_domain_table_hash: &str,
    external_import_table_hash: &str,
    backend_artifact_payload_table_hash: &str,
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let material = format!(
        "{container_section_table_hash}\t{loader_symbol_table_hash}\t{relocation_table_hash}\t{compatibility_domain_table_hash}\t{external_import_table_hash}\t{backend_artifact_payload_table_hash}\n"
    );
    hash_bytes(material.as_bytes())
}

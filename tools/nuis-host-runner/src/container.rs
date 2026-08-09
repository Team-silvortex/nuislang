use crate::{
    container_backend_payload::{scan_backend_artifact_payloads, BackendArtifactPayloadSummary},
    container_bootstrap::{
        relocation_records, section_records, ContainerRelocationRecord, ContainerSectionRecord,
    },
    container_metadata_binding::{scan_metadata_bindings, MetadataBindingSummary},
    container_provider_dispatch::{scan_provider_dispatch, ProviderDispatchSummary},
    container_toml::{
        array_table_blocks, bool_value, bool_value_from_lines, first_array_table_block,
        string_array_value, string_value, string_value_from_lines, usize_value,
        usize_value_from_lines,
    },
};

pub(super) const CONTAINER_SCHEMA: &str = "nuis-nsld-container-v1";
pub(super) const CONTAINER_SCHEMA_VERSION: usize = 1;
pub(super) const CONTAINER_KIND: &str = "deterministic-hetero-container";
pub(super) const CONTAINER_PRODUCER: &str = "nsld";
pub(super) const CONTAINER_MAGIC: &str = "NUISNSLD";
pub(super) const CONTAINER_VERSION: usize = 1;
const CONTAINER_CAPSULE_END_MARKER: &[u8] = b"\n# nuis-nsld-container-end-v1\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContainerLoaderSymbolSummary {
    pub(super) status: String,
    pub(super) symbol_id: Option<String>,
    pub(super) symbol_kind: Option<String>,
    pub(super) symbol_name: Option<String>,
    pub(super) lifecycle_hook: Option<String>,
    pub(super) section_id: Option<String>,
    pub(super) offset: Option<usize>,
    pub(super) size_bytes: Option<usize>,
    pub(super) payload_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContainerSectionSummary {
    pub(super) status: String,
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) first_section_id: Option<String>,
    pub(super) first_section_kind: Option<String>,
    pub(super) entry_section_found: bool,
    pub(super) entries: Vec<ContainerSectionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContainerRelocationSummary {
    pub(super) status: String,
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) first_relocation_kind: Option<String>,
    pub(super) first_source_section_id: Option<String>,
    pub(super) first_target_symbol_id: Option<String>,
    pub(super) first_targets_loader_symbol: bool,
    pub(super) first_source_matches_loader_symbol: bool,
    pub(super) entries: Vec<ContainerRelocationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CompatibilityDomainSummary {
    pub(super) status: String,
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) first_domain_kind: Option<String>,
    pub(super) required_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExternalImportEntry {
    pub(super) import_kind: String,
    pub(super) import_name: String,
    pub(super) provider: String,
    pub(super) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExternalImportSummary {
    pub(super) status: String,
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) first_import_kind: Option<String>,
    pub(super) first_import_name: Option<String>,
    pub(super) required_imports: Vec<String>,
    pub(super) entries: Vec<ExternalImportEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContainerLoaderSummary {
    pub(super) status: String,
    pub(super) container_schema: Option<String>,
    pub(super) container_schema_version: Option<usize>,
    pub(super) container_kind: Option<String>,
    pub(super) container_producer: Option<String>,
    pub(super) container_producer_phase: Option<String>,
    pub(super) container_ready: Option<bool>,
    pub(super) container_blockers: Vec<String>,
    pub(super) container_magic: Option<String>,
    pub(super) container_version: Option<usize>,
    pub(super) container_metadata_table_hash: Option<String>,
    pub(super) container_section_table_hash: Option<String>,
    pub(super) container_hash: Option<String>,
    pub(super) container_section: ContainerSectionSummary,
    pub(super) compatibility_domain: CompatibilityDomainSummary,
    pub(super) external_import: ExternalImportSummary,
    pub(super) backend_artifact_payload: BackendArtifactPayloadSummary,
    pub(super) metadata_binding: MetadataBindingSummary,
    pub(super) provider_dispatch: ProviderDispatchSummary,
    pub(super) loader_symbol_table_hash: Option<String>,
    pub(super) relocation_table_hash: Option<String>,
    pub(super) compatibility_domain_table_hash: Option<String>,
    pub(super) external_import_table_hash: Option<String>,
    pub(super) backend_artifact_payload_table_hash: Option<String>,
    pub(super) container_payload_size_bytes: Option<usize>,
    pub(super) container_payload_hash: Option<String>,
    pub(super) container_payload_path: Option<String>,
    pub(super) loader_readiness: Option<String>,
    pub(super) loader_blockers: Vec<String>,
    pub(super) loader_entry_kind: Option<String>,
    pub(super) loader_entry_abi_contract: Option<String>,
    pub(super) loader_entry_machine_arch: Option<String>,
    pub(super) loader_entry_symbol: Option<String>,
    pub(super) loader_entry_section_id: Option<String>,
    pub(super) loader_symbol_count: Option<usize>,
    pub(super) loader_symbol: ContainerLoaderSymbolSummary,
    pub(super) relocation: ContainerRelocationSummary,
    pub(super) handoff_status: String,
    pub(super) handoff_ready: bool,
    pub(super) handoff_blockers: Vec<String>,
}

pub(super) fn scan_container_loader(
    region: Option<&[u8]>,
    payload_kind: &str,
) -> ContainerLoaderSummary {
    if !matches!(payload_kind, "nsld-container-toml" | "toml-like") {
        return empty_container_loader("not-container-toml", Vec::new());
    }
    let Some(region) = region else {
        return empty_container_loader("not-mapped", Vec::new());
    };
    let source_region = container_capsule_region(region);
    let Ok(source) = std::str::from_utf8(source_region) else {
        return empty_container_loader(
            "invalid-utf8",
            vec!["container-loader:invalid-utf8".to_owned()],
        );
    };
    let container_schema = string_value(source, "schema");
    let container_schema_version = usize_value(source, "schema_version");
    let container_kind = string_value(source, "container_kind");
    let container_producer = string_value(source, "producer");
    let container_producer_phase = string_value(source, "producer_phase");
    let container_ready = bool_value(source, "ready");
    let container_blockers = string_array_value(source, "blockers");
    let container_magic = string_value(source, "container_magic");
    let container_version = usize_value(source, "container_version");
    let container_metadata_table_hash = string_value(source, "metadata_table_hash");
    let container_section_table_hash = string_value(source, "container_section_table_hash");
    let container_hash = string_value(source, "container_hash");
    let container_section_count = usize_value(source, "section_count");
    let compatibility_domain_count = usize_value(source, "compatibility_domain_count");
    let external_import_count = usize_value(source, "external_import_count");
    let backend_artifact_payload_count = usize_value(source, "backend_artifact_payload_count");
    let loader_symbol_table_hash = string_value(source, "loader_symbol_table_hash");
    let relocation_table_hash = string_value(source, "relocation_table_hash");
    let compatibility_domain_table_hash = string_value(source, "compatibility_domain_table_hash");
    let external_import_table_hash = string_value(source, "external_import_table_hash");
    let backend_artifact_payload_table_hash =
        string_value(source, "backend_artifact_payload_table_hash");
    let container_payload_size_bytes = usize_value(source, "payload_size_bytes");
    let container_payload_hash = string_value(source, "payload_hash");
    let container_payload_path = string_value(source, "payload_path");
    let loader_readiness = string_value(source, "loader_readiness");
    let loader_blockers = string_array_value(source, "loader_blockers");
    let loader_entry_kind = string_value(source, "loader_entry_kind");
    let loader_entry_abi_contract = string_value(source, "loader_entry_abi_contract");
    let loader_entry_machine_arch = string_value(source, "loader_entry_machine_arch");
    let loader_entry_symbol = string_value(source, "loader_entry_symbol");
    let loader_entry_section_id = string_value(source, "loader_entry_section_id");
    let loader_symbol_count = usize_value(source, "loader_symbol_count");
    let loader_symbol = scan_first_loader_symbol(source);
    let relocation_count = usize_value(source, "relocation_count");
    let relocation = scan_relocations(source, relocation_count, &loader_symbol);
    let container_section = scan_container_sections(
        source,
        container_section_count,
        loader_entry_section_id.as_deref(),
    );
    let compatibility_domain = scan_compatibility_domains(source, compatibility_domain_count);
    let external_import = scan_external_imports(source, external_import_count);
    let backend_artifact_payload =
        scan_backend_artifact_payloads(source, backend_artifact_payload_count);
    let metadata_binding = scan_metadata_bindings(source);
    let provider_dispatch = scan_provider_dispatch(source, &metadata_binding);
    let mut summary = ContainerLoaderSummary {
        status: "parsed".to_owned(),
        container_schema,
        container_schema_version,
        container_kind,
        container_producer,
        container_producer_phase,
        container_ready,
        container_blockers,
        container_magic,
        container_version,
        container_metadata_table_hash,
        container_section_table_hash,
        container_hash,
        container_section,
        compatibility_domain,
        external_import,
        backend_artifact_payload,
        metadata_binding,
        provider_dispatch,
        loader_symbol_table_hash,
        relocation_table_hash,
        compatibility_domain_table_hash,
        external_import_table_hash,
        backend_artifact_payload_table_hash,
        container_payload_size_bytes,
        container_payload_hash,
        container_payload_path,
        loader_readiness,
        loader_blockers,
        loader_entry_kind,
        loader_entry_abi_contract,
        loader_entry_machine_arch,
        loader_entry_symbol,
        loader_entry_section_id,
        loader_symbol_count,
        loader_symbol,
        relocation,
        handoff_status: "blocked".to_owned(),
        handoff_ready: false,
        handoff_blockers: Vec::new(),
    };
    summary.handoff_blockers = container_loader_handoff_blockers(&summary);
    summary.handoff_ready = summary.handoff_blockers.is_empty();
    summary.handoff_status = if summary.handoff_ready {
        "ready"
    } else {
        "blocked"
    }
    .to_owned();
    summary
}

fn container_capsule_region(region: &[u8]) -> &[u8] {
    if let Some(offset) = region
        .windows(CONTAINER_CAPSULE_END_MARKER.len())
        .position(|window| window == CONTAINER_CAPSULE_END_MARKER)
    {
        return &region[..offset + CONTAINER_CAPSULE_END_MARKER.len()];
    }
    region
        .iter()
        .position(|byte| *byte == 0)
        .and_then(|end| region.get(..end))
        .unwrap_or(region)
}

fn empty_container_loader(status: &str, handoff_blockers: Vec<String>) -> ContainerLoaderSummary {
    ContainerLoaderSummary {
        status: status.to_owned(),
        container_schema: None,
        container_schema_version: None,
        container_kind: None,
        container_producer: None,
        container_producer_phase: None,
        container_ready: None,
        container_blockers: Vec::new(),
        container_magic: None,
        container_version: None,
        container_metadata_table_hash: None,
        container_section_table_hash: None,
        container_hash: None,
        container_section: ContainerSectionSummary::empty(status),
        compatibility_domain: CompatibilityDomainSummary::empty(status),
        external_import: ExternalImportSummary::empty(status),
        backend_artifact_payload: BackendArtifactPayloadSummary::empty(status),
        metadata_binding: MetadataBindingSummary::not_applicable(),
        provider_dispatch: ProviderDispatchSummary::not_applicable(None, None),
        loader_symbol_table_hash: None,
        relocation_table_hash: None,
        compatibility_domain_table_hash: None,
        external_import_table_hash: None,
        backend_artifact_payload_table_hash: None,
        container_payload_size_bytes: None,
        container_payload_hash: None,
        container_payload_path: None,
        loader_readiness: None,
        loader_blockers: Vec::new(),
        loader_entry_kind: None,
        loader_entry_abi_contract: None,
        loader_entry_machine_arch: None,
        loader_entry_symbol: None,
        loader_entry_section_id: None,
        loader_symbol_count: None,
        loader_symbol: ContainerLoaderSymbolSummary::empty(status),
        relocation: ContainerRelocationSummary::empty(status),
        handoff_status: status.to_owned(),
        handoff_ready: false,
        handoff_blockers,
    }
}

impl ContainerLoaderSymbolSummary {
    fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            symbol_id: None,
            symbol_kind: None,
            symbol_name: None,
            lifecycle_hook: None,
            section_id: None,
            offset: None,
            size_bytes: None,
            payload_hash: None,
        }
    }
}

impl ContainerSectionSummary {
    fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            declared_count: None,
            parsed_count: 0,
            first_section_id: None,
            first_section_kind: None,
            entry_section_found: false,
            entries: Vec::new(),
        }
    }
}

impl ContainerRelocationSummary {
    fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            declared_count: None,
            parsed_count: 0,
            first_relocation_kind: None,
            first_source_section_id: None,
            first_target_symbol_id: None,
            first_targets_loader_symbol: false,
            first_source_matches_loader_symbol: false,
            entries: Vec::new(),
        }
    }
}

impl CompatibilityDomainSummary {
    fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            declared_count: None,
            parsed_count: 0,
            first_domain_kind: None,
            required_count: 0,
        }
    }
}

impl ExternalImportSummary {
    fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            declared_count: None,
            parsed_count: 0,
            first_import_kind: None,
            first_import_name: None,
            required_imports: Vec::new(),
            entries: Vec::new(),
        }
    }
}

fn scan_first_loader_symbol(source: &str) -> ContainerLoaderSymbolSummary {
    let Some(block) = first_array_table_block(source, "loader_symbol") else {
        return ContainerLoaderSymbolSummary::empty("missing");
    };
    ContainerLoaderSymbolSummary {
        status: "parsed".to_owned(),
        symbol_id: string_value_from_lines(&block, "symbol_id"),
        symbol_kind: string_value_from_lines(&block, "symbol_kind"),
        symbol_name: string_value_from_lines(&block, "symbol_name"),
        lifecycle_hook: string_value_from_lines(&block, "lifecycle_hook"),
        section_id: string_value_from_lines(&block, "section_id"),
        offset: usize_value_from_lines(&block, "offset"),
        size_bytes: usize_value_from_lines(&block, "size_bytes"),
        payload_hash: string_value_from_lines(&block, "payload_hash"),
    }
}

fn scan_compatibility_domains(
    source: &str,
    declared_count: Option<usize>,
) -> CompatibilityDomainSummary {
    let blocks = array_table_blocks(source, "compatibility_domain");
    let first = blocks.first();
    let required_count = blocks
        .iter()
        .filter(|block| bool_value_from_lines(block, "required").unwrap_or(false))
        .count();
    CompatibilityDomainSummary {
        status: if blocks.is_empty() {
            "missing".to_owned()
        } else {
            "parsed".to_owned()
        },
        declared_count,
        parsed_count: blocks.len(),
        first_domain_kind: first.and_then(|block| string_value_from_lines(block, "domain_kind")),
        required_count,
    }
}

fn scan_external_imports(source: &str, declared_count: Option<usize>) -> ExternalImportSummary {
    let blocks = array_table_blocks(source, "external_import");
    let first = blocks.first();
    let required_imports = blocks
        .iter()
        .filter(|block| bool_value_from_lines(block, "required").unwrap_or(false))
        .map(|block| {
            let kind = string_value_from_lines(block, "import_kind")
                .unwrap_or_else(|| "<unknown-kind>".to_owned());
            let name = string_value_from_lines(block, "import_name")
                .unwrap_or_else(|| "<unknown-name>".to_owned());
            format!("{kind}:{name}")
        })
        .collect();
    let entries = blocks
        .iter()
        .filter_map(|block| {
            Some(ExternalImportEntry {
                import_kind: string_value_from_lines(block, "import_kind")?,
                import_name: string_value_from_lines(block, "import_name")?,
                provider: string_value_from_lines(block, "provider")?,
                required: bool_value_from_lines(block, "required")?,
            })
        })
        .collect();
    ExternalImportSummary {
        status: if blocks.is_empty() {
            "missing".to_owned()
        } else {
            "parsed".to_owned()
        },
        declared_count,
        parsed_count: blocks.len(),
        first_import_kind: first.and_then(|block| string_value_from_lines(block, "import_kind")),
        first_import_name: first.and_then(|block| string_value_from_lines(block, "import_name")),
        required_imports,
        entries,
    }
}

fn scan_relocations(
    source: &str,
    declared_count: Option<usize>,
    loader_symbol: &ContainerLoaderSymbolSummary,
) -> ContainerRelocationSummary {
    let entries = relocation_records(source);
    let first = entries.first();
    let first_target_symbol_id = first.map(|entry| entry.target_symbol_id.clone());
    let first_source_section_id = first.map(|entry| entry.source_section_id.clone());
    ContainerRelocationSummary {
        status: if entries.is_empty() {
            "missing".to_owned()
        } else {
            "parsed".to_owned()
        },
        declared_count,
        parsed_count: entries.len(),
        first_relocation_kind: first.map(|entry| entry.relocation_kind.clone()),
        first_targets_loader_symbol: first_target_symbol_id.as_deref()
            == loader_symbol.symbol_id.as_deref(),
        first_source_matches_loader_symbol: first_source_section_id.as_deref()
            == loader_symbol.section_id.as_deref(),
        first_source_section_id,
        first_target_symbol_id,
        entries,
    }
}

fn scan_container_sections(
    source: &str,
    declared_count: Option<usize>,
    loader_entry_section_id: Option<&str>,
) -> ContainerSectionSummary {
    let entries = section_records(source);
    let first = entries.first();
    let entry_section_found = loader_entry_section_id
        .is_some_and(|entry| entries.iter().any(|section| section.section_id == entry));
    ContainerSectionSummary {
        status: if entries.is_empty() {
            "missing".to_owned()
        } else {
            "parsed".to_owned()
        },
        declared_count,
        parsed_count: entries.len(),
        first_section_id: first.map(|section| section.section_id.clone()),
        first_section_kind: first.map(|section| section.section_kind.clone()),
        entry_section_found,
        entries,
    }
}

fn container_loader_handoff_blockers(summary: &ContainerLoaderSummary) -> Vec<String> {
    let container_schema = summary.container_schema.as_deref();
    let container_schema_version = summary.container_schema_version;
    let container_kind = summary.container_kind.as_deref();
    let container_producer = summary.container_producer.as_deref();
    let container_ready = summary.container_ready;
    let container_blockers = &summary.container_blockers;
    let container_magic = summary.container_magic.as_deref();
    let container_version = summary.container_version;
    let container_section = &summary.container_section;
    let compatibility_domain = &summary.compatibility_domain;
    let external_import = &summary.external_import;
    let container_payload_size_bytes = summary.container_payload_size_bytes;
    let container_payload_hash = summary.container_payload_hash.as_deref();
    let loader_readiness = summary.loader_readiness.as_deref();
    let loader_blockers = &summary.loader_blockers;
    let loader_entry_kind = summary.loader_entry_kind.as_deref();
    let loader_entry_abi_contract = summary.loader_entry_abi_contract.as_deref();
    let loader_entry_machine_arch = summary.loader_entry_machine_arch.as_deref();
    let loader_entry_symbol = summary.loader_entry_symbol.as_deref();
    let loader_entry_section_id = summary.loader_entry_section_id.as_deref();
    let loader_symbol_count = summary.loader_symbol_count;
    let loader_symbol = &summary.loader_symbol;
    let relocation = &summary.relocation;
    let metadata_binding = &summary.metadata_binding;
    let provider_dispatch = &summary.provider_dispatch;
    let mut blockers = Vec::new();
    blockers.extend(metadata_binding.blockers.iter().cloned());
    blockers.extend(provider_dispatch.blockers.iter().cloned());
    let host_assisted_loader = loader_readiness == Some("host-assisted");
    match container_schema {
        Some(CONTAINER_SCHEMA) => {}
        Some(_) => blockers.push("container:schema-unsupported".to_owned()),
        None => blockers.push("container:schema-missing".to_owned()),
    }
    match container_schema_version {
        Some(CONTAINER_SCHEMA_VERSION) => {}
        Some(_) => blockers.push("container:schema-version-unsupported".to_owned()),
        None => blockers.push("container:schema-version-missing".to_owned()),
    }
    match container_kind {
        Some(CONTAINER_KIND) => {}
        Some(_) => blockers.push("container:kind-unsupported".to_owned()),
        None => blockers.push("container:kind-missing".to_owned()),
    }
    match container_producer {
        Some(CONTAINER_PRODUCER) => {}
        Some(_) => blockers.push("container:producer-unsupported".to_owned()),
        None => blockers.push("container:producer-missing".to_owned()),
    }
    match container_ready {
        Some(true) => {}
        Some(false) => blockers.push("container:not-ready".to_owned()),
        None => blockers.push("container:ready-missing".to_owned()),
    }
    blockers.extend(
        container_blockers
            .iter()
            .map(|blocker| format!("container:blocker:{blocker}")),
    );
    match container_magic {
        Some(CONTAINER_MAGIC) => {}
        Some(_) => blockers.push("container:magic-unsupported".to_owned()),
        None => blockers.push("container:magic-missing".to_owned()),
    }
    match container_version {
        Some(CONTAINER_VERSION) => {}
        Some(_) => blockers.push("container:version-unsupported".to_owned()),
        None => blockers.push("container:version-missing".to_owned()),
    }
    match container_section.declared_count {
        Some(0) => blockers.push("container:sections-missing".to_owned()),
        Some(expected) if expected != container_section.parsed_count => {
            blockers.push("container:section-count-mismatch".to_owned())
        }
        Some(_) => {}
        None => blockers.push("container:section-count-missing".to_owned()),
    }
    if container_section.declared_count.unwrap_or(0) > 0 && container_section.status != "parsed" {
        blockers.push("container:section-table-missing".to_owned());
    }
    match compatibility_domain.declared_count {
        Some(expected) if expected != compatibility_domain.parsed_count => {
            blockers.push("container:compatibility-domain-count-mismatch".to_owned())
        }
        Some(_) => {}
        None => blockers.push("container:compatibility-domain-count-missing".to_owned()),
    }
    if compatibility_domain.declared_count.unwrap_or(0) > 0
        && compatibility_domain.status != "parsed"
    {
        blockers.push("container:compatibility-domain-table-missing".to_owned());
    }
    match external_import.declared_count {
        Some(expected) if expected != external_import.parsed_count => {
            blockers.push("container:external-import-count-mismatch".to_owned())
        }
        Some(_) => {}
        None => blockers.push("container:external-import-count-missing".to_owned()),
    }
    if external_import.declared_count.unwrap_or(0) > 0 && external_import.status != "parsed" {
        blockers.push("container:external-import-table-missing".to_owned());
    }
    if !host_assisted_loader {
        blockers.extend(
            external_import
                .required_imports
                .iter()
                .map(|import| format!("container-external-import:required:{import}")),
        );
    }
    if container_payload_size_bytes.is_none() {
        blockers.push("container:payload-size-missing".to_owned());
    }
    if container_payload_hash.is_none_or(str::is_empty) {
        blockers.push("container:payload-hash-missing".to_owned());
    }
    match loader_readiness {
        Some("self-contained" | "host-assisted") => {}
        Some("blocked") => blockers.push("container-loader:readiness-blocked".to_owned()),
        Some(_) => blockers.push("container-loader:readiness-unsupported".to_owned()),
        None => blockers.push("container-loader:readiness-missing".to_owned()),
    }
    blockers.extend(
        loader_blockers
            .iter()
            .filter(|blocker| !(host_assisted_loader && blocker.starts_with("external-import:")))
            .map(|blocker| format!("container-loader:blocker:{blocker}")),
    );
    if loader_entry_symbol.is_none_or(str::is_empty) {
        blockers.push("container-loader:entry-symbol-missing".to_owned());
    }
    if loader_entry_kind.is_none_or(str::is_empty) {
        blockers.push("container-loader:entry-kind-missing".to_owned());
    }
    match loader_entry_abi_contract {
        Some(nuis_runtime::NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1) => {}
        Some(_) => blockers.push("container-loader:entry-abi-unsupported".to_owned()),
        None => blockers.push("container-loader:entry-abi-missing".to_owned()),
    }
    match loader_entry_machine_arch {
        Some(value) if nuis_runtime::canonical_machine_arch(value) == Some(value) => {}
        Some(_) => blockers.push("container-loader:entry-machine-arch-unsupported".to_owned()),
        None => blockers.push("container-loader:entry-machine-arch-missing".to_owned()),
    }
    if loader_entry_section_id.is_none_or(str::is_empty) {
        blockers.push("container-loader:entry-section-missing".to_owned());
    }
    if loader_entry_section_id.is_some_and(|entry| !entry.is_empty())
        && !container_section.entry_section_found
    {
        blockers.push("container-loader:entry-section-not-found".to_owned());
    }
    if loader_symbol_count.unwrap_or(0) == 0 {
        blockers.push("container-loader:symbols-missing".to_owned());
    }
    if loader_symbol_count.unwrap_or(0) > 0 {
        if loader_symbol.status != "parsed" {
            blockers.push("container-loader:symbol-table-missing".to_owned());
        } else {
            match (loader_entry_kind, loader_symbol.symbol_kind.as_deref()) {
                (Some(entry_kind), Some(symbol_kind)) if entry_kind == symbol_kind => {}
                (Some(_), Some(_)) => {
                    blockers.push("container-loader:entry-kind-mismatch".to_owned())
                }
                (_, None) => blockers.push("container-loader:symbol-kind-missing".to_owned()),
                (None, Some(_)) => {}
            }
            match loader_symbol.lifecycle_hook.as_deref() {
                Some("on_lifecycle_bootstrap") => {}
                Some(_) => blockers.push("container-loader:lifecycle-hook-unsupported".to_owned()),
                None => blockers.push("container-loader:lifecycle-hook-missing".to_owned()),
            }
            if loader_entry_symbol.is_some_and(|entry| {
                loader_symbol
                    .symbol_name
                    .as_deref()
                    .is_some_and(|symbol| symbol != entry)
            }) {
                blockers.push("container-loader:entry-symbol-mismatch".to_owned());
            }
            if loader_entry_section_id.is_some_and(|entry| {
                loader_symbol
                    .section_id
                    .as_deref()
                    .is_some_and(|section| section != entry)
            }) {
                blockers.push("container-loader:entry-section-mismatch".to_owned());
            }
            if loader_symbol
                .symbol_name
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("container-loader:symbol-name-missing".to_owned());
            }
            if loader_symbol
                .section_id
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("container-loader:symbol-section-missing".to_owned());
            }
            let symbol_range_valid = loader_symbol
                .section_id
                .as_deref()
                .zip(loader_symbol.offset)
                .zip(loader_symbol.size_bytes)
                .and_then(|((section_id, offset), size)| {
                    let section = container_section
                        .entries
                        .iter()
                        .find(|section| section.section_id == section_id)?;
                    Some(
                        size > 0
                            && offset >= section.offset
                            && offset
                                .checked_add(size)
                                .zip(section.offset.checked_add(section.size_bytes))
                                .is_some_and(|(symbol_end, section_end)| symbol_end <= section_end),
                    )
                })
                .unwrap_or(false);
            if !symbol_range_valid {
                blockers.push("container-loader:symbol-range-invalid".to_owned());
            }
            if loader_symbol
                .payload_hash
                .as_deref()
                .is_none_or(|hash| !valid_hash(hash))
            {
                blockers.push("container-loader:symbol-hash-invalid".to_owned());
            }
        }
    }
    match relocation.declared_count {
        Some(0) => blockers.push("container:relocations-missing".to_owned()),
        Some(expected) if expected != relocation.parsed_count => {
            blockers.push("container:relocation-count-mismatch".to_owned())
        }
        Some(_) => {}
        None => blockers.push("container:relocation-count-missing".to_owned()),
    }
    if relocation.declared_count.unwrap_or(0) > 0 {
        if relocation.status != "parsed" {
            blockers.push("container:relocation-table-missing".to_owned());
        } else {
            if !relocation.first_targets_loader_symbol {
                blockers.push("container-loader:first-relocation-target-mismatch".to_owned());
            }
            if !relocation.first_source_matches_loader_symbol {
                blockers.push("container-loader:first-relocation-source-mismatch".to_owned());
            }
        }
    }
    blockers
}

fn valid_hash(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

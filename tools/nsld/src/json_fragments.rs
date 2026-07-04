use super::{
    container::NsldContainerSectionEntry,
    json_fields::*,
    reports::{
        NsldAssembleSectionDiagnostic, NsldClockEdgeDiagnostic, NsldDataSegmentDiagnostic,
        NsldDomainDiagnostic, NsldLinkInputDiagnostic, NsldLinkUnitDiagnostic,
        NsldObjectFileLayoutRecordDiagnostic, NsldObjectRelocationSeedDiagnostic,
        NsldObjectSectionDiagnostic, NsldSidecarCapabilityDiagnostic,
    },
};

pub(crate) fn nsld_link_inputs_json(inputs: &[NsldLinkInputDiagnostic]) -> String {
    inputs
        .iter()
        .map(|input| {
            let fields = vec![
                json_usize_field("order_index", input.order_index),
                json_string_field("input_id", &input.input_id),
                json_string_field("input_kind", &input.input_kind),
                json_string_field("domain_family", &input.domain_family),
                json_string_field("package_id", &input.package_id),
                json_string_field("path", &input.path),
                json_string_field("native_ir", &input.native_ir),
                json_string_field("dispatch_lowering", &input.dispatch_lowering),
                json_usize_field("contract_count", input.contract_count),
                json_usize_field("content_bytes", input.content_bytes),
                json_string_field("content_hash", &input.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_link_units_json(units: &[NsldLinkUnitDiagnostic]) -> String {
    units
        .iter()
        .map(|unit| {
            let fields = vec![
                json_usize_field("order_index", unit.order_index),
                json_string_field("unit_id", &unit.unit_id),
                json_string_field("unit_kind", &unit.unit_kind),
                json_string_field("domain_family", &unit.domain_family),
                json_string_field("package_id", &unit.package_id),
                json_string_field("backend_family", &unit.backend_family),
                json_string_field("lowering_target", &unit.lowering_target),
                json_string_field("packaging_role", &unit.packaging_role),
                json_string_array_field("link_input_ids", &unit.link_input_ids),
                json_usize_field("clock_edge_count", unit.clock_edge_count),
                json_usize_field("data_segment_count", unit.data_segment_count),
                json_bool_field("requires_host_wrapper", unit.requires_host_wrapper),
                json_string_field("deterministic_order_key", &unit.deterministic_order_key),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_assemble_sections_json(sections: &[NsldAssembleSectionDiagnostic]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("section_id", &section.section_id),
                json_string_field("section_kind", &section.section_kind),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_bool_field("required", section.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_container_sections_json(sections: &[NsldContainerSectionEntry]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("section_id", &section.section_id),
                json_string_field("section_kind", &section.section_kind),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_string_field("payload_hash", &section.payload_hash),
                json_bool_field("required", section.required),
                json_usize_field("offset", section.offset),
                json_usize_field("size_bytes", section.size_bytes),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn compatibility_domain_summary_json(
    count: Option<usize>,
    table_hash: Option<&str>,
    domain_id: Option<&str>,
    domain_kind: Option<&str>,
    paradigm: Option<&str>,
    lifecycle_hook: Option<&str>,
    abi_family: Option<&str>,
    wrapper_policy: Option<&str>,
    required: Option<bool>,
) -> String {
    let fields = vec![
        json_optional_usize_field("count", count),
        json_optional_string_field("table_hash", table_hash),
        json_optional_string_field("domain_id", domain_id),
        json_optional_string_field("domain_kind", domain_kind),
        json_optional_string_field("paradigm", paradigm),
        json_optional_string_field("lifecycle_hook", lifecycle_hook),
        json_optional_string_field("abi_family", abi_family),
        json_optional_string_field("wrapper_policy", wrapper_policy),
        json_optional_bool_field("required", required),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_sections_json(sections: &[NsldObjectSectionDiagnostic]) -> String {
    sections
        .iter()
        .map(|section| {
            let fields = vec![
                json_usize_field("order_index", section.order_index),
                json_string_field("source_section_id", &section.source_section_id),
                json_string_field("source_section_kind", &section.source_section_kind),
                json_string_field("object_section_name", &section.object_section_name),
                json_string_field("object_section_role", &section.object_section_role),
                json_string_field("source_path", &section.source_path),
                json_string_field("source_hash", &section.source_hash),
                json_usize_field("source_size_bytes", section.source_size_bytes),
                json_usize_field("payload_offset_seed", section.payload_offset_seed),
                json_usize_field("file_offset_seed", section.file_offset_seed),
                json_usize_field("file_size_seed", section.file_size_seed),
                json_usize_field("alignment", section.alignment),
                json_bool_field("required", section.required),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_object_relocation_seeds_json(
    seeds: &[NsldObjectRelocationSeedDiagnostic],
) -> String {
    seeds
        .iter()
        .map(|seed| {
            let fields = vec![
                json_usize_field("order_index", seed.order_index),
                json_string_field("relocation_seed_id", &seed.relocation_seed_id),
                json_string_field("relocation_seed_kind", &seed.relocation_seed_kind),
                json_string_field("source_section_id", &seed.source_section_id),
                json_usize_field("source_offset_seed", seed.source_offset_seed),
                json_string_field("target_symbol", &seed.target_symbol),
                json_isize_field("addend", seed.addend),
                json_bool_field("native_relocation_ready", seed.native_relocation_ready),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_object_file_layout_records_json(
    records: &[NsldObjectFileLayoutRecordDiagnostic],
) -> String {
    records
        .iter()
        .map(|record| {
            let fields = vec![
                json_usize_field("order_index", record.order_index),
                json_string_field("record_id", &record.record_id),
                json_string_field("record_kind", &record.record_kind),
                json_usize_field("file_offset", record.file_offset),
                json_usize_field("size_bytes", record.size_bytes),
                json_usize_field("alignment", record.alignment),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn domains_json(domains: &[NsldDomainDiagnostic]) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = vec![
                json_string_field("domain_family", &domain.domain_family),
                json_string_field("package_id", &domain.package_id),
                json_string_field("kind", &domain.kind),
                json_string_field("packaging_role", &domain.packaging_role),
                json_string_field("lowering_target", &domain.lowering_target),
                json_string_field("backend_family", &domain.backend_family),
                json_bool_field("alignment_consistent", domain.alignment_consistent),
                json_string_array_field("alignment_issues", &domain.alignment_issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn sidecar_capabilities_json(
    capabilities: &[NsldSidecarCapabilityDiagnostic],
) -> String {
    capabilities
        .iter()
        .map(|capability| {
            let fields = vec![
                json_string_field("domain_family", &capability.domain_family),
                json_string_field("package_id", &capability.package_id),
                json_string_field("path", &capability.path),
                json_bool_field("valid", capability.valid),
                json_string_field("capability_owner", &capability.capability_owner),
                json_string_field("frontend_ir", &capability.frontend_ir),
                json_string_field("native_ir", &capability.native_ir),
                json_string_field("dispatch_lowering", &capability.dispatch_lowering),
                json_usize_field("content_bytes", capability.content_bytes),
                json_string_field("content_hash", &capability.content_hash),
                json_string_array_field("validation_contracts", &capability.validation_contracts),
                json_string_array_field("issues", &capability.issues),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn clock_edges_json(edges: &[NsldClockEdgeDiagnostic]) -> String {
    edges
        .iter()
        .map(|edge| {
            let fields = vec![
                json_usize_field("index", edge.index),
                json_string_field("from", &edge.from),
                json_string_field("to", &edge.to),
                json_string_field("relation", &edge.relation),
                json_string_field("source", &edge.source),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn data_segments_json(segments: &[NsldDataSegmentDiagnostic]) -> String {
    segments
        .iter()
        .map(|segment| {
            let fields = vec![
                json_usize_field("index", segment.index),
                json_string_field("segment_id", &segment.segment_id),
                json_string_field("domain_family", &segment.domain_family),
                json_string_field("owner_package", &segment.owner_package),
                json_string_field("order_key", &segment.order_key),
                json_string_field("access_phase", &segment.access_phase),
                json_string_field("source_path", &segment.source_path),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

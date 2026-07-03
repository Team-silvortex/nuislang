use super::{
    fnv1a64_hex,
    reports::{
        NsldAssembleSectionDiagnostic, NsldObjectRelocationSeedDiagnostic,
        NsldObjectSectionDiagnostic,
    },
};
use std::fs;

pub(crate) fn nsld_object_plan_hash(
    target_arch: &str,
    target_os: &str,
    object_format: &str,
    section_table_hash: &str,
    object_layout_hash: &str,
    relocation_seed_table_hash: &str,
    source_container_path: &str,
    source_payload_path: &str,
    object_sections: &[NsldObjectSectionDiagnostic],
    relocation_seeds: &[NsldObjectRelocationSeedDiagnostic],
    blockers: &[String],
) -> String {
    let section_material = object_sections
        .iter()
        .map(|section| {
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                section.order_index,
                section.source_section_id,
                section.source_section_kind,
                section.object_section_name,
                section.object_section_role,
                section.source_size_bytes,
                section.payload_offset_seed,
                section.file_offset_seed,
                section.file_size_seed,
                section.alignment
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let relocation_material = relocation_seeds
        .iter()
        .map(|seed| {
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}",
                seed.order_index,
                seed.relocation_seed_id,
                seed.relocation_seed_kind,
                seed.source_section_id,
                seed.source_offset_seed,
                seed.target_symbol,
                seed.addend,
                seed.native_relocation_ready
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let material = format!(
        "target_arch={target_arch}\ntarget_os={target_os}\nobject_format={object_format}\nsection_table_hash={section_table_hash}\nobject_layout_hash={object_layout_hash}\nrelocation_seed_table_hash={relocation_seed_table_hash}\nsource_container_path={source_container_path}\nsource_payload_path={source_payload_path}\nobject_sections={section_material}\nrelocation_seeds={relocation_material}\nblockers={}\n",
        blockers.join("|")
    );
    fnv1a64_hex(material.as_bytes())
}

pub(crate) fn nsld_object_layout_hash(object_sections: &[NsldObjectSectionDiagnostic]) -> String {
    let mut material = String::new();
    for section in object_sections {
        material.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            section.order_index,
            section.source_section_id,
            section.object_section_name,
            section.source_size_bytes,
            section.payload_offset_seed,
            section.file_offset_seed,
            section.file_size_seed,
            section.alignment
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

pub(crate) fn nsld_relocation_seed_table_hash(
    relocation_seeds: &[NsldObjectRelocationSeedDiagnostic],
) -> String {
    let mut material = String::new();
    for seed in relocation_seeds {
        material.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            seed.order_index,
            seed.relocation_seed_id,
            seed.relocation_seed_kind,
            seed.source_section_id,
            seed.source_offset_seed,
            seed.target_symbol,
            seed.addend,
            seed.native_relocation_ready
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

pub(crate) fn object_section_layout(
    sections: &[NsldAssembleSectionDiagnostic],
) -> Vec<NsldObjectSectionDiagnostic> {
    let mut next_file_offset_seed = 0usize;
    sections
        .iter()
        .map(|section| {
            let alignment = object_section_alignment(&section.section_kind);
            next_file_offset_seed = align_to(next_file_offset_seed, alignment);
            let source_size_bytes = source_size_bytes(&section.source_path);
            let file_offset_seed = next_file_offset_seed;
            let file_size_seed = source_size_bytes;
            next_file_offset_seed = next_file_offset_seed.saturating_add(file_size_seed);
            NsldObjectSectionDiagnostic {
                order_index: section.order_index,
                source_section_id: section.section_id.clone(),
                source_section_kind: section.section_kind.clone(),
                object_section_name: object_section_name(
                    &section.section_kind,
                    section.order_index,
                ),
                object_section_role: object_section_role(&section.section_kind),
                source_path: section.source_path.clone(),
                source_hash: section.source_hash.clone(),
                source_size_bytes,
                payload_offset_seed: section.order_index,
                file_offset_seed,
                file_size_seed,
                alignment,
                required: section.required,
            }
        })
        .collect()
}

pub(crate) fn object_relocation_seeds(
    object_sections: &[NsldObjectSectionDiagnostic],
) -> Vec<NsldObjectRelocationSeedDiagnostic> {
    object_sections
        .iter()
        .filter(|section| section.required)
        .enumerate()
        .map(|(index, section)| NsldObjectRelocationSeedDiagnostic {
            order_index: index,
            relocation_seed_id: format!(
                "orel{index:04}.{}",
                sanitize_section_token(&section.source_section_kind)
            ),
            relocation_seed_kind: object_relocation_seed_kind(&section.object_section_role),
            source_section_id: section.source_section_id.clone(),
            source_offset_seed: 0,
            target_symbol: format!(
                "__nuis_section_{}",
                sanitize_section_token(&section.source_section_id)
            ),
            addend: 0,
            native_relocation_ready: false,
        })
        .collect()
}

fn object_section_name(section_kind: &str, order_index: usize) -> String {
    match section_kind {
        "compiled-artifact" => ".nuis.text.compiled".to_owned(),
        "nsld-link-input-table" => ".nuis.meta.link_inputs".to_owned(),
        "nsld-link-unit-table" => ".nuis.meta.link_units".to_owned(),
        "nsld-link-bundle" => ".nuis.meta.link_bundle".to_owned(),
        "lowering-sidecar-input" => format!(".nuis.ir.sidecar.{order_index:04}"),
        "hetero-data-segment" => format!(".nuis.data.hetero.{order_index:04}"),
        other => format!(
            ".nuis.section.{}.{}",
            order_index,
            sanitize_section_token(other)
        ),
    }
}

fn object_section_role(section_kind: &str) -> String {
    match section_kind {
        "compiled-artifact" => "native-bootstrap-input".to_owned(),
        "nsld-link-input-table"
        | "nsld-link-unit-table"
        | "nsld-link-bundle"
        | "lowering-sidecar-input" => "metadata".to_owned(),
        "hetero-data-segment" => "data".to_owned(),
        _ => "extension".to_owned(),
    }
}

fn object_relocation_seed_kind(object_section_role: &str) -> String {
    match object_section_role {
        "native-bootstrap-input" => "bootstrap-entry-seed".to_owned(),
        "metadata" => "metadata-address-seed".to_owned(),
        "data" => "data-address-seed".to_owned(),
        _ => "extension-address-seed".to_owned(),
    }
}

fn object_section_alignment(section_kind: &str) -> usize {
    match section_kind {
        "compiled-artifact" => 16,
        "hetero-data-segment" => 16,
        _ => 8,
    }
}

fn align_to(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

fn source_size_bytes(source_path: &str) -> usize {
    fs::metadata(source_path)
        .map(|metadata| metadata.len() as usize)
        .unwrap_or(0)
}

fn sanitize_section_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

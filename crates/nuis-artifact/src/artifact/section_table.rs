use std::{collections::BTreeSet, path::Path};

use crate::{
    envelope::{decode_nuis_executable_envelope_binary, encode_nuis_executable_envelope_binary},
    protocol::{
        supported_compiled_artifact_sections, COMPILED_ARTIFACT_MAGIC,
        COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML, COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY,
        COMPILED_ARTIFACT_SECTION_HOST_BINARY, COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
        COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML, COMPILED_ARTIFACT_SECTION_METADATA_TOML,
        COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION,
    },
    toml::parse_required_toml_string,
    ArtifactError,
};

use super::{
    encode_u32_len,
    metadata::{
        parse_lifecycle_contract, render_compiled_artifact_metadata, render_lifecycle_contract,
        render_lowering_index,
    },
    validate_lowering_index_against_build_manifest, NuisCompiledArtifact,
    NuisCompiledArtifactSection, NuisCompiledArtifactSectionTable,
};

impl NuisCompiledArtifactSectionTable {
    pub fn section_names(&self) -> Vec<&str> {
        self.sections
            .iter()
            .map(|section| section.name.as_str())
            .collect()
    }

    pub fn section_bytes(&self, name: &str) -> Result<&[u8], ArtifactError> {
        section_bytes(self, name)
    }

    pub fn section_utf8(&self, name: &str) -> Result<&str, ArtifactError> {
        section_utf8(self, name)
    }

    pub fn contains_section(&self, name: &str) -> bool {
        self.sections.iter().any(|section| section.name == name)
    }
}

fn encode_u64_len(len: usize, what: &str) -> Result<[u8; 8], ArtifactError> {
    let len = u64::try_from(len)
        .map_err(|_| ArtifactError::new(format!("{what} exceeds u64 and cannot be encoded")))?;
    Ok(len.to_le_bytes())
}

pub fn encode_nuis_compiled_artifact_section_table_binary(
    artifact: &NuisCompiledArtifact,
) -> Result<Vec<u8>, ArtifactError> {
    let table = compiled_artifact_to_section_table(artifact)?;
    encode_nuis_compiled_artifact_section_table(&table)
}

pub fn encode_nuis_compiled_artifact_section_table(
    table: &NuisCompiledArtifactSectionTable,
) -> Result<Vec<u8>, ArtifactError> {
    let mut out = Vec::new();
    out.extend_from_slice(COMPILED_ARTIFACT_MAGIC);
    out.extend_from_slice(&COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION.to_le_bytes());
    out.extend_from_slice(&encode_u32_len(
        table.sections.len(),
        "compiled artifact section count",
    )?);
    for section in &table.sections {
        out.extend_from_slice(&encode_u32_len(
            section.name.len(),
            "compiled artifact section name",
        )?);
        out.extend_from_slice(&encode_u64_len(
            section.bytes.len(),
            "compiled artifact section payload",
        )?);
    }
    for section in &table.sections {
        out.extend_from_slice(section.name.as_bytes());
        out.extend_from_slice(&section.bytes);
    }
    Ok(out)
}

pub fn decode_nuis_compiled_artifact_section_table_binary(
    bytes: &[u8],
) -> Result<NuisCompiledArtifactSectionTable, ArtifactError> {
    if bytes.len() < 10 {
        return Err(ArtifactError::new(
            "nuis compiled artifact section table is too short",
        ));
    }
    if &bytes[..4] != COMPILED_ARTIFACT_MAGIC {
        return Err(ArtifactError::new(
            "nuis compiled artifact section table has invalid magic",
        ));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION {
        return Err(ArtifactError::new(format!(
            "unsupported nuis compiled artifact section table version `{version}`"
        )));
    }
    let section_count = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    let mut offset = 10usize;
    let mut section_meta = Vec::with_capacity(section_count);
    for _ in 0..section_count {
        if offset + 12 > bytes.len() {
            return Err(ArtifactError::new(
                "nuis compiled artifact section table header is truncated",
            ));
        }
        let name_len = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;
        let payload_len = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        offset += 8;
        section_meta.push((name_len, payload_len));
    }
    let total_payload_len = section_meta
        .iter()
        .map(|(name_len, payload_len)| name_len + payload_len)
        .sum::<usize>();
    if bytes.len() != offset + total_payload_len {
        return Err(ArtifactError::new(format!(
            "nuis compiled artifact section table length mismatch: header says {total_payload_len} payload bytes, actual {}",
            bytes.len().saturating_sub(offset)
        )));
    }
    let mut sections = Vec::with_capacity(section_count);
    for (name_len, payload_len) in section_meta {
        let name =
            String::from_utf8(take_exact(bytes, &mut offset, name_len)?).map_err(|error| {
                ArtifactError::new(format!(
                    "compiled artifact section name is not valid UTF-8: {error}"
                ))
            })?;
        let section_bytes = take_exact(bytes, &mut offset, payload_len)?;
        sections.push(NuisCompiledArtifactSection {
            name,
            bytes: section_bytes,
        });
    }
    let table = NuisCompiledArtifactSectionTable { sections };
    validate_compiled_artifact_section_table(&table)?;
    Ok(table)
}

pub fn validate_compiled_artifact_section_table(
    table: &NuisCompiledArtifactSectionTable,
) -> Result<(), ArtifactError> {
    let mut seen = BTreeSet::new();
    for section in &table.sections {
        if section.name.trim().is_empty() {
            return Err(ArtifactError::new(
                "compiled artifact section table contains an empty section name",
            ));
        }
        if !seen.insert(section.name.as_str()) {
            return Err(ArtifactError::new(format!(
                "compiled artifact section table contains duplicate section `{}`",
                section.name
            )));
        }
    }
    for required in supported_compiled_artifact_sections() {
        if !table.contains_section(required) {
            return Err(ArtifactError::new(format!(
                "compiled artifact section table is missing required section `{required}`"
            )));
        }
    }
    Ok(())
}

fn take_exact(bytes: &[u8], offset: &mut usize, len: usize) -> Result<Vec<u8>, ArtifactError> {
    if *offset + len > bytes.len() {
        return Err(ArtifactError::new(
            "nuis compiled artifact binary payload is truncated",
        ));
    }
    let value = bytes[*offset..*offset + len].to_vec();
    *offset += len;
    Ok(value)
}

pub(super) fn compiled_artifact_to_section_table(
    artifact: &NuisCompiledArtifact,
) -> Result<NuisCompiledArtifactSectionTable, ArtifactError> {
    let metadata = render_compiled_artifact_metadata(artifact);
    let lifecycle = render_lifecycle_contract(&artifact.lifecycle);
    let lowering_index = render_lowering_index(artifact)?;
    let envelope = encode_nuis_executable_envelope_binary(&artifact.envelope)?;
    Ok(NuisCompiledArtifactSectionTable {
        sections: vec![
            NuisCompiledArtifactSection {
                name: COMPILED_ARTIFACT_SECTION_METADATA_TOML.to_owned(),
                bytes: metadata.into_bytes(),
            },
            NuisCompiledArtifactSection {
                name: COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY.to_owned(),
                bytes: envelope,
            },
            NuisCompiledArtifactSection {
                name: COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML.to_owned(),
                bytes: lifecycle.into_bytes(),
            },
            NuisCompiledArtifactSection {
                name: COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML.to_owned(),
                bytes: artifact.build_manifest_source.as_bytes().to_vec(),
            },
            NuisCompiledArtifactSection {
                name: COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML.to_owned(),
                bytes: lowering_index.into_bytes(),
            },
            NuisCompiledArtifactSection {
                name: COMPILED_ARTIFACT_SECTION_HOST_BINARY.to_owned(),
                bytes: artifact.binary_blob.clone(),
            },
        ],
    })
}

pub(super) fn section_table_to_compiled_artifact(
    table: &NuisCompiledArtifactSectionTable,
) -> Result<NuisCompiledArtifact, ArtifactError> {
    let metadata = section_utf8(table, COMPILED_ARTIFACT_SECTION_METADATA_TOML)?;
    let lifecycle_source = section_utf8(table, COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML)?;
    let build_manifest_source =
        section_utf8(table, COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML)?.to_owned();
    let envelope = decode_nuis_executable_envelope_binary(section_bytes(
        table,
        COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY,
    )?)?;
    let binary_blob = section_bytes(table, COMPILED_ARTIFACT_SECTION_HOST_BINARY)?.to_vec();
    let lifecycle = parse_lifecycle_contract(lifecycle_source)?;
    validate_lowering_index_against_build_manifest(
        section_utf8(table, COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)?,
        &build_manifest_source,
    )?;
    Ok(NuisCompiledArtifact {
        schema: parse_required_toml_string(
            metadata,
            "artifact_schema",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        packaging_mode: parse_required_toml_string(
            metadata,
            "packaging_mode",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        cpu_target_abi: parse_required_toml_string(
            metadata,
            "cpu_target_abi",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        cpu_target_machine_arch: parse_required_toml_string(
            metadata,
            "cpu_target_machine_arch",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        cpu_target_machine_os: parse_required_toml_string(
            metadata,
            "cpu_target_machine_os",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        cpu_target_object_format: parse_required_toml_string(
            metadata,
            "cpu_target_object_format",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        cpu_target_calling_abi: parse_required_toml_string(
            metadata,
            "cpu_target_calling_abi",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        binary_name: parse_required_toml_string(
            metadata,
            "binary_name",
            Path::new("<compiled-artifact-metadata>"),
        )?,
        binary_bytes: binary_blob.len(),
        build_manifest_bytes: build_manifest_source.len(),
        envelope,
        lifecycle,
        build_manifest_source,
        binary_blob,
    })
}

fn section_bytes<'a>(
    table: &'a NuisCompiledArtifactSectionTable,
    name: &str,
) -> Result<&'a [u8], ArtifactError> {
    table
        .sections
        .iter()
        .find(|section| section.name == name)
        .map(|section| section.bytes.as_slice())
        .ok_or_else(|| ArtifactError::new(format!("compiled artifact is missing section `{name}`")))
}

fn section_utf8<'a>(
    table: &'a NuisCompiledArtifactSectionTable,
    name: &str,
) -> Result<&'a str, ArtifactError> {
    std::str::from_utf8(section_bytes(table, name)?).map_err(|error| {
        ArtifactError::new(format!(
            "compiled artifact section `{name}` is not valid UTF-8: {error}"
        ))
    })
}

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    decode_domain_payload_blob,
    envelope::{decode_nuis_executable_envelope_binary, encode_nuis_executable_envelope_binary},
    parse_build_manifest_from_source,
    protocol::{
        supported_compiled_artifact_sections, COMPILED_ARTIFACT_BINARY_VERSION,
        COMPILED_ARTIFACT_MAGIC, COMPILED_ARTIFACT_SCHEMA_V1,
        COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML, COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY,
        COMPILED_ARTIFACT_SECTION_HOST_BINARY, COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
        COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML, COMPILED_ARTIFACT_SECTION_METADATA_TOML,
        COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION, DOMAIN_PAYLOAD_SECTION_CONTRACT_TOML,
    },
    toml::{
        escape_toml_string, parse_optional_map_string, parse_optional_toml_string_array,
        parse_required_map_string_in_block, parse_required_toml_string, parse_required_toml_usize,
        render_string_array,
    },
    ArtifactError, BuildManifestDomainBuildUnit, NuisExecutableEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisLifecycleContract {
    pub schema: String,
    pub bootstrap_entry: String,
    pub tick_policy: String,
    pub shutdown_policy: String,
    pub yalivia_rpc: String,
    pub hook_surface: Vec<String>,
    pub export_surface: Vec<String>,
    pub runtime_capability_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifact {
    pub schema: String,
    pub packaging_mode: String,
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub binary_name: String,
    pub binary_bytes: usize,
    pub build_manifest_bytes: usize,
    pub envelope: NuisExecutableEnvelope,
    pub lifecycle: NuisLifecycleContract,
    pub build_manifest_source: String,
    pub binary_blob: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifactSection {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifactSectionTable {
    pub sections: Vec<NuisCompiledArtifactSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisLoweringIndex {
    pub schema: String,
    pub packaging_mode: String,
    pub domain_unit_count: usize,
    pub units: Vec<NuisLoweringIndexUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisLoweringIndexUnit {
    pub package_id: String,
    pub domain_family: String,
    pub backend_family: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub artifact_ir_sidecar_path: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
}

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

fn encode_u32_len(len: usize, what: &str) -> Result<[u8; 4], ArtifactError> {
    let len = u32::try_from(len)
        .map_err(|_| ArtifactError::new(format!("{what} exceeds 4 GiB and cannot be encoded")))?;
    Ok(len.to_le_bytes())
}

fn encode_u64_len(len: usize, what: &str) -> Result<[u8; 8], ArtifactError> {
    let len = u64::try_from(len)
        .map_err(|_| ArtifactError::new(format!("{what} exceeds u64 and cannot be encoded")))?;
    Ok(len.to_le_bytes())
}

pub fn encode_nuis_compiled_artifact_binary(
    artifact: &NuisCompiledArtifact,
) -> Result<Vec<u8>, ArtifactError> {
    let envelope = encode_nuis_executable_envelope_binary(&artifact.envelope)?;
    let packaging_mode = artifact.packaging_mode.as_bytes();
    let cpu_target_abi = artifact.cpu_target_abi.as_bytes();
    let cpu_target_machine_arch = artifact.cpu_target_machine_arch.as_bytes();
    let cpu_target_machine_os = artifact.cpu_target_machine_os.as_bytes();
    let cpu_target_object_format = artifact.cpu_target_object_format.as_bytes();
    let cpu_target_calling_abi = artifact.cpu_target_calling_abi.as_bytes();
    let binary_name = artifact.binary_name.as_bytes();
    let lifecycle_schema = artifact.lifecycle.schema.as_bytes();
    let lifecycle_bootstrap_entry = artifact.lifecycle.bootstrap_entry.as_bytes();
    let lifecycle_tick_policy = artifact.lifecycle.tick_policy.as_bytes();
    let lifecycle_shutdown_policy = artifact.lifecycle.shutdown_policy.as_bytes();
    let lifecycle_yalivia_rpc = artifact.lifecycle.yalivia_rpc.as_bytes();
    let lifecycle_hook_surface = render_string_array(&artifact.lifecycle.hook_surface).into_bytes();
    let lifecycle_export_surface =
        render_string_array(&artifact.lifecycle.export_surface).into_bytes();
    let lifecycle_runtime_capability_flags =
        render_string_array(&artifact.lifecycle.runtime_capability_flags).into_bytes();
    let build_manifest_source = artifact.build_manifest_source.as_bytes();
    let binary_blob = &artifact.binary_blob;
    let mut out = Vec::new();
    out.extend_from_slice(COMPILED_ARTIFACT_MAGIC);
    out.extend_from_slice(&COMPILED_ARTIFACT_BINARY_VERSION.to_le_bytes());
    out.extend_from_slice(&encode_u32_len(
        envelope.len(),
        "compiled artifact envelope",
    )?);
    out.extend_from_slice(&encode_u32_len(
        packaging_mode.len(),
        "compiled artifact packaging_mode",
    )?);
    out.extend_from_slice(&encode_u32_len(
        cpu_target_abi.len(),
        "compiled artifact cpu_target_abi",
    )?);
    out.extend_from_slice(&encode_u32_len(
        cpu_target_machine_arch.len(),
        "compiled artifact cpu_target_machine_arch",
    )?);
    out.extend_from_slice(&encode_u32_len(
        cpu_target_machine_os.len(),
        "compiled artifact cpu_target_machine_os",
    )?);
    out.extend_from_slice(&encode_u32_len(
        cpu_target_object_format.len(),
        "compiled artifact cpu_target_object_format",
    )?);
    out.extend_from_slice(&encode_u32_len(
        cpu_target_calling_abi.len(),
        "compiled artifact cpu_target_calling_abi",
    )?);
    out.extend_from_slice(&encode_u32_len(
        binary_name.len(),
        "compiled artifact binary_name",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_schema.len(),
        "compiled artifact lifecycle_schema",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_bootstrap_entry.len(),
        "compiled artifact lifecycle_bootstrap_entry",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_tick_policy.len(),
        "compiled artifact lifecycle_tick_policy",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_shutdown_policy.len(),
        "compiled artifact lifecycle_shutdown_policy",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_yalivia_rpc.len(),
        "compiled artifact lifecycle_yalivia_rpc",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_hook_surface.len(),
        "compiled artifact lifecycle_hook_surface",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_export_surface.len(),
        "compiled artifact lifecycle_export_surface",
    )?);
    out.extend_from_slice(&encode_u32_len(
        lifecycle_runtime_capability_flags.len(),
        "compiled artifact lifecycle_runtime_capability_flags",
    )?);
    out.extend_from_slice(&encode_u32_len(
        build_manifest_source.len(),
        "compiled artifact build_manifest_source",
    )?);
    out.extend_from_slice(&encode_u32_len(
        binary_blob.len(),
        "compiled artifact binary_blob",
    )?);
    out.extend_from_slice(envelope.as_slice());
    out.extend_from_slice(packaging_mode);
    out.extend_from_slice(cpu_target_abi);
    out.extend_from_slice(cpu_target_machine_arch);
    out.extend_from_slice(cpu_target_machine_os);
    out.extend_from_slice(cpu_target_object_format);
    out.extend_from_slice(cpu_target_calling_abi);
    out.extend_from_slice(binary_name);
    out.extend_from_slice(lifecycle_schema);
    out.extend_from_slice(lifecycle_bootstrap_entry);
    out.extend_from_slice(lifecycle_tick_policy);
    out.extend_from_slice(lifecycle_shutdown_policy);
    out.extend_from_slice(lifecycle_yalivia_rpc);
    out.extend_from_slice(lifecycle_hook_surface.as_slice());
    out.extend_from_slice(lifecycle_export_surface.as_slice());
    out.extend_from_slice(lifecycle_runtime_capability_flags.as_slice());
    out.extend_from_slice(build_manifest_source);
    out.extend_from_slice(binary_blob);
    Ok(out)
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

pub fn parse_nuis_lowering_index_from_source(
    source: &str,
    path: &Path,
) -> Result<NuisLoweringIndex, ArtifactError> {
    let schema = parse_required_toml_string(source, "schema", path)?;
    let packaging_mode = parse_required_toml_string(source, "packaging_mode", path)?;
    let domain_unit_count = parse_required_toml_usize(source, "domain_unit_count", path)?;
    let units = parse_lowering_unit_blocks(source, path)?;
    if units.len() != domain_unit_count {
        return Err(ArtifactError::new(format!(
            "`{}` lowering index domain_unit_count mismatch: declared={}, actual={}",
            path.display(),
            domain_unit_count,
            units.len()
        )));
    }
    Ok(NuisLoweringIndex {
        schema,
        packaging_mode,
        domain_unit_count,
        units,
    })
}

fn parse_lowering_unit_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<NuisLoweringIndexUnit>, ArtifactError> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[lowering_unit]]" {
            if in_block {
                rows.push(parse_lowering_unit_row(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_lowering_unit_row(&current, path)?);
                current.clear();
                in_block = false;
            }
            continue;
        }
        if in_block {
            if let Some((key, value)) = line.split_once('=') {
                current.insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }
    }
    if in_block {
        rows.push(parse_lowering_unit_row(&current, path)?);
    }
    Ok(rows)
}

fn parse_lowering_unit_row(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<NuisLoweringIndexUnit, ArtifactError> {
    Ok(NuisLoweringIndexUnit {
        package_id: parse_required_map_string_in_block(
            values,
            "package_id",
            path,
            "lowering_unit",
        )?,
        domain_family: parse_required_map_string_in_block(
            values,
            "domain_family",
            path,
            "lowering_unit",
        )?,
        backend_family: parse_optional_map_string(values, "backend_family"),
        selected_lowering_target: parse_optional_map_string(values, "selected_lowering_target"),
        artifact_ir_sidecar_path: parse_optional_map_string(values, "artifact_ir_sidecar_path"),
        contract_family: parse_required_map_string_in_block(
            values,
            "contract_family",
            path,
            "lowering_unit",
        )?,
        packaging_role: parse_required_map_string_in_block(
            values,
            "packaging_role",
            path,
            "lowering_unit",
        )?,
    })
}

pub fn decode_nuis_compiled_artifact_binary(
    bytes: &[u8],
) -> Result<NuisCompiledArtifact, ArtifactError> {
    if bytes.len() < 6 {
        return Err(ArtifactError::new(
            "nuis compiled artifact binary is too short",
        ));
    }
    if &bytes[..4] != COMPILED_ARTIFACT_MAGIC {
        return Err(ArtifactError::new(
            "nuis compiled artifact binary has invalid magic",
        ));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version == COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION {
        let table = decode_nuis_compiled_artifact_section_table_binary(bytes)?;
        return section_table_to_compiled_artifact(&table);
    }
    if version != COMPILED_ARTIFACT_BINARY_VERSION {
        return Err(ArtifactError::new(format!(
            "unsupported nuis compiled artifact binary version `{version}`"
        )));
    }
    if bytes.len() < 78 {
        return Err(ArtifactError::new(
            "nuis compiled artifact binary is too short",
        ));
    }
    let mut offset = 6usize;
    let next_len = |bytes: &[u8], offset: &mut usize| -> Result<usize, ArtifactError> {
        if *offset + 4 > bytes.len() {
            return Err(ArtifactError::new(
                "nuis compiled artifact binary header is truncated",
            ));
        }
        let value = u32::from_le_bytes([
            bytes[*offset],
            bytes[*offset + 1],
            bytes[*offset + 2],
            bytes[*offset + 3],
        ]) as usize;
        *offset += 4;
        Ok(value)
    };
    let envelope_len = next_len(bytes, &mut offset)?;
    let packaging_mode_len = next_len(bytes, &mut offset)?;
    let cpu_target_abi_len = next_len(bytes, &mut offset)?;
    let cpu_target_machine_arch_len = next_len(bytes, &mut offset)?;
    let cpu_target_machine_os_len = next_len(bytes, &mut offset)?;
    let cpu_target_object_format_len = next_len(bytes, &mut offset)?;
    let cpu_target_calling_abi_len = next_len(bytes, &mut offset)?;
    let binary_name_len = next_len(bytes, &mut offset)?;
    let lifecycle_schema_len = next_len(bytes, &mut offset)?;
    let lifecycle_bootstrap_entry_len = next_len(bytes, &mut offset)?;
    let lifecycle_tick_policy_len = next_len(bytes, &mut offset)?;
    let lifecycle_shutdown_policy_len = next_len(bytes, &mut offset)?;
    let lifecycle_yalivia_rpc_len = next_len(bytes, &mut offset)?;
    let lifecycle_hook_surface_len = next_len(bytes, &mut offset)?;
    let lifecycle_export_surface_len = next_len(bytes, &mut offset)?;
    let lifecycle_runtime_capability_flags_len = next_len(bytes, &mut offset)?;
    let build_manifest_source_len = next_len(bytes, &mut offset)?;
    let binary_blob_len = next_len(bytes, &mut offset)?;
    let total_payload_len = envelope_len
        + packaging_mode_len
        + cpu_target_abi_len
        + cpu_target_machine_arch_len
        + cpu_target_machine_os_len
        + cpu_target_object_format_len
        + cpu_target_calling_abi_len
        + binary_name_len
        + lifecycle_schema_len
        + lifecycle_bootstrap_entry_len
        + lifecycle_tick_policy_len
        + lifecycle_shutdown_policy_len
        + lifecycle_yalivia_rpc_len
        + lifecycle_hook_surface_len
        + lifecycle_export_surface_len
        + lifecycle_runtime_capability_flags_len
        + build_manifest_source_len
        + binary_blob_len;
    if bytes.len() != offset + total_payload_len {
        return Err(ArtifactError::new(format!(
            "nuis compiled artifact binary length mismatch: header says {total_payload_len} payload bytes, actual {}",
            bytes.len().saturating_sub(offset)
        )));
    }
    let take_bytes =
        |bytes: &[u8], offset: &mut usize, len: usize| -> Result<Vec<u8>, ArtifactError> {
            if *offset + len > bytes.len() {
                return Err(ArtifactError::new(
                    "nuis compiled artifact binary payload is truncated",
                ));
            }
            let value = bytes[*offset..*offset + len].to_vec();
            *offset += len;
            Ok(value)
        };
    let envelope =
        decode_nuis_executable_envelope_binary(&take_bytes(bytes, &mut offset, envelope_len)?)?;
    let packaging_mode = String::from_utf8(take_bytes(bytes, &mut offset, packaging_mode_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "compiled artifact packaging_mode is not valid UTF-8: {error}"
            ))
        })?;
    let cpu_target_abi = String::from_utf8(take_bytes(bytes, &mut offset, cpu_target_abi_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "compiled artifact cpu_target_abi is not valid UTF-8: {error}"
            ))
        })?;
    let cpu_target_machine_arch =
        String::from_utf8(take_bytes(bytes, &mut offset, cpu_target_machine_arch_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact cpu_target_machine_arch is not valid UTF-8: {error}"
                ))
            },
        )?;
    let cpu_target_machine_os =
        String::from_utf8(take_bytes(bytes, &mut offset, cpu_target_machine_os_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact cpu_target_machine_os is not valid UTF-8: {error}"
                ))
            },
        )?;
    let cpu_target_object_format = String::from_utf8(take_bytes(
        bytes,
        &mut offset,
        cpu_target_object_format_len,
    )?)
    .map_err(|error| {
        ArtifactError::new(format!(
            "compiled artifact cpu_target_object_format is not valid UTF-8: {error}"
        ))
    })?;
    let cpu_target_calling_abi =
        String::from_utf8(take_bytes(bytes, &mut offset, cpu_target_calling_abi_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact cpu_target_calling_abi is not valid UTF-8: {error}"
                ))
            },
        )?;
    let binary_name =
        String::from_utf8(take_bytes(bytes, &mut offset, binary_name_len)?).map_err(|error| {
            ArtifactError::new(format!(
                "compiled artifact binary_name is not valid UTF-8: {error}"
            ))
        })?;
    let lifecycle_schema = String::from_utf8(take_bytes(bytes, &mut offset, lifecycle_schema_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "compiled artifact lifecycle_schema is not valid UTF-8: {error}"
            ))
        })?;
    let lifecycle_bootstrap_entry = String::from_utf8(take_bytes(
        bytes,
        &mut offset,
        lifecycle_bootstrap_entry_len,
    )?)
    .map_err(|error| {
        ArtifactError::new(format!(
            "compiled artifact lifecycle_bootstrap_entry is not valid UTF-8: {error}"
        ))
    })?;
    let lifecycle_tick_policy =
        String::from_utf8(take_bytes(bytes, &mut offset, lifecycle_tick_policy_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact lifecycle_tick_policy is not valid UTF-8: {error}"
                ))
            },
        )?;
    let lifecycle_shutdown_policy = String::from_utf8(take_bytes(
        bytes,
        &mut offset,
        lifecycle_shutdown_policy_len,
    )?)
    .map_err(|error| {
        ArtifactError::new(format!(
            "compiled artifact lifecycle_shutdown_policy is not valid UTF-8: {error}"
        ))
    })?;
    let lifecycle_yalivia_rpc =
        String::from_utf8(take_bytes(bytes, &mut offset, lifecycle_yalivia_rpc_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact lifecycle_yalivia_rpc is not valid UTF-8: {error}"
                ))
            },
        )?;
    let lifecycle_hook_surface_source =
        String::from_utf8(take_bytes(bytes, &mut offset, lifecycle_hook_surface_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact lifecycle_hook_surface is not valid UTF-8: {error}"
                ))
            },
        )?;
    let lifecycle_export_surface_source = String::from_utf8(take_bytes(
        bytes,
        &mut offset,
        lifecycle_export_surface_len,
    )?)
    .map_err(|error| {
        ArtifactError::new(format!(
            "compiled artifact lifecycle_export_surface is not valid UTF-8: {error}"
        ))
    })?;
    let lifecycle_runtime_capability_flags_source = String::from_utf8(take_bytes(
        bytes,
        &mut offset,
        lifecycle_runtime_capability_flags_len,
    )?)
    .map_err(|error| {
        ArtifactError::new(format!(
            "compiled artifact lifecycle_runtime_capability_flags is not valid UTF-8: {error}"
        ))
    })?;
    let build_manifest_source =
        String::from_utf8(take_bytes(bytes, &mut offset, build_manifest_source_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "compiled artifact build_manifest_source is not valid UTF-8: {error}"
                ))
            },
        )?;
    let binary_blob = take_bytes(bytes, &mut offset, binary_blob_len)?;
    Ok(NuisCompiledArtifact {
        schema: COMPILED_ARTIFACT_SCHEMA_V1.to_owned(),
        packaging_mode,
        cpu_target_abi,
        cpu_target_machine_arch,
        cpu_target_machine_os,
        cpu_target_object_format,
        cpu_target_calling_abi,
        binary_name,
        binary_bytes: binary_blob.len(),
        build_manifest_bytes: build_manifest_source.len(),
        envelope,
        lifecycle: NuisLifecycleContract {
            schema: lifecycle_schema,
            bootstrap_entry: lifecycle_bootstrap_entry,
            tick_policy: lifecycle_tick_policy,
            shutdown_policy: lifecycle_shutdown_policy,
            yalivia_rpc: lifecycle_yalivia_rpc,
            hook_surface: parse_optional_toml_string_array(
                &format!("hook_surface = {lifecycle_hook_surface_source}"),
                "hook_surface",
            )
            .unwrap_or_default(),
            export_surface: parse_optional_toml_string_array(
                &format!("export_surface = {lifecycle_export_surface_source}"),
                "export_surface",
            )
            .unwrap_or_default(),
            runtime_capability_flags: parse_optional_toml_string_array(
                &format!("runtime_capability_flags = {lifecycle_runtime_capability_flags_source}"),
                "runtime_capability_flags",
            )
            .unwrap_or_default(),
        },
        build_manifest_source,
        binary_blob,
    })
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

fn compiled_artifact_to_section_table(
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

fn section_table_to_compiled_artifact(
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

fn validate_lowering_index_against_build_manifest(
    lowering_index_source: &str,
    build_manifest_source: &str,
) -> Result<(), ArtifactError> {
    let lowering_index = parse_nuis_lowering_index_from_source(
        lowering_index_source,
        Path::new("<compiled-artifact-lowering-index>"),
    )?;
    let build_manifest = parse_build_manifest_from_source(
        build_manifest_source,
        Path::new("<compiled-artifact-build-manifest>"),
    )?;
    if lowering_index.packaging_mode != build_manifest.packaging_mode {
        return Err(ArtifactError::new(format!(
            "compiled artifact lowering index packaging_mode `{}` does not match build manifest packaging_mode `{}`",
            lowering_index.packaging_mode, build_manifest.packaging_mode
        )));
    }
    if lowering_index.units.len() != build_manifest.domain_build_units.len() {
        return Err(ArtifactError::new(format!(
            "compiled artifact lowering index unit count `{}` does not match build manifest domain build unit count `{}`",
            lowering_index.units.len(),
            build_manifest.domain_build_units.len()
        )));
    }
    for (index, (lowering_unit, manifest_unit)) in lowering_index
        .units
        .iter()
        .zip(build_manifest.domain_build_units.iter())
        .enumerate()
    {
        validate_lowering_unit_matches_manifest_unit(index, lowering_unit, manifest_unit)?;
    }
    Ok(())
}

fn validate_lowering_unit_matches_manifest_unit(
    index: usize,
    lowering_unit: &NuisLoweringIndexUnit,
    manifest_unit: &BuildManifestDomainBuildUnit,
) -> Result<(), ArtifactError> {
    let unit_label = format!(
        "lowering unit #{index} ({}/{})",
        lowering_unit.package_id, lowering_unit.domain_family
    );
    validate_lowering_unit_string(
        &unit_label,
        "package_id",
        &lowering_unit.package_id,
        &manifest_unit.package_id,
    )?;
    validate_lowering_unit_string(
        &unit_label,
        "domain_family",
        &lowering_unit.domain_family,
        &manifest_unit.domain_family,
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "backend_family",
        lowering_unit.backend_family.as_deref(),
        manifest_unit.backend_family.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "selected_lowering_target",
        lowering_unit.selected_lowering_target.as_deref(),
        manifest_unit.selected_lowering_target.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "artifact_ir_sidecar_path",
        lowering_unit.artifact_ir_sidecar_path.as_deref(),
        manifest_unit.artifact_ir_sidecar_path.as_deref(),
    )?;
    validate_lowering_unit_string(
        &unit_label,
        "contract_family",
        &lowering_unit.contract_family,
        &manifest_unit.contract_family,
    )?;
    validate_lowering_unit_string(
        &unit_label,
        "packaging_role",
        &lowering_unit.packaging_role,
        &manifest_unit.packaging_role,
    )
}

fn validate_lowering_unit_string(
    unit_label: &str,
    field: &str,
    lowering_value: &str,
    manifest_value: &str,
) -> Result<(), ArtifactError> {
    if lowering_value == manifest_value {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiled artifact lowering index {unit_label} field `{field}` value `{lowering_value}` does not match build manifest value `{manifest_value}`"
    )))
}

fn validate_lowering_unit_option(
    unit_label: &str,
    field: &str,
    lowering_value: Option<&str>,
    manifest_value: Option<&str>,
) -> Result<(), ArtifactError> {
    if lowering_value == manifest_value {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiled artifact lowering index {unit_label} field `{field}` value `{}` does not match build manifest value `{}`",
        lowering_value.unwrap_or("<none>"),
        manifest_value.unwrap_or("<none>")
    )))
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

fn render_compiled_artifact_metadata(artifact: &NuisCompiledArtifact) -> String {
    format!(
        "artifact_schema = \"{}\"\npackaging_mode = \"{}\"\ncpu_target_abi = \"{}\"\ncpu_target_machine_arch = \"{}\"\ncpu_target_machine_os = \"{}\"\ncpu_target_object_format = \"{}\"\ncpu_target_calling_abi = \"{}\"\nbinary_name = \"{}\"\n",
        escape_toml_string(&artifact.schema),
        escape_toml_string(&artifact.packaging_mode),
        escape_toml_string(&artifact.cpu_target_abi),
        escape_toml_string(&artifact.cpu_target_machine_arch),
        escape_toml_string(&artifact.cpu_target_machine_os),
        escape_toml_string(&artifact.cpu_target_object_format),
        escape_toml_string(&artifact.cpu_target_calling_abi),
        escape_toml_string(&artifact.binary_name),
    )
}

fn render_lifecycle_contract(lifecycle: &NuisLifecycleContract) -> String {
    format!(
        "lifecycle_schema = \"{}\"\nlifecycle_bootstrap_entry = \"{}\"\nlifecycle_tick_policy = \"{}\"\nlifecycle_shutdown_policy = \"{}\"\nlifecycle_yalivia_rpc = \"{}\"\nlifecycle_hook_surface = {}\nlifecycle_export_surface = {}\nlifecycle_runtime_capability_flags = {}\n",
        escape_toml_string(&lifecycle.schema),
        escape_toml_string(&lifecycle.bootstrap_entry),
        escape_toml_string(&lifecycle.tick_policy),
        escape_toml_string(&lifecycle.shutdown_policy),
        escape_toml_string(&lifecycle.yalivia_rpc),
        render_string_array(&lifecycle.hook_surface),
        render_string_array(&lifecycle.export_surface),
        render_string_array(&lifecycle.runtime_capability_flags),
    )
}

fn render_lowering_index(artifact: &NuisCompiledArtifact) -> Result<String, ArtifactError> {
    let manifest = parse_build_manifest_from_source(
        &artifact.build_manifest_source,
        Path::new("<compiled-artifact-build-manifest>"),
    )?;
    let mut out = String::new();
    out.push_str("schema = \"nuis-lowering-index-v1\"\n");
    out.push_str(&format!(
        "packaging_mode = \"{}\"\n",
        escape_toml_string(&artifact.packaging_mode)
    ));
    out.push_str(&format!(
        "domain_unit_count = {}\n",
        manifest.domain_build_units.len()
    ));
    for unit in &manifest.domain_build_units {
        out.push_str("\n[[lowering_unit]]\n");
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        if let Some(value) = &unit.backend_family {
            out.push_str(&format!(
                "backend_family = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.selected_lowering_target {
            out.push_str(&format!(
                "selected_lowering_target = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_ir_sidecar_path {
            out.push_str(&format!(
                "artifact_ir_sidecar_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        out.push_str(&format!(
            "contract_family = \"{}\"\n",
            escape_toml_string(&unit.contract_family)
        ));
        out.push_str(&format!(
            "packaging_role = \"{}\"\n",
            escape_toml_string(&unit.packaging_role)
        ));
    }
    Ok(out)
}

fn parse_lifecycle_contract(source: &str) -> Result<NuisLifecycleContract, ArtifactError> {
    let path = Path::new("<compiled-artifact-lifecycle>");
    Ok(NuisLifecycleContract {
        schema: parse_required_toml_string(source, "lifecycle_schema", path)?,
        bootstrap_entry: parse_required_toml_string(source, "lifecycle_bootstrap_entry", path)?,
        tick_policy: parse_required_toml_string(source, "lifecycle_tick_policy", path)?,
        shutdown_policy: parse_required_toml_string(source, "lifecycle_shutdown_policy", path)?,
        yalivia_rpc: parse_required_toml_string(source, "lifecycle_yalivia_rpc", path)?,
        hook_surface: parse_optional_toml_string_array(source, "lifecycle_hook_surface")
            .unwrap_or_default(),
        export_surface: parse_optional_toml_string_array(source, "lifecycle_export_surface")
            .unwrap_or_default(),
        runtime_capability_flags: parse_optional_toml_string_array(
            source,
            "lifecycle_runtime_capability_flags",
        )
        .unwrap_or_default(),
    })
}

pub fn write_nuis_compiled_artifact(
    path: &Path,
    artifact: &NuisCompiledArtifact,
) -> Result<(), ArtifactError> {
    let out = encode_nuis_compiled_artifact_binary(artifact)?;
    fs::write(path, out).map_err(|error| {
        ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
    })
}

pub fn parse_nuis_compiled_artifact(path: &Path) -> Result<NuisCompiledArtifact, ArtifactError> {
    let bytes = fs::read(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    decode_nuis_compiled_artifact_binary(&bytes)
}

pub fn materialize_embedded_artifact_support(
    artifact: &NuisCompiledArtifact,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, ArtifactError> {
    fs::create_dir_all(output_dir).map_err(|error| {
        ArtifactError::new(format!(
            "failed to create materialization directory `{}`: {error}",
            output_dir.display()
        ))
    })?;
    let manifest = parse_build_manifest_from_source(
        &artifact.build_manifest_source,
        Path::new("<embedded-build-manifest>"),
    )?;
    let mut written = Vec::new();

    if let Some(source) = &manifest.bridge_registry_inline {
        let path = output_dir.join("nuis.bridge.registry.toml");
        fs::write(&path, source).map_err(|error| {
            ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
        })?;
        written.push(path);
    }
    if let Some(source) = &manifest.host_bridge_plan_index_inline {
        let path = output_dir.join("nuis.host-bridge.plan-index.toml");
        fs::write(&path, source).map_err(|error| {
            ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
        })?;
        written.push(path);
    }

    for unit in manifest
        .domain_build_units
        .iter()
        .filter(|unit| unit.is_heterogeneous())
    {
        materialize_domain_unit_support(output_dir, unit, &mut written)?;
    }

    Ok(written)
}

fn materialize_domain_unit_support(
    output_dir: &Path,
    unit: &BuildManifestDomainBuildUnit,
    written: &mut Vec<PathBuf>,
) -> Result<(), ArtifactError> {
    if let Some(source) = &unit.artifact_stub_inline {
        let path = output_dir.join(format!("nuis.domain.{}.artifact.toml", unit.domain_family));
        fs::write(&path, source).map_err(|error| {
            ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
        })?;
        written.push(path);
    }

    if let Some(hex_blob) = &unit.artifact_payload_blob_inline {
        let blob = decode_hex_bytes(hex_blob)?;
        let blob_path = output_dir.join(format!("nuis.domain.{}.payload.bin", unit.domain_family));
        fs::write(&blob_path, &blob).map_err(|error| {
            ArtifactError::new(format!(
                "failed to write `{}`: {error}",
                blob_path.display()
            ))
        })?;
        written.push(blob_path);

        let decoded = decode_domain_payload_blob(&blob)?;
        if let Some(contract_section) = decoded
            .sections
            .iter()
            .find(|section| section.name == DOMAIN_PAYLOAD_SECTION_CONTRACT_TOML)
        {
            let path = output_dir.join(format!("nuis.domain.{}.payload.toml", unit.domain_family));
            fs::write(&path, &contract_section.bytes).map_err(|error| {
                ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
            })?;
            written.push(path);
        }
        if let Some(ir_sidecar_section) = decoded
            .sections
            .iter()
            .find(|section| section.name.ends_with("_ir_sidecar"))
        {
            let path = materialized_support_path(
                output_dir,
                unit.artifact_ir_sidecar_path.as_deref(),
                &format!("nuis.domain.{}.lowering.ir.txt", unit.domain_family),
            );
            fs::write(&path, &ir_sidecar_section.bytes).map_err(|error| {
                ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
            })?;
            written.push(path);
        }
    }

    if let Some(source) = &unit.artifact_bridge_stub_inline {
        let path = output_dir.join(format!(
            "nuis.domain.{}.bridge.stub.txt",
            unit.domain_family
        ));
        fs::write(&path, source).map_err(|error| {
            ArtifactError::new(format!("failed to write `{}`: {error}", path.display()))
        })?;
        written.push(path);
    }

    Ok(())
}

fn materialized_support_path(
    output_dir: &Path,
    original: Option<&str>,
    fallback_name: &str,
) -> PathBuf {
    if let Some(original) = original {
        let candidate = Path::new(original);
        if let Some(file_name) = candidate.file_name() {
            return output_dir.join(file_name);
        }
    }
    output_dir.join(fallback_name)
}

fn decode_hex_bytes(value: &str) -> Result<Vec<u8>, ArtifactError> {
    if value.len() % 2 != 0 {
        return Err(ArtifactError::new("hex payload length must be even"));
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let chunk = std::str::from_utf8(&bytes[index..index + 2])
            .map_err(|_| ArtifactError::new("hex payload is not valid UTF-8"))?;
        let byte = u8::from_str_radix(chunk, 16)
            .map_err(|_| ArtifactError::new(format!("invalid hex byte `{chunk}`")))?;
        out.push(byte);
        index += 2;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        decode_nuis_compiled_artifact_binary, decode_nuis_compiled_artifact_section_table_binary,
        encode_domain_payload_blob, encode_nuis_compiled_artifact_section_table,
        encode_nuis_compiled_artifact_section_table_binary, materialize_embedded_artifact_support,
        parse_nuis_lowering_index_from_source,
        protocol::{
            COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML,
            COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY, COMPILED_ARTIFACT_SECTION_HOST_BINARY,
            COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
            COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML, COMPILED_ARTIFACT_SECTION_METADATA_TOML,
        },
        DomainBuildUnitPayloadBlob, DomainBuildUnitPayloadBlobSection, NuisCompiledArtifact,
        NuisExecutableEnvelope, NuisLifecycleContract,
    };

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nuis_artifact_{label}_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hex_encode_bytes(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{byte:02x}");
        }
        out
    }

    fn sample_compiled_artifact() -> NuisCompiledArtifact {
        let build_manifest_source = r#"manifest_schema = "nuis-build-manifest-v1"
input = "/tmp/demo.ns"
output_dir = "/tmp/out"
packaging_mode = "native-cpu-llvm"
path = "/tmp/out/nuis.executable.envelope.toml"
schema = "nuis-executable-envelope-v1"
package_count = 1
artifact_path = "/tmp/out/nuis.compiled.artifact"
artifact_schema = "nuis-compiled-artifact-v1"
artifact_binary_name = "demo"
artifact_binary_bytes = 3
lifecycle_schema = "nuis-lifecycle-contract-v1"
lifecycle_bootstrap_entry = "nuis.bootstrap.lifecycle.v1"
lifecycle_tick_policy = "cooperative"
lifecycle_shutdown_policy = "graceful"
lifecycle_yalivia_rpc = "disabled"
lifecycle_hook_surface = ["on_bootstrap"]
lifecycle_export_surface = ["main"]
lifecycle_runtime_capability_flags = ["runtime.tick"]
function_kind = "function-node"
graph_kind = "function-graph"
default_time_mode = "logical"
cpu_target_abi = "cpu.arm64.apple_aapcs64"
cpu_target_machine_arch = "arm64"
cpu_target_machine_os = "darwin"
cpu_target_object_format = "mach-o"
cpu_target_calling_abi = "aapcs64-darwin"
cpu_target_clang = "arm64-apple-darwin"
cpu_target_cross = false

[[execution_contract]]
package_id = "official.cpu"
domain_family = "cpu"

[[domain_build_unit]]
package_id = "official.cpu"
domain_family = "cpu"
backend_family = "llvm"
selected_lowering_target = "llvm"
contract_family = "nustar.cpu"
packaging_role = "host-binary"
"#
        .to_owned();
        NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
            cpu_target_machine_arch: "arm64".to_owned(),
            cpu_target_machine_os: "darwin".to_owned(),
            cpu_target_object_format: "mach-o".to_owned(),
            cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
            binary_name: "demo".to_owned(),
            binary_bytes: 3,
            build_manifest_bytes: build_manifest_source.len(),
            envelope: NuisExecutableEnvelope {
                schema: "nuis-executable-envelope-v1".to_owned(),
                executable_kind: "native".to_owned(),
                package_count: 1,
                domain_families: vec!["cpu".to_owned()],
                contract_families: vec!["nustar.cpu".to_owned()],
                function_kind: "function-node".to_owned(),
                graph_kind: "function-graph".to_owned(),
                default_time_mode: "logical".to_owned(),
            },
            lifecycle: NuisLifecycleContract {
                schema: "nuis-lifecycle-contract-v1".to_owned(),
                bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
                tick_policy: "cooperative".to_owned(),
                shutdown_policy: "graceful".to_owned(),
                yalivia_rpc: "disabled".to_owned(),
                hook_surface: vec!["on_bootstrap".to_owned()],
                export_surface: vec!["main".to_owned()],
                runtime_capability_flags: vec!["runtime.tick".to_owned()],
            },
            build_manifest_source,
            binary_blob: b"bin".to_vec(),
        }
    }

    #[test]
    fn section_table_artifact_roundtrips_through_generic_decoder() {
        let artifact = sample_compiled_artifact();

        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let decoded = decode_nuis_compiled_artifact_binary(&encoded).unwrap();
        let section_names = table
            .sections
            .iter()
            .map(|section| section.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            section_names,
            vec![
                COMPILED_ARTIFACT_SECTION_METADATA_TOML,
                COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY,
                COMPILED_ARTIFACT_SECTION_LIFECYCLE_TOML,
                COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML,
                COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML,
                COMPILED_ARTIFACT_SECTION_HOST_BINARY,
            ]
        );
        let lowering_index = table
            .section_utf8(COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        assert!(lowering_index.contains("schema = \"nuis-lowering-index-v1\""));
        assert!(lowering_index.contains("domain_unit_count = 1"));
        assert!(lowering_index.contains("backend_family = \"llvm\""));
        assert!(lowering_index.contains("selected_lowering_target = \"llvm\""));
        assert_eq!(decoded, artifact);
    }

    #[test]
    fn lowering_index_parser_validates_generated_section() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let lowering_index_source = table
            .section_utf8(COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        let lowering_index = parse_nuis_lowering_index_from_source(
            lowering_index_source,
            std::path::Path::new("<test-lowering-index>"),
        )
        .unwrap();

        assert_eq!(lowering_index.schema, "nuis-lowering-index-v1");
        assert_eq!(lowering_index.packaging_mode, "native-cpu-llvm");
        assert_eq!(lowering_index.domain_unit_count, 1);
        assert_eq!(lowering_index.units[0].domain_family, "cpu");
        assert_eq!(
            lowering_index.units[0].selected_lowering_target.as_deref(),
            Some("llvm")
        );
    }

    #[test]
    fn section_table_rejects_lowering_index_build_manifest_drift() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        let lowering_section = table
            .sections
            .iter_mut()
            .find(|section| section.name == COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .unwrap();
        let drifted = std::str::from_utf8(&lowering_section.bytes)
            .unwrap()
            .replace(
                "selected_lowering_target = \"llvm\"",
                "selected_lowering_target = \"shader-msl\"",
            );
        lowering_section.bytes = drifted.into_bytes();

        let encoded_with_drift = encode_nuis_compiled_artifact_section_table(&table).unwrap();
        let error = decode_nuis_compiled_artifact_binary(&encoded_with_drift).unwrap_err();

        assert!(error
            .to_string()
            .contains("field `selected_lowering_target` value `shader-msl`"));
    }

    #[test]
    fn section_table_rejects_missing_required_section() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        table
            .sections
            .retain(|section| section.name != COMPILED_ARTIFACT_SECTION_HOST_BINARY);

        let encoded_without_host = encode_nuis_compiled_artifact_section_table(&table).unwrap();
        let error =
            decode_nuis_compiled_artifact_section_table_binary(&encoded_without_host).unwrap_err();

        assert!(error
            .to_string()
            .contains("missing required section `host_binary`"));
    }

    #[test]
    fn section_table_rejects_duplicate_section_names() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
        table.sections.push(table.sections[0].clone());

        let encoded_with_duplicate = encode_nuis_compiled_artifact_section_table(&table).unwrap();
        let error = decode_nuis_compiled_artifact_section_table_binary(&encoded_with_duplicate)
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("duplicate section `metadata_toml`"));
    }

    #[test]
    fn section_table_exposes_lookup_helpers_for_linker_consumers() {
        let artifact = sample_compiled_artifact();
        let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        let table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();

        assert!(table.contains_section(COMPILED_ARTIFACT_SECTION_ENVELOPE_BINARY));
        assert!(table
            .section_names()
            .contains(&COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML));
        assert_eq!(
            table
                .section_utf8(COMPILED_ARTIFACT_SECTION_BUILD_MANIFEST_TOML)
                .unwrap(),
            artifact.build_manifest_source
        );
        assert_eq!(
            table
                .section_bytes(COMPILED_ARTIFACT_SECTION_HOST_BINARY)
                .unwrap(),
            artifact.binary_blob
        );
    }

    #[test]
    fn materializes_embedded_heterogeneous_support_files() {
        let blob = DomainBuildUnitPayloadBlob {
            domain_family: "network".to_owned(),
            package_id: "official.network".to_owned(),
            backend_family: Some("urlsession".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("socket-io".to_owned()),
            selected_lowering_target: Some("urlsession".to_owned()),
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "toml".to_owned(),
            sections: vec![
                DomainBuildUnitPayloadBlobSection {
                    name: "contract_toml".to_owned(),
                    bytes: br#"schema = "nuis-domain-build-payload-v1""#.to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "lowering_plan".to_owned(),
                    bytes: b"lowering".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "backend_stub".to_owned(),
                    bytes: b"backend".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "bridge_plan".to_owned(),
                    bytes: b"bridge-plan".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "network_ir_sidecar".to_owned(),
                    bytes: b"schema = \"nuis-network-ir-sidecar-v1\"".to_vec(),
                },
            ],
        };
        let encoded_blob = encode_domain_payload_blob(&blob).unwrap();
        let manifest = format!(
            r#"manifest_schema = "nuis-build-manifest-v1"
input = "/tmp/demo.ns"
output_dir = "/tmp/out"
packaging_mode = "window-aot-bundle"
path = "/tmp/out/nuis.executable.envelope.toml"
schema = "nuis-executable-envelope-v1"
package_count = 2
artifact_path = "/tmp/out/nuis.compiled.artifact"
artifact_schema = "nuis-compiled-artifact-v1"
artifact_binary_name = "demo.bin"
artifact_binary_bytes = 3
lifecycle_schema = "nuis-lifecycle-contract-v1"
lifecycle_bootstrap_entry = "nuis.bootstrap.lifecycle.v1"
lifecycle_tick_policy = "cooperative"
lifecycle_shutdown_policy = "graceful"
lifecycle_yalivia_rpc = "yalivia.rpc.bootstrap.v1"
lifecycle_hook_surface = ["on_bootstrap"]
lifecycle_export_surface = ["tick_export"]
lifecycle_runtime_capability_flags = ["runtime.tick"]
function_kind = "function-node"
graph_kind = "function-graph"
default_time_mode = "logical"
cpu_target_abi = "cpu.x86_64.sysv64"
cpu_target_machine_arch = "x86_64"
cpu_target_machine_os = "linux"
cpu_target_object_format = "elf"
cpu_target_calling_abi = "sysv64"
cpu_target_clang = "x86_64-unknown-linux-gnu"
cpu_target_cross = true
bridge_registry_path = "/tmp/out/nuis.bridge.registry.toml"
bridge_registry_schema = "nuis-bridge-registry-v1"
bridge_registry_units = 1
bridge_registry_inline = "schema = \"nuis-bridge-registry-v1\"\nbridge_count = 1\ndomains = [\"network\"]\n"
host_bridge_plan_index_path = "/tmp/out/nuis.host-bridge.plan-index.toml"
host_bridge_plan_index_schema = "nuis-host-bridge-plan-index-v1"
host_bridge_plan_units = 1
host_bridge_plan_index_inline = "schema = \"nuis-host-bridge-plan-index-v1\"\nplan_count = 1\ndomains = [\"network\"]\n"

[[artifact_hash]]
kind = "artifact"
path = "/tmp/out/nuis.compiled.artifact"
bytes = 3
fnv1a64 = "0x0000000000000000"

[[execution_contract]]
package_id = "official.cpu"
domain_family = "cpu"

[[execution_contract]]
package_id = "official.network"
domain_family = "network"

[[domain_build_unit]]
package_id = "official.cpu"
domain_family = "cpu"
selected_lowering_target = "llvm"
contract_family = "nustar.cpu"
packaging_role = "host-binary"

[[domain_build_unit]]
package_id = "official.network"
domain_family = "network"
backend_family = "urlsession"
selected_lowering_target = "urlsession"
artifact_stub_path = "/tmp/out/nuis.domain.network.artifact.toml"
artifact_stub_inline = "schema = \"nuis-domain-build-unit-v1\""
artifact_payload_path = "/tmp/out/nuis.domain.network.payload.toml"
artifact_bridge_stub_path = "/tmp/out/nuis.domain.network.bridge.stub.txt"
artifact_ir_sidecar_path = "/tmp/out/nuis.domain.network.lowering.ir.txt"
artifact_bridge_stub_inline = "schema = \"nuis-host-bridge-spec-v1\""
artifact_payload_blob_path = "/tmp/out/nuis.domain.network.payload.bin"
artifact_payload_blob_bytes = {blob_bytes}
artifact_payload_format = "ndpb-v2"
artifact_payload_blob_inline = "{blob_hex}"
contract_family = "nustar.network"
packaging_role = "hetero-contract"
"#,
            blob_bytes = encoded_blob.len(),
            blob_hex = hex_encode_bytes(&encoded_blob),
        );
        let artifact = NuisCompiledArtifact {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            packaging_mode: "window-aot-bundle".to_owned(),
            cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
            cpu_target_machine_arch: "x86_64".to_owned(),
            cpu_target_machine_os: "linux".to_owned(),
            cpu_target_object_format: "elf".to_owned(),
            cpu_target_calling_abi: "sysv64".to_owned(),
            binary_name: "demo.bin".to_owned(),
            binary_bytes: 3,
            build_manifest_bytes: manifest.len(),
            envelope: NuisExecutableEnvelope {
                schema: "nuis-executable-envelope-v1".to_owned(),
                executable_kind: "window-aot-bundle".to_owned(),
                package_count: 2,
                domain_families: vec!["cpu".to_owned(), "network".to_owned()],
                contract_families: vec!["nustar.cpu".to_owned(), "nustar.network".to_owned()],
                function_kind: "function-node".to_owned(),
                graph_kind: "function-graph".to_owned(),
                default_time_mode: "logical".to_owned(),
            },
            lifecycle: NuisLifecycleContract {
                schema: "nuis-lifecycle-contract-v1".to_owned(),
                bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
                tick_policy: "cooperative".to_owned(),
                shutdown_policy: "graceful".to_owned(),
                yalivia_rpc: "yalivia.rpc.bootstrap.v1".to_owned(),
                hook_surface: vec!["on_bootstrap".to_owned()],
                export_surface: vec!["tick_export".to_owned()],
                runtime_capability_flags: vec!["runtime.tick".to_owned()],
            },
            build_manifest_source: manifest,
            binary_blob: b"bin".to_vec(),
        };

        let out = temp_dir("materialize_support");
        let written = materialize_embedded_artifact_support(&artifact, &out).unwrap();

        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.bridge.registry.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.host-bridge.plan-index.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.artifact.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.payload.toml")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.payload.bin")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.bridge.stub.txt")));
        assert!(written
            .iter()
            .any(|path| path.ends_with("nuis.domain.network.lowering.ir.txt")));
        assert_eq!(
            fs::read(out.join("nuis.domain.network.payload.bin")).unwrap(),
            encoded_blob
        );
        assert_eq!(
            fs::read(out.join("nuis.domain.network.payload.toml")).unwrap(),
            br#"schema = "nuis-domain-build-payload-v1""#
        );
        assert_eq!(
            fs::read_to_string(out.join("nuis.domain.network.bridge.stub.txt")).unwrap(),
            r#"schema = "nuis-host-bridge-spec-v1""#
        );
        assert_eq!(
            fs::read_to_string(out.join("nuis.domain.network.lowering.ir.txt")).unwrap(),
            r#"schema = "nuis-network-ir-sidecar-v1""#
        );
    }
}

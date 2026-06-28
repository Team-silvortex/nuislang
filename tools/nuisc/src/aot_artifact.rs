use std::{
    fs,
    path::{Component, Path},
};

use nuis_artifact::protocol::{
    COMPILED_ARTIFACT_BINARY_VERSION, COMPILED_ARTIFACT_MAGIC,
    COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML, COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION,
};
use nuis_artifact::{
    decode_nuis_compiled_artifact_binary as shared_decode_nuis_compiled_artifact_binary,
    decode_nuis_compiled_artifact_section_table_binary as shared_decode_nuis_compiled_artifact_section_table_binary,
    decode_nuis_executable_envelope_binary as shared_decode_nuis_executable_envelope_binary,
    encode_nuis_compiled_artifact_binary as shared_encode_nuis_compiled_artifact_binary,
    encode_nuis_compiled_artifact_section_table_binary as shared_encode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_executable_envelope_binary as shared_encode_nuis_executable_envelope_binary,
    parse_nuis_compiled_artifact as shared_parse_nuis_compiled_artifact,
    parse_nuis_executable_envelope as shared_parse_nuis_executable_envelope,
    parse_nuis_executable_envelope_from_source as shared_parse_nuis_executable_envelope_from_source,
    parse_nuis_lowering_index_from_source as shared_parse_nuis_lowering_index_from_source,
    render_nuis_executable_envelope as shared_render_nuis_executable_envelope,
    write_nuis_compiled_artifact as shared_write_nuis_compiled_artifact,
    write_nuis_executable_envelope as shared_write_nuis_executable_envelope, NuisCompiledArtifact,
    NuisExecutableEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifactLoweringUnitInspect {
    pub package_id: String,
    pub domain_family: String,
    pub backend_family: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub artifact_ir_sidecar_path: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisCompiledArtifactContainerInspect {
    pub magic: String,
    pub binary_version: u16,
    pub container_kind: String,
    pub section_count: usize,
    pub section_names: Vec<String>,
    pub section_table_valid: bool,
    pub lowering_unit_count: usize,
    pub lowering_domain_families: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub lowering_units: Vec<NuisCompiledArtifactLoweringUnitInspect>,
}

pub(crate) fn validate_artifact_binary_name(
    field: &str,
    value: &str,
    context: &Path,
) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty() || path.components().count() != 1 {
        return Err(format!(
            "`{}` has unsafe {field} `{}`; expected a single file name",
            context.display(),
            value
        ));
    }
    match path.components().next() {
        Some(Component::Normal(_)) => Ok(()),
        _ => Err(format!(
            "`{}` has unsafe {field} `{}`; expected a plain file name",
            context.display(),
            value
        )),
    }
}

pub fn validate_nuis_compiled_artifact_layout(
    path: &Path,
    artifact: &NuisCompiledArtifact,
) -> Result<(), String> {
    validate_artifact_binary_name("binary_name", &artifact.binary_name, path)
}

pub fn render_nuis_executable_envelope(envelope: &NuisExecutableEnvelope) -> String {
    shared_render_nuis_executable_envelope(envelope)
}

pub fn encode_nuis_executable_envelope_binary(
    envelope: &NuisExecutableEnvelope,
) -> Result<Vec<u8>, String> {
    shared_encode_nuis_executable_envelope_binary(envelope).map_err(|error| error.to_string())
}

pub fn decode_nuis_executable_envelope_binary(
    bytes: &[u8],
) -> Result<NuisExecutableEnvelope, String> {
    shared_decode_nuis_executable_envelope_binary(bytes).map_err(|error| error.to_string())
}

pub fn write_nuis_executable_envelope(
    path: &Path,
    envelope: &NuisExecutableEnvelope,
) -> Result<(), String> {
    shared_write_nuis_executable_envelope(path, envelope).map_err(|error| error.to_string())
}

pub fn encode_nuis_compiled_artifact_binary(
    artifact: &NuisCompiledArtifact,
) -> Result<Vec<u8>, String> {
    shared_encode_nuis_compiled_artifact_binary(artifact).map_err(|error| error.to_string())
}

pub fn encode_nuis_compiled_artifact_section_table_binary(
    artifact: &NuisCompiledArtifact,
) -> Result<Vec<u8>, String> {
    shared_encode_nuis_compiled_artifact_section_table_binary(artifact)
        .map_err(|error| error.to_string())
}

pub fn decode_nuis_compiled_artifact_binary(bytes: &[u8]) -> Result<NuisCompiledArtifact, String> {
    shared_decode_nuis_compiled_artifact_binary(bytes).map_err(|error| error.to_string())
}

pub fn inspect_nuis_compiled_artifact_container(
    path: &Path,
) -> Result<NuisCompiledArtifactContainerInspect, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    if bytes.len() < 6 {
        return Err(format!(
            "`{}` is too short to be a nuis compiled artifact container",
            path.display()
        ));
    }
    if &bytes[..4] != COMPILED_ARTIFACT_MAGIC {
        return Err(format!(
            "`{}` has invalid nuis artifact magic",
            path.display()
        ));
    }
    let binary_version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let magic = std::str::from_utf8(COMPILED_ARTIFACT_MAGIC)
        .unwrap_or("NART")
        .to_owned();
    if binary_version == COMPILED_ARTIFACT_BINARY_VERSION {
        return Ok(NuisCompiledArtifactContainerInspect {
            magic,
            binary_version,
            container_kind: "compiled-artifact-v1".to_owned(),
            section_count: 0,
            section_names: Vec::new(),
            section_table_valid: true,
            lowering_unit_count: 0,
            lowering_domain_families: Vec::new(),
            lowering_targets: Vec::new(),
            lowering_units: Vec::new(),
        });
    }
    if binary_version == COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION {
        let table = shared_decode_nuis_compiled_artifact_section_table_binary(&bytes)
            .map_err(|error| error.to_string())?;
        let _ = shared_decode_nuis_compiled_artifact_binary(&bytes).map_err(|error| {
            format!(
                "`{}` has inconsistent nuis artifact section payloads: {error}",
                path.display()
            )
        })?;
        let lowering_index_source = table
            .section_utf8(COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
            .map_err(|error| error.to_string())?;
        let lowering_index = shared_parse_nuis_lowering_index_from_source(
            lowering_index_source,
            Path::new("<compiled-artifact-lowering-index>"),
        )
        .map_err(|error| error.to_string())?;
        let mut lowering_domain_families = lowering_index
            .units
            .iter()
            .map(|unit| unit.domain_family.clone())
            .collect::<Vec<_>>();
        lowering_domain_families.sort();
        lowering_domain_families.dedup();
        let mut lowering_targets = lowering_index
            .units
            .iter()
            .filter_map(|unit| unit.selected_lowering_target.clone())
            .collect::<Vec<_>>();
        lowering_targets.sort();
        lowering_targets.dedup();
        let lowering_units = lowering_index
            .units
            .iter()
            .map(|unit| NuisCompiledArtifactLoweringUnitInspect {
                package_id: unit.package_id.clone(),
                domain_family: unit.domain_family.clone(),
                backend_family: unit.backend_family.clone(),
                selected_lowering_target: unit.selected_lowering_target.clone(),
                artifact_ir_sidecar_path: unit.artifact_ir_sidecar_path.clone(),
                contract_family: unit.contract_family.clone(),
                packaging_role: unit.packaging_role.clone(),
            })
            .collect();
        return Ok(NuisCompiledArtifactContainerInspect {
            magic,
            binary_version,
            container_kind: "compiled-artifact-section-table-v2".to_owned(),
            section_count: table.sections.len(),
            section_names: table
                .section_names()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            section_table_valid: true,
            lowering_unit_count: lowering_index.domain_unit_count,
            lowering_domain_families,
            lowering_targets,
            lowering_units,
        });
    }
    Err(format!(
        "`{}` has unsupported nuis artifact binary version `{binary_version}`",
        path.display()
    ))
}

pub fn write_nuis_compiled_artifact(
    path: &Path,
    artifact: &NuisCompiledArtifact,
) -> Result<(), String> {
    shared_write_nuis_compiled_artifact(path, artifact).map_err(|error| error.to_string())
}

pub fn parse_nuis_compiled_artifact(path: &Path) -> Result<NuisCompiledArtifact, String> {
    shared_parse_nuis_compiled_artifact(path).map_err(|error| error.to_string())
}

pub fn parse_nuis_executable_envelope(path: &Path) -> Result<NuisExecutableEnvelope, String> {
    shared_parse_nuis_executable_envelope(path).map_err(|error| error.to_string())
}

pub fn parse_nuis_executable_envelope_from_source(
    source: &str,
    path: &Path,
) -> Result<NuisExecutableEnvelope, String> {
    shared_parse_nuis_executable_envelope_from_source(source, path)
        .map_err(|error| error.to_string())
}

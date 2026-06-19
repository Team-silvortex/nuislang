use std::{fs, path::Path};

use crate::{
    toml::{
        escape_toml_string, parse_optional_toml_string, parse_required_toml_string,
        parse_required_toml_string_array, parse_required_toml_usize, render_string_array,
    },
    ArtifactError,
};

const NUIS_ENVELOPE_BINARY_MAGIC: &[u8; 4] = b"NENV";
const NUIS_ENVELOPE_BINARY_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisExecutableEnvelope {
    pub schema: String,
    pub executable_kind: String,
    pub package_count: usize,
    pub domain_families: Vec<String>,
    pub contract_families: Vec<String>,
    pub function_kind: String,
    pub graph_kind: String,
    pub default_time_mode: String,
}

pub fn render_nuis_executable_envelope(envelope: &NuisExecutableEnvelope) -> String {
    let mut out = String::new();
    out.push_str("envelope_schema = \"nuis-executable-envelope-v1\"\n");
    out.push_str(&format!(
        "executable_kind = \"{}\"\n",
        escape_toml_string(&envelope.executable_kind)
    ));
    out.push_str(&format!("package_count = {}\n", envelope.package_count));
    out.push_str(&format!(
        "domain_families = {}\n",
        render_string_array(&envelope.domain_families)
    ));
    out.push_str(&format!(
        "contract_families = {}\n",
        render_string_array(&envelope.contract_families)
    ));
    out.push_str(&format!(
        "function_kind = \"{}\"\n",
        escape_toml_string(&envelope.function_kind)
    ));
    out.push_str(&format!(
        "graph_kind = \"{}\"\n",
        escape_toml_string(&envelope.graph_kind)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        escape_toml_string(&envelope.default_time_mode)
    ));
    out
}

pub fn encode_nuis_executable_envelope_binary(
    envelope: &NuisExecutableEnvelope,
) -> Result<Vec<u8>, ArtifactError> {
    let payload = render_nuis_executable_envelope(envelope).into_bytes();
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| ArtifactError::new("nuis executable envelope payload exceeds 4 GiB"))?;
    let mut out = Vec::with_capacity(4 + 2 + 4 + payload.len());
    out.extend_from_slice(NUIS_ENVELOPE_BINARY_MAGIC);
    out.extend_from_slice(&NUIS_ENVELOPE_BINARY_VERSION.to_le_bytes());
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

pub fn decode_nuis_executable_envelope_binary(
    bytes: &[u8],
) -> Result<NuisExecutableEnvelope, ArtifactError> {
    if bytes.len() < 10 {
        return Err(ArtifactError::new("nuis executable envelope binary is too short"));
    }
    if &bytes[..4] != NUIS_ENVELOPE_BINARY_MAGIC {
        return Err(ArtifactError::new("nuis executable envelope binary has invalid magic"));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != NUIS_ENVELOPE_BINARY_VERSION {
        return Err(ArtifactError::new(format!(
            "unsupported nuis executable envelope binary version `{version}`"
        )));
    }
    let payload_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    if bytes.len() != 10 + payload_len {
        return Err(ArtifactError::new(format!(
            "nuis executable envelope binary length mismatch: header says {payload_len} payload bytes, actual {}",
            bytes.len().saturating_sub(10)
        )));
    }
    let payload = std::str::from_utf8(&bytes[10..]).map_err(|error| {
        ArtifactError::new(format!(
            "nuis executable envelope payload is not valid UTF-8: {error}"
        ))
    })?;
    parse_nuis_executable_envelope_from_source(payload, Path::new("<nuis-envelope-binary>"))
}

pub fn write_nuis_executable_envelope(
    path: &Path,
    envelope: &NuisExecutableEnvelope,
) -> Result<(), ArtifactError> {
    let out = render_nuis_executable_envelope(envelope);
    fs::write(path, out)
        .map_err(|error| ArtifactError::new(format!("failed to write `{}`: {error}", path.display())))
}

pub fn parse_nuis_executable_envelope(path: &Path) -> Result<NuisExecutableEnvelope, ArtifactError> {
    let source = fs::read_to_string(path)
        .map_err(|error| ArtifactError::new(format!("failed to read `{}`: {error}", path.display())))?;
    parse_nuis_executable_envelope_from_source(&source, path)
}

pub fn parse_nuis_executable_envelope_from_source(
    source: &str,
    path: &Path,
) -> Result<NuisExecutableEnvelope, ArtifactError> {
    let schema = parse_optional_toml_string(source, "envelope_schema")
        .or_else(|| parse_optional_toml_string(source, "schema"))
        .ok_or_else(|| ArtifactError::new(format!("`{}` is missing required key `schema`", path.display())))?;
    let executable_kind = parse_required_toml_string(source, "executable_kind", path)?;
    let package_count = parse_required_toml_usize(source, "package_count", path)?;
    let domain_families = parse_required_toml_string_array(source, "domain_families", path)?;
    let contract_families = parse_required_toml_string_array(source, "contract_families", path)?;
    let function_kind = parse_required_toml_string(source, "function_kind", path)?;
    let graph_kind = parse_required_toml_string(source, "graph_kind", path)?;
    let default_time_mode = parse_required_toml_string(source, "default_time_mode", path)?;

    if package_count != domain_families.len() || package_count != contract_families.len() {
        return Err(ArtifactError::new(format!(
            "`{}` nuis envelope package_count mismatch: package_count={}, domains={}, contract_families={}",
            path.display(),
            package_count,
            domain_families.len(),
            contract_families.len()
        )));
    }

    Ok(NuisExecutableEnvelope {
        schema,
        executable_kind,
        package_count,
        domain_families,
        contract_families,
        function_kind,
        graph_kind,
        default_time_mode,
    })
}

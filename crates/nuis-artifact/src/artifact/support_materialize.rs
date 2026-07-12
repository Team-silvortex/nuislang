use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    decode_domain_payload_blob, parse_build_manifest_from_source,
    protocol::DOMAIN_PAYLOAD_SECTION_CONTRACT_TOML, ArtifactError, BuildManifestDomainBuildUnit,
};

use super::NuisCompiledArtifact;

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
    if !value.len().is_multiple_of(2) {
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

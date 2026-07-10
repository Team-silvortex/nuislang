use std::path::Path;

use nuis_artifact::{
    decode_domain_payload_blob, parse_bridge_registry, parse_build_manifest_from_source,
    parse_clock_protocol, parse_host_bridge_plan_index, parse_nuis_compiled_artifact,
    BridgeRegistry, BuildManifest, ClockProtocol, DomainBuildUnitPayloadBlob, HostBridgePlanIndex,
    NuisCompiledArtifact,
};

use crate::{LoadedExecutable, RuntimeError};

#[derive(Debug, Default)]
pub struct RuntimeLoader;

impl RuntimeLoader {
    pub fn load_from_artifact_path(
        &self,
        artifact_path: &Path,
    ) -> Result<LoadedExecutable, RuntimeError> {
        let artifact = parse_nuis_compiled_artifact(artifact_path).map_err(|error| {
            RuntimeError::new(format!("failed to load compiled artifact: {error}"))
        })?;
        self.load_from_compiled_artifact(artifact)
    }

    pub fn load_from_compiled_artifact(
        &self,
        artifact: NuisCompiledArtifact,
    ) -> Result<LoadedExecutable, RuntimeError> {
        let manifest = self.load_embedded_manifest(&artifact)?;
        let domain_payload_blobs = self.load_domain_payload_blobs(&manifest)?;
        let bridge_registry = self.load_bridge_registry(&manifest)?;
        let host_bridge_plan_index = self.load_host_bridge_plan_index(&manifest)?;
        let clock_protocol = self.load_clock_protocol(&manifest)?;
        Ok(LoadedExecutable {
            envelope: artifact.envelope.clone(),
            domain_units: manifest.domain_build_units.clone(),
            domain_payload_blobs,
            bridge_registry,
            host_bridge_plan_index,
            clock_protocol,
            artifact,
            manifest,
        })
    }

    fn load_embedded_manifest(
        &self,
        artifact: &NuisCompiledArtifact,
    ) -> Result<BuildManifest, RuntimeError> {
        parse_build_manifest_from_source(
            &artifact.build_manifest_source,
            Path::new("<embedded-build-manifest>"),
        )
        .map_err(|error| {
            RuntimeError::new(format!("failed to parse embedded build manifest: {error}"))
        })
    }

    fn load_bridge_registry(
        &self,
        manifest: &BuildManifest,
    ) -> Result<Option<BridgeRegistry>, RuntimeError> {
        if let Some(source) = &manifest.bridge_registry_inline {
            return nuis_artifact::parse_bridge_registry_from_source(
                source,
                Path::new("<embedded-bridge-registry>"),
            )
            .map(Some)
            .map_err(|error| {
                RuntimeError::new(format!("failed to parse embedded bridge registry: {error}"))
            });
        }
        let Some(path) = &manifest.bridge_registry_path else {
            return Ok(None);
        };
        parse_bridge_registry(Path::new(path))
            .map(Some)
            .map_err(|error| RuntimeError::new(format!("failed to parse bridge registry: {error}")))
    }

    fn load_host_bridge_plan_index(
        &self,
        manifest: &BuildManifest,
    ) -> Result<Option<HostBridgePlanIndex>, RuntimeError> {
        if let Some(source) = &manifest.host_bridge_plan_index_inline {
            return nuis_artifact::parse_host_bridge_plan_index_from_source(
                source,
                Path::new("<embedded-host-bridge-plan-index>"),
            )
            .map(Some)
            .map_err(|error| {
                RuntimeError::new(format!(
                    "failed to parse embedded host bridge plan index: {error}"
                ))
            });
        }
        let Some(path) = &manifest.host_bridge_plan_index_path else {
            return Ok(None);
        };
        parse_host_bridge_plan_index(Path::new(path))
            .map(Some)
            .map_err(|error| {
                RuntimeError::new(format!("failed to parse host bridge plan index: {error}"))
            })
    }

    fn load_clock_protocol(
        &self,
        manifest: &BuildManifest,
    ) -> Result<Option<ClockProtocol>, RuntimeError> {
        let protocol = if let Some(source) = &manifest.clock_protocol_inline {
            nuis_artifact::parse_clock_protocol_from_source(
                source,
                Path::new("<embedded-clock-protocol>"),
            )
            .map_err(|error| {
                RuntimeError::new(format!("failed to parse embedded clock protocol: {error}"))
            })?
        } else if let Some(path) = &manifest.clock_protocol_path {
            parse_clock_protocol(Path::new(path)).map_err(|error| {
                RuntimeError::new(format!("failed to parse clock protocol: {error}"))
            })?
        } else if manifest.clock_protocol_schema.is_some() || manifest.clock_protocol_domains > 0 {
            return Err(RuntimeError::new(
                "build manifest declares clock protocol metadata but has no clock protocol artifact"
                    .to_owned(),
            ));
        } else {
            return Ok(None);
        };

        validate_loaded_clock_protocol(manifest, &protocol)?;
        Ok(Some(protocol))
    }

    fn load_domain_payload_blobs(
        &self,
        manifest: &BuildManifest,
    ) -> Result<Vec<DomainBuildUnitPayloadBlob>, RuntimeError> {
        let mut blobs = Vec::new();
        for unit in manifest
            .domain_build_units
            .iter()
            .filter(|unit| unit.is_heterogeneous())
        {
            let payload = if let Some(path) = &unit.artifact_payload_blob_path {
                match std::fs::read(path) {
                    Ok(bytes) => bytes,
                    Err(path_error) => {
                        if let Some(inline) = &unit.artifact_payload_blob_inline {
                            decode_hex_bytes(inline).map_err(RuntimeError::new)?
                        } else {
                            return Err(RuntimeError::new(format!(
                                "failed to read domain payload blob for `{}` from `{}`: {path_error}",
                                unit.domain_family, path
                            )));
                        }
                    }
                }
            } else if let Some(inline) = &unit.artifact_payload_blob_inline {
                decode_hex_bytes(inline).map_err(RuntimeError::new)?
            } else {
                return Err(RuntimeError::new(format!(
                    "missing domain payload blob for heterogeneous domain `{}`",
                    unit.domain_family
                )));
            };
            let blob = decode_domain_payload_blob(&payload).map_err(|error| {
                RuntimeError::new(format!(
                    "failed to decode domain payload blob for `{}`: {error}",
                    unit.domain_family
                ))
            })?;
            if blob.domain_family != unit.domain_family {
                return Err(RuntimeError::new(format!(
                    "domain payload blob family mismatch for `{}`: blob reports `{}`",
                    unit.domain_family, blob.domain_family
                )));
            }
            if blob.package_id != unit.package_id {
                return Err(RuntimeError::new(format!(
                    "domain payload blob package mismatch for `{}`: blob reports `{}`",
                    unit.domain_family, blob.package_id
                )));
            }
            if blob.selected_lowering_target != unit.selected_lowering_target {
                return Err(RuntimeError::new(format!(
                    "domain payload blob lowering target mismatch for `{}`",
                    unit.domain_family
                )));
            }
            blobs.push(blob);
        }
        Ok(blobs)
    }
}

fn validate_loaded_clock_protocol(
    manifest: &BuildManifest,
    protocol: &ClockProtocol,
) -> Result<(), RuntimeError> {
    if let Some(schema) = &manifest.clock_protocol_schema {
        if schema != "nuis-clock-protocol-v1" {
            return Err(RuntimeError::new(format!(
                "unsupported manifest clock protocol schema `{schema}`"
            )));
        }
    }
    if protocol.schema != "nuis-clock-protocol-v1" {
        return Err(RuntimeError::new(format!(
            "unsupported clock protocol schema `{}`",
            protocol.schema
        )));
    }
    if !protocol.validation_valid {
        return Err(RuntimeError::new(
            "clock protocol validation flag is false".to_owned(),
        ));
    }
    if manifest.clock_protocol_domains > 0
        && manifest.clock_protocol_domains != protocol.domains.len()
    {
        return Err(RuntimeError::new(format!(
            "clock protocol domain count mismatch: manifest={}, protocol={}",
            manifest.clock_protocol_domains,
            protocol.domains.len()
        )));
    }
    for domain in &protocol.domains {
        let present = manifest.domain_build_units.iter().any(|unit| {
            unit.domain_family == domain.domain_family && unit.package_id == domain.package_id
        });
        if !present {
            return Err(RuntimeError::new(format!(
                "clock protocol domain `{}` package `{}` is not present in build manifest",
                domain.domain_family, domain.package_id
            )));
        }
    }
    Ok(())
}

fn decode_hex_bytes(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("hex payload length must be even".to_owned());
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let chunk = std::str::from_utf8(&bytes[index..index + 2])
            .map_err(|_| "hex payload is not valid UTF-8".to_owned())?;
        let byte =
            u8::from_str_radix(chunk, 16).map_err(|_| format!("invalid hex byte `{chunk}`"))?;
        out.push(byte);
        index += 2;
    }
    Ok(out)
}

#[cfg(test)]
mod tests;

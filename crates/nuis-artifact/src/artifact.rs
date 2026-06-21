use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    decode_domain_payload_blob,
    envelope::{decode_nuis_executable_envelope_binary, encode_nuis_executable_envelope_binary},
    parse_build_manifest_from_source,
    toml::{parse_optional_toml_string_array, render_string_array},
    ArtifactError, BuildManifestDomainBuildUnit, NuisExecutableEnvelope,
};

const NUIS_COMPILED_ARTIFACT_MAGIC: &[u8; 4] = b"NART";
const NUIS_COMPILED_ARTIFACT_VERSION: u16 = 1;

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

fn encode_u32_len(len: usize, what: &str) -> Result<[u8; 4], ArtifactError> {
    let len = u32::try_from(len)
        .map_err(|_| ArtifactError::new(format!("{what} exceeds 4 GiB and cannot be encoded")))?;
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
    out.extend_from_slice(NUIS_COMPILED_ARTIFACT_MAGIC);
    out.extend_from_slice(&NUIS_COMPILED_ARTIFACT_VERSION.to_le_bytes());
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

pub fn decode_nuis_compiled_artifact_binary(
    bytes: &[u8],
) -> Result<NuisCompiledArtifact, ArtifactError> {
    if bytes.len() < 78 {
        return Err(ArtifactError::new(
            "nuis compiled artifact binary is too short",
        ));
    }
    if &bytes[..4] != NUIS_COMPILED_ARTIFACT_MAGIC {
        return Err(ArtifactError::new(
            "nuis compiled artifact binary has invalid magic",
        ));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != NUIS_COMPILED_ARTIFACT_VERSION {
        return Err(ArtifactError::new(format!(
            "unsupported nuis compiled artifact binary version `{version}`"
        )));
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
        schema: "nuis-compiled-artifact-v1".to_owned(),
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
            .find(|section| section.name == "contract_toml")
        {
            let path = output_dir.join(format!("nuis.domain.{}.payload.toml", unit.domain_family));
            fs::write(&path, &contract_section.bytes).map_err(|error| {
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
        encode_domain_payload_blob, materialize_embedded_artifact_support,
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

    #[test]
    fn materializes_embedded_heterogeneous_support_files() {
        let blob = DomainBuildUnitPayloadBlob {
            domain_family: "network".to_owned(),
            package_id: "official.network".to_owned(),
            backend_family: Some("urlsession".to_owned()),
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
    }
}

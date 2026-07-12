use crate::{
    envelope::{decode_nuis_executable_envelope_binary, encode_nuis_executable_envelope_binary},
    protocol::{
        COMPILED_ARTIFACT_BINARY_VERSION, COMPILED_ARTIFACT_MAGIC, COMPILED_ARTIFACT_SCHEMA_V1,
        COMPILED_ARTIFACT_SECTION_TABLE_BINARY_VERSION,
    },
    toml::{parse_optional_toml_string_array, render_string_array},
    ArtifactError,
};

use super::{
    decode_nuis_compiled_artifact_section_table_binary, encode_u32_len,
    section_table::section_table_to_compiled_artifact, NuisCompiledArtifact, NuisLifecycleContract,
};

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

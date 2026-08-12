use std::collections::BTreeSet;

use crate::ArtifactError;

use super::{encode_u32_len, NuisCompiledArtifactHostObject};

const HOST_OBJECT_BUNDLE_MAGIC: &[u8; 4] = b"NHOB";
const HOST_OBJECT_BUNDLE_VERSION: u16 = 1;

pub(super) fn encode_host_object_bundle(
    objects: &[NuisCompiledArtifactHostObject],
) -> Result<Vec<u8>, ArtifactError> {
    validate_host_objects(objects)?;
    let mut out = Vec::new();
    out.extend_from_slice(HOST_OBJECT_BUNDLE_MAGIC);
    out.extend_from_slice(&HOST_OBJECT_BUNDLE_VERSION.to_le_bytes());
    out.extend_from_slice(&encode_u32_len(objects.len(), "host object count")?);
    for object in objects {
        out.extend_from_slice(&encode_u32_len(object.object_id.len(), "host object id")?);
        out.extend_from_slice(&encode_u32_len(object.role.len(), "host object role")?);
        out.extend_from_slice(&encode_u32_len(
            object.object_format.len(),
            "host object format",
        )?);
        out.extend_from_slice(&encode_u32_len(object.bytes.len(), "host object payload")?);
        out.extend_from_slice(object.object_id.as_bytes());
        out.extend_from_slice(object.role.as_bytes());
        out.extend_from_slice(object.object_format.as_bytes());
        out.extend_from_slice(&object.bytes);
    }
    Ok(out)
}

pub(super) fn decode_host_object_bundle(
    bytes: &[u8],
) -> Result<Vec<NuisCompiledArtifactHostObject>, ArtifactError> {
    if bytes.len() < 10 {
        return Err(ArtifactError::new("host object bundle is too short"));
    }
    if &bytes[..4] != HOST_OBJECT_BUNDLE_MAGIC {
        return Err(ArtifactError::new("host object bundle has invalid magic"));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != HOST_OBJECT_BUNDLE_VERSION {
        return Err(ArtifactError::new(format!(
            "unsupported host object bundle version `{version}`"
        )));
    }
    let mut offset = 6;
    let count = next_u32(bytes, &mut offset, "host object count")? as usize;
    let mut objects = Vec::with_capacity(count);
    for index in 0..count {
        let object_id_len = next_u32(bytes, &mut offset, "host object id length")? as usize;
        let role_len = next_u32(bytes, &mut offset, "host object role length")? as usize;
        let object_format_len = next_u32(bytes, &mut offset, "host object format length")? as usize;
        let payload_len = next_u32(bytes, &mut offset, "host object payload length")? as usize;
        let object_id = take_utf8(bytes, &mut offset, object_id_len, index, "id")?;
        let role = take_utf8(bytes, &mut offset, role_len, index, "role")?;
        let object_format = take_utf8(bytes, &mut offset, object_format_len, index, "format")?;
        let payload = take_bytes(bytes, &mut offset, payload_len, index)?.to_vec();
        objects.push(NuisCompiledArtifactHostObject {
            object_id,
            role,
            object_format,
            bytes: payload,
        });
    }
    if offset != bytes.len() {
        return Err(ArtifactError::new(format!(
            "host object bundle has {} trailing byte(s)",
            bytes.len() - offset
        )));
    }
    validate_host_objects(&objects)?;
    Ok(objects)
}

fn validate_host_objects(objects: &[NuisCompiledArtifactHostObject]) -> Result<(), ArtifactError> {
    let mut ids = BTreeSet::new();
    for object in objects {
        if !valid_identity(&object.object_id) {
            return Err(ArtifactError::new(format!(
                "host object id `{}` is not a stable identity token",
                object.object_id
            )));
        }
        if !ids.insert(object.object_id.as_str()) {
            return Err(ArtifactError::new(format!(
                "host object bundle contains duplicate id `{}`",
                object.object_id
            )));
        }
        if !valid_identity(&object.role) {
            return Err(ArtifactError::new(format!(
                "host object role `{}` is not a stable identity token",
                object.role
            )));
        }
        if !valid_identity(&object.object_format) {
            return Err(ArtifactError::new(format!(
                "host object format `{}` is not a stable identity token",
                object.object_format
            )));
        }
        if object.bytes.is_empty() {
            return Err(ArtifactError::new(format!(
                "host object `{}` has an empty payload",
                object.object_id
            )));
        }
    }
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn next_u32(bytes: &[u8], offset: &mut usize, label: &str) -> Result<u32, ArtifactError> {
    let raw: [u8; 4] = bytes
        .get(*offset..offset.saturating_add(4))
        .ok_or_else(|| ArtifactError::new(format!("{label} is truncated")))?
        .try_into()
        .map_err(|_| ArtifactError::new(format!("{label} is malformed")))?;
    *offset += 4;
    Ok(u32::from_le_bytes(raw))
}

fn take_utf8(
    bytes: &[u8],
    offset: &mut usize,
    len: usize,
    index: usize,
    field: &str,
) -> Result<String, ArtifactError> {
    String::from_utf8(take_bytes(bytes, offset, len, index)?.to_vec()).map_err(|error| {
        ArtifactError::new(format!(
            "host object {index} {field} is not valid UTF-8: {error}"
        ))
    })
}

fn take_bytes<'a>(
    bytes: &'a [u8],
    offset: &mut usize,
    len: usize,
    index: usize,
) -> Result<&'a [u8], ArtifactError> {
    let end = offset
        .checked_add(len)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| ArtifactError::new(format!("host object {index} payload is truncated")))?;
    let value = &bytes[*offset..end];
    *offset = end;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_object_bundle_roundtrips_role_and_payload_identity() {
        let objects = vec![
            NuisCompiledArtifactHostObject {
                object_id: "host.program-llvm".to_owned(),
                role: "program-llvm".to_owned(),
                object_format: "mach-o".to_owned(),
                bytes: vec![1, 2, 3],
            },
            NuisCompiledArtifactHostObject {
                object_id: "host.runtime-shim".to_owned(),
                role: "runtime-shim".to_owned(),
                object_format: "mach-o".to_owned(),
                bytes: vec![4, 5],
            },
        ];

        let encoded = encode_host_object_bundle(&objects).unwrap();
        assert_eq!(decode_host_object_bundle(&encoded).unwrap(), objects);
    }

    #[test]
    fn host_object_bundle_rejects_duplicate_ids() {
        let object = NuisCompiledArtifactHostObject {
            object_id: "host.duplicate".to_owned(),
            role: "program-llvm".to_owned(),
            object_format: "mach-o".to_owned(),
            bytes: vec![1],
        };

        assert!(encode_host_object_bundle(&[object.clone(), object])
            .unwrap_err()
            .to_string()
            .contains("duplicate id"));
    }
}

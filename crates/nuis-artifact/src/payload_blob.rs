use crate::{ArtifactError, BuildManifestDomainBuildUnit};

const NUIS_DOMAIN_PAYLOAD_BLOB_MAGIC: &[u8; 4] = b"NDPB";
const NUIS_DOMAIN_PAYLOAD_BLOB_VERSION: u16 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainBuildUnitPayloadBlob {
    pub domain_family: String,
    pub package_id: String,
    pub backend_family: Option<String>,
    pub vendor: Option<String>,
    pub device_class: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
    pub payload_kind: String,
    pub payload_format: String,
    pub sections: Vec<DomainBuildUnitPayloadBlobSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainBuildUnitPayloadBlobSection {
    pub name: String,
    pub bytes: Vec<u8>,
}

fn encode_u32_len(len: usize, what: &str) -> Result<[u8; 4], ArtifactError> {
    let len = u32::try_from(len)
        .map_err(|_| ArtifactError::new(format!("{what} exceeds 4 GiB and cannot be encoded")))?;
    Ok(len.to_le_bytes())
}

pub fn encode_domain_payload_blob(
    blob: &DomainBuildUnitPayloadBlob,
) -> Result<Vec<u8>, ArtifactError> {
    let domain_family = blob.domain_family.as_bytes();
    let package_id = blob.package_id.as_bytes();
    let backend_family = blob.backend_family.as_deref().unwrap_or("").as_bytes();
    let vendor = blob.vendor.as_deref().unwrap_or("").as_bytes();
    let device_class = blob.device_class.as_deref().unwrap_or("").as_bytes();
    let selected_lowering_target = blob
        .selected_lowering_target
        .as_deref()
        .unwrap_or("")
        .as_bytes();
    let contract_family = blob.contract_family.as_bytes();
    let packaging_role = blob.packaging_role.as_bytes();
    let payload_kind = blob.payload_kind.as_bytes();
    let payload_format = blob.payload_format.as_bytes();

    let mut out = Vec::new();
    out.extend_from_slice(NUIS_DOMAIN_PAYLOAD_BLOB_MAGIC);
    out.extend_from_slice(&NUIS_DOMAIN_PAYLOAD_BLOB_VERSION.to_le_bytes());
    out.extend_from_slice(&encode_u32_len(
        domain_family.len(),
        "domain payload blob domain_family",
    )?);
    out.extend_from_slice(&encode_u32_len(
        package_id.len(),
        "domain payload blob package_id",
    )?);
    out.extend_from_slice(&encode_u32_len(
        backend_family.len(),
        "domain payload blob backend_family",
    )?);
    out.extend_from_slice(&encode_u32_len(
        vendor.len(),
        "domain payload blob vendor",
    )?);
    out.extend_from_slice(&encode_u32_len(
        device_class.len(),
        "domain payload blob device_class",
    )?);
    out.extend_from_slice(&encode_u32_len(
        selected_lowering_target.len(),
        "domain payload blob selected_lowering_target",
    )?);
    out.extend_from_slice(&encode_u32_len(
        contract_family.len(),
        "domain payload blob contract_family",
    )?);
    out.extend_from_slice(&encode_u32_len(
        packaging_role.len(),
        "domain payload blob packaging_role",
    )?);
    out.extend_from_slice(&encode_u32_len(
        payload_kind.len(),
        "domain payload blob payload_kind",
    )?);
    out.extend_from_slice(&encode_u32_len(
        payload_format.len(),
        "domain payload blob payload_format",
    )?);
    out.extend_from_slice(&encode_u32_len(
        blob.sections.len(),
        "domain payload blob section_count",
    )?);
    for section in &blob.sections {
        out.extend_from_slice(&encode_u32_len(
            section.name.len(),
            "domain payload blob section_name",
        )?);
        out.extend_from_slice(&encode_u32_len(
            section.bytes.len(),
            "domain payload blob section_payload",
        )?);
    }
    out.extend_from_slice(domain_family);
    out.extend_from_slice(package_id);
    out.extend_from_slice(backend_family);
    out.extend_from_slice(vendor);
    out.extend_from_slice(device_class);
    out.extend_from_slice(selected_lowering_target);
    out.extend_from_slice(contract_family);
    out.extend_from_slice(packaging_role);
    out.extend_from_slice(payload_kind);
    out.extend_from_slice(payload_format);
    for section in &blob.sections {
        out.extend_from_slice(section.name.as_bytes());
        out.extend_from_slice(&section.bytes);
    }
    Ok(out)
}

pub fn decode_domain_payload_blob(
    bytes: &[u8],
) -> Result<DomainBuildUnitPayloadBlob, ArtifactError> {
    if bytes.len() < 54 {
        return Err(ArtifactError::new("domain payload blob is too short"));
    }
    if &bytes[..4] != NUIS_DOMAIN_PAYLOAD_BLOB_MAGIC {
        return Err(ArtifactError::new("domain payload blob has invalid magic"));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != NUIS_DOMAIN_PAYLOAD_BLOB_VERSION {
        return Err(ArtifactError::new(format!(
            "unsupported domain payload blob version `{version}`"
        )));
    }
    let mut offset = 6usize;
    let next_len = |bytes: &[u8], offset: &mut usize| -> Result<usize, ArtifactError> {
        if *offset + 4 > bytes.len() {
            return Err(ArtifactError::new(
                "domain payload blob header is truncated",
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
    let domain_family_len = next_len(bytes, &mut offset)?;
    let package_id_len = next_len(bytes, &mut offset)?;
    let backend_family_len = next_len(bytes, &mut offset)?;
    let vendor_len = next_len(bytes, &mut offset)?;
    let device_class_len = next_len(bytes, &mut offset)?;
    let selected_lowering_target_len = next_len(bytes, &mut offset)?;
    let contract_family_len = next_len(bytes, &mut offset)?;
    let packaging_role_len = next_len(bytes, &mut offset)?;
    let payload_kind_len = next_len(bytes, &mut offset)?;
    let payload_format_len = next_len(bytes, &mut offset)?;
    let section_count = next_len(bytes, &mut offset)?;
    let mut section_header_len = 0usize;
    let mut sections_meta = Vec::new();
    for _ in 0..section_count {
        let section_name_len = next_len(bytes, &mut offset)?;
        let section_payload_len = next_len(bytes, &mut offset)?;
        section_header_len += section_name_len + section_payload_len;
        sections_meta.push((section_name_len, section_payload_len));
    }
    let total_payload_len = domain_family_len
        + package_id_len
        + backend_family_len
        + vendor_len
        + device_class_len
        + selected_lowering_target_len
        + contract_family_len
        + packaging_role_len
        + payload_kind_len
        + payload_format_len
        + section_header_len;
    if bytes.len() != offset + total_payload_len {
        return Err(ArtifactError::new(format!(
            "domain payload blob length mismatch: header says {total_payload_len} payload bytes, actual {}",
            bytes.len().saturating_sub(offset)
        )));
    }
    let take_bytes =
        |bytes: &[u8], offset: &mut usize, len: usize| -> Result<Vec<u8>, ArtifactError> {
            if *offset + len > bytes.len() {
                return Err(ArtifactError::new(
                    "domain payload blob payload is truncated",
                ));
            }
            let value = bytes[*offset..*offset + len].to_vec();
            *offset += len;
            Ok(value)
        };
    let domain_family = String::from_utf8(take_bytes(bytes, &mut offset, domain_family_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob domain_family is not valid UTF-8: {error}"
            ))
        })?;
    let package_id =
        String::from_utf8(take_bytes(bytes, &mut offset, package_id_len)?).map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob package_id is not valid UTF-8: {error}"
            ))
        })?;
    let backend_family = String::from_utf8(take_bytes(bytes, &mut offset, backend_family_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob backend_family is not valid UTF-8: {error}"
            ))
        })?;
    let vendor = String::from_utf8(take_bytes(bytes, &mut offset, vendor_len)?).map_err(
        |error| ArtifactError::new(format!(
            "domain payload blob vendor is not valid UTF-8: {error}"
        )),
    )?;
    let device_class =
        String::from_utf8(take_bytes(bytes, &mut offset, device_class_len)?).map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob device_class is not valid UTF-8: {error}"
            ))
        })?;
    let selected_lowering_target = String::from_utf8(take_bytes(
        bytes,
        &mut offset,
        selected_lowering_target_len,
    )?)
    .map_err(|error| {
        ArtifactError::new(format!(
            "domain payload blob selected_lowering_target is not valid UTF-8: {error}"
        ))
    })?;
    let contract_family = String::from_utf8(take_bytes(bytes, &mut offset, contract_family_len)?)
        .map_err(|error| {
        ArtifactError::new(format!(
            "domain payload blob contract_family is not valid UTF-8: {error}"
        ))
    })?;
    let packaging_role = String::from_utf8(take_bytes(bytes, &mut offset, packaging_role_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob packaging_role is not valid UTF-8: {error}"
            ))
        })?;
    let payload_kind = String::from_utf8(take_bytes(bytes, &mut offset, payload_kind_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob payload_kind is not valid UTF-8: {error}"
            ))
        })?;
    let payload_format = String::from_utf8(take_bytes(bytes, &mut offset, payload_format_len)?)
        .map_err(|error| {
            ArtifactError::new(format!(
                "domain payload blob payload_format is not valid UTF-8: {error}"
            ))
        })?;
    let mut sections = Vec::new();
    for (section_name_len, section_payload_len) in sections_meta {
        let name = String::from_utf8(take_bytes(bytes, &mut offset, section_name_len)?).map_err(
            |error| {
                ArtifactError::new(format!(
                    "domain payload blob section name is not valid UTF-8: {error}"
                ))
            },
        )?;
        let section_bytes = take_bytes(bytes, &mut offset, section_payload_len)?;
        sections.push(DomainBuildUnitPayloadBlobSection {
            name,
            bytes: section_bytes,
        });
    }
    Ok(DomainBuildUnitPayloadBlob {
        domain_family,
        package_id,
        backend_family: (!backend_family.is_empty()).then_some(backend_family),
        vendor: (!vendor.is_empty()).then_some(vendor),
        device_class: (!device_class.is_empty()).then_some(device_class),
        selected_lowering_target: (!selected_lowering_target.is_empty())
            .then_some(selected_lowering_target),
        contract_family,
        packaging_role,
        payload_kind,
        payload_format,
        sections,
    })
}

impl DomainBuildUnitPayloadBlob {
    pub fn from_domain_unit_and_sections(
        unit: &BuildManifestDomainBuildUnit,
        sections: Vec<DomainBuildUnitPayloadBlobSection>,
    ) -> Self {
        Self {
            domain_family: unit.domain_family.clone(),
            package_id: unit.package_id.clone(),
            backend_family: unit.backend_family.clone(),
            vendor: unit.vendor.clone(),
            device_class: unit.device_class.clone(),
            selected_lowering_target: unit.selected_lowering_target.clone(),
            contract_family: unit.contract_family.clone(),
            packaging_role: unit.packaging_role.clone(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "toml".to_owned(),
            sections,
        }
    }

    pub fn section(&self, name: &str) -> Option<&DomainBuildUnitPayloadBlobSection> {
        self.sections.iter().find(|section| section.name == name)
    }

    pub fn section_bytes(&self, name: &str) -> Option<&[u8]> {
        self.section(name).map(|section| section.bytes.as_slice())
    }

    pub fn section_text(&self, name: &str) -> Option<Result<&str, std::str::Utf8Error>> {
        self.section_bytes(name).map(std::str::from_utf8)
    }

    pub fn ir_sidecar_section(&self) -> Option<&DomainBuildUnitPayloadBlobSection> {
        self.sections
            .iter()
            .find(|section| section.name.ends_with("_ir_sidecar"))
    }

    pub fn ir_sidecar_text(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.ir_sidecar_section()
            .map(|section| std::str::from_utf8(&section.bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decode_domain_payload_blob, encode_domain_payload_blob, DomainBuildUnitPayloadBlob,
        DomainBuildUnitPayloadBlobSection,
    };

    #[test]
    fn roundtrips_domain_payload_blob() {
        let blob = DomainBuildUnitPayloadBlob {
            domain_family: "shader".to_owned(),
            package_id: "official.shader".to_owned(),
            backend_family: Some("metal".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("apple-silicon-gpu".to_owned()),
            selected_lowering_target: Some("metal".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "toml".to_owned(),
            sections: vec![
                DomainBuildUnitPayloadBlobSection {
                    name: "contract_toml".to_owned(),
                    bytes: b"a".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "lowering_plan".to_owned(),
                    bytes: b"b".to_vec(),
                },
            ],
        };
        let encoded = encode_domain_payload_blob(&blob).unwrap();
        let decoded = decode_domain_payload_blob(&encoded).unwrap();
        assert_eq!(decoded, blob);
        assert_eq!(decoded.section("contract_toml").unwrap().bytes, b"a".to_vec());
        assert_eq!(decoded.section_text("lowering_plan").unwrap().unwrap(), "b");
        assert!(decoded.ir_sidecar_section().is_none());
    }
}

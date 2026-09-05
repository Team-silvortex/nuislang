use std::{io::Read, sync::Arc};

pub const MAX_UPLOAD_BYTES: usize = 16 * 1024 * 1024;

/// Immutable contiguous little-endian data. Only its descriptor enters the control header.
/// A parsed descriptor grants no file, pointer, or device authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchUpload {
    pub element_type: String,
    pub shape: Vec<usize>,
    pub byte_length: usize,
    pub content_hash: String,
    payload: Option<Arc<[u8]>>,
}

impl DispatchUpload {
    pub fn new(element_type: &str, shape: Vec<usize>, bytes: Vec<u8>) -> Result<Self, String> {
        let result = Self {
            element_type: element_type.to_owned(),
            shape,
            byte_length: bytes.len(),
            content_hash: crate::provider_runtime_ipc::hash_bytes(&bytes),
            payload: Some(bytes.into()),
        };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        let width = match self.element_type.as_str() {
            "u8" => 1usize,
            "u32" | "f32" => 4,
            _ => return Err("unsupported runtime upload element type".to_owned()),
        };
        let length = self.shape.iter().try_fold(width, |size, extent| {
            (*extent > 0).then(|| size.checked_mul(*extent)).flatten()
        });
        if self.shape.is_empty()
            || self.shape.len() > 4
            || self.byte_length == 0
            || self.byte_length > MAX_UPLOAD_BYTES
            || length != Some(self.byte_length)
            || !self.content_hash.strip_prefix("0x").is_some_and(|hex| {
                hex.len() == 16
                    && hex
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(
                "runtime upload descriptor exceeds shape, length or identity limits".to_owned(),
            );
        }
        if let Some(bytes) = &self.payload {
            if bytes.len() != self.byte_length
                || crate::provider_runtime_ipc::hash_bytes(bytes) != self.content_hash
            {
                return Err("runtime upload payload identity mismatch".to_owned());
            }
        }
        Ok(())
    }

    pub fn payload(&self) -> Result<&[u8], String> {
        self.validate()?;
        self.payload
            .as_deref()
            .ok_or_else(|| "runtime upload payload is missing".to_owned())
    }

    pub fn descriptor(&self) -> Self {
        Self {
            payload: None,
            ..self.clone()
        }
    }

    pub(super) fn to_wire(&self) -> Result<String, String> {
        self.validate()?;
        let shape = self
            .shape
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("x");
        Ok(format!(
            "immutable-upload-le:{}:{shape}:{}:{}",
            self.element_type, self.byte_length, self.content_hash
        ))
    }

    pub(super) fn parse(wire: &str) -> Result<Self, String> {
        let fields = wire.split(':').collect::<Vec<_>>();
        let [element_type, shape, length, hash] = fields.as_slice() else {
            return Err("runtime upload wire fields are invalid".to_owned());
        };
        let result = Self {
            element_type: (*element_type).to_owned(),
            shape: shape
                .split('x')
                .map(|s| s.parse::<usize>().map_err(|_| "invalid upload shape"))
                .collect::<Result<_, _>>()?,
            byte_length: length.parse().map_err(|_| "invalid upload length")?,
            content_hash: (*hash).to_owned(),
            payload: None,
        };
        result.validate()?;
        Ok(result)
    }

    pub(crate) fn read_payload(&mut self, reader: &mut impl Read) -> Result<(), String> {
        self.validate()?;
        let mut bytes = vec![0; self.byte_length];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| format!("runtime upload read failed: {error}"))?;
        self.payload = Some(bytes.into());
        self.validate()
    }
}

/// Small owned immutable payload, contiguous and little-endian on every host.
/// Large resources require a separate bounded carrier, not a larger control header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchResource {
    pub element_type: String,
    pub shape: Vec<usize>,
    pub bytes: Vec<u8>,
}

impl DispatchResource {
    pub fn validate(&self) -> Result<(), String> {
        let width = match self.element_type.as_str() {
            "u8" => 1usize,
            "u32" | "f32" => 4,
            _ => return Err("unsupported runtime resource element type".to_owned()),
        };
        if self.shape.is_empty() || self.shape.len() > 4 || self.bytes.len() > 64 {
            return Err("runtime resource exceeds shape or byte budget".to_owned());
        }
        let length = self.shape.iter().try_fold(width, |size, extent| {
            (*extent > 0).then(|| size.checked_mul(*extent)).flatten()
        });
        if length != Some(self.bytes.len()) {
            return Err("runtime resource shape does not match byte length".to_owned());
        }
        Ok(())
    }

    pub fn content_hash(&self) -> String {
        crate::provider_runtime_ipc::hash_bytes(&self.bytes)
    }

    pub fn hex_bytes(&self) -> String {
        self.bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
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
            "immutable-le:{}:{shape}:{}:{}",
            self.element_type,
            self.content_hash(),
            self.hex_bytes()
        ))
    }

    pub(super) fn parse(wire: &str) -> Result<Self, String> {
        let fields = wire.split(':').collect::<Vec<_>>();
        let [element_type, shape, hash, hex] = fields.as_slice() else {
            return Err("runtime resource wire fields are invalid".to_owned());
        };
        if hex.len() > 128 || hex.len() % 2 != 0 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("runtime resource byte encoding is invalid".to_owned());
        }
        let result = Self {
            element_type: (*element_type).to_owned(),
            shape: shape
                .split('x')
                .map(|extent| {
                    extent
                        .parse::<usize>()
                        .map_err(|_| "invalid runtime resource shape")
                })
                .collect::<Result<_, _>>()?,
            bytes: (0..hex.len())
                .step_by(2)
                .map(|offset| {
                    u8::from_str_radix(&hex[offset..offset + 2], 16)
                        .map_err(|_| "invalid runtime resource byte")
                })
                .collect::<Result<_, _>>()?,
        };
        result.validate()?;
        if result.content_hash() != *hash {
            return Err("runtime resource content identity mismatch".to_owned());
        }
        Ok(result)
    }
}

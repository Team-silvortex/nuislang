use yir_core::{
    provider_runtime_ipc::{DispatchArguments, DispatchResource},
    ShaderBindingSet, Value,
};

pub const SHADER_FRAGMENT_UNIFORM_CONTRACT: &str = "nuis-shader-fragment-uniform-v1";
pub const FRAGMENT_UNIFORM_CAPABILITY_MARKER: &str = "// nuis-fragment-uniform-f32x4 ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketBinding {
    pub slot: usize,
    words: [i64; 4],
    width: usize,
}

impl PacketBinding {
    pub fn from_words(slot: usize, input: &[i64]) -> Result<Self, String> {
        if slot > 30 || !(1..=4).contains(&input.len()) {
            return Err("invalid packet uniform slot or width".into());
        }
        if input
            .iter()
            .any(|word| !(-(1i64 << 24)..=(1i64 << 24)).contains(word))
        {
            return Err("packet uniform requires the exact f32 integer range".into());
        }
        let mut words = [0; 4];
        words[..input.len()].copy_from_slice(input);
        Ok(Self {
            slot,
            words,
            width: input.len(),
        })
    }
    pub fn words(&self) -> &[i64] {
        &self.words[..self.width]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UniformInput {
    Packed(ShaderFragmentUniform),
    Packet(PacketBinding),
}

/// One immutable group-zero fragment vec4<f32>. The slot is not a backend pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderFragmentUniform {
    pub slot: usize,
    pub bytes: [u8; 16],
}

impl ShaderFragmentUniform {
    pub fn from_bindings(bindings: &ShaderBindingSet) -> Result<Self, String> {
        let [binding] = bindings.bindings.as_slice() else {
            return Err("fragment uniform requires exactly one binding".to_owned());
        };
        if !matches!(binding.kind.as_str(), "uniform" | "uniform_binding") {
            return Err("fragment uniform requires a read-only uniform binding".to_owned());
        }
        Self::from_binding(binding)
    }

    pub(crate) fn from_binding(binding: &yir_core::ShaderBinding) -> Result<Self, String> {
        Ok(UniformInput::read(binding)?.reference())
    }
}

impl UniformInput {
    pub(crate) fn reference(self) -> ShaderFragmentUniform {
        match self {
            Self::Packed(uniform) => uniform,
            Self::Packet(packet) => {
                let mut bytes = [0; 16];
                for (word, dst) in packet.words().iter().zip(bytes.chunks_exact_mut(4)) {
                    dst.copy_from_slice(&(*word as f32).to_le_bytes());
                }
                ShaderFragmentUniform {
                    slot: packet.slot,
                    bytes,
                }
            }
        }
    }

    pub(crate) fn read(binding: &yir_core::ShaderBinding) -> Result<Self, String> {
        if binding.slot > 30 {
            return Err("invalid fragment uniform slot".into());
        }
        if let Value::DataWindow(window) = binding.value.as_ref() {
            let Value::Struct(packet) = window.base.as_ref() else {
                return Err("packet uniform requires an inline record".into());
            };
            if !window.immutable
                || window.offset != 0
                || window.len != 1
                || !(1..=4).contains(&packet.fields.len())
            {
                return Err(
                    "packet uniform requires a single immutable slot with 1..4 fields".into(),
                );
            }
            let mut names = std::collections::BTreeSet::new();
            let mut words = [0; 4];
            for ((name, value), lane) in packet.fields.iter().zip(&mut words) {
                let Value::Int(word) = value else {
                    return Err("packet uniform requires i64 fields".into());
                };
                if !names.insert(name) || name.is_empty() {
                    return Err("packet uniform requires distinct named fields".into());
                }
                *lane = *word;
            }
            return Ok(Self::Packet(PacketBinding::from_words(
                binding.slot,
                &words[..packet.fields.len()],
            )?));
        }
        let mut bytes = [0; 16];
        for (value, chunk) in uniform_tuple(binding.value.as_ref())?
            .into_iter()
            .zip(bytes.chunks_exact_mut(4))
        {
            let Value::F32(value) = value else {
                return Err(
                    "fragment uniform elements must be f32, without implicit conversion".to_owned(),
                );
            };
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        let result = ShaderFragmentUniform {
            slot: binding.slot,
            bytes,
        };
        result.validate()?;
        Ok(Self::Packed(result))
    }
}

impl ShaderFragmentUniform {
    pub fn validate(&self) -> Result<(), String> {
        if self.slot > 30
            || self
                .bytes
                .chunks_exact(4)
                .any(|chunk| !f32::from_le_bytes(chunk.try_into().unwrap()).is_finite())
        {
            return Err("fragment uniform slot or finite-f32 payload is invalid".to_owned());
        }
        Ok(())
    }

    pub fn bind_dispatch(self, arguments: &mut DispatchArguments) -> Result<(), String> {
        self.validate()?;
        arguments.contract = SHADER_FRAGMENT_UNIFORM_CONTRACT.to_owned();
        arguments.resources.insert(
            format!("fragment.uniform.{}", self.slot),
            DispatchResource {
                element_type: "f32".to_owned(),
                shape: vec![4],
                bytes: self.bytes.to_vec(),
            },
        );
        arguments.to_wire()?;
        Ok(())
    }

    pub fn from_dispatch(arguments: &DispatchArguments) -> Result<Option<Self>, String> {
        if arguments.contract == crate::SHADER_UNBOUND_DRAW_CONTRACT
            && arguments.resources.is_empty()
            && arguments.uploads.is_empty()
        {
            return Ok(None);
        }
        arguments.to_wire()?;
        if arguments.contract != SHADER_FRAGMENT_UNIFORM_CONTRACT
            || arguments.resources.len() != 1
            || !arguments.uploads.is_empty()
        {
            return Err("unsupported fragment uniform argument contract or count".to_owned());
        }
        let (name, resource) = arguments.resources.first_key_value().unwrap();
        let slot = name
            .strip_prefix("fragment.uniform.")
            .ok_or("unsupported fragment resource kind")?
            .parse::<usize>()
            .map_err(|_| "invalid fragment uniform slot")?;
        if name != &format!("fragment.uniform.{slot}")
            || resource.element_type != "f32"
            || resource.shape != [4]
        {
            return Err("fragment uniform type, shape or slot is invalid".to_owned());
        }
        let result = Self {
            slot,
            bytes: resource
                .bytes
                .as_slice()
                .try_into()
                .map_err(|_| "fragment uniform must contain 16 bytes")?,
        };
        result.validate()?;
        Ok(Some(result))
    }
}

fn uniform_tuple(value: &Value) -> Result<[&Value; 4], String> {
    match value {
        Value::Tuple(values) => match values.as_slice() {
            [a, b, c, d] => Ok([a, b, c, d]),
            _ => Err("fragment uniform requires exactly four f32 values".to_owned()),
        },
        // NIR tuple expressions lower to the canonical indexed Tuple aggregate.
        Value::Struct(value) if value.type_name == "Tuple" => match value.fields.as_slice() {
            [(a, x), (b, y), (c, z), (d, w)]
                if (a.as_str(), b.as_str(), c.as_str(), d.as_str()) == ("0", "1", "2", "3") =>
            {
                Ok([x, y, z, w])
            }
            _ => Err("fragment uniform Tuple fields must be exactly 0,1,2,3".to_owned()),
        },
        _ => Err("fragment uniform requires a tuple of four f32 values".to_owned()),
    }
}

/// Compiler-emitted reflection is inside the content-addressed code asset.
pub fn fragment_uniform_capability(source: &str) -> Result<Option<usize>, String> {
    let mut entries = source
        .lines()
        .filter_map(|line| line.strip_prefix(FRAGMENT_UNIFORM_CAPABILITY_MARKER));
    let slot = match entries.next() {
        None => return Ok(None),
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| "invalid fragment uniform capability slot")?,
    };
    if slot > 30 || entries.next().is_some() {
        return Err("invalid or duplicate fragment uniform capability".to_owned());
    }
    Ok(Some(slot))
}

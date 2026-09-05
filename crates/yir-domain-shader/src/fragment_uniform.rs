use yir_core::{
    provider_runtime_ipc::{DispatchArguments, DispatchResource},
    ShaderBindingSet, Value,
};

pub const SHADER_FRAGMENT_UNIFORM_CONTRACT: &str = "nuis-shader-fragment-uniform-v1";
pub const FRAGMENT_UNIFORM_CAPABILITY_MARKER: &str = "// nuis-fragment-uniform-f32x4 ";

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
        let values = uniform_tuple(binding.value.as_ref())?;
        let mut bytes = [0; 16];
        for (value, chunk) in values.into_iter().zip(bytes.chunks_exact_mut(4)) {
            let Value::F32(value) = value else {
                return Err(
                    "fragment uniform elements must be f32, without implicit conversion".to_owned(),
                );
            };
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        let result = Self {
            slot: binding.slot,
            bytes,
        };
        result.validate()?;
        Ok(result)
    }

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

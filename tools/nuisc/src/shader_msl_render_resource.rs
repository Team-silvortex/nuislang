use yir_domain_shader::{ShaderFragmentStorageCapability, FRAGMENT_STORAGE_CAPABILITY_MARKER};

#[derive(Clone, Copy)]
pub(super) enum FragmentResource<'a> {
    Uniform {
        slot: u32,
        name: &'a str,
    },
    Storage {
        capability: ShaderFragmentStorageCapability,
        name: &'a str,
    },
}

pub(super) fn parse<'a>(
    source: &str,
    summary: &'a crate::shader_source::InlineWgslSummary,
) -> Result<Option<FragmentResource<'a>>, String> {
    let binding = match summary.bindings.as_slice() {
        [] => return Ok(None),
        [binding] => binding,
        _ => return Err("canonical render supports at most one fragment resource".to_owned()),
    };
    if binding.group != 0
        || binding.binding > 30
        || !super::valid_ident(&binding.name)
        || matches!(binding.name.as_str(), "uv" | "input" | "nuis_storage_read")
    {
        return Err("canonical render requires a group-zero fragment resource".to_owned());
    }
    let compact_type = compact(&binding.ty);
    let address = compact(binding.address_space.as_deref().unwrap_or_default());
    let resource = if binding.kind == "uniform"
        && address == "uniform"
        && compact_type == "vec4<f32>"
    {
        FragmentResource::Uniform {
            slot: binding.binding,
            name: &binding.name,
        }
    } else if address == "storage,read" {
        let count = compact_type
            .strip_prefix("array<u32,")
            .and_then(|value| value.strip_suffix('>'))
            .ok_or("fragment storage requires a fixed-size read-only array<u32,N>")?;
        let element_count = count
            .parse::<usize>()
            .map_err(|_| "invalid fragment storage element count")?;
        if count != element_count.to_string() {
            return Err("fragment storage element count must be canonical".to_owned());
        }
        let capability = ShaderFragmentStorageCapability {
            slot: binding.binding as usize,
            element_count,
        };
        capability.validate()?;
        FragmentResource::Storage {
            capability,
            name: &binding.name,
        }
    } else {
        return Err("canonical render requires group-zero fragment vec4<f32> uniform or read-only u32 storage".to_owned());
    };
    let declaration = format!(
        "@group(0)@binding({})var<{address}>{}:{compact_type};",
        binding.binding, binding.name
    );
    if compact(source).matches(&declaration).count() != 1 {
        return Err("unsupported fragment resource declaration or initializer".to_owned());
    }
    Ok(Some(resource))
}

fn compact(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

impl FragmentResource<'_> {
    pub(super) fn parameter(self) -> String {
        match self {
            Self::Uniform { slot, name } => format!(", constant float4& {name} [[buffer({slot})]]"),
            Self::Storage { capability, name } => format!(
                ", const device NuisFragmentStorage& {name} [[buffer({})]]",
                capability.slot
            ),
        }
    }

    pub(super) fn metadata(self) -> String {
        match self {
            Self::Uniform { slot, .. } => format!(
                "{}{slot}\n",
                yir_domain_shader::FRAGMENT_UNIFORM_CAPABILITY_MARKER
            ),
            Self::Storage { capability, .. } => format!(
                "{FRAGMENT_STORAGE_CAPABILITY_MARKER}{}:{}\n",
                capability.slot, capability.element_count
            ),
        }
    }

    pub(super) fn prelude(self) -> String {
        match self {
            Self::Uniform { .. } => String::new(),
            Self::Storage { capability, .. } => format!(
                "struct NuisFragmentStorage {{ uint values[{}]; }};\n\
                 uint nuis_storage_read(const device NuisFragmentStorage& buffer, uint index) {{\n\
                     return index < {}u ? buffer.values[index] : 0u;\n\
                 }};\n",
                capability.element_count, capability.element_count
            ),
        }
    }
}

/// Lower only reads of the admitted array; every index is bounds checked on the GPU.
pub(super) fn lower_reads(
    expression: &str,
    resource: Option<FragmentResource<'_>>,
) -> Result<String, String> {
    let mut rest = expression;
    let mut out = String::new();
    while let Some(open) = rest.find('[') {
        let Some(FragmentResource::Storage { name, .. }) = resource else {
            return Err("fragment array read has no admitted storage resource".to_owned());
        };
        let before = rest[..open].trim_end();
        let start = before
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(0, |index| index + 1);
        if &before[start..] != name {
            return Err("fragment array read must target the admitted resource".to_owned());
        }
        let close = rest[open + 1..]
            .find(']')
            .map(|index| open + 1 + index)
            .ok_or("unclosed fragment storage index")?;
        let index = rest[open + 1..close].trim();
        if index.is_empty() || index.contains(['[', ']']) {
            return Err("nested or empty fragment storage index is unsupported".to_owned());
        }
        out.push_str(&before[..start]);
        out.push_str(&format!("nuis_storage_read({name}, {index})"));
        rest = &rest[close + 1..];
    }
    if rest.contains(']') {
        return Err("unmatched fragment storage index".to_owned());
    }
    out.push_str(rest);
    Ok(out)
}

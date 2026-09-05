use crate::provider_request::{ProviderRequest, ProviderScalarBinding};
use yir_core::provider_runtime_ipc::DispatchArguments;
use yir_domain_shader::{ShaderDrawArguments, ShaderFragmentUniform, SHADER_UNBOUND_DRAW_CONTRACT};

pub(super) fn prepare_runtime_arguments(
    request: &mut ProviderRequest,
    arguments: Option<&DispatchArguments>,
) -> Result<DispatchArguments, String> {
    if !super::is_rgba8_render(request) {
        return Err("Metal adapter has no runtime argument binding for this operation".to_owned());
    }
    let [width, height] = request.output_bindings[0].shape.as_slice() else {
        return Err("Metal draw output must be two-dimensional".to_owned());
    };
    let (vertex_count, instance_count) = render_draw_counts(request)?;
    let admitted_slot = uniform_slot(request)?;
    let uniform = arguments
        .map(ShaderFragmentUniform::from_dispatch)
        .transpose()?
        .flatten();
    if uniform.map(|uniform| uniform.slot) != admitted_slot {
        return Err(
            "Metal fragment uniform is missing or differs from admitted code capability".to_owned(),
        );
    }
    let draw = match arguments {
        Some(arguments) => {
            let mut scalars = arguments.clone();
            scalars.contract = SHADER_UNBOUND_DRAW_CONTRACT.to_owned();
            scalars.resources.clear();
            ShaderDrawArguments::from_dispatch(&scalars)?
        }
        None => ShaderDrawArguments {
            width: *width,
            height: *height,
            vertex_count: vertex_count as u64,
            instance_count: instance_count as u64,
        },
    };
    if (draw.width, draw.height) != (*width, *height) {
        return Err("Metal runtime draw dimensions differ from admitted output".to_owned());
    }
    validate_counts(draw.vertex_count, draw.instance_count)?;
    if request
        .kernel
        .scalar_bindings
        .iter()
        .any(|binding| binding.name == "fragment_uniform_bytes")
    {
        return Err("Metal fragment uniform bytes must come from this runtime dispatch".to_owned());
    }
    let mut result = draw.to_dispatch();
    if let Some(uniform) = uniform {
        uniform.bind_dispatch(&mut result)?;
    }
    // Runtime inputs cannot replace the admitted code asset, entry set, or output capability.
    for (name, value) in [
        ("vertex_count", draw.vertex_count),
        ("instance_count", draw.instance_count),
    ] {
        request
            .kernel
            .scalar_bindings
            .retain(|scalar| scalar.name != name);
        request.kernel.scalar_bindings.push(ProviderScalarBinding {
            name: name.to_owned(),
            value_type: "u64".to_owned(),
            value: value.to_string(),
        });
    }
    if let Some(uniform) = uniform {
        request.kernel.scalar_bindings.push(ProviderScalarBinding {
            name: "fragment_uniform_bytes".to_owned(),
            value_type: "symbol".to_owned(),
            value: uniform
                .bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        });
    }
    Ok(result)
}

pub(super) fn uniform_slot(request: &ProviderRequest) -> Result<Option<usize>, String> {
    let Some(binding) = unique_scalar(request, "fragment_uniform_slot")? else {
        return Ok(None);
    };
    let slot = binding
        .value
        .parse::<usize>()
        .map_err(|_| "invalid Metal uniform slot")?;
    if binding.value_type != "u64" || slot > 30 || binding.value != slot.to_string() {
        return Err("invalid Metal uniform slot type or range".to_owned());
    }
    Ok(Some(slot))
}

pub(super) fn uniform_upload(request: &ProviderRequest) -> Result<String, String> {
    let slot = uniform_slot(request)?;
    let bytes = unique_scalar(request, "fragment_uniform_bytes")?;
    match (slot, bytes) {
        (None, None) => Ok("none".to_owned()),
        (Some(slot), Some(bytes)) if bytes.value_type == "symbol" => {
            let hex = &bytes.value;
            if hex.len() != 32
                || !hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err("invalid Metal uniform bytes".to_owned());
            }
            let mut data = [0; 16];
            for (index, byte) in data.iter_mut().enumerate() {
                *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
                    .map_err(|_| "invalid uniform byte")?;
            }
            ShaderFragmentUniform { slot, bytes: data }.validate()?;
            Ok(format!("{slot}:{hex}"))
        }
        _ => Err("Metal uniform requires both admitted slot and runtime bytes".to_owned()),
    }
}

fn unique_scalar<'a>(
    request: &'a ProviderRequest,
    name: &str,
) -> Result<Option<&'a ProviderScalarBinding>, String> {
    let mut bindings = request
        .kernel
        .scalar_bindings
        .iter()
        .filter(|binding| binding.name == name);
    let binding = bindings.next();
    if bindings.next().is_some() {
        return Err(format!("duplicate Metal scalar `{name}`"));
    }
    Ok(binding)
}

pub(super) fn render_draw_counts(request: &ProviderRequest) -> Result<(usize, usize), String> {
    let count = |name: &str, fallback| {
        let mut bindings = request
            .kernel
            .scalar_bindings
            .iter()
            .filter(|scalar| scalar.name == name);
        let value = match bindings.next() {
            None => fallback,
            Some(scalar) if scalar.value_type == "u64" => scalar
                .value
                .parse::<u64>()
                .map_err(|_| format!("invalid Metal draw scalar `{name}`"))?,
            Some(_) => return Err(format!("invalid Metal draw scalar type `{name}`")),
        };
        if bindings.next().is_some() {
            return Err(format!("duplicate Metal draw scalar `{name}`"));
        }
        Ok(value)
    };
    let vertices = count("vertex_count", 4)?;
    let instances = count("instance_count", 1)?;
    validate_counts(vertices, instances)?;
    Ok((vertices as usize, instances as usize))
}

fn validate_counts(vertices: u64, instances: u64) -> Result<(), String> {
    // This unbound projection only admits the existing four-vertex procedural stage.
    // Wider or resource-backed draws require a separate registered capability.
    if !(1..=4).contains(&vertices) || !(1..=256).contains(&instances) {
        return Err("Metal unbound draw count exceeds admitted vertex/instance budget".to_owned());
    }
    Ok(())
}

#[cfg(test)]
#[path = "provider_execution_metal_draw_tests.rs"]
mod tests;

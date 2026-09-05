use crate::provider_request::{ProviderRequest, ProviderScalarBinding};
use yir_core::provider_runtime_ipc::DispatchArguments;
use yir_domain_shader::ShaderDrawArguments;

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
    let draw = match arguments {
        Some(arguments) => ShaderDrawArguments::from_dispatch(arguments)?,
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
    // Runtime scalars cannot replace the admitted code asset, entry set, or output capability.
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
    Ok(draw.to_dispatch())
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

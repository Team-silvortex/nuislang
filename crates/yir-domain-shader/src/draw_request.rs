use crate::{
    describe::describe_shader_node,
    execute_effects::push_shader_event,
    render_pass::{prepare_render_pass, ValidatedRenderPass},
    ShaderMod,
};
use yir_core::{
    provider_runtime_ipc::DispatchArguments, ExecutionState, FrameSurface, Node, Resource, Value,
};

pub const SHADER_UNBOUND_DRAW_CONTRACT: &str = "nuis-shader-unbound-draw-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderDrawArguments {
    pub width: usize,
    pub height: usize,
    pub vertex_count: u64,
    pub instance_count: u64,
}

impl ShaderDrawArguments {
    pub fn to_dispatch(self) -> DispatchArguments {
        DispatchArguments {
            contract: SHADER_UNBOUND_DRAW_CONTRACT.to_owned(),
            scalars: [
                ("width".to_owned(), self.width as u64),
                ("height".to_owned(), self.height as u64),
                ("vertex_count".to_owned(), self.vertex_count),
                ("instance_count".to_owned(), self.instance_count),
            ]
            .into(),
            resources: Default::default(),
        }
    }

    pub fn from_dispatch(arguments: &DispatchArguments) -> Result<Self, String> {
        arguments.to_wire()?;
        if arguments.contract != SHADER_UNBOUND_DRAW_CONTRACT
            || arguments.scalars.len() != 4
            || !arguments.resources.is_empty()
        {
            return Err("unsupported shader runtime argument contract or field count".to_owned());
        }
        let scalar = |name: &str| {
            arguments
                .scalars
                .get(name)
                .copied()
                .filter(|value| *value > 0)
                .ok_or_else(|| format!("shader runtime argument `{name}` must be positive u64"))
        };
        let result = Self {
            width: usize::try_from(scalar("width")?)
                .map_err(|_| "shader width exceeds host range")?,
            height: usize::try_from(scalar("height")?)
                .map_err(|_| "shader height exceeds host range")?,
            vertex_count: scalar("vertex_count")?,
            instance_count: scalar("instance_count")?,
        };
        ShaderDrawDescriptor::new(result.width, result.height)?;
        Ok(result)
    }
}

/// Validated output extent, without allocating or rasterizing a reference frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderDrawDescriptor {
    width: usize,
    height: usize,
    rgba8_byte_length: usize,
    pub(crate) vertex_count: u64,
    pub(crate) instance_count: u64,
    pub(crate) unbound_rgba8_triangle_strip: bool,
    pub(crate) fragment_uniform: Option<crate::ShaderFragmentUniform>,
}

impl ShaderDrawDescriptor {
    pub(crate) fn new(width: usize, height: usize) -> Result<Self, String> {
        let rgba8_byte_length = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .filter(|bytes| *bytes != 0)
            .ok_or("shader.draw_instanced has empty or overflowing output dimensions")?;
        Ok(Self {
            width,
            height,
            rgba8_byte_length,
            vertex_count: 0,
            instance_count: 0,
            unbound_rgba8_triangle_strip: false,
            fragment_uniform: None,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn rgba8_byte_length(&self) -> usize {
        self.rgba8_byte_length
    }

    pub fn provider_arguments(&self) -> Result<DispatchArguments, String> {
        if !self.unbound_rgba8_triangle_strip {
            return Err("provider draw requires unbound rgba8_unorm triangle_strip; resource bindings or other pass formats/topologies are unsupported".to_owned());
        }
        let mut arguments = ShaderDrawArguments {
            width: self.width,
            height: self.height,
            vertex_count: self.vertex_count,
            instance_count: self.instance_count,
        }
        .to_dispatch();
        if let Some(uniform) = self.fragment_uniform {
            uniform.bind_dispatch(&mut arguments)?;
        }
        Ok(arguments)
    }
}

impl ShaderMod {
    pub fn validate_draw_instanced(
        &self,
        node: &Node,
        resource: &Resource,
        state: &ExecutionState,
    ) -> Result<ShaderDrawDescriptor, String> {
        Ok(prepare_draw_instanced(node, resource, state)?.descriptor)
    }

    pub fn record_draw_instanced(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
        frame: &FrameSurface,
    ) {
        push_shader_event(node, resource, state, "draw_instanced", frame);
    }
}

pub(crate) fn execute_draw_instanced(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let frame = prepare_draw_instanced(node, resource, state)?.rasterize_reference();
    ShaderMod.record_draw_instanced(node, resource, state, &frame);
    Ok(Value::Frame(frame))
}

fn prepare_draw_instanced<'a>(
    node: &Node,
    resource: &Resource,
    state: &'a ExecutionState,
) -> Result<ValidatedRenderPass<'a>, String> {
    if node.op.module != "shader" || node.op.instruction != "draw_instanced" {
        return Err("draw validation requires shader.draw_instanced".to_owned());
    }
    describe_shader_node(node, resource)?;
    let pass = match state.expect_value(&node.op.args[0])? {
        Value::RenderPass(pass) => pass,
        other => {
            return Err(format!(
                "shader.draw_instanced expects render pass, got {other}"
            ))
        }
    };
    let packet = match state.expect_value(&node.op.args[1])? {
        Value::DataWindow(window) => window.base.as_ref(),
        other => other,
    };
    let vertex_count = resolve_draw_count(state, node, 2, "vertex_count")?;
    let instance_count = resolve_draw_count(state, node, 3, "instance_count")?;
    let bindings = match node.op.args.get(4) {
        Some(name) => match state.expect_value(name)? {
            Value::BindingSet(bindings) => Some(bindings),
            other => {
                return Err(format!(
                    "shader.draw_instanced expects bind_set value, got {other}"
                ))
            }
        },
        None => None,
    };
    prepare_render_pass(pass, packet, vertex_count, instance_count, bindings)
}

fn resolve_draw_count(
    state: &ExecutionState,
    node: &Node,
    index: usize,
    label: &str,
) -> Result<i64, String> {
    let raw = &node.op.args[index];
    if let Ok(value) = raw.parse::<i64>() {
        return Ok(value);
    }
    match state.expect_value(raw)? {
        Value::Int(value) => Ok(*value),
        Value::I32(value) => Ok(i64::from(*value)),
        Value::Bool(value) => Ok(i64::from(*value)),
        other => Err(format!(
            "node `{}` expects integer-like {label} value, got {other}",
            node.name
        )),
    }
}

#[cfg(test)]
#[path = "draw_request_tests.rs"]
mod tests;

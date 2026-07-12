use super::{
    draw_render_pass_surface, frame_surface,
    sphere_render::{draw_ball_surface, draw_sphere_surface_with_size},
};
use yir_core::{ExecutionState, FrameSurface, Node, Resource, Value};

pub(crate) fn execute_shader_effect_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Option<Value>, String> {
    let value = match node.op.instruction.as_str() {
        "clear" => execute_clear(node, resource, state),
        "overlay" => execute_overlay(node, resource, state),
        "dispatch" => execute_dispatch(node, resource, state),
        "draw_instanced" => execute_draw_instanced(node, resource, state),
        "draw_ball" => execute_draw_ball(node, resource, state),
        "draw_sphere" => execute_draw_sphere(node, resource, state),
        "print" => execute_print(node, resource, state),
        _ => return Ok(None),
    }?;
    Ok(Some(value))
}

fn execute_clear(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let target = match state.expect_value(&node.op.args[0])?.clone() {
        Value::Target(target) => target,
        other => return Err(format!("shader.clear expects target value, got {}", other)),
    };
    let fill = node.op.args[1].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid clear fill `{}`",
            node.name, node.op.args[1]
        )
    })?;
    let frame = frame_surface::clear_target_surface(&target, fill);
    push_shader_event(node, resource, state, "clear", &Value::Frame(frame.clone()));
    Ok(Value::Frame(frame))
}

fn execute_overlay(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let base = match state.expect_value(&node.op.args[0])?.clone() {
        Value::Frame(frame) => frame,
        other => return Err(format!("shader.overlay expects base frame, got {}", other)),
    };
    let top = match state.expect_value(&node.op.args[1])?.clone() {
        Value::Frame(frame) => frame,
        other => return Err(format!("shader.overlay expects top frame, got {}", other)),
    };
    let frame = frame_surface::overlay_surfaces(&base, &top)?;
    push_shader_event(
        node,
        resource,
        state,
        "overlay",
        &Value::Frame(frame.clone()),
    );
    Ok(Value::Frame(frame))
}

fn execute_dispatch(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let value = state.expect_value(&node.op.args[0])?.clone();
    push_shader_event(node, resource, state, "dispatch", &value);
    Ok(value)
}

fn execute_draw_instanced(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let pass = match state.expect_value(&node.op.args[0])?.clone() {
        Value::RenderPass(pass) => pass,
        other => {
            return Err(format!(
                "shader.draw_instanced expects render pass, got {}",
                other
            ))
        }
    };
    let packet = unwrap_data_window(state.expect_value(&node.op.args[1])?.clone());
    let vertex_count = resolve_draw_count(state, node, 2, "vertex_count")?;
    let instance_count = resolve_draw_count(state, node, 3, "instance_count")?;
    let bindings = match node.op.args.get(4) {
        Some(name) => match state.expect_value(name)?.clone() {
            Value::BindingSet(bindings) => Some(bindings),
            other => {
                return Err(format!(
                    "shader.draw_instanced expects bind_set value, got {}",
                    other
                ))
            }
        },
        None => None,
    };
    let frame = draw_render_pass_surface(
        &pass,
        &packet,
        vertex_count,
        instance_count,
        bindings.as_ref(),
    )?;
    push_shader_event(
        node,
        resource,
        state,
        "draw_instanced",
        &Value::Frame(frame.clone()),
    );
    Ok(Value::Frame(frame))
}

fn execute_draw_ball(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let value = state.expect_value(&node.op.args[0])?.clone();
    let frame = draw_ball_surface(&value)?;
    push_shader_event(
        node,
        resource,
        state,
        "draw_ball",
        &Value::Frame(frame.clone()),
    );
    Ok(Value::Frame(frame))
}

fn execute_draw_sphere(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let value = state.expect_value(&node.op.args[0])?.clone();
    let frame = draw_sphere_surface(&value)?;
    push_shader_event(
        node,
        resource,
        state,
        "draw_sphere",
        &Value::Frame(frame.clone()),
    );
    Ok(Value::Frame(frame))
}

fn execute_print(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let value = state.expect_value(&node.op.args[0])?.clone();
    push_shader_event(node, resource, state, "print", &value);
    Ok(Value::Unit)
}

fn push_shader_event(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
    instruction: &str,
    value: &Value,
) {
    state.push_resource_event(
        resource,
        format!(
            "effect shader.{} @{} [{}]: {}",
            instruction, node.resource, resource.kind.raw, value
        ),
    );
}

fn draw_sphere_surface(value: &Value) -> Result<FrameSurface, String> {
    draw_sphere_surface_with_size(value, 48, 32)
}

fn unwrap_data_window(value: Value) -> Value {
    match value {
        Value::DataWindow(window) => (*window.base).clone(),
        other => other,
    }
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
        Value::I32(value) => Ok(*value as i64),
        Value::Bool(value) => Ok(if *value { 1 } else { 0 }),
        other => Err(format!(
            "node `{}` expects integer-like {} value, got {}",
            node.name, label, other
        )),
    }
}

use super::{
    draw_request::execute_draw_instanced,
    frame_surface,
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

pub(crate) fn push_shader_event(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
    instruction: &str,
    value: &impl std::fmt::Display,
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

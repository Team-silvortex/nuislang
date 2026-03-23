use yir_core::{ExecutionState, FrameSurface, InstructionSemantics, Node, RegisteredMod, Resource, Value};

pub struct ShaderMod;

impl RegisteredMod for ShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        require_shader_resource(node, resource)?;

        match node.op.instruction.as_str() {
            "const" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const <name> <resource> <value>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid integer literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "add" | "mul" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <lhs> <rhs>`",
                        node.name, node.op.instruction
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "pack_ball_state" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.pack_ball_state <name> <resource> <color> <speed>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "dispatch" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.dispatch <name> <resource> <input>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "draw_ball" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.draw_ball <name> <resource> <packet>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "print" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.print <name> <resource> <input>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            other => Err(format!("unknown shader instruction `{other}`")),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        match node.op.instruction.as_str() {
            "const" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid integer literal `{}`",
                    node.name, node.op.args[0]
                )
            })?)),
            "add" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? + state.expect_int(&node.op.args[1])?,
            )),
            "mul" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?,
            )),
            "pack_ball_state" => {
                let color = state.expect_value(&node.op.args[0])?.clone();
                let speed = state.expect_value(&node.op.args[1])?.clone();
                Ok(Value::Tuple(vec![color, speed]))
            }
            "dispatch" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(resource, format!(
                    "effect shader.dispatch @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ));
                Ok(value)
            }
            "draw_ball" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let frame = draw_ball_surface(&value)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.draw_ball @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Frame(frame))
            }
            "print" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(resource, format!(
                    "effect shader.print @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ));
                Ok(Value::Unit)
            }
            other => Err(format!("unknown shader instruction `{other}`")),
        }
    }
}

fn require_shader_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("shader") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses shader mod on non-shader resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn draw_ball_surface(value: &Value) -> Result<FrameSurface, String> {
    let (color, speed) = match value {
        Value::Tuple(items) if items.len() == 2 => match (&items[0], &items[1]) {
            (Value::Int(color), Value::Int(speed)) => (*color, *speed),
            _ => return Err("shader.draw_ball expects (int, int)".to_owned()),
        },
        _ => return Err("shader.draw_ball expects a 2-tuple packet".to_owned()),
    };

    let width = 16usize;
    let height = 9usize;
    let center_x = (speed.rem_euclid(width as i64)) as usize;
    let center_y = ((speed / 2).rem_euclid(height as i64)) as usize;
    let glyph = match color.rem_euclid(3) {
        0 => 'o',
        1 => 'O',
        _ => '@',
    };

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        for x in 0..width {
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);
            if dx <= 1 && dy <= 1 {
                row.push(glyph);
            } else {
                row.push('.');
            }
        }
        rows.push(row);
    }

    Ok(FrameSurface { width, height, rows })
}

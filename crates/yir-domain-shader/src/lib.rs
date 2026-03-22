use yir_core::{ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, Value};

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
            "dispatch" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.dispatch <name> <resource> <input>`",
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
            "dispatch" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_event(format!(
                    "effect shader.dispatch @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ));
                Ok(value)
            }
            "print" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_event(format!(
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

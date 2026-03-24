use std::env;

use yir_core::{ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, Value};

pub struct CpuMod;

impl RegisteredMod for CpuMod {
    fn module_name(&self) -> &'static str {
        "cpu"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        require_cpu_resource(node, resource)?;

        match node.op.instruction.as_str() {
            "const" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const <name> <resource> <value>`",
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
                        "node `{}` expects `cpu.{} <name> <resource> <lhs> <rhs>`",
                        node.name, node.op.instruction
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "window" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.window <name> <resource> <width> <height> <title>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!("node `{}` has invalid width `{}`", node.name, node.op.args[0])
                })?;
                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!("node `{}` has invalid height `{}`", node.name, node.op.args[1])
                })?;

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "input_i64" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.input_i64 <name> <resource> <channel> <default>`",
                        node.name
                    ));
                }

                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid default integer literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "present_frame" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.present_frame <name> <resource> <frame>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "print" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.print <name> <resource> <input>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            other => Err(format!("unknown cpu instruction `{other}`")),
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
            "window" => {
                let width = node.op.args[0].parse::<i64>().map_err(|_| {
                    format!("node `{}` has invalid width `{}`", node.name, node.op.args[0])
                })?;
                let height = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!("node `{}` has invalid height `{}`", node.name, node.op.args[1])
                })?;
                let title = &node.op.args[2];
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.window @{} [{}] {}x{} title={}",
                        node.resource, resource.kind.raw, width, height, title
                    ),
                );
                Ok(Value::Tuple(vec![Value::Int(width), Value::Int(height)]))
            }
            "input_i64" => {
                let channel = &node.op.args[0];
                let default_value = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid default integer literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                let env_name = format!("NUIS_UI_{}", normalize_channel(channel));
                let sampled = env::var(&env_name)
                    .ok()
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(default_value);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.input_i64 @{} [{}] channel={} value={} source={}",
                        node.resource,
                        resource.kind.raw,
                        channel,
                        sampled,
                        if sampled == default_value { "default" } else { "env" }
                    ),
                );
                Ok(Value::Int(sampled))
            }
            "present_frame" => {
                let frame = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.present_frame @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Unit)
            }
            "print" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(resource, format!(
                    "effect cpu.print @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ));
                Ok(Value::Unit)
            }
            other => Err(format!("unknown cpu instruction `{other}`")),
        }
    }
}

fn require_cpu_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("cpu") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses cpu mod on non-cpu resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn normalize_channel(channel: &str) -> String {
    channel
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

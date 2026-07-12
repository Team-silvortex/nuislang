use super::*;

pub(super) fn describe_cpu_host_node(node: &Node) -> Result<Option<InstructionSemantics>, String> {
    let semantics = match node.op.instruction.as_str() {
        "target_config" => {
            if node.op.args.len() != 3 && node.op.args.len() != 5 {
                return Err(format!(
                    "node `{}` expects `cpu.target_config <name> <resource> <arch> <abi> <vector_bits> [isa_family isa_features]`",
                    node.name
                ));
            }

            node.op.args[2].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid vector width `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "instantiate_unit" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.instantiate_unit <name> <resource> <domain> <unit>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::effect(Vec::new()))
        }
        "bind_core" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.bind_core <name> <resource> <core_index>`",
                    node.name
                ));
            }

            node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid core index `{}`",
                    node.name, node.op.args[0]
                )
            })?;

            Ok(InstructionSemantics::effect(Vec::new()))
        }
        "window" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.window <name> <resource> <width> <height> <title>`",
                    node.name
                ));
            }

            node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid width `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            node.op.args[1].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid height `{}`",
                    node.name, node.op.args[1]
                )
            })?;

            Ok(InstructionSemantics::effect(Vec::new()))
        }
        "extern_call_i64" | "extern_call_i32" => {
            if node.op.args.len() < 2 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <abi> <symbol> [args...]`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args[2..].to_vec()))
        }
        "input_i64" => {
            if node.op.args.len() != 2 && node.op.args.len() != 5 {
                return Err(format!(
                    "node `{}` expects `cpu.input_i64 <name> <resource> <channel> <default> [<min> <max> <step>]`",
                    node.name
                ));
            }

            node.op.args[1].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid default integer literal `{}`",
                    node.name, node.op.args[1]
                )
            })?;
            if node.op.args.len() == 5 {
                let min_value = node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid min integer literal `{}`",
                        node.name, node.op.args[2]
                    )
                })?;
                let max_value = node.op.args[3].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid max integer literal `{}`",
                        node.name, node.op.args[3]
                    )
                })?;
                let step_value = node.op.args[4].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid step integer literal `{}`",
                        node.name, node.op.args[4]
                    )
                })?;
                if min_value > max_value {
                    return Err(format!(
                        "node `{}` requires min <= max, got {} > {}",
                        node.name, min_value, max_value
                    ));
                }
                if step_value <= 0 {
                    return Err(format!(
                        "node `{}` requires positive step, got `{}`",
                        node.name, step_value
                    ));
                }
            }

            Ok(InstructionSemantics::effect(Vec::new()))
        }
        "tick_i64" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.tick_i64 <name> <resource> <start> <step>`",
                    node.name
                ));
            }

            node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid tick start literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            node.op.args[1].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid tick step literal `{}`",
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
        "guard_print" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.guard_print <name> <resource> <condition> <print>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        _ => return Ok(None),
    };
    semantics.map(Some)
}

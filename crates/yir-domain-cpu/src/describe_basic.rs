use super::*;

pub(super) fn describe_cpu_basic_node(node: &Node) -> Result<Option<InstructionSemantics>, String> {
    let semantics = match node.op.instruction.as_str() {
        "text" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.text <name> <resource> <string>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(Vec::new()))
        }
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
        "project_profile_ref" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.project_profile_ref <name> <resource> <domain> <unit> <slot>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_bool" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.const_bool <name> <resource> <true|false>`",
                    node.name
                ));
            }
            match node.op.args[0].as_str() {
                "true" | "false" => Ok(InstructionSemantics::pure(Vec::new())),
                _ => Err(format!(
                    "node `{}` has invalid bool literal `{}`",
                    node.name, node.op.args[0]
                )),
            }
        }
        "const_i32" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.const_i32 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<i32>().map_err(|_| {
                format!(
                    "node `{}` has invalid i32 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_i64" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.const_i64 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid i64 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_f32" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.const_f32 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<f32>().map_err(|_| {
                format!(
                    "node `{}` has invalid f32 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_f64" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.const_f64 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<f64>().map_err(|_| {
                format!(
                    "node `{}` has invalid f64 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "struct" => {
            if node.op.args.is_empty() {
                return Err(format!(
                    "node `{}` expects `cpu.struct <name> <resource> <type_name> [field=value]...`",
                    node.name
                ));
            }
            for entry in &node.op.args[1..] {
                let Some((field, value_name)) = entry.split_once('=') else {
                    return Err(format!(
                        "node `{}` has invalid struct field binding `{}`",
                        node.name, entry
                    ));
                };
                if field.trim().is_empty() || value_name.trim().is_empty() {
                    return Err(format!(
                        "node `{}` has empty struct field binding `{}`",
                        node.name, entry
                    ));
                }
            }
            Ok(InstructionSemantics::pure(
                node.op.args[1..]
                    .iter()
                    .filter_map(|entry| {
                        entry
                            .split_once('=')
                            .map(|(_, value)| value.trim())
                            .filter(|value| !cpu_struct_field_is_literal(value))
                            .map(str::to_owned)
                    })
                    .collect(),
            ))
        }
        "field" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.field <name> <resource> <struct> <field_name>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "variant_is" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.variant_is <name> <resource> <value> <variant_name>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "variant_field" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.variant_field <name> <resource> <value> <variant_name> <field_name>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "null" => {
            if !node.op.args.is_empty() {
                return Err(format!(
                    "node `{}` expects `cpu.null <name> <resource>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "borrow" | "borrow_end" | "move_ptr" | "neg" | "not" | "await" | "async_value" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <input>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "param_bool" | "param_i32" | "param_i64" | "param_f32" | "param_f64" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <index>`",
                    node.name, node.op.instruction
                ));
            }
            node.op.args[0].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid parameter index `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "call_bool" | "call_i32" | "call_i64" | "call_f32" | "call_f64" => {
            if node.op.args.is_empty() {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <callee> [arg...]`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(
                node.op.args.iter().skip(1).cloned().collect(),
            ))
        }
        "return_bool" | "return_i32" | "return_i64" | "return_f32" | "return_f64" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <value>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        _ => return Ok(None),
    };
    semantics.map(Some)
}

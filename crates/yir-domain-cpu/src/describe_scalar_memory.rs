use super::*;

pub(super) fn describe_cpu_scalar_memory_node(
    node: &Node,
) -> Result<Option<InstructionSemantics>, String> {
    let semantics = match node.op.instruction.as_str() {
        "add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "and"
        | "or" | "xor" | "shl" | "shr" | "add_i32" | "sub_i32" | "mul_i32" | "div_i32"
        | "add_f32" | "sub_f32" | "mul_f32" | "div_f32" | "add_f64" | "sub_f64" | "mul_f64"
        | "div_f64" | "eq_i32" | "lt_i32" | "gt_i32" | "eq_f32" | "lt_f32" | "gt_f32"
        | "eq_f64" | "lt_f64" | "gt_f64" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <lhs> <rhs>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "cast_bool_to_i64" | "cast_i32_to_i64" | "cast_i64_to_bool" | "cast_i64_to_i32"
        | "cast_i32_to_f32" | "cast_i32_to_f64" | "cast_f32_to_f64" | "cast_f64_to_f32" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <input>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "select" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.select <name> <resource> <cond> <then> <else>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "alloc_node" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.alloc_node <name> <resource> <value> <next_ptr>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "alloc_buffer" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.alloc_buffer <name> <resource> <len> <fill>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "load_value" | "load_next" | "is_null" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <ptr>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "buffer_len" | "copy_buffer_owned" | "owned_bytes_len" | "drop_owned_bytes" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <buffer_ptr>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "load_at" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.load_at <name> <resource> <buffer_ptr> <index>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "store_value" | "store_next" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <ptr> <value>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "store_at" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.store_at <name> <resource> <buffer_ptr> <index> <value>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "free" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.free <name> <resource> <ptr>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "madd" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.madd <name> <resource> <lhs> <rhs> <acc>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        _ => return Ok(None),
    };
    semantics.map(Some)
}

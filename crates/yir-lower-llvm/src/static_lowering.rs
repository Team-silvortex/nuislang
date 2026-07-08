use std::collections::BTreeMap;

use yir_core::Node;

use super::{fresh_reg, LlvmValueRef};

pub(crate) fn lower_cpu_static_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }

    match node.op.instruction.as_str() {
        "input_i64" => {
            let reg = fresh_reg(next_reg);
            body.push(format!(
                "  ; static AOT lowering freezes cpu.input_i64 `{}` to its default value",
                node.op.args[0]
            ));
            body.push(format!("  {reg} = add i64 0, {}", node.op.args[1]));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        "tick_i64" => {
            let start = node.op.args.first().map(String::as_str).unwrap_or("0");
            let step = node.op.args.get(1).map(String::as_str).unwrap_or("1");
            let reg = fresh_reg(next_reg);
            body.push("  ; static AOT lowering freezes cpu.tick_i64 to start + step".to_owned());
            body.push(format!("  {reg} = add i64 {start}, {step}"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        "target_config" | "bind_core" | "instantiate_unit" | "window" | "present_frame" => {
            registers.insert(node.name.clone(), LlvmValueRef::Void);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

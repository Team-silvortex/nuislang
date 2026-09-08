use std::collections::BTreeMap;

use yir_core::Node;

use super::{value_ref::coerce_to_i64, LlvmValueRef};

pub(crate) fn lower_cpu_param_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }
    if !matches!(
        node.op.instruction.as_str(),
        "param_bool" | "param_i32" | "param_i64"
    ) {
        return Ok(false);
    }

    let binding = registers.get(&node.name).ok_or_else(|| {
        format!(
            "CPU parameter `{}` has no binding in the selected LLVM function context",
            node.name
        )
    })?;
    if let Some(input) = coerce_to_i64(binding, body, next_reg) {
        *last_cpu_value = Some(input);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbound_scalar_parameter_reports_context_without_panicking_or_mutating_ir() {
        for instruction in ["param_bool", "param_i32", "param_i64"] {
            let node = Node {
                name: "callback.arg".to_owned(),
                resource: "cpu0".to_owned(),
                op: yir_core::Operation {
                    module: "cpu".to_owned(),
                    instruction: instruction.to_owned(),
                    args: vec![],
                },
            };
            let mut body = vec![];
            let mut next = 7;
            let mut last = Some("%prior".to_owned());
            let error =
                lower_cpu_param_node(&node, &mut body, &BTreeMap::new(), &mut next, &mut last)
                    .unwrap_err();
            assert!(error.contains("callback.arg"));
            assert!(error.contains("selected LLVM function context"));
            assert!(body.is_empty());
            assert_eq!(next, 7);
            assert_eq!(last.as_deref(), Some("%prior"));
            let registers =
                BTreeMap::from([(node.name.clone(), LlvmValueRef::I64("%bound".to_owned()))]);
            assert!(
                lower_cpu_param_node(&node, &mut body, &registers, &mut next, &mut last).unwrap()
            );
            assert_eq!(last.as_deref(), Some("%bound"));
        }
    }
}

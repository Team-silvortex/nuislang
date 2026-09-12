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
        "param_bool" | "param_i32" | "param_i64" | "param_f32" | "param_f64"
    ) {
        return Ok(false);
    }

    let binding = registers.get(&node.name).ok_or_else(|| {
        format!(
            "CPU parameter `{}` has no binding in the selected LLVM function context",
            node.name
        )
    })?;
    if matches!(node.op.instruction.as_str(), "param_f32" | "param_f64") {
        // Float parameters are already typed SSA bindings, not fallback integer returns.
        if !matches!(
            (node.op.instruction.as_str(), binding),
            ("param_f32", LlvmValueRef::F32(_)) | ("param_f64", LlvmValueRef::F64(_))
        ) {
            return Err(format!(
                "CPU floating parameter `{}` has a mismatched LLVM binding",
                node.name
            ));
        }
        return Ok(true);
    }
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

    #[test]
    fn floating_parameters_remain_typed_and_never_convert_to_fallback_integers() {
        for (instruction, binding) in [
            ("param_f32", LlvmValueRef::F32("%float".to_owned())),
            ("param_f64", LlvmValueRef::F64("%double".to_owned())),
        ] {
            let node = Node {
                name: "arg".to_owned(),
                resource: "cpu0".to_owned(),
                op: yir_core::Operation::parse(&format!("cpu.{instruction}"), vec!["0".to_owned()])
                    .unwrap(),
            };
            let mut body = Vec::new();
            let mut next = 7;
            let mut last = Some("%prior".to_owned());
            for registers in [
                BTreeMap::new(),
                BTreeMap::from([("arg".to_owned(), LlvmValueRef::I64("%wrong".to_owned()))]),
            ] {
                assert!(
                    lower_cpu_param_node(&node, &mut body, &registers, &mut next, &mut last)
                        .is_err()
                );
            }
            assert!(lower_cpu_param_node(
                &node,
                &mut body,
                &BTreeMap::from([("arg".to_owned(), binding)]),
                &mut next,
                &mut last
            )
            .unwrap());
            assert!(body.is_empty());
            assert_eq!(next, 7);
            assert_eq!(last.as_deref(), Some("%prior"));
        }
    }
}

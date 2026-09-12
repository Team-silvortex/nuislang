use super::*;

/// Strict native functions may not use the generic backend's scalar coercions.
pub(super) fn validate(
    node: &Node,
    registers: &BTreeMap<String, LlvmValueRef>,
    signatures: &BTreeMap<String, CpuHelperSignature>,
    return_kind: CpuCallScalarKind,
    aggregate_return: bool,
) -> Result<(), String> {
    if matches!(
        node.op.instruction.as_str(),
        "loop_while_i64"
            | "loop_while_i64_chain"
            | "loop_while_scalar_chain"
            | "loop_while_i64_effect"
    ) {
        for name in &node.op.args[..3] {
            require_value(node, name, CpuCallScalarKind::I64, registers)?;
        }
        if node.op.instruction.ends_with("_chain") {
            for pair in node.op.args[5..].chunks_exact(2) {
                require_value(node, &pair[0], CpuCallScalarKind::I64, registers)?;
            }
        }
    }
    if let Some(call) = native_session::loops::scoped::parse(node)? {
        let signature = signatures
            .get(call.callee)
            .ok_or_else(|| format!("native scoped call `{}` has no emitted helper", node.name))?;
        if signature.params.len() != call.operands.len() {
            return Err(format!(
                "native scoped call `{}` argument count drift",
                node.name
            ));
        }
        if let Some(initial) = call.initial {
            require_value(node, initial, CpuCallScalarKind::I64, registers)?;
        }
        for (operand, kind) in call.operands.iter().zip(&signature.params) {
            if matches!(operand.as_str(), "$current" | "$carry") {
                if *kind != CpuCallScalarKind::I64 {
                    return Err(format!(
                        "native scoped call `{}` requires exact i64 loop-state parameters",
                        node.name
                    ));
                }
            } else {
                require_value(node, operand, *kind, registers)?;
            }
        }
    }
    if node.op.instruction == "guard_return" {
        if !matches!(
            registers.get(&node.op.args[0]),
            Some(LlvmValueRef::Bool { .. } | LlvmValueRef::I64(_))
        ) {
            return Err(format!(
                "native scalar guard `{}` requires a bool or i64 condition",
                node.name
            ));
        }
        if !aggregate_return && node.op.args.len() != 2 {
            return Err("native scalar helper guard cannot carry an aggregate layout".to_owned());
        }
    }
    if node.op.instruction.starts_with("call_") {
        let signature = signatures
            .get(&node.op.args[0])
            .ok_or_else(|| format!("native scalar call `{}` has no emitted helper", node.name))?;
        if node.op.args.len() - 1 != signature.params.len() {
            return Err(format!(
                "native scalar call `{}` argument count drift",
                node.name
            ));
        }
        for (name, kind) in node.op.args[1..].iter().zip(&signature.params) {
            require_value(node, name, *kind, registers)?;
        }
    }
    if !aggregate_return {
        if node.op.instruction.starts_with("return_") {
            require_value(node, &node.op.args[0], return_kind, registers)?;
        } else if node.op.instruction == "guard_return" {
            require_value(node, &node.op.args[1], return_kind, registers)?;
        }
    }
    Ok(())
}

fn require_value(
    node: &Node,
    name: &str,
    kind: CpuCallScalarKind,
    registers: &BTreeMap<String, LlvmValueRef>,
) -> Result<(), String> {
    if registers
        .get(name)
        .and_then(|value| crate::call_lowering::lower_scalar_value_arg(value, &kind))
        .is_none()
    {
        return Err(format!(
            "native scalar `{}` input `{name}` does not exactly match its declared scalar kind",
            node.name
        ));
    }
    Ok(())
}

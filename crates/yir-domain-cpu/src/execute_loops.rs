use yir_core::{ExecutionState, Node, Value};

pub(super) fn evaluate_i64_loop_final(node: &Node, state: &ExecutionState) -> Result<i64, String> {
    let initial = state.expect_int(required_arg(node, 0, "initial")?)?;
    let limit = state.expect_int(required_arg(node, 1, "limit")?)?;
    let step = state.expect_int(required_arg(node, 2, "step")?)?;
    let compare = required_arg(node, 3, "compare kind")?;
    let step_kind = required_arg(node, 4, "step kind")?;
    let delta = match step_kind {
        "add" => i128::from(step),
        "sub" => -i128::from(step),
        other => {
            return Err(format!(
                "node `{}` has invalid loop step kind `{other}`",
                node.name
            ))
        }
    };
    let initial_wide = i128::from(initial);
    let limit_wide = i128::from(limit);

    let final_value = match compare {
        "eq" if initial != limit => initial_wide,
        "eq" => {
            if delta == 0 {
                return Err(non_terminating_loop(node));
            }
            return Ok(match step_kind {
                "add" => initial.wrapping_add(step),
                "sub" => initial.wrapping_sub(step),
                _ => unreachable!(),
            });
        }
        "ne" if initial == limit => initial_wide,
        "ne" => {
            if delta == 0 || !modular_step_reaches(initial, limit, delta) {
                return Err(non_terminating_loop(node));
            }
            limit_wide
        }
        "lt" if initial >= limit => initial_wide,
        "lt" => advance_relational(node, initial_wide, limit_wide, delta, false)?,
        "le" if initial > limit => initial_wide,
        "le" => advance_relational(node, initial_wide, limit_wide, delta, true)?,
        "gt" if initial <= limit => initial_wide,
        "gt" => retreat_relational(node, initial_wide, limit_wide, delta, false)?,
        "ge" if initial < limit => initial_wide,
        "ge" => retreat_relational(node, initial_wide, limit_wide, delta, true)?,
        other => {
            return Err(format!(
                "node `{}` has invalid loop compare kind `{other}`",
                node.name
            ))
        }
    };

    i64::try_from(final_value).map_err(|_| {
        format!(
            "node `{}` loop induction overflows i64 before reaching its exit",
            node.name
        )
    })
}

pub(super) fn render_loop_effect_action(
    node: &Node,
    state: &ExecutionState,
    current: &Value,
) -> Result<String, String> {
    let args = &node.op.args;
    let arity = args
        .get(7)
        .ok_or_else(|| missing_action(node))?
        .parse::<usize>()
        .map_err(|_| {
            format!(
                "node `{}` has invalid loop action arity `{}`",
                node.name, args[7]
            )
        })?;
    if args.len() != 8 + arity {
        return Err(format!(
            "node `{}` declares {arity} loop action operands but provides {}",
            node.name,
            args.len().saturating_sub(8)
        ));
    }
    let module = args.get(5).ok_or_else(|| missing_action(node))?;
    let instruction = args.get(6).ok_or_else(|| missing_action(node))?;

    match (module.as_str(), instruction.as_str(), arity) {
        ("cpu", "owned_bytes_copy_drop", 1) => {
            let value = state.expect_value(&args[8])?;
            Ok(format!("cpu.owned_bytes_copy_drop({value:?})"))
        }
        ("cpu", "scoped_call", arity) if arity >= 1 => {
            let callee = &args[8];
            let operands = resolve_scoped_operands(&args[9..], state, current)?;
            Ok(format!("cpu.scoped_call {callee}({operands:?})"))
        }
        ("cpu", "scoped_call_owned_return", arity) if arity >= 2 => {
            let callee = &args[8];
            let result = &args[9];
            let operands = resolve_scoped_operands(&args[10..], state, current)?;
            Ok(format!(
                "cpu.scoped_call_owned_return {callee} -> {result}({operands:?})"
            ))
        }
        ("cpu", "scoped_call_owned_struct_return", arity) if arity >= 4 => {
            let callee = &args[8];
            let result = &args[9];
            let operands = resolve_scoped_operands(&args[11..], state, current)?;
            Ok(format!(
                "cpu.scoped_call_owned_struct_return {callee} -> {result}({operands:?})"
            ))
        }
        (module, instruction, _) => Err(format!(
            "node `{}` references unregistered loop action `{module}.{instruction}`",
            node.name
        )),
    }
}

fn advance_relational(
    node: &Node,
    initial: i128,
    limit: i128,
    delta: i128,
    inclusive: bool,
) -> Result<i128, String> {
    if delta <= 0 {
        return Err(non_terminating_loop(node));
    }
    let distance = limit - initial;
    let iterations = if inclusive {
        distance / delta + 1
    } else {
        (distance + delta - 1) / delta
    };
    Ok(initial + iterations * delta)
}

fn retreat_relational(
    node: &Node,
    initial: i128,
    limit: i128,
    delta: i128,
    inclusive: bool,
) -> Result<i128, String> {
    if delta >= 0 {
        return Err(non_terminating_loop(node));
    }
    let magnitude = -delta;
    let distance = initial - limit;
    let iterations = if inclusive {
        distance / magnitude + 1
    } else {
        (distance + magnitude - 1) / magnitude
    };
    Ok(initial + iterations * delta)
}

fn modular_step_reaches(initial: i64, limit: i64, delta: i128) -> bool {
    const MODULUS: u128 = 1_u128 << 64;
    let step = delta.rem_euclid(MODULUS as i128) as u128;
    let distance = (i128::from(limit) - i128::from(initial)).rem_euclid(MODULUS as i128) as u128;
    distance % greatest_common_divisor(step, MODULUS) == 0
}

fn greatest_common_divisor(mut lhs: u128, mut rhs: u128) -> u128 {
    while rhs != 0 {
        (lhs, rhs) = (rhs, lhs % rhs);
    }
    lhs
}

fn resolve_scoped_operands(
    operands: &[String],
    state: &ExecutionState,
    current: &Value,
) -> Result<Vec<Value>, String> {
    operands
        .iter()
        .map(|operand| {
            if operand == "$current" {
                return Ok(current.clone());
            }
            let value_name = if let Some(input) = operand
                .strip_prefix("copy_owned:")
                .or_else(|| operand.strip_prefix("move_owned:"))
            {
                input
            } else if let Some((_, input)) = yir_core::parse_loop_owned_struct_carry(operand)? {
                input
            } else {
                operand
            };
            state.expect_value(value_name).cloned()
        })
        .collect()
}

fn required_arg<'a>(node: &'a Node, index: usize, role: &str) -> Result<&'a str, String> {
    node.op
        .args
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("node `{}` is missing its loop {role}", node.name))
}

fn missing_action(node: &Node) -> String {
    format!("node `{}` is missing loop action metadata", node.name)
}

fn non_terminating_loop(node: &Node) -> String {
    format!(
        "node `{}` has a loop induction that cannot reach its exit",
        node.name
    )
}

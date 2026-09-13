use yir_core::{loop_carry_contract::ScopedI64Carries, Node};

/// Native subset of the existing scoped-call payload, not a new loop contract.
pub(crate) struct ScopedCall<'a> {
    pub callee: &'a str,
    pub operands: &'a [String],
    pub initial: Option<&'a str>,
    pub carries: Option<ScopedI64Carries<'a>>,
}

pub(crate) fn parse(node: &Node) -> Result<Option<ScopedCall<'_>>, String> {
    if node.op.instruction != "loop_while_i64_effect" {
        return Ok(None);
    }
    let fail = || {
        format!(
            "native scalar loop `{}` does not admit this scoped action",
            node.name
        )
    };
    let args = &node.op.args;
    if args.len() < 9 || args[5] != "cpu" {
        return Err(fail());
    }
    if let Some(carry) = yir_core::loop_carry_contract::parse_scoped_i64_carry(args)? {
        return Ok(Some(ScopedCall {
            callee: carry.callee,
            operands: carry.operands,
            initial: Some(carry.initial),
            carries: None,
        }));
    }
    if let Some(carries) = yir_core::loop_carry_contract::parse_scoped_i64_carries(args)? {
        if carries.seeds.len() > super::super::MAX_SCALAR_SLOTS {
            return Err(fail());
        }
        return Ok(Some(ScopedCall {
            callee: carries.callee,
            operands: carries.operands,
            initial: None,
            carries: Some(carries),
        }));
    }
    if args[6] != "scoped_call"
        || args[7]
            .parse::<usize>()
            .ok()
            .is_none_or(|arity| arity == 0 || arity.checked_add(8) != Some(args.len()))
    {
        return Err(fail());
    }
    let named = |value: &str| {
        !value.is_empty() && !value.contains(['$', ':']) && !value.chars().any(char::is_whitespace)
    };
    if !named(&args[8]) || args[9..].iter().any(|arg| arg != "$current" && !named(arg)) {
        return Err(fail());
    }
    Ok(Some(ScopedCall {
        callee: &args[8],
        operands: &args[9..],
        initial: None,
        carries: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_scoped_multi_carry_slot_bound_is_not_a_precombined_arity() {
        for count in [1, 2, 7, 64, 65] {
            let fields = (0..count)
                .map(|i| format!("carry{i}:i64"))
                .collect::<Vec<_>>()
                .join(";");
            let mut args = "begin end step lt add cpu scoped_call_i64_carries"
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            args.extend([
                (count + 2).to_string(),
                "advance".to_owned(),
                format!("State{{{fields}}}"),
            ]);
            args.extend(
                (0..count)
                    .rev()
                    .map(|i| yir_core::encode_loop_owned_struct_carry(i, &format!("seed{i}"))),
            );
            let mut node = Node {
                name: "loop".to_owned(),
                resource: "cpu".to_owned(),
                op: yir_core::Operation::parse("cpu.loop_while_i64_effect", args).unwrap(),
            };
            for action in ["scoped_call_i64_carries", "scoped_call_i64_carries_break"] {
                node.op.args[6] = action.to_owned();
                if count <= 64 {
                    let carries = parse(&node).unwrap().unwrap().carries.unwrap();
                    assert_eq!(carries.seeds.len(), count);
                    assert_eq!(carries.break_on_return, action.ends_with("_break"));
                } else {
                    assert!(parse(&node)
                        .err()
                        .expect("carry bound must reject")
                        .contains("does not admit this scoped action"));
                }
            }
        }
    }
}

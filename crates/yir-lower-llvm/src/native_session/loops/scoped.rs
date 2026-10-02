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
            return Err(format!(
                "{}: {} carried words exceed the {}-word native limit (including private return/control state)",
                fail(),
                carries.seeds.len(),
                super::super::MAX_SCALAR_SLOTS,
            ));
        }
        return Ok(Some(ScopedCall {
            callee: carries.callee,
            operands: carries.operands,
            initial: None,
            carries: Some(carries),
        }));
    }
    let call = yir_core::loop_carry_contract::parse_scoped_call(args)?.ok_or_else(fail)?;
    if call
        .operands
        .iter()
        .any(|arg| arg.starts_with("copy_owned:") || arg.starts_with("move_owned:"))
    {
        return Err(fail());
    }
    Ok(Some(ScopedCall {
        callee: call.callee,
        operands: call.operands,
        initial: None,
        carries: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_scoped_readonly_records_keep_carry_and_resource_boundaries() {
        for scalar in [false, true] {
            let mut args = "begin end step lt add cpu scoped_call 0 helper"
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if scalar {
                args[6] = "scoped_call_i64_carry".into();
                args.extend(["seed".into(), "$carry".into()]);
            }
            args.push("$value_record:Input{x:f32;i:i64}|gain|$current".into());
            args[7] = (args.len() - 8).to_string();
            let mut node = Node {
                name: "loop".into(),
                resource: "cpu0".into(),
                op: yir_core::Operation::parse("cpu.loop_while_i64_effect", args).unwrap(),
            };
            let call = parse(&node).unwrap().unwrap();
            assert_eq!(call.initial, if scalar { Some("seed") } else { None });
            assert!(call.carries.is_none());
            for invalid in [
                "$value_record:Input{x:i64}|$owned_struct_carry:0:seed",
                "$value_record:Input{x:i64}|$carry",
                "$value_record:Input{x:f32}|$current",
                "$value_record:Input{x:i64}|copy_owned:seed",
                "copy_owned:seed",
                "move_owned:seed",
            ] {
                *node.op.args.last_mut().unwrap() = invalid.into();
                assert!(parse(&node).is_err(), "{invalid}");
            }
        }
    }

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
                    let mut renamed = node.clone();
                    renamed.op.args[9] = renamed.op.args[9].replace("carry0:", "value:");
                    assert!(
                        parse(&renamed).is_err(),
                        "ordinary field names cannot redefine a scoped carry"
                    );
                } else {
                    let error = parse(&node).err().expect("carry bound must reject");
                    assert!(error.contains("65 carried words exceed the 64-word native limit"));
                    assert!(error.contains("including private return/control state"));
                }
            }
        }
    }

    #[test]
    fn independent_seed_slots_still_obey_native_state_bounds() {
        for count in [1, 7, 64, 65] {
            let seeds = (0..count).map(|i| format!("seed{i}")).collect::<Vec<_>>();
            let fields = (0..count)
                .map(|i| format!("carry{i}:i64"))
                .collect::<Vec<_>>()
                .join(";");
            let mut args = "begin end step lt add cpu scoped_call_i64_carries 0 advance"
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            args.push(format!("State{{{fields}}}"));
            args.extend(yir_core::loop_carry_contract::encode_scoped_i64_seeds(
                &seeds,
            ));
            args[7] = (args.len() - 8).to_string();
            let mut node = Node {
                name: "loop".into(),
                resource: "cpu".into(),
                op: yir_core::Operation::parse("cpu.loop_while_i64_effect", args).unwrap(),
            };
            for action in ["scoped_call_i64_carries", "scoped_call_i64_carries_break"] {
                node.op.args[6] = action.into();
                if count <= 64 {
                    let parsed = parse(&node).unwrap().unwrap();
                    assert!(parsed.operands.is_empty());
                    assert_eq!(parsed.carries.unwrap().seeds.len(), count);
                } else {
                    assert!(parse(&node).is_err());
                }
            }
        }
    }
}

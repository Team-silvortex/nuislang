use super::*;

const SOURCE: &str =
    include_str!("../tests/native_application_bridge/return_i32_i64_private_pair.ns");

#[test]
fn native_return_i32_i64_private_pair_keeps_exact_owned_layout_guarded_order_and_policy_vetoes() {
    let (_, mut module, mode) = compiled(SOURCE);
    let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
    assert!(bridge.state_layout.source().ends_with(";result:bool}"));
    assert!(module
        .functions
        .iter()
        .any(|f| f.name.starts_with("__nuis_effect_call_")
            && f.result.as_ref().is_some_and(|result| result.ty == "i32")));
    assert!(module
        .nodes
        .iter()
        .any(|n| n.op.instruction == "call_owned_struct"
            && n.op
                .args
                .get(1)
                .is_some_and(|layout| layout.ends_with("{owner:i32;child:i64}"))));
    let child = module
        .functions
        .iter()
        .find(|f| f.name == "child_initial")
        .unwrap();
    assert!(!width_safe_narrow(&module, &child.parameters[0].node));
    for node in module
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "cast_i64_to_i32")
    {
        assert!(
            width_safe_narrow(&module, &node.op.args[0]),
            "unexpected i64 payload narrowing at {}",
            node.name
        );
    }
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_"))
    {
        let sensitive = function
            .body_nodes
            .iter()
            .filter_map(|name| {
                let node = module.nodes.iter().find(|n| &n.name == name).unwrap();
                (node.op.instruction.starts_with("call_")
                    || matches!(node.op.instruction.as_str(), "guard_return" | "div" | "rem"))
                .then_some(node)
            })
            .collect::<Vec<_>>();
        assert_eq!(sensitive[0].op.instruction, "guard_return");
        for pair in sensitive.windows(2) {
            let mut seen = std::collections::BTreeSet::new();
            let mut pending = vec![pair[0].name.as_str()];
            while let Some(from) = pending.pop() {
                for edge in module.edges.iter().filter(|edge| {
                    edge.from == from
                        && matches!(edge.kind, yir_core::EdgeKind::Effect)
                        && function.body_nodes.contains(&edge.to)
                }) {
                    if seen.insert(edge.to.as_str()) {
                        pending.push(edge.to.as_str());
                    }
                }
            }
            assert!(
                seen.contains(pair[1].name.as_str()),
                "{} lacks an Effect path",
                function.name
            );
        }
    }
    assert!(!bridge
        .llvm_ir
        .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!bridge
        .llvm_ir
        .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    module
        .edges
        .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
    let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
    assert!(error.contains("lacks dependency order"), "{error}");
}

fn node<'a>(module: &'a yir_core::YirModule, name: &str) -> Option<&'a yir_core::Node> {
    module.nodes.iter().find(|n| n.name == name)
}

fn constant(module: &yir_core::YirModule, name: &str, depth: usize) -> Option<i64> {
    if depth >= 8 {
        return None;
    }
    let n = node(module, name)?;
    match (n.op.instruction.as_str(), n.op.args.as_slice()) {
        ("const_i64", [value]) => value.parse().ok(),
        ("sub", [lhs, rhs]) => {
            constant(module, lhs, depth + 1)?.checked_sub(constant(module, rhs, depth + 1)?)
        }
        ("cast_i32_to_i64" | "cast_i64_to_i32", [input]) => {
            let value = constant(module, input, depth + 1)?;
            i32::try_from(value).ok().map(i64::from)
        }
        _ => None,
    }
}

fn binary<'a>(module: &'a yir_core::YirModule, name: &str, op: &str) -> Option<(&'a str, &'a str)> {
    let n = node(module, name)?;
    let [lhs, rhs] = n.op.args.as_slice() else {
        return None;
    };
    (n.op.instruction == op).then_some((lhs.as_str(), rhs.as_str()))
}

fn unary<'a>(module: &'a yir_core::YirModule, name: &str, op: &str) -> Option<&'a str> {
    let n = node(module, name)?;
    let [input] = n.op.args.as_slice() else {
        return None;
    };
    (n.op.instruction == op).then_some(input.as_str())
}

fn canonical_i32_codec(module: &yir_core::YirModule, name: &str, depth: usize) -> bool {
    if depth >= 8 {
        return false;
    }
    let Some(n) = node(module, name) else {
        return false;
    };
    match (n.op.instruction.as_str(), n.op.args.as_slice()) {
        ("const_i64", [word]) => word
            .parse::<i64>()
            .is_ok_and(|v| (0..=8_589_934_591).contains(&v)),
        ("select_i64", [_, yes, no]) => {
            canonical_i32_codec(module, yes, depth + 1)
                && canonical_i32_codec(module, no, depth + 1)
        }
        ("call_i64", [callee, ..]) if callee.starts_with("__nuis_effect_call_") => {
            let Some(function) = module.functions.iter().find(|f| f.name == *callee) else {
                return false;
            };
            let Some(result) = &function.result else {
                return false;
            };
            result.ty == "i64"
                && unary(module, &result.node, "return_i64")
                    .is_some_and(|returned| canonical_i32_codec(module, returned, depth + 1))
        }
        ("add", [biased, flag]) => {
            let Some((value, bias)) = binary(module, biased, "add") else {
                return false;
            };
            let Some((boolean, weight)) = binary(module, flag, "mul") else {
                return false;
            };
            unary(module, value, "cast_i32_to_i64").is_some()
                && constant(module, bias, 0) == Some(2_147_483_648)
                && unary(module, boolean, "cast_bool_to_i64").is_some()
                && constant(module, weight, 0) == Some(4_294_967_296)
        }
        _ => false,
    }
}

fn width_safe_narrow(module: &yir_core::YirModule, name: &str) -> bool {
    if matches!(constant(module, name, 0), Some(0 | -1)) {
        return true;
    }
    let Some((lhs, rhs)) = binary(module, name, "sub") else {
        return false;
    };
    // Existing i32 arithmetic widens both operands; -1 - any i32 remains in range.
    if unary(module, lhs, "cast_i32_to_i64").is_some()
        && unary(module, rhs, "cast_i32_to_i64").is_some()
        && constant(module, lhs, 0) == Some(-1)
    {
        return true;
    }
    // Existing bool/i32 restoration is separate from the new independently typed pair.
    if constant(module, rhs, 0) != Some(2_147_483_648) {
        return false;
    }
    let Some((carrier, flag)) = binary(module, lhs, "sub") else {
        return false;
    };
    let Some((boolean, weight)) = binary(module, flag, "mul") else {
        return false;
    };
    let Some(predicate) = unary(module, boolean, "cast_bool_to_i64") else {
        return false;
    };
    let Some((observed, threshold)) = binary(module, predicate, "ge") else {
        return false;
    };
    observed == carrier
        && constant(module, weight, 0) == Some(4_294_967_296)
        && constant(module, threshold, 0) == Some(4_294_967_296)
        && canonical_i32_codec(module, carrier, 0)
}

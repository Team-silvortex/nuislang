use super::*;

#[test]
fn private_record_copies_scalar_calls_prune_unobserved_copies_but_keep_checked_fields() {
    for binding in [
        "",
        "let selected = snapshot.count; const forwarded: i64 = selected;",
    ] {
        let operand = if binding.is_empty() {
            "snapshot.count"
        } else {
            "forwarded"
        };
        let mut module = module(
            &format!("let snapshot = {COPY}; {binding} return read_count({operand});"),
            "",
        );
        let original = helper(&mut module).body.clone();
        normalize_and_verify(&mut module);
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &helper(&mut module).body[0]
        else {
            panic!()
        };
        assert!(inputs(&fields[0].1).is_empty());
        assert!(inputs(&fields[1].1).is_empty());
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields: before, .. },
            ..
        } = &original[0]
        else {
            panic!()
        };
        assert_eq!(fields[2], before[2]);
        assert_eq!(&helper(&mut module).body[1..], &original[1..]);
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before);
    }
}

#[test]
fn private_record_copies_scalar_calls_require_exact_nested_scalar_paths() {
    for (field, kind) in [
        ("value", "f64"),
        ("gain", "f32"),
        ("tag", "i32"),
        ("enabled", "bool"),
    ] {
        let mut module = module(
            &format!("let snapshot = {COPY}; let used = scalar(snapshot.right.{field}); return snapshot.count;"),
            &format!("fn scalar(value: {kind}) -> {kind} {{ return value; }}"),
        );
        normalize_and_verify(&mut module);
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &helper(&mut module).body[0]
        else {
            panic!()
        };
        assert!(inputs(&fields[0].1).is_empty());
        let NirExpr::StructLiteral { fields: right, .. } = &fields[1].1 else {
            panic!("a scalar observation should permit exact subrecord projection")
        };
        for (name, value) in right {
            assert_eq!(inputs(value).is_empty(), name != field);
        }
        assert!(matches!(fields[2].1, NirExpr::Binary { .. }));
    }
}

#[test]
fn private_record_copies_scalar_calls_do_not_authorize_aggregate_transport() {
    for operand in ["snapshot", "saved"] {
        let binding = if operand == "saved" {
            "let saved = snapshot;"
        } else {
            ""
        };
        let mut module = module(
            &format!("let snapshot = {COPY}; {binding} let used = consume({operand}); return snapshot.count;"),
            "",
        );
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before);
    }
    let mut module = module(
        &format!("let snapshot = {COPY}; let used = leaf(snapshot.right); return snapshot.count;"),
        "fn leaf(value: Leaf) -> i64 { return 0; }",
    );
    let before = module.clone();
    normalize_and_verify(&mut module);
    assert_eq!(module, before);
}

#[test]
fn private_record_copies_scalar_calls_keep_unknown_paths_and_budget_failure_conservative() {
    let mut module = module(
        &format!("let snapshot = {COPY}; return read_count(snapshot.count);"),
        "",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let function = helper(&mut module).clone();
    for remaining in [0, 1, 8, 32] {
        assert!(normalized(&function, &layouts, &BTreeSet::new(), remaining).is_none());
        assert_eq!(helper(&mut module), &function);
    }
    for field in ["missing", "count.child"] {
        let path = field.split('.').map(String::from).collect::<Vec<_>>();
        let NirStmt::Return(Some(NirExpr::Call { args, .. })) =
            helper(&mut module).body.last_mut().unwrap()
        else {
            panic!()
        };
        args[0] = source_value("snapshot", &path);
        let before = module.clone();
        normalize(helper(&mut module), &layouts, &BTreeSet::new());
        assert_eq!(module, before);
    }
}

#[test]
fn private_record_copies_scalar_calls_keep_constructed_and_encoded_transport_provenance() {
    for body in [
        format!("let snapshot = {COPY}; return consume_count(Count {{ value: snapshot.count }});"),
        format!("let snapshot = {COPY}; let packet = Count {{ value: snapshot.count }}; return consume_count(packet);"),
    ] {
        let mut module = module(&body, "struct Count { value: i64 } fn consume_count(value: Count) -> i64 { return value.value; }");
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before);
    }
    let mut module = module(
        &format!("let snapshot = {COPY}; return read_count(snapshot.count);"),
        "",
    );
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) =
        helper(&mut module).body.last_mut().unwrap()
    else {
        panic!()
    };
    args[0] = NirExpr::CastI32ToI64(Box::new(NirExpr::CastI64ToI32(Box::new(args[0].clone()))));
    let before = module.clone();
    normalize_and_verify(&mut module);
    assert_eq!(module, before);
}

#[test]
fn private_record_copies_scalar_calls_preserve_selected_constructor_work_differentially() {
    let execute = |module: &NirModule| {
        let mut yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
        yir.nodes.reverse();
        yir.functions.reverse();
        for function in &mut yir.functions {
            function.body_nodes.reverse();
        }
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .map_err(|_| ())?;
        let main = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap();
        Ok(trace.values[&main.result.as_ref().unwrap().node].clone())
    };
    for count in [0, 1, 2, 3] {
        for divisor in [0, 1, 2, 4] {
            for position in ["none", "before", "after"] {
                let guard = "if state.count == 2 { return read_count(state.left.used); }";
                let before_guard = if position == "before" { guard } else { "" };
                let after_guard = if position == "after" { guard } else { "" };
                let mut module = crate::frontend::parse_nuis_module(&format!(
                    "mod cpu Main {{
                    struct Leaf {{ used: i64, unused: i64 }}
                    struct State {{ left: Leaf, right: Leaf, count: i64 }}
                    @noinline fn checked(divisor: i64) -> i64 {{ return 10 / divisor; }}
                    @noinline fn read_count(value: i64) -> i64 {{ return value; }}
                    @noinline fn helper(state: State, other: State) -> i64 {{
                        if state.count == 0 {{ return 0; }}
                        {before_guard}
                        let snapshot = State {{
                            left: Leaf {{ used: state.left.used, unused: state.left.unused }},
                            right: state.right, count: checked(other.count - state.count)
                        }};
                        {after_guard}
                        return read_count(snapshot.left.used);
                    }}
                    fn main() -> i64 {{ return helper(
                        State {{ left: Leaf {{ used: 7, unused: 99 }}, right: Leaf {{ used: 101, unused: 103 }}, count: {count} }},
                        State {{ left: Leaf {{ used: 0, unused: 0 }}, right: Leaf {{ used: 0, unused: 0 }}, count: {divisor} }});
                    }} }}"
                ))
                .unwrap();
                let expected = if count == 0 {
                    Ok(yir_core::Value::Int(0))
                } else if position == "before" && count == 2 {
                    Ok(yir_core::Value::Int(7))
                } else if divisor == count {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(7))
                };
                assert_eq!(
                    execute(&module),
                    expected,
                    "before: {count}/{divisor}/{position}"
                );
                let original = module.clone();
                normalize_and_verify(&mut module);
                assert_ne!(module, original);
                assert_eq!(
                    execute(&module),
                    expected,
                    "after: {count}/{divisor}/{position}"
                );
            }
        }
    }
}

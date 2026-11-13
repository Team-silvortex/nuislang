use super::*;

fn source(op: &str) -> String {
    logical_local_tests::source()
        .replace("gate && observe(inner, divisor)", &format!("(gate && observe(inner, divisor)) {op} (inner || observe(gate, divisor))"))
        .replace("const ready: bool = inner || observe(gate, divisor)", &format!("const ready: bool = (inner || observe(gate, divisor)) {op} (gate && observe(inner, divisor))"))
}

#[test]
fn iteration_eager_locals_lower_both_operands_without_purity_leaks() {
    for op in ["==", "!=", "^"] {
        for inferred in [false, true] {
            for reversed in [false, true] {
                let text = if inferred {
                    source(op).replace("let ready: bool", "let ready")
                } else {
                    source(op)
                };
                let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
                if reversed {
                    module.functions.reverse();
                }
                let layouts = control_values::layouts(&module);
                let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
                assert!(catalog.contains_key("owner"), "{op}");
                assert!(
                    !scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("owner")
                );
                let retained = catalog.keys().cloned().collect();
                let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
                let mut helpers = Vec::new();
                let mut guarded = BTreeSet::new();
                scalar_control::outline(
                    &mut module,
                    &retained,
                    &mut names,
                    &mut helpers,
                    &mut guarded,
                    &catalog,
                    &layouts,
                );
                let lazy = helpers
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_scalar_predicate"))
                    .collect::<Vec<_>>();
                assert_eq!(lazy.len(), 4, "both sides, both arms: {op}");
                assert!(lazy.iter().all(
                    |f| guarded.contains(&f.name) && f.return_type == Some(scalar_type("bool"))
                ));
                module.functions.extend(helpers);
                crate::nir_verify::verify_nir_module(&module).unwrap();
            }
        }
    }
}

#[test]
fn iteration_eager_locals_keep_argument_print_scope_and_type_boundaries() {
    let original = source("==");
    for (from, to) in [
        ("let ready: bool", "let ready: i64"),
        ("const ready: bool", "const ready: i64"),
        ("observe(inner, divisor)", "observe(ready, divisor)"),
        (
            "observe(inner, divisor)",
            "observe((inner && gate) == gate, divisor)",
        ),
        ("observe(inner, divisor)", "missing(inner, divisor)"),
        ("== (inner || observe(gate, divisor))", "== divisor"),
        (
            "if gate {",
            "if (gate && observe(inner, divisor)) == inner {",
        ),
        ("print(63)", "print((inner && gate) == gate)"),
        ("print(joined)", "print(ready)"),
    ] {
        let text = original.replace(from, to);
        assert_ne!(text, original);
        if let Ok(module) = crate::frontend::parse_nuis_module(&text) {
            let before = module.clone();
            let layouts = control_values::layouts(&module);
            assert!(
                !scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("owner"),
                "{from} -> {to}"
            );
            assert_eq!(module, before);
        }
    }
}

#[test]
fn iteration_eager_locals_share_all_root_budgets_without_field_permission() {
    use conditional_values::prefix::{computed_initializer_expression_roots, Root};
    let layouts = BTreeMap::from([("Flags".into(), vec![("flag".into(), scalar_type("bool"))])]);
    let logical = |count| {
        let mut value = NirExpr::Var("gate".into());
        for _ in 0..count {
            value = NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Var("gate".into())),
            };
        }
        value
    };
    let record = |value| NirExpr::StructLiteral {
        type_name: "Flags".into(),
        type_args: vec![],
        fields: vec![("flag".into(), value)],
    };
    let predicate = logical(1);
    let field = record(logical(1));
    for (count, expected) in [(15, true), (16, false)] {
        let value = NirExpr::Binary {
            op: NirBinaryOp::Xor,
            lhs: Box::new(logical(15)),
            rhs: Box::new(logical(count)),
        };
        let root = |value, eager| Root {
            value,
            depth: 0,
            logical: true,
            eager,
        };
        assert_eq!(
            computed_initializer_expression_roots(
                vec![
                    root(&value, true),
                    root(&predicate, false),
                    root(&field, true)
                ],
                &layouts
            ),
            expected,
            "shared 32/33 edges across eager local, predicate and field"
        );
        assert!(!computed_initializer_expression_roots(
            vec![root(&value, false)],
            &layouts
        ));
        let nested = record(value.clone());
        assert!(!computed_initializer_expression_roots(
            vec![root(&nested, true)],
            &layouts
        ));
        let argument = NirExpr::Call {
            callee: "observe".into(),
            args: vec![value.clone()],
        };
        assert!(!computed_initializer_expression_roots(
            vec![root(&argument, true)],
            &layouts
        ));
    }
}

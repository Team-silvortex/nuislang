use super::*;

fn source() -> String {
    record_tests::source("i64")
        .replace("fn main()", "@noinline fn observe(value: bool, divisor: i64) -> bool { print(61); let checked = 0 / divisor; print(62); return value; } fn main()")
        .replace("tag: 0 / divisor, flag: false", "tag: 0 / divisor, flag: gate && observe(inner, divisor)")
        .replace("tag: 0, flag: true", "tag: 0, flag: inner || observe(gate, divisor)")
}

#[test]
fn iteration_logical_record_fields_keep_effects_guarded_and_scope_private() {
    for reversed in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(&source()).unwrap();
        if reversed {
            module.functions.reverse();
            module.structs.reverse();
        }
        let layouts = control_values::layouts(&module);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        assert!(catalog.contains_key("owner"));
        assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("owner"));
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
        assert_eq!(lazy.len(), 2);
        assert!(lazy
            .iter()
            .all(|f| guarded.contains(&f.name) && f.return_type == Some(scalar_type("bool"))));
        module.functions.extend(helpers);
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn iteration_logical_record_fields_reject_hidden_roots_and_unproved_calls() {
    let original = source();
    for (from, to) in [
        (
            "flag: gate && observe(inner, divisor)",
            "flag: (gate && observe(inner, divisor)) == inner",
        ),
        (
            "flag: gate && observe(inner, divisor)",
            "flag: observe(gate && inner, divisor)",
        ),
        (
            "flag: gate && observe(inner, divisor)",
            "flag: gate && observe(staged.flag, divisor)",
        ),
        (
            "flag: gate && observe(inner, divisor)",
            "flag: gate && missing(inner, divisor)",
        ),
        (
            "flag: gate && observe(inner, divisor)",
            "flag: gate && divisor",
        ),
        ("print(staged.tag)", "print(leaf(left, divisor).tag)"),
        ("print(staged.tag)", "print(staged.flag)"),
        (
            "let checked = 0 / divisor; print(62)",
            "let checked = missing(divisor); print(62)",
        ),
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
fn iteration_logical_record_fields_share_edges_across_nested_fields_and_arms() {
    let module = crate::frontend::parse_nuis_module(&source()).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut layouts = layouts;
    layouts.insert("Flags".into(), vec![("flag".into(), scalar_type("bool"))]);
    layouts.insert("Wrap".into(), vec![("flags".into(), scalar_type("Flags"))]);
    let scope = BTreeMap::from([("gate".into(), scalar_type("bool"))]);
    let gate = |count| {
        let mut value = NirExpr::Var("gate".into());
        for _ in 0..count {
            value = NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Var("gate".into())),
            };
        }
        NirExpr::StructLiteral {
            type_name: "Wrap".into(),
            type_args: vec![],
            fields: vec![(
                "flags".into(),
                NirExpr::StructLiteral {
                    type_name: "Flags".into(),
                    type_args: vec![],
                    fields: vec![("flag".into(), value)],
                },
            )],
        }
    };
    let arm = |count| {
        vec![NirStmt::Let {
            name: "joined".into(),
            ty: Some(scalar_type("Wrap")),
            value: gate(count),
        }]
    };
    let suffix = [NirStmt::Return(Some(NirExpr::Var("joined".into())))];
    for (right, expected) in [(16, true), (17, false)] {
        let join = NirStmt::If {
            condition: NirExpr::Var("gate".into()),
            then_body: arm(16),
            else_body: arm(right),
        };
        assert_eq!(
            prefixed_declaration(&join, &scope, &suffix, &catalog, &layouts).is_some(),
            expected,
            "shared 32/33 logical edges"
        );
        assert!(prefixed::declaration(&join, &scope, &suffix, &catalog, &layouts).is_none());
    }
    let value = gate(1);
    assert!(!conditional_values::prefix::computed_call_expression_roots(
        vec![(&value, 0, false)],
        &catalog
    ));
    assert!(
        conditional_values::prefix::computed_record_call_expression_roots(
            vec![(&value, 0, false)],
            &catalog,
            &layouts
        )
    );
}

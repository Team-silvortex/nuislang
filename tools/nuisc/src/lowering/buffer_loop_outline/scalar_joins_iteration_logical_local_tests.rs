use super::*;

pub(super) fn source() -> &'static str {
    "mod cpu Main {
        @noinline fn observe(value: bool, divisor: i64) -> bool {
            print(61); let checked = 0 / divisor; print(62); return value;
        }
        @noinline fn leaf(value: i64) -> i64 { print(value); return value; }
        fn owner(outer: bool, gate: bool, inner: bool, divisor: i64) -> i64 {
            if outer {
                let joined: i64 = if gate {
                    let before = 0 / divisor; print(60);
                    let ready: bool = gate && observe(inner, divisor);
                    let selected: i64 = if ready {
                        let local = leaf(1); local
                    } else { let local = leaf(2); local };
                    selected
                } else {
                    const ready: bool = inner || observe(gate, divisor);
                    print(63); let after = 0 / divisor;
                    let selected: i64 = if ready {
                        let local = leaf(3); local
                    } else { let local = leaf(4); local };
                    selected
                };
                print(joined); return joined;
            }
            return 0;
        }
        fn main() -> i64 { return 0; }
    }"
}

#[test]
fn iteration_logical_locals_keep_let_const_guards_and_dependent_branches() {
    for inferred in [false, true] {
        for reversed in [false, true] {
            let text = if inferred {
                source().replace("let ready: bool", "let ready")
            } else {
                source().into()
            };
            let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
            if reversed {
                module.functions.reverse();
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
}

#[test]
fn iteration_logical_locals_reject_hidden_roots_scope_type_and_print_leaks() {
    for (from, to) in [
        (
            "gate && observe(inner, divisor)",
            "(gate && observe(inner, divisor)) < inner",
        ),
        (
            "gate && observe(inner, divisor)",
            "observe(gate && inner, divisor)",
        ),
        (
            "gate && observe(inner, divisor)",
            "gate && observe(ready, divisor)",
        ),
        (
            "gate && observe(inner, divisor)",
            "gate && observe(selected == 1, divisor)",
        ),
        (
            "gate && observe(inner, divisor)",
            "gate && missing(inner, divisor)",
        ),
        ("gate && observe(inner, divisor)", "gate && divisor"),
        ("let ready: bool", "let ready: i64"),
        ("let ready: bool", "let gate: bool"),
        ("const ready: bool", "const ready: i64"),
        ("print(63)", "print(inner || gate)"),
        ("print(63)", "print(leaf(divisor))"),
        ("print(joined)", "print(ready)"),
        ("print(63)", "let alias = ready; let alias = ready"),
    ] {
        let text = source().replace(from, to);
        assert_ne!(text, source());
        let Ok(module) = crate::frontend::parse_nuis_module(&text) else {
            // Some lexical/type failures are rejected before NIR admission.
            continue;
        };
        let before = module.clone();
        let layouts = control_values::layouts(&module);
        assert!(
            !scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("owner"),
            "{from} -> {to}"
        );
        assert_eq!(module, before);
    }
}

#[test]
fn iteration_logical_locals_share_budget_with_record_fields_and_predicates() {
    let layouts = BTreeMap::from([("Flags".into(), vec![("flag".into(), scalar_type("bool"))])]);
    let scope = BTreeMap::from([("gate".into(), scalar_type("bool"))]);
    let catalog = BTreeMap::new();
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
    let arm = |count| {
        vec![
            NirStmt::Let {
                name: "ready".into(),
                ty: Some(scalar_type("bool")),
                value: logical(count),
            },
            NirStmt::Let {
                name: "joined".into(),
                ty: Some(scalar_type("Flags")),
                value: NirExpr::StructLiteral {
                    type_name: "Flags".into(),
                    type_args: vec![],
                    fields: vec![("flag".into(), logical(7))],
                },
            },
        ]
    };
    let suffix = [NirStmt::Return(Some(NirExpr::Var("joined".into())))];
    for (count, expected) in [(9, true), (10, false)] {
        let join = NirStmt::If {
            condition: logical(1),
            then_body: arm(8),
            else_body: arm(count),
        };
        assert_eq!(
            prefixed_declaration(&join, &scope, &suffix, &catalog, &layouts).is_some(),
            expected,
            "shared 32/33 logical edges across local/field/predicate roots"
        );
    }
}

#[test]
fn iteration_logical_locals_lower_terminal_bool_join_initializers() {
    let source = "mod cpu Main {
        @noinline fn observe(value: bool) -> bool { print(61); return value; }
        fn owner(gate: bool, inner: bool) -> bool {
            let joined: bool = if gate {
                print(60); gate && observe(inner)
            } else {
                print(62); inner || observe(gate)
            };
            print(63); return joined;
        }
        fn main() -> i64 { return 0; }
    }";
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(catalog.contains_key("owner"));
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
    assert_eq!(
        helpers
            .iter()
            .filter(|f| f.name.starts_with("__nuis_scalar_predicate"))
            .count(),
        2
    );
    module.functions.extend(helpers);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

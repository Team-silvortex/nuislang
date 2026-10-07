use super::*;
#[path = "conditional_returns_effect_typed_return_fixtures.rs"]
mod fixtures;
use fixtures::{source, MODES, TYPES};

#[test]
fn conditional_return_typed_handoff_outlines_exact_selected_regions_and_bool_exit_signals() {
    for ty in TYPES {
        for mode in MODES {
            for constant in [false, true] {
                let mut module =
                    crate::frontend::parse_nuis_module(&source(ty, mode, constant)).unwrap();
                let before = module.clone();
                let generated = outline_test(&mut module);
                assert!(!generated.is_empty(), "{ty} {mode} const={constant}");
                let signals = module
                    .structs
                    .iter()
                    .filter(|s| s.name.starts_with("__nuis_return_signal"))
                    .collect::<Vec<_>>();
                assert_eq!(
                    signals.is_empty(),
                    mode.ends_with("-no-exit"),
                    "{ty} {mode}"
                );
                for signal in signals {
                    assert_eq!(signal.fields[0].name, "exited");
                    assert_eq!(signal.fields[0].ty, scalar_type("bool"));
                    assert_eq!(signal.fields[1].name, "value");
                    assert_eq!(signal.fields[1].ty, scalar_type(ty));
                }
                for helper in module
                    .functions
                    .iter()
                    .filter(|f| generated.contains(&f.name))
                {
                    assert_eq!(helper.params[0].ty, scalar_type("bool"));
                    assert!(!format!("{:?}", helper.body).contains("Print("));
                }
                for name in ["choose", "observe", "finish"] {
                    assert_eq!(
                        module.functions.iter().find(|f| f.name == name),
                        before.functions.iter().find(|f| f.name == name)
                    );
                }
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let once = module.clone();
                assert!(outline_test(&mut module).is_empty());
                assert_eq!(module, once);
            }
        }
    }
}

#[test]
fn conditional_return_typed_handoff_does_not_admit_ordinary_tails_or_typed_print_captures() {
    for ty in TYPES {
        let text = format!(
            "mod cpu Main {{
            @noinline fn relay(value: {ty}) -> {ty} {{ return value; }}
            fn event(gate: bool, value: {ty}) -> {ty} {{
                print(99); if gate {{ return relay(value); }} return value;
            }} fn main() -> i64 {{ return 0; }}
        }}"
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, before);
        // A computed print cannot gain authority from the enclosing result kind.
        let text = source(ty, "complete", false).replace("print(80);", "print(finish(selected));");
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{ty}");
        assert_eq!(module, before);
    }
}

#[test]
fn conditional_return_typed_handoff_rejects_original_scope_type_effect_and_budget_violations() {
    for ty in TYPES {
        for mutation in [
            "return-kind",
            "missing",
            "reference",
            "optional",
            "generic",
            "effectful",
            "duplicate",
            "declared",
            "unreachable",
            "aggregate",
            "statements",
            "depth",
        ] {
            let mut module =
                crate::frontend::parse_nuis_module(&source(ty, "complete", false)).unwrap();
            if mutation == "effectful" {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "finish")
                    .unwrap()
                    .body
                    .insert(0, NirStmt::Print(NirExpr::Int(9)));
            } else if matches!(mutation, "reference" | "optional" | "generic") {
                let param = event(&mut module)
                    .params
                    .iter_mut()
                    .find(|p| p.name == "fallback")
                    .unwrap();
                match mutation {
                    "reference" => param.ty.is_ref = true,
                    "optional" => param.ty.is_optional = true,
                    _ => param.ty.generic_args.push(scalar_type("i64")),
                }
            } else {
                let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
                    panic!()
                };
                let insertion = then_body.len() - 1;
                match mutation {
                    "return-kind" | "missing" => {
                        *then_body.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Var(
                            if mutation == "missing" {
                                "not_defined"
                            } else {
                                "outer"
                            }
                            .into(),
                        )));
                    }
                    "duplicate" => then_body.insert(
                        insertion,
                        NirStmt::Let {
                            name: "fallback".into(),
                            ty: Some(scalar_type(ty)),
                            value: NirExpr::Var("selected".into()),
                        },
                    ),
                    "declared" => then_body.insert(
                        insertion,
                        NirStmt::Let {
                            name: "wrong_annotation".into(),
                            ty: Some(scalar_type("bool")),
                            value: NirExpr::Var("selected".into()),
                        },
                    ),
                    "unreachable" => then_body.push(NirStmt::Let {
                        name: "after_return".into(),
                        ty: Some(scalar_type(ty)),
                        value: NirExpr::Var("selected".into()),
                    }),
                    "aggregate" => then_body.insert(
                        insertion,
                        NirStmt::Let {
                            name: "unused_record".into(),
                            ty: Some(scalar_type("State")),
                            value: NirExpr::StructLiteral {
                                type_name: "State".into(),
                                type_args: vec![],
                                fields: [
                                    ("outer", "outer"),
                                    ("gate", "gate"),
                                    ("nested", "nested"),
                                    ("early", "early"),
                                    ("value", "value"),
                                    ("fallback", "fallback"),
                                    ("left", "left"),
                                    ("right", "right"),
                                    ("result", "selected"),
                                ]
                                .into_iter()
                                .map(|(name, value)| (name.into(), NirExpr::Var(value.into())))
                                .collect(),
                            },
                        },
                    ),
                    "statements" => {
                        for n in 0..33 {
                            then_body.insert(
                                insertion,
                                NirStmt::Let {
                                    name: format!("extra_{n}"),
                                    ty: Some(scalar_type("i64")),
                                    value: NirExpr::Int(0),
                                },
                            );
                        }
                    }
                    "depth" => {
                        let mut tail = vec![then_body.pop().unwrap()];
                        for _ in 0..65 {
                            tail = vec![NirStmt::If {
                                condition: NirExpr::Var("nested".into()),
                                then_body: tail,
                                else_body: vec![NirStmt::Return(Some(NirExpr::Var(
                                    "fallback".into(),
                                )))],
                            }];
                        }
                        then_body.extend(tail);
                    }
                    _ => unreachable!(),
                }
            }
            let before = module.clone();
            assert!(outline_test(&mut module).is_empty(), "{ty} {mutation}");
            assert_eq!(module, before, "atomic veto: {ty} {mutation}");
        }
    }
}

#[test]
fn conditional_return_typed_handoff_shares_original_and_staged_statement_ledgers() {
    for ty in TYPES {
        let mut first_veto = None;
        for count in 0..=32 {
            let mut module =
                crate::frontend::parse_nuis_module(&source(ty, "partial", false)).unwrap();
            let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
                panic!()
            };
            let insertion = then_body.len() - 1;
            for n in 0..count {
                then_body.insert(
                    insertion,
                    NirStmt::Let {
                        name: format!("budget_{n}"),
                        ty: Some(scalar_type("i64")),
                        value: NirExpr::Int(0),
                    },
                );
            }
            let before = module.clone();
            let outlined = !outline_test(&mut module).is_empty();
            if outlined {
                assert!(
                    first_veto.is_none(),
                    "a veto cannot buy a later larger budget"
                );
                crate::nir_verify::verify_nir_module(&module).unwrap();
            } else {
                first_veto.get_or_insert(count);
                assert_eq!(module, before, "{ty} count={count}");
            }
        }
        let boundary = first_veto.unwrap();
        assert!(boundary > 0 && boundary < 32, "{ty}: {boundary}");
    }
}

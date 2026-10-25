use super::record_return_tests::{attempt, edges, producer, reject};
use super::*;

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/joined_record_return.ns"
    ))
    .unwrap();
    if simplify {
        crate::optimize::simplify_nir_module(&mut module);
    }
    module
}

fn small(join: &str, value: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
            struct R {{ value:i64, tag:bool }}
            struct Other {{ value:i64, tag:bool }}
            @noinline fn calc(value:i64, div:i64)->i64 {{ return value+0/div; }}
            @noinline fn returned_record(flag:bool,value:i64,div:i64)->R {{
                print(86); {join} print(89);
                return R {{ value:{value}, tag:flag && (1/div != 0) }};
            }}
        }}"
    ))
    .unwrap()
}

#[test]
fn joined_record_return_exports_only_the_typed_result_and_is_order_independent() {
    for simplify in [false, true] {
        for reversed in [false, true] {
            let mut module = module(simplify);
            if reversed {
                module.functions.reverse();
            }
            let before = producer(&mut module).clone();
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let f = producer(&mut module);
            assert_eq!(f.params, before.params);
            assert_eq!(f.return_type, before.return_type);
            assert!(matches!(f.body[0], NirStmt::Print(NirExpr::Int(86))));
            assert!(matches!(&f.body[1], NirStmt::Let { name, ty:Some(ty), .. }
                if name == "joined" && ty == &scalar_type("Leaf")));
            assert!(matches!(f.body[2], NirStmt::Print(NirExpr::Int(89))));
            assert_eq!(edges(&f.body), 0);
            let before = module.clone();
            assert!(!attempt(&mut module, None));
            assert_eq!(module, before);
        }
    }
}

#[test]
fn joined_record_return_reuses_scalar_record_and_bool_join_authority() {
    for (join, value) in [
        ("let joined:i64=if flag { const local=calc(value,div); local } else { const local=calc(0-value,div); 0-local };", "joined"),
        ("let first:i64=if flag { value } else { 0-value }; let joined:i64=if flag { first } else { 0-first };", "joined"),
        ("let joined:R=if flag { R{value:value,tag:flag && (1/div!=0)} } else { R{tag:flag || (1/div!=0),value:0-value} };", "joined.value"),
        ("let joined:bool=if flag { flag && (1/div!=0) } else { flag || (1/div!=0) };", "value"),
    ] {
        let mut module = small(join, value);
        if join.starts_with("let joined:bool") {
            let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) = producer(&mut module).body.last_mut().unwrap() else { panic!() };
            let NirExpr::Binary { lhs, .. } = &mut fields[1].1 else { panic!() };
            *lhs = Box::new(NirExpr::Var("joined".into()));
        }
        assert!(attempt(&mut module, None), "{join}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert_eq!(edges(&producer(&mut module).body), 0);
    }
}

#[test]
fn joined_record_return_revokes_all_join_dependencies_before_mutating() {
    for name in ["join_decide", "join_unused", "return_prefix"] {
        let mut module = module(true);
        let before = module.clone();
        assert!(!attempt(&mut module, Some(name)));
        assert_eq!(module, before);
        module
            .functions
            .iter_mut()
            .find(|f| f.name == name)
            .unwrap()
            .body
            .insert(0, NirStmt::Print(NirExpr::Int(99)));
        reject(&mut module);
    }
}

#[test]
fn joined_record_return_rejects_partial_nominal_shadowing_effect_and_scope_leaks() {
    for mutation in 0..12 {
        let mut module = if mutation == 1 {
            small("let joined:R=if flag { R{value:value,tag:flag} } else { R{value:value,tag:flag} };", "joined.value")
        } else {
            small(
                "let joined:i64=if flag { value } else { 0-value };",
                "joined",
            )
        };
        let f = producer(&mut module);
        if mutation == 5 {
            let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) =
                f.body.last_mut().unwrap()
            else {
                panic!()
            };
            fields[0].1 = NirExpr::Var("local".into());
        }
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &mut f.body[1]
        else {
            panic!()
        };
        match mutation {
            0 => {
                let NirStmt::Let { value, .. } = &mut else_body[0] else {
                    panic!()
                };
                *value = NirExpr::Bool(true);
            }
            1 => {
                let NirStmt::Let {
                    value: NirExpr::StructLiteral { type_name, .. },
                    ..
                } = &mut else_body[0]
                else {
                    panic!()
                };
                *type_name = "Other".into();
            }
            2 => else_body.clear(),
            3 => {
                let NirStmt::Let { name, .. } = &mut else_body[0] else {
                    panic!()
                };
                *name = "other".into();
            }
            4 => then_body.insert(0, NirStmt::Print(NirExpr::Int(99))),
            5 | 6 => then_body.insert(
                0,
                NirStmt::Let {
                    name: if mutation == 5 { "local" } else { "value" }.into(),
                    ty: Some(scalar_type("i64")),
                    value: NirExpr::Int(1),
                },
            ),
            7 => {
                for stmt in then_body.iter_mut().chain(else_body.iter_mut()) {
                    let NirStmt::Let { name, .. } = stmt else {
                        panic!()
                    };
                    *name = "value".into();
                }
            }
            8 => *condition = NirExpr::Var("value".into()),
            9 => {
                let NirStmt::Let { value, .. } = &mut then_body[0] else {
                    panic!()
                };
                *value = NirExpr::Call {
                    callee: "calc".into(),
                    args: vec![
                        NirExpr::Int(1),
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs: Box::new(NirExpr::Bool(true)),
                            rhs: Box::new(NirExpr::Bool(false)),
                        },
                    ],
                };
            }
            10 => then_body.insert(
                0,
                NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body: vec![],
                },
            ),
            11 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut else_body[0] else {
                    panic!()
                };
                ty.is_ref = true;
            }
            _ => unreachable!(),
        }
        reject(&mut module);
    }
}

#[test]
fn joined_record_return_shares_logical_work_across_join_and_return_roots() {
    fn chain(count: usize) -> NirExpr {
        (0..count).fold(NirExpr::Var("flag".into()), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Bool(true)),
        })
    }
    for (count, accepted) in [(31, true), (32, false)] {
        let mut module = small("let joined:bool=if flag { flag } else { false };", "value");
        let NirStmt::If { then_body, .. } = &mut producer(&mut module).body[1] else {
            panic!()
        };
        let NirStmt::Let { value, .. } = then_body.last_mut().unwrap() else {
            panic!()
        };
        *value = chain(count);
        // The unrelated lazy return field consumes the last logical edge.
        assert_eq!(attempt(&mut module, None), accepted);
        if !accepted {
            reject(&mut module);
        }
    }
}

use super::record_return_tests::{attempt, edges, producer, reject};
use super::*;

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/local_record_return.ns"
    ))
    .unwrap();
    if simplify {
        crate::optimize::simplify_nir_module(&mut module);
    }
    module
}

fn small(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
            struct R {{ value:i64, tag:bool }}
            struct Other {{ value:i64, tag:bool }}
            @noinline fn make(value:i64,flag:bool,div:i64)->R {{ return R{{value:value,tag:flag&&(1/div!=0)}}; }}
            @noinline fn returned_record(input:R,value:i64,flag:bool,div:i64)->R {{ {body} }}
        }}"
    ))
    .unwrap()
}

const LOCAL: &str = "let local:R=R{value:value,tag:flag&&(1/div!=0)}; print(83); return local;";

#[test]
fn local_record_return_certifies_original_initializers_without_a_literal_return() {
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
            assert!(
                matches!(f.body.last(), Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "completed")
            );
            assert!(matches!(
                f.body[f.body.len() - 2],
                NirStmt::Print(NirExpr::Int(84))
            ));
            assert_eq!(edges(&f.body), 0);
            let NirStmt::If { then_body, .. } = &f.body[3] else {
                panic!()
            };
            assert!(
                matches!(then_body.last(), Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "completed")
            );
            let before = module.clone();
            assert!(!attempt(&mut module, None));
            assert_eq!(module, before);
        }
    }
}

#[test]
fn local_record_return_supports_fresh_let_const_inference_and_inherited_scopes() {
    for body in [
        LOCAL.to_owned(),
        LOCAL.replace("let local:R", "let local"),
        LOCAL.replace("let local:R", "const local:R"),
        "let local:R=R{value:value,tag:flag&&(1/div!=0)}; if flag { print(81); return local; } print(82); return local;".into(),
        "if flag { const local:R=R{value:value,tag:flag&&(1/div!=0)}; print(81); return local; } const local:R=R{tag:flag||(1/div!=0),value:0-value}; print(82); return local;".into(),
    ] {
        let mut module = small(&body);
        assert!(attempt(&mut module, None), "{body}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert_eq!(edges(&producer(&mut module).body), 0);
    }
}

#[test]
fn local_record_return_does_not_borrow_unreturned_origins_or_general_alias_authority() {
    for body in [
        "return input;",
        "let local:R=R{value:value,tag:flag&&(1/div!=0)}; return input;",
        "let local:R=R{value:value,tag:flag&&(1/div!=0)}; let alias:R=input; return alias;",
        "let local:R=make(value,flag,div); return local;",
        "let local:R=R{value:value,tag:flag}; return local;",
        "let local:R=if flag { R{value:value,tag:flag&&(1/div!=0)} } else { input }; return local;",
    ] {
        reject(&mut small(body));
    }
    for mutation in 0..9 {
        let mut module = small(LOCAL);
        let f = producer(&mut module);
        match mutation {
            0 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut f.body[0] else {
                    panic!()
                };
                *ty = scalar_type("Other");
            }
            1 => {
                let NirStmt::Let {
                    value: NirExpr::StructLiteral { type_name, .. },
                    ..
                } = &mut f.body[0]
                else {
                    panic!()
                };
                *type_name = "Other".into();
            }
            2 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut f.body[0] else {
                    panic!()
                };
                ty.is_ref = true;
            }
            3 => {
                let NirStmt::Let { name, .. } = &mut f.body[0] else {
                    panic!()
                };
                *name = "input".into();
            }
            4 => {
                f.body.insert(1, f.body[0].clone());
            }
            5 => {
                f.body.insert(
                    1,
                    NirStmt::Let {
                        name: "local".into(),
                        ty: None,
                        value: NirExpr::Var("input".into()),
                    },
                );
            }
            6 => {
                let stmt = f.body.remove(0);
                f.body.insert(
                    0,
                    NirStmt::If {
                        condition: NirExpr::Var("flag".into()),
                        then_body: vec![stmt],
                        else_body: vec![],
                    },
                );
            }
            7 => {
                f.body.insert(
                    0,
                    NirStmt::While {
                        condition: NirExpr::Bool(false),
                        body: vec![],
                    },
                );
            }
            8 => {
                let NirStmt::Let {
                    value: NirExpr::StructLiteral { fields, .. },
                    ..
                } = &mut f.body[0]
                else {
                    panic!()
                };
                fields[1].1 = NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(fields[1].1.clone()),
                    rhs: Box::new(NirExpr::Bool(true)),
                };
            }
            _ => unreachable!(),
        }
        reject(&mut module);
    }
}

#[test]
fn local_record_return_revokes_initializer_and_post_construction_dependencies_atomically() {
    for name in [
        "field_head",
        "field_tail",
        "field_spare",
        "outer_word",
        "outer_unused",
        "join_decide",
        "join_unused",
        "return_prefix",
        "local_after",
    ] {
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
fn local_record_return_keeps_whole_function_expression_and_statement_budgets() {
    for (count, accepted) in [(32, true), (33, false)] {
        let mut module = small(LOCAL);
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut producer(&mut module).body[0]
        else {
            panic!()
        };
        fields[1].1 = (0..count).fold(NirExpr::Var("flag".into()), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Bool(true)),
        });
        assert_eq!(attempt(&mut module, None), accepted);
        if !accepted {
            reject(&mut module);
        }
    }
    for (count, accepted) in [(29, true), (30, false)] {
        let mut module = small(LOCAL);
        for index in 0..count {
            producer(&mut module).body.insert(
                0,
                NirStmt::Let {
                    name: format!("padding_{index}"),
                    ty: None,
                    value: NirExpr::Int(1),
                },
            );
        }
        assert_eq!(attempt(&mut module, None), accepted);
        if !accepted {
            reject(&mut module);
        }
    }
}

#[test]
fn local_record_return_executes_unused_lazy_fields_before_post_construction_work() {
    for (op, gate, div, post, expected) in [
        ("&&", false, 0, 1, Some(7)),
        ("||", true, 0, -1, Some(7)),
        ("&&", true, 1, 1, Some(7)),
        ("||", false, 1, 1, Some(7)),
        ("&&", true, 0, 1, None),
        ("||", false, 0, 1, None),
        ("&&", false, 0, 0, None),
        ("||", true, 0, 0, None),
    ] {
        let source=format!("mod cpu Main {{ struct R {{ value:i64,unused:bool }} @noinline fn checked(div:i64)->bool{{return 1/div!=0;}} @noinline fn returned_record(value:i64,gate:bool,div:i64,post:i64)->R{{print(81);let completed:R=R{{value:value,unused:gate {op} checked(div)}};print(83);print(1/post);print(84);return completed;}} fn main()->i64{{return returned_record(7,{gate},{div},{post}).value;}} }}");
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        );
        if let Some(expected) = expected {
            let trace = result.unwrap();
            let main = compiled
                .yir
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap();
            assert_eq!(
                trace.values[&main.result.as_ref().unwrap().node],
                yir_core::Value::Int(expected)
            );
        } else {
            assert!(result.unwrap_err().contains("zero"));
        }
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}

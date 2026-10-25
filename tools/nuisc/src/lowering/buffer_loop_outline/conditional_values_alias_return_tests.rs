use super::record_return_tests::{attempt, edges, producer, reject};
use super::*;

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/alias_record_return.ns"
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
            @noinline fn identity(value:R)->R {{ return value; }}
            @noinline fn returned_record(input:R,value:i64,flag:bool,div:i64)->R {{ {body} }}
        }}"
    ))
    .unwrap()
}

const ROOT: &str = "let completed:R=R{value:value,tag:flag&&(1/div!=0)};";
const CHAIN: &str =
    "let alias:R=completed; print(83); const final_record:R=alias; print(84); return final_record;";

fn chain_module() -> NirModule {
    small(&format!("{ROOT}{CHAIN}"))
}

#[test]
fn alias_record_return_preserves_original_copies_and_adds_no_reconstruction_helpers() {
    for simplify in [false, true] {
        let mut baseline = crate::frontend::parse_nuis_module(include_str!(
            "../../../tests/native_application_bridge/local_record_return.ns"
        ))
        .unwrap();
        if simplify {
            crate::optimize::simplify_nir_module(&mut baseline);
        }
        assert!(attempt(&mut baseline, None));
        for reversed in [false, true] {
            let mut module = module(simplify);
            if reversed {
                module.functions.reverse();
            }
            let before = producer(&mut module).clone();
            assert!(attempt(&mut module, None));
            assert_eq!(module.functions.len(), baseline.functions.len());
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let f = producer(&mut module);
            assert_eq!(f.params, before.params);
            assert_eq!(f.return_type, before.return_type);
            assert_eq!(edges(&f.body), 0);
            assert!(
                matches!(f.body.last(),Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name=="final_record")
            );
            if !simplify {
                assert!(f.body.iter().any(|s| matches!(s,NirStmt::Let { name, ty:Some(ty), value:NirExpr::Var(origin) } if name=="final_record" && origin=="alias" && ty==&scalar_type("Record"))));
                let NirStmt::If { then_body, .. } = &f.body[3] else {
                    panic!()
                };
                assert!(then_body.iter().any(|s| matches!(s,NirStmt::Let { name, value:NirExpr::Var(origin), .. } if name=="alias" && origin=="completed")));
                assert!(then_body.iter().any(|s| matches!(s,NirStmt::Const { name, value:NirExpr::Var(origin), .. } if name=="final_record" && origin=="alias")));
            }
            let before = module.clone();
            assert!(!attempt(&mut module, None));
            assert_eq!(module, before);
        }
    }
}

#[test]
fn alias_record_return_propagates_only_fresh_exact_let_const_values_through_lexical_scopes() {
    for tail in [
        CHAIN,
        "let alias=completed; const final_record:R=alias; return final_record;",
        "const alias:R=completed; let final_record=alias; return final_record;",
        "let first:R=completed; let second:R=completed; let third=second; return third;",
        "let alias=completed; if flag { const child:R=alias; print(81); return child; } const child:R=alias; print(82); return child;",
    ] {
        let mut module=small(&format!("{ROOT}{tail}"));
        assert!(attempt(&mut module,None),"{tail}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert_eq!(edges(&producer(&mut module).body),0);
    }
}

#[test]
fn alias_record_return_rejects_unrelated_origins_repacking_join_only_results_and_stale_names() {
    for body in [
        "let alias:R=input; let final_record:R=alias; return final_record;",
        "let completed:R=make(value,flag,div); let alias:R=completed; return alias;",
        "let completed:R=R{value:value,tag:flag}; let alias:R=completed; return alias;",
        "let completed:R=if flag { R{value:value,tag:flag&&(1/div!=0)} } else { input }; let alias:R=completed; return alias;",
    ] { reject(&mut small(body)); }
    for tail in [
        "let alias:R=input; return alias;",
        "let alias:R=identity(completed); return alias;",
        "let alias:R=R{value:completed.value,tag:completed.tag}; return alias;",
        "let alias:R=completed; return input;",
    ] {
        reject(&mut small(&format!("{ROOT}{tail}")));
    }
    for mutation in 0..9 {
        let mut module = chain_module();
        let f = producer(&mut module);
        match mutation {
            0 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut f.body[1] else {
                    panic!()
                };
                *ty = scalar_type("Other");
            }
            1 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut f.body[1] else {
                    panic!()
                };
                ty.is_ref = true;
            }
            2 => {
                let NirStmt::Let { value, .. } = &mut f.body[1] else {
                    panic!()
                };
                *value = NirExpr::Var("final_record".into());
            }
            3 => {
                let NirStmt::Let { value, .. } = &mut f.body[1] else {
                    panic!()
                };
                *value = NirExpr::Var("alias".into());
            }
            4 | 5 => f.body.insert(
                2,
                NirStmt::Let {
                    name: if mutation == 4 { "alias" } else { "completed" }.into(),
                    ty: Some(scalar_type("R")),
                    value: NirExpr::Var("input".into()),
                },
            ),
            6 => {
                let stmt = f.body.remove(1);
                f.body.insert(
                    1,
                    NirStmt::If {
                        condition: NirExpr::Var("flag".into()),
                        then_body: vec![stmt],
                        else_body: vec![],
                    },
                );
            }
            7 => {
                let NirStmt::Const { name, .. } = &mut f.body[3] else {
                    panic!()
                };
                *name = "input".into();
            }
            8 => f.body.insert(
                1,
                NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body: vec![],
                },
            ),
            _ => unreachable!(),
        }
        reject(&mut module);
    }
}

#[test]
fn alias_record_return_revokes_original_initializer_and_intervening_dependencies_atomically() {
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
fn alias_record_return_chains_consume_the_existing_whole_function_budgets() {
    for (count, accepted) in [(27, true), (28, false)] {
        let mut body = ROOT.to_owned();
        let mut previous = "completed".to_owned();
        for i in 0..count {
            let name = format!("copy_{i}");
            body.push_str(&format!("let {name}:R={previous};"));
            previous = name;
        }
        body.push_str(&format!(
            "print(83);const after:i64=1/div;print(84);return {previous};"
        ));
        let mut module = small(&body);
        let before = module.clone();
        assert_eq!(attempt(&mut module, None), accepted);
        if !accepted {
            assert_eq!(module, before);
        }
    }
    for (count, accepted) in [(32, true), (33, false)] {
        let mut module = chain_module();
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
}

#[test]
fn alias_record_return_executes_unsimplified_copy_chains_without_replaying_fields() {
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
        let source=format!("mod cpu Main {{ struct R {{ value:i64,unused:bool }} @noinline fn checked(div:i64)->bool{{return 1/div!=0;}} @noinline fn returned_record(value:i64,gate:bool,div:i64,post:i64)->R{{print(81);let completed:R=R{{value:value,unused:gate {op} checked(div)}};let alias=completed;print(83);const after:i64=1/post;const final_record:R=alias;print(84);return final_record;}} fn main()->i64{{return returned_record(7,{gate},{div},{post}).value;}} }}");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert!(attempt(&mut module, None));
        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        );
        if let Some(expected) = expected {
            let trace = result.unwrap();
            let main = yir
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
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}

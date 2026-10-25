use super::record_return_tests::{attempt, edges, producer, reject};
use super::*;

const LAZY: &str = "R{value:value,tag:flag&&(1/div!=0)}";
const OTHER: &str = "R{value:0-value,tag:flag||(1/div!=0)}";

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/origin_record_join.ns"
    ))
    .unwrap();
    if simplify {
        crate::optimize::simplify_nir_module(&mut module);
    }
    module
}

fn small(yes: &str, no: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
            struct R {{value:i64,tag:bool}}
            struct Other {{value:i64,tag:bool}}
            @noinline fn make(value:i64,flag:bool,div:i64)->R {{return {LAZY};}}
            @noinline fn returned_record(input:R,value:i64,flag:bool,div:i64)->R {{
                print(81);let merged:R=if flag {{{yes}}}else{{{no}}};
                print(83);const after:i64=1/div;print(84);
                let final_record=merged;return final_record;
            }}
        }}"
    ))
    .unwrap()
}

#[test]
fn origin_record_join_exports_only_the_common_result_and_keeps_original_effect_order() {
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
            assert!(matches!(f.body[2], NirStmt::Print(NirExpr::Int(89))));
            assert!(
                matches!(&f.body[4],NirStmt::Let { name,ty:Some(ty),.. } if name=="merged" && ty==&scalar_type("Record"))
            );
            assert!(matches!(f.body[5], NirStmt::Print(NirExpr::Int(83))));
            assert!(f.body.iter().all(|s| !matches!(s,NirStmt::Let { name,.. }|NirStmt::Const { name,.. } if matches!(name.as_str(),"alias"|"completed"))));
            assert_eq!(edges(&f.body), 0);
            let before = module.clone();
            assert!(!attempt(&mut module, None));
            assert_eq!(module, before);
        }
    }
}

#[test]
fn origin_record_join_accepts_original_literals_local_copies_nested_joins_and_inherited_roots() {
    let left = format!("const completed:R={LAZY};let alias=completed;alias");
    let right = format!("let completed:R={OTHER};const alias:R=completed;alias");
    let nested = format!("let child:R=if flag {{{left}}}else{{{right}}};const alias:R=child;alias");
    let discarded = format!("if flag {{let ignored:R={LAZY};}} {left}");
    for (yes, no) in [
        (LAZY, OTHER),
        (left.as_str(), right.as_str()),
        (nested.as_str(), right.as_str()),
        (discarded.as_str(), right.as_str()),
    ] {
        let mut module = small(yes, no);
        assert!(attempt(&mut module, None), "{yes} / {no}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
    let mut module = small("input", "input");
    let f = producer(&mut module);
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &mut f.body[1]
    else {
        panic!()
    };
    for body in [then_body, else_body] {
        let NirStmt::Let { value, .. } = body.last_mut().unwrap() else {
            panic!()
        };
        *value = NirExpr::Var("root".into());
    }
    let mut root = small(LAZY, OTHER);
    let NirStmt::If { then_body, .. } = &mut producer(&mut root).body[1] else {
        panic!()
    };
    let NirStmt::Let { value, .. } = &then_body[0] else {
        panic!()
    };
    f.body.insert(
        0,
        NirStmt::Let {
            name: "root".into(),
            ty: Some(scalar_type("R")),
            value: value.clone(),
        },
    );
    assert!(attempt(&mut module, None));
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn origin_record_join_requires_both_results_and_rejects_unused_roots_and_private_state() {
    let unused = format!("const completed:R={LAZY}; input");
    let discarded = format!("if flag {{let ignored:R={LAZY};}} input");
    let repack = format!("const completed:R={LAZY};R{{value:completed.value,tag:completed.tag}}");
    let partial_nested = format!("let child:R=if flag {{{LAZY}}}else{{input}};child");
    for bad in [
        "input",
        "make(value,flag,div)",
        "R{value:value,tag:flag}",
        unused.as_str(),
        discarded.as_str(),
        repack.as_str(),
        partial_nested.as_str(),
    ] {
        for reversed in [false, true] {
            let mut module = if reversed {
                small(bad, LAZY)
            } else {
                small(LAZY, bad)
            };
            reject(&mut module);
        }
    }
    for mutation in 0..8 {
        let mut module = small(LAZY, OTHER);
        let f = producer(&mut module);
        if mutation == 6 {
            let NirStmt::Let { value, .. } = &mut f.body[5] else {
                panic!()
            };
            *value = NirExpr::Var("private".into());
        }
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut f.body[1]
        else {
            panic!()
        };
        match mutation {
            0 => else_body.clear(),
            1 => {
                let NirStmt::Let { name, .. } = &mut else_body[0] else {
                    panic!()
                };
                *name = "other".into();
            }
            2 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut else_body[0] else {
                    panic!()
                };
                *ty = scalar_type("Other");
            }
            3 => {
                let NirStmt::Let { ty: Some(ty), .. } = &mut else_body[0] else {
                    panic!()
                };
                ty.is_ref = true;
            }
            4 => then_body.insert(0, NirStmt::Print(NirExpr::Int(99))),
            5 => then_body.insert(
                0,
                NirStmt::Let {
                    name: "value".into(),
                    ty: Some(scalar_type("i64")),
                    value: NirExpr::Int(1),
                },
            ),
            6 => {}
            7 => then_body.insert(
                0,
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
fn origin_record_join_revokes_selected_field_prefix_and_post_dependencies_before_allocation() {
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
fn origin_record_join_shares_expression_edges_and_nested_depth_before_provenance_walks() {
    for (left, accepted) in [(16, true), (17, false)] {
        let mut module = small(LAZY, OTHER);
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut producer(&mut module).body[1]
        else {
            panic!()
        };
        for (body, count) in [(then_body, left), (else_body, 16)] {
            let NirStmt::Let {
                value: NirExpr::StructLiteral { fields, .. },
                ..
            } = &mut body[0]
            else {
                panic!()
            };
            fields[1].1 = (0..count).fold(NirExpr::Var("flag".into()), |lhs, _| NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(NirExpr::Bool(true)),
            });
        }
        assert_eq!(attempt(&mut module, None), accepted);
        if !accepted {
            reject(&mut module);
        }
    }
    for (depth, accepted) in [(8, true), (9, false)] {
        let mut arm = LAZY.to_owned();
        for level in 1..depth {
            arm = format!("let child_{level}:R=if flag {{{arm}}}else{{{OTHER}}};child_{level}");
        }
        let mut module = small(&arm, OTHER);
        assert_eq!(attempt(&mut module, None), accepted);
        if !accepted {
            reject(&mut module);
        }
    }
}

#[test]
fn origin_record_join_executes_raw_two_arm_provenance_without_eager_unselected_fields() {
    for (op, outer, gate, selector, div, post, expected) in [
        ("&&", true, false, 1, 0, 1, Some(7)),
        ("&&", true, true, 1, 1, 1, Some(7)),
        ("||", false, true, 1, 0, 1, Some(-7)),
        ("||", false, false, 1, 1, 1, Some(-7)),
        ("&&", true, true, 1, 0, 1, None),
        ("||", false, false, 1, 0, 1, None),
        ("&&", true, false, 0, 0, 0, None),
        ("||", false, true, 1, 0, 0, None),
    ] {
        let source=format!("mod cpu Main {{struct R{{value:i64,unused:bool}} @noinline fn checked(div:i64)->bool{{return 1/div!=0;}} @noinline fn choose(outer:bool,div:i64)->bool{{return outer==(1/div!=0);}} @noinline fn returned_record(value:i64,outer:bool,gate:bool,selector:i64,div:i64,post:i64)->R{{print(81);let merged:R=if choose(outer,selector){{const completed:R=R{{value:value,unused:gate {op} checked(div)}};let alias=completed;alias}}else{{let completed:R=R{{unused:gate {op} checked(div),value:0-value}};const alias:R=completed;alias}};print(83);const after:i64=1/post;let final_record=merged;print(84);return final_record;}}fn main()->i64{{return returned_record(7,{outer},{gate},{selector},{div},{post}).value;}}}}");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert!(attempt(&mut module, None));
        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        );
        if let Some(expected) = expected {
            let trace = result.unwrap();
            let calls = |name: &str| {
                trace
                    .events
                    .iter()
                    .filter(|event| {
                        event.contains("cpu.call") && event.contains(&format!("] {name}("))
                    })
                    .count()
            };
            assert_eq!(calls("choose"), 1);
            assert_eq!(
                calls("checked"),
                usize::from(if op == "&&" { gate } else { !gate })
            );
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

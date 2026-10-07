use super::*;

#[path = "conditional_returns_effect_control_fixtures.rs"]
mod fixtures;
use fixtures::{expected, source, Input, INPUTS};
#[path = "conditional_returns_effect_control_budget_tests.rs"]
mod budget;
#[path = "conditional_returns_effect_control_native_tests.rs"]
mod native;

#[test]
fn conditional_return_control_regions_preserve_selected_branches_order_and_exits() {
    let mut cases = 0;
    for kind in ["atom", "computed", "logical"] {
        for nested in [false, true] {
            for mode in ["return", "partial", "zero"] {
                for shape in ["then", "else", "both"] {
                    for (index, input) in INPUTS.into_iter().enumerate() {
                        let text = source(kind, nested, mode, shape, index % 3, input);
                        let (result, prints, calls) =
                            expected(kind, nested, mode, shape, index % 3, input);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 432);
}

#[test]
fn conditional_return_control_regions_keep_condition_work_unused_checks_and_snapshot_order() {
    for gate in [false, true] {
        let input = Input {
            outer: true,
            gate,
            nested: true,
            value: 2,
            right: 2,
            tail: 2,
            early: false,
        };
        let text = source("computed", true, "return", "then", 1, input);
        let (result, prints, calls) = expected("computed", true, "return", "then", 1, input);
        execute(&text, result, &prints, calls);
        for reversed in [false, true] {
            let trace = events(&text, reversed);
            let positions = trace
                .iter()
                .enumerate()
                .filter(|(_, e)| e.contains("cpu.call_bool") && e.contains("] helper("))
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let prints = trace
                .iter()
                .enumerate()
                .filter(|(_, e)| e.contains("cpu.guard_print ") && e.contains("if true then print"))
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            assert_eq!(positions.len(), usize::from(gate) + 1);
            assert!(
                prints[0] < positions[positions.len() - 1]
                    && positions[positions.len() - 1] < prints[1],
                "{trace:?}"
            );
            assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 1);
        }
    }
    let base = source("computed", false, "return", "then", 1, INPUTS[2]);
    // The condition's selected check alone may support a real pure return.
    let only_condition = base
        .replace("let ready = gate && helper(produce(value));", "")
        .replace("return ready;", "return true;")
        .replace(
            "let local = observe(right); print(local);  const copy: i64 = local; print(copy);",
            "print(44);",
        )
        .replace("let local = observe(tail); print(local);", "print(55);");
    execute(&only_condition, Some(11), &[99, 88, 44, 89, 11], 1);
    let empty_condition = only_condition
        .replace("print(44);", "")
        .replace("print(55);", "");
    execute(&empty_condition, Some(11), &[99, 88, 89, 11], 1);
    execute(
        &empty_condition.replace(
            "event(true, true, true, 2, 2, 2, false)",
            "event(true, true, true, 0, 2, 2, false)",
        ),
        None,
        &[99, 88],
        0,
    );
    let unused = base.replace(
        "let local = observe(right);",
        "let ignored = observe(tail); let local = observe(right);",
    );
    execute(&unused, Some(11), &[99, 88, 50, 50, 89, 11], 2);
    execute(
        &unused.replace(
            "event(true, true, true, 2, 2, 2, false)",
            "event(true, true, true, 2, 2, 0, false)",
        ),
        None,
        &[99, 88],
        2,
    );
    let entry = base.replace("if outer {", "if helper(produce(tail)) && outer {");
    execute(&entry, Some(11), &[99, 88, 50, 50, 89, 11], 3);
    let chained = base.replace(
        "print(89);",
        "let after = observe(tail); print(after); print(89);",
    );
    execute(&chained, Some(11), &[99, 88, 50, 50, 50, 89, 11], 2);
}

#[test]
fn conditional_return_control_regions_validate_original_child_scopes_and_veto_atomically() {
    for mutation in [
        "escape",
        "sibling",
        "shadow",
        "condition-private",
        "condition-kind",
        "condition-leaf",
        "internal-return",
        "mutation",
        "resource",
        "borrow",
        "effectful-callee",
        "no-return",
        "print-only",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&source(
            "computed", false, "return", "both", 1, INPUTS[2],
        ))
        .unwrap();
        if mutation == "effectful-callee" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "helper")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1)));
        } else {
            let parent = event(&mut module);
            if mutation == "borrow" {
                parent.params[3].ty.is_ref = true;
            }
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &mut parent.body[2]
            else {
                panic!()
            };
            if mutation == "no-return" {
                then_body.pop();
                else_body.pop();
            } else {
                let NirStmt::If {
                    condition,
                    then_body: yes,
                    else_body: no,
                } = &mut else_body[2]
                else {
                    panic!()
                };
                match mutation {
                    "escape" => {
                        *else_body.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Binary {
                            op: NirBinaryOp::Gt,
                            lhs: Box::new(NirExpr::Var("local".into())),
                            rhs: Box::new(NirExpr::Int(0)),
                        }))
                    }
                    "sibling" => {
                        no[0] = NirStmt::Let {
                            name: "other".into(),
                            ty: None,
                            value: NirExpr::Var("copy".into()),
                        }
                    }
                    "shadow" => {
                        let NirStmt::Let { name, .. } = &mut yes[0] else {
                            panic!()
                        };
                        *name = "ready".into();
                    }
                    "condition-private" => {
                        *condition = NirExpr::Var("__nuis_effect_condition_0".into())
                    }
                    "condition-kind" => *condition = NirExpr::Int(1),
                    "condition-leaf" => {
                        *condition = NirExpr::Binary {
                            op: NirBinaryOp::Eq,
                            lhs: Box::new(NirExpr::Binary {
                                op: NirBinaryOp::And,
                                lhs: Box::new(NirExpr::Bool(true)),
                                rhs: Box::new(NirExpr::Bool(false)),
                            }),
                            rhs: Box::new(NirExpr::Bool(true)),
                        }
                    }
                    "internal-return" => yes.push(NirStmt::Return(Some(NirExpr::Int(0)))),
                    "mutation" => {
                        yes[0] = NirStmt::Let {
                            name: "right".into(),
                            ty: None,
                            value: NirExpr::Int(1),
                        }
                    }
                    "resource" => {
                        let NirStmt::Let { value, .. } = &mut yes[0] else {
                            panic!()
                        };
                        *value = NirExpr::Call {
                            callee: "produce".into(),
                            args: vec![NirExpr::Var("right".into())],
                        };
                    }
                    "print-only" => {
                        for arm in [then_body, else_body] {
                            arm.remove(0);
                            *arm.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Bool(true)));
                            let NirStmt::If {
                                condition,
                                then_body,
                                else_body,
                            } = &mut arm[1]
                            else {
                                panic!()
                            };
                            *condition = NirExpr::Var("gate".into());
                            *then_body = vec![NirStmt::Print(NirExpr::Binary {
                                op: NirBinaryOp::Div,
                                lhs: Box::new(NirExpr::Int(100)),
                                rhs: Box::new(NirExpr::Var("right".into())),
                            })];
                            *else_body = vec![];
                        }
                    }
                    "borrow" => {}
                    _ => unreachable!(),
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_control_regions_keep_pure_helpers_saved_paths_and_idempotence() {
    let text = source("logical", true, "return", "both", 2, INPUTS[2]);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let before = module.clone();
    let generated = outline_test(&mut module);
    assert!(!generated.is_empty());
    assert!(format!("{:?}", event(&mut module).body).contains("__nuis_effect_path"));
    for function in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert!(!format!("{:?}", function.body).contains("Print("));
        assert!(function.params.iter().all(|p| scalar(&p.ty)));
    }
    for name in ["produce", "helper", "observe"] {
        assert_eq!(
            module.functions.iter().find(|f| f.name == name),
            before.functions.iter().find(|f| f.name == name)
        );
    }
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let once = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, once);
    let text = text
        .replace("ready", "__nuis_effect_condition_0")
        .replace("nested", "__nuis_effect_path_0");
    let (result, prints, calls) = expected("logical", true, "return", "both", 2, INPUTS[2]);
    execute(&text, result, &prints, calls);
}

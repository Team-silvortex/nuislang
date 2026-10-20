use super::*;
use crate::frontend::parse_nuis_module;

#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_return_parent_break_cases.rs"]
mod breaking;
#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_return_parent_continue_cases.rs"]
mod continuing;
#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_return_parent_mixed_cases.rs"]
mod mixed;
#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_return_selected_cases.rs"]
mod selected;
#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_return_parent_steps_cases.rs"]
mod steps;

fn source() -> String {
    mixed::source(mixed::variants()[0])
}

fn prepared(source: &str) -> (NirFunction, control_values::CarryLayouts, Prepared) {
    let module = parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let plan = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    assert!(verify_independent(function, &plan, &layouts).is_some());
    (function.clone(), layouts, plan)
}

fn edit_block(body: &mut Vec<NirStmt>, edit: &mut impl FnMut(&mut Vec<NirStmt>) -> bool) -> bool {
    if edit(body) {
        return true;
    }
    body.iter_mut().any(|stmt| match stmt {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => edit_block(then_body, edit) || edit_block(else_body, edit),
        NirStmt::While { body, .. } => edit_block(body, edit),
        _ => false,
    })
}

#[test]
fn return_effect_prefix_certifies_all_existing_typed_selected_and_local_exit_matrices() {
    let matrices = [
        selected::variants()
            .into_iter()
            .map(selected::source)
            .collect::<Vec<_>>(),
        steps::variants().into_iter().map(steps::source).collect(),
        continuing::variants()
            .into_iter()
            .map(continuing::source)
            .collect(),
        breaking::variants()
            .into_iter()
            .map(breaking::source)
            .collect(),
        mixed::variants().into_iter().map(mixed::source).collect(),
    ];
    for sources in matrices {
        assert!(!sources.is_empty());
        for source in sources {
            let (function, layouts, plan) = prepared(&source);
            let module = parse_nuis_module(&source).unwrap();
            let before = module.clone();
            let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
            assert!(catalog.contains_key("work"));
            assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("work"));
            let checked = prepare_effects(&function, &layouts, &catalog)
                .unwrap()
                .unwrap();
            assert_eq!(checked.body, plan.body);
            assert_eq!(
                normalize_effects(&function, &layouts),
                Some(Some(plan.body))
            );
            assert_eq!(module, before);
        }
    }
}

#[test]
fn return_effect_prefix_rejects_removed_replayed_reordered_and_hoisted_prefix_work() {
    let module = parse_nuis_module(&source()).unwrap();
    let catalog =
        scalar_helpers::collect_iteration_calls(&module, &control_values::layouts(&module));
    for mutation in [
        "remove",
        "duplicate",
        "reorder",
        "hoist",
        "unused-check",
        "bound-call",
        "inactive-arm",
    ] {
        let (function, layouts, mut plan) = prepared(&source());
        let changed = edit_block(&mut plan.body, &mut |body| {
            if mutation == "inactive-arm" {
                let Some(stmt) = body
                    .iter_mut()
                    .find(|s| **s == NirStmt::Print(NirExpr::Int(9)))
                else {
                    return false;
                };
                *stmt = NirStmt::Print(NirExpr::Int(99));
                return true;
            }
            if mutation == "unused-check" || mutation == "bound-call" {
                let name = if mutation == "unused-check" {
                    "fallback"
                } else {
                    "width"
                };
                let Some(NirStmt::Let { value, .. }) = body
                    .iter_mut()
                    .find(|s| matches!(s, NirStmt::Let { name: n, .. } if n == name))
                else {
                    return false;
                };
                *value = NirExpr::Int(1);
                return true;
            }
            let Some(index) = body
                .iter()
                .position(|s| *s == NirStmt::Print(NirExpr::Int(7)))
            else {
                return false;
            };
            match mutation {
                "remove" => {
                    body.remove(index);
                }
                "duplicate" => body.insert(index, body[index].clone()),
                "reorder" => body.swap(index, index - 1),
                "hoist" => {
                    let work = body.remove(index - 1);
                    body.insert(0, work);
                }
                _ => unreachable!(),
            }
            true
        });
        assert!(changed, "{mutation}");
        if ["remove", "duplicate", "inactive-arm"].contains(&mutation) {
            assert!(
                scalar_helpers::validate_iteration_body(&function, &plan.body, &catalog, &layouts)
                    .is_some(),
                "type-valid but effect-invalid: {mutation}"
            );
        }
        assert!(
            verify_independent(&function, &plan, &layouts).is_none(),
            "{mutation}"
        );
    }
}

#[test]
fn return_effect_prefix_rejects_early_pending_wrong_payload_and_changed_call_arguments() {
    for variant in mixed::variants() {
        for mutation in [
            "pending-first",
            "pending-zero",
            "no-payload",
            "argument",
            "no-break",
        ] {
            let (function, layouts, mut plan) = prepared(&mixed::source(variant));
            let NirStmt::Let { name: payload, .. } = &plan.body[1] else {
                panic!()
            };
            let payload = payload.clone();
            assert!(edit_block(&mut plan.body, &mut |body| {
                let Some(index) = body.iter().position(|s| {
                    matches!(s, NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. }
                        if name == &payload && callee == "returned")
                }) else {
                    return false;
                };
                match mutation {
                    "pending-first" => body.swap(index, index + 1),
                    "pending-zero" => {
                        let NirStmt::Let { value, .. } = &mut body[index + 1] else {
                            panic!()
                        };
                        *value = NirExpr::Int(0);
                    }
                    "no-payload" => {
                        body.remove(index);
                    }
                    "argument" => {
                        let NirStmt::Let {
                            value: NirExpr::Call { args, .. },
                            ..
                        } = &mut body[index]
                        else {
                            panic!()
                        };
                        *args.last_mut().unwrap() = NirExpr::Int(0);
                    }
                    "no-break" => {
                        body.remove(index + 2);
                    }
                    _ => unreachable!(),
                }
                true
            }));
            assert!(
                verify_independent(&function, &plan, &layouts).is_none(),
                "{variant:?} {mutation}"
            );
        }
    }
}

#[test]
fn return_effect_prefix_requires_each_immediate_scoped_pending_propagation() {
    for mutation in [
        "remove",
        "duplicate",
        "unconditional",
        "wrong-flag",
        "wrong-exit",
        "extra-else",
    ] {
        let (function, layouts, mut plan) = prepared(&source());
        let flag = plan.signal.name().to_owned();
        assert!(edit_block(&mut plan.body, &mut |body| {
            let Some(index) = body.iter().position(|s| {
                matches!(s,
                NirStmt::If { condition: NirExpr::Binary { lhs, .. }, .. }
                    if lhs.as_ref() == &NirExpr::Var(flag.clone()))
            }) else {
                return false;
            };
            match mutation {
                "remove" => {
                    body.remove(index);
                }
                "duplicate" => body.insert(index, body[index].clone()),
                _ => {
                    let NirStmt::If {
                        condition,
                        then_body,
                        else_body,
                    } = &mut body[index]
                    else {
                        panic!()
                    };
                    match mutation {
                        "unconditional" => *condition = NirExpr::Bool(true),
                        "wrong-flag" => *condition = NirExpr::Var("gate".into()),
                        "wrong-exit" => *then_body = vec![NirStmt::Continue],
                        "extra-else" => else_body.push(NirStmt::Print(NirExpr::Int(99))),
                        _ => unreachable!(),
                    }
                }
            }
            true
        }));
        assert!(
            verify_independent(&function, &plan, &layouts).is_none(),
            "{mutation}"
        );
    }
}

#[test]
fn return_effect_prefix_keeps_leading_trailing_steps_and_local_exits_exact() {
    for variant in mixed::variants() {
        for mutation in ["step", "local-continue", "local-break"] {
            let (function, layouts, mut plan) = prepared(&mixed::source(variant));
            assert!(edit_block(&mut plan.body, &mut |body| {
                if mutation == "step" {
                    let Some(NirStmt::While { condition, body }) =
                        body.iter_mut().find(|s| matches!(s, NirStmt::While { .. }))
                    else {
                        return false;
                    };
                    let leading = induction::parse(condition, body).unwrap().leading;
                    let index = if leading { 0 } else { body.len() - 1 };
                    let NirStmt::Let { value, .. } = &mut body[index] else {
                        panic!()
                    };
                    *value = NirExpr::Int(0);
                    return true;
                }
                let wanted = if mutation == "local-continue" {
                    NirStmt::Continue
                } else {
                    NirStmt::Break
                };
                if !body.iter().any(|s| *s == NirStmt::Print(NirExpr::Int(25))) {
                    return false;
                }
                let Some(stmt) = body.iter_mut().find(|s| **s == wanted) else {
                    return false;
                };
                *stmt = if mutation == "local-continue" {
                    NirStmt::Break
                } else {
                    NirStmt::Continue
                };
                true
            }));
            assert!(
                verify_independent(&function, &plan, &layouts).is_none(),
                "{variant:?} {mutation}"
            );
        }
    }
}

#[test]
fn return_effect_prefix_requires_fresh_independent_storage_and_exact_seed_types() {
    for mutation in ["parameter", "binding", "same-storage", "seed", "type"] {
        let (function, layouts, mut plan) = prepared(&source());
        if mutation == "parameter" {
            plan.signal = Signal("rounds".into());
            let NirStmt::Let { name, .. } = &mut plan.body[0] else {
                panic!()
            };
            *name = "rounds".into();
        } else {
            let NirStmt::Let { name, ty, value } = &mut plan.body[1] else {
                panic!()
            };
            match mutation {
                "binding" => *name = "current".into(),
                "same-storage" => *name = plan.signal.name().into(),
                "seed" => {
                    *value = NirExpr::Call {
                        callee: "checkpoint".into(),
                        args: vec![NirExpr::Int(0)],
                    }
                }
                "type" => *ty = Some(scalar_type("i64")),
                _ => unreachable!(),
            }
        }
        assert!(
            verify_independent(&function, &plan, &layouts).is_none(),
            "{mutation}"
        );
    }
}

#[test]
fn return_effect_prefix_bounds_fail_closed_without_mutating_or_granting_source() {
    let (function, layouts, _) = prepared(&source());
    let mut large = function.clone();
    large
        .body
        .splice(0..0, (0..40_000).map(|_| NirStmt::Print(NirExpr::Int(1))));
    let before = large.clone();
    assert!(normalize_effects(&large, &layouts).is_none());
    assert_eq!(large, before);
    let mut value = NirExpr::Int(0);
    for _ in 0..65 {
        value = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(value),
            rhs: Box::new(NirExpr::Int(1)),
        };
    }
    for stmt in [
        NirStmt::Print(value),
        NirStmt::Await(NirExpr::Int(0)),
        NirStmt::Print(NirExpr::AllocBuffer {
            len: Box::new(NirExpr::Int(1)),
            fill: Box::new(NirExpr::Int(0)),
        }),
    ] {
        let mut function = function.clone();
        function.body.insert(0, stmt);
        assert!(normalize_effects(&function, &layouts).is_none());
    }
}

#[test]
fn return_effect_prefix_is_not_call_type_or_purity_admission() {
    let mut module = parse_nuis_module(&source()).unwrap();
    let layouts = control_values::layouts(&module);
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "work")
        .unwrap();
    assert!(edit_block(&mut function.body, &mut |body| {
        let Some(NirStmt::Let {
            value: NirExpr::Call { callee, .. },
            ..
        }) = body
            .iter_mut()
            .find(|s| matches!(s, NirStmt::Let { name, .. } if name == "width"))
        else {
            return false;
        };
        *callee = "unregistered".into();
        true
    }));
    let plan = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    assert!(verify_independent(function, &plan, &layouts).is_some());
    assert!(!scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("work"));
    assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("work"));
}

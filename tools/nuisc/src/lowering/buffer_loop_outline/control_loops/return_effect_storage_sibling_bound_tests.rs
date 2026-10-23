use super::tests::plans;
use super::*;

#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_sibling_bounds_cases.rs"]
pub(super) mod cases;

#[test]
fn effect_return_storage_sibling_bounds_admit_completed_snapshot_without_new_authority() {
    for variant in cases::variants() {
        let source = cases::source(variant)
            .replace("frozen", "saved_extent")
            .replace("carry", "working_copy")
            .replace("bound_", "snapshot_");
        let (module, layouts, catalog) = plans(&source, "work");
        let before = module.clone();
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        assert!(verify(
            function,
            &independent,
            &shared,
            "working_copy",
            &layouts,
            &catalog
        )
        .is_some());
        for callee in ["advance", "snapshot_seed", "snapshot_extent"] {
            let mut denied = catalog.clone();
            denied.remove(callee);
            assert!(prepare_effects(function, &layouts, &denied).is_none());
            assert!(verify(
                function,
                &independent,
                &shared,
                "working_copy",
                &layouts,
                &denied
            )
            .is_none());
        }
        assert_eq!(shared.signal.name(), independent.signal.name());
        assert_eq!(module, before);
    }
}

pub(super) fn mutate(body: &mut Vec<NirStmt>, mutation: usize) -> bool {
    if let Some(second) = body
        .iter()
        .position(|s| matches!(s, NirStmt::Let { name, .. } if name == "second_width"))
    {
        if mutation == 0 {
            assert!(matches!(
                body[second + 1],
                NirStmt::Print(NirExpr::Int(945))
            ));
            body.swap(second, second + 1);
        } else {
            let NirStmt::Let {
                value: NirExpr::Call { args, .. },
                ..
            } = &mut body[second]
            else {
                unreachable!()
            };
            match mutation {
                1 => args[0] = NirExpr::Var("carry".into()),
                2 => {
                    args[0] = NirExpr::Call {
                        callee: "bound_seed".into(),
                        args: vec![
                            NirExpr::Var("carry".into()),
                            NirExpr::Var("trips".into()),
                            NirExpr::Var("root".into()),
                        ],
                    }
                }
                3 => args[1] = NirExpr::Var("divisor".into()),
                _ => unreachable!(),
            }
        }
        return true;
    }
    for stmt in body {
        match stmt {
            NirStmt::While { body, .. } => {
                if mutate(body, mutation) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if mutate(then_body, mutation) || mutate(else_body, mutation) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_sibling_bounds_reject_typed_recomputation_effect_reordering_and_argument_bypasses(
) {
    let (module, layouts, catalog) = plans(&cases::source(cases::variants()[0]), "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    for mutation in 0..4 {
        let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        assert!(mutate(&mut shared.body, mutation));
        assert!(
            scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
                .is_some(),
            "mutation {mutation}"
        );
        assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    }
}

#[test]
fn effect_return_storage_sibling_bounds_oracle_distinguishes_child_and_parent_exits() {
    for variant in cases::variants() {
        let mut call = cases::base(variant);
        call.mode = 9;
        let (state, trace) = call.execute(true, None).unwrap();
        assert_eq!(state.total, if variant.leading { 27 } else { 23 });
        assert_eq!(trace.iter().filter(|&&v| v == 930).count(), 1);
        assert_eq!(
            trace
                .windows(5)
                .filter(|w| *w == [932, 933, 2, 1, 16])
                .count(),
            2
        );
        for mode in [1, 4, 5] {
            call.mode = mode;
            call.second = -1;
            let (_, trace) = call.execute(false, None).unwrap();
            assert!(!trace.iter().any(|v| matches!(v, 945 | 946 | 947)));
        }
        for mode in [2, 3] {
            call.mode = mode;
            call.second = 1;
            let (_, trace) = call.execute(false, None).unwrap();
            assert!(trace.contains(&946));
        }
    }
}

#[test]
fn effect_return_storage_sibling_bounds_preserve_candidate_fallback_and_original_read_rejection() {
    let source = cases::source(cases::variants()[0])
        .replace(
            "let mut carry = seed;",
            "let mut candidate_0 = seed; let mut carry = seed;",
        )
        .replace("print(941);", "candidate_0 = candidate_0; print(941);")
        .replace(
            "print(943);",
            "print(943); let snapshot = candidate_0.total;",
        );
    let (module, layouts, catalog) = plans(&source, "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let rejected = rewrite(
        function,
        &layouts,
        Some("candidate_0".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(verify(
        function,
        &independent,
        &rejected,
        "candidate_0",
        &layouts,
        &catalog
    )
    .is_none());
    let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
    assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_some());
    let local = rewrite(
        function,
        &layouts,
        Some("frozen".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(verify(function, &independent, &local, "frozen", &layouts, &catalog).is_none());
    let source = source.replace("print(943);", "print(943); let forbidden = carry.total;");
    let module = crate::frontend::parse_nuis_module(&source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    assert!(!catalog.contains_key("work"));
    assert!(prepare_effects(function, &layouts, &catalog).is_none());
}

pub(super) fn run_entry(
    mut module: yir_core::YirModule,
    reversed: bool,
) -> Result<(yir_core::Value, Vec<i64>), ()> {
    if reversed {
        module.nodes.reverse();
        module.functions.reverse();
        for function in &mut module.functions {
            function.body_nodes.reverse();
        }
    }
    let entry = module
        .functions
        .iter()
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap();
    let result = entry.result.as_ref().unwrap().node.clone();
    yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&module),
        &yir_verify::default_registry(),
    )
    .map(|trace| {
        let printed = trace
            .events
            .iter()
            .filter(|e| {
                e.contains("cpu.print")
                    || (e.contains("cpu.guard_print") && e.contains("if true then print"))
            })
            .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
            .collect();
        (trace.values[&result].clone(), printed)
    })
    .map_err(|_| ())
}

#[test]
fn effect_return_storage_sibling_bounds_match_independent_values_effects_and_four_word_reduction() {
    use super::execution_tests::{lowered, maximum_carry_width};
    for variant in cases::variants() {
        let mut calls = (0..=9)
            .map(|mode| {
                let mut call = cases::base(variant);
                call.mode = mode;
                call
            })
            .collect::<Vec<_>>();
        for (_, mut failure) in cases::failures(variant) {
            failure.input = failure.previous.take().unwrap();
            calls.push(failure);
        }
        for call in calls {
            let args = call
                .words()
                .into_iter()
                .enumerate()
                .map(|(slot, word)| match slot {
                    0 | 4 => (word != 0).to_string(),
                    2 => format!("{:?}", f32::from_bits(word as u32)),
                    3 => format!("{:?}", f64::from_bits(word as u64)),
                    _ if word == i64::MIN => "(-9223372036854775807 - 1)".into(),
                    _ => word.to_string(),
                })
                .collect::<Vec<_>>()
                .join(",");
            let source = cases::source(variant).replace(
                "fn main() -> i64 { return 0; }",
                &format!("fn main() -> i64 {{ let result = start({args}); return result.total; }}"),
            );
            let independent = lowered(&source, true);
            let shared = lowered(&source, false);
            assert_eq!(
                maximum_carry_width(&shared) + 4,
                maximum_carry_width(&independent)
            );
            for reversed in [false, true] {
                let actual = run_entry(shared.clone(), reversed);
                assert_eq!(actual, run_entry(independent.clone(), reversed));
                assert_eq!(
                    actual,
                    call.execute(false, None)
                        .map(|(state, trace)| (yir_core::Value::Int(state.total), trace))
                        .map_err(|_| ()),
                    "{call:?}"
                );
            }
        }
    }
}

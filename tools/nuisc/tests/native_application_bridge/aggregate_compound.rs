use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_conditional::{Fixture, BASE};

#[derive(Clone, Copy)]
enum Logic {
    And,
    Or,
    AndOr,
    OrAnd,
    Grouped,
}

impl Logic {
    fn source(self, state: &str) -> String {
        let a = format!("{state} > pivot");
        let b = format!("{state} < limit");
        let c = format!("seed == {state}");
        match self {
            Self::And => format!("{a} && {b}"),
            Self::Or => format!("{a} || {b}"),
            Self::AndOr => format!("{a} && {b} || {c}"),
            Self::OrAnd => format!("{a} || {b} && {c}"),
            Self::Grouped => format!("({a} || {b}) && {c}"),
        }
    }

    fn evaluate(self, state: i64, case: Case, pivot: i64, count: &mut i64) -> bool {
        let mut leaf = |which| {
            *count += 1;
            match which {
                0 => state > pivot,
                1 => state < case.limit,
                2 => case.seed == state,
                _ => unreachable!(),
            }
        };
        match self {
            Self::And => leaf(0) && leaf(1),
            Self::Or => leaf(0) || leaf(1),
            Self::AndOr => leaf(0) && leaf(1) || leaf(2),
            Self::OrAnd => leaf(0) || leaf(1) && leaf(2),
            Self::Grouped => (leaf(0) || leaf(1)) && leaf(2),
        }
    }
}

fn source(fixture: Fixture, logic: Logic) -> String {
    assert_eq!(fixture.compare, ">");
    assert!(!fixture.reversed);
    let mut source = fixture.source();
    for slot in 0..fixture.count {
        if slot > 0 && slot % 2 == 0 {
            continue;
        }
        let state = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        let original = format!("if {state} > pivot");
        assert!(source.contains(&original));
        source = source.replace(&original, &format!("if {}", logic.source(&state)));
        if fixture.keep_first_else {
            source = source.replace(
                &format!(" else {{ let carry{slot}: i64 = carry{slot}; }}"),
                "",
            );
        }
    }
    source
}

fn execute(fixture: Fixture, logic: Logic, cases: &[(Case, i64)]) {
    let mut counts = Vec::new();
    let observations = cases
        .iter()
        .map(|&(case, pivot)| {
            let mut count = 0;
            let observation = fixture.expected_with_predicate(case, pivot, &mut |state| {
                logic.evaluate(state, case, pivot, &mut count)
            });
            // A failed native preflight executes none of the oracle's trial steps.
            if observation.state.is_none() && observation.reference_error.is_none() {
                count = 0;
            }
            counts.push(count);
            observation
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_with_predicates(
        &source(fixture, logic),
        &observations,
        "loop_while_scalar_cond_chain_body",
        !fixture.op.is_empty(),
        Some(&counts),
    );
}

#[test]
fn compound_carries_match_independent_oracle_and_actual_short_circuit_counts() {
    for (shape_index, shape) in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix]
        .into_iter()
        .enumerate()
    {
        for (logic_index, logic) in [Logic::And, Logic::Or, Logic::AndOr, Logic::OrAnd]
            .into_iter()
            .enumerate()
        {
            let descending = logic_index % 2 == 1;
            let fixture = Fixture {
                shape,
                descending,
                op: if shape_index % 2 == 0 { "/" } else { "%" },
                keep_first_else: logic_index % 2 == 0,
                ..Fixture::default()
            };
            let mut cases = Vec::new();
            for enabled in [false, true] {
                for trips in [0, 1, 7] {
                    for stride in [1, 2] {
                        for seed in [-7, 0, i64::MAX] {
                            for pivot in [-2, 2] {
                                cases.push((
                                    Case {
                                        enabled,
                                        limit: if descending {
                                            -trips * stride
                                        } else {
                                            trips * stride
                                        },
                                        stride,
                                        seed,
                                        ..BASE
                                    },
                                    pivot,
                                ));
                            }
                        }
                    }
                }
            }
            assert_eq!(cases.len(), 72);
            execute(fixture, logic, &cases);
        }
    }
}

#[test]
fn compound_precedence_grouping_and_variable_widths_execute() {
    for count in [1, 7] {
        for descending in [false, true] {
            let fixture = Fixture {
                count,
                descending,
                op: "",
                keep_first_else: true,
                ..Fixture::default()
            };
            let mut cases = Vec::new();
            for seed in [-1, 1, i64::MAX] {
                for pivot in [i64::MIN, -2, 2, i64::MAX] {
                    cases.push((
                        Case {
                            seed,
                            limit: if descending { -7 } else { 7 },
                            ..BASE
                        },
                        pivot,
                    ));
                }
            }
            execute(fixture, Logic::Grouped, &cases);
        }
    }
}

#[test]
fn compound_carry_preflight_and_checked_traps_preserve_selected_path_evaluation() {
    let failures = [
        Case { stride: 0, ..BASE },
        Case { stride: -1, ..BASE },
        Case {
            initial: i64::MAX - 1,
            limit: i64::MAX,
            stride: 2,
            ..BASE
        },
        Case {
            limit: 65537,
            ..BASE
        },
        Case { divisor: 0, ..BASE },
        Case {
            limit: 0,
            divisor: -1,
            seed: i64::MAX - 1,
            ..BASE
        },
    ];
    for (index, failure) in failures.into_iter().enumerate() {
        let fixture = Fixture {
            shape: if index % 2 == 0 {
                Shape::Callee
            } else {
                Shape::Inline
            },
            op: if index % 2 == 0 { "/" } else { "%" },
            keep_first_else: true,
            ..Fixture::default()
        };
        execute(
            fixture,
            Logic::AndOr,
            &[
                (
                    Case {
                        enabled: false,
                        ..failure
                    },
                    2,
                ),
                (BASE, 2),
                (failure, 2),
            ],
        );
    }
    for (shape, failure) in [(Shape::Prefix, failures[0]), (Shape::Suffix, failures[4])] {
        execute(
            Fixture {
                shape,
                keep_first_else: true,
                ..Fixture::default()
            },
            Logic::Grouped,
            &[(
                Case {
                    enabled: false,
                    ..failure
                },
                2,
            )],
        );
    }
    for descending in [false, true] {
        execute(
            Fixture {
                descending,
                op: "",
                keep_first_else: true,
                ..Fixture::default()
            },
            Logic::OrAnd,
            &[
                (
                    Case {
                        limit: if descending { -131072 } else { 131072 },
                        stride: 2,
                        ..BASE
                    },
                    2,
                ),
                (
                    Case {
                        limit: 0,
                        stride: 0,
                        ..BASE
                    },
                    2,
                ),
            ],
        );
    }
    execute(
        Fixture {
            descending: true,
            op: "",
            keep_first_else: true,
            ..Fixture::default()
        },
        Logic::And,
        &[(
            Case {
                initial: i64::MIN + 1,
                limit: i64::MIN,
                stride: 2,
                ..BASE
            },
            2,
        )],
    );
}

#[test]
fn compound_rhs_kind_drift_rejects_even_in_skipped_leaves_and_zero_trip_loops() {
    for zero_trip in [false, true] {
        let source = source(Fixture::default(), Logic::And);
        let source = if zero_trip {
            source.replace("index < limit", "index < initial")
        } else {
            source
        };
        let project = Project::with_source(&source);
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        emit_registered(&module, "counter").unwrap();
        let original = module
            .nodes
            .iter()
            .find(|n| n.op.instruction == "loop_while_scalar_cond_chain")
            .unwrap();
        assert_eq!(original.op.args[6], "and");
        assert_eq!(original.op.args[9], "current_lt");
        for (kind, literal) in [
            ("bool", "true"),
            ("i32", "1"),
            ("f32", "1.0"),
            ("f64", "1.0"),
        ] {
            let mut drift = module.clone();
            let name = format!("{}_rhs_drift", original.name);
            let mut constant = original.clone();
            constant.name = name.clone();
            constant.op =
                yir_core::Operation::parse(&format!("cpu.const_{kind}"), vec![literal.into()])
                    .unwrap();
            drift.nodes.push(constant);
            let function = drift
                .functions
                .iter_mut()
                .find(|f| f.body_nodes.contains(&original.name))
                .unwrap();
            function.body_nodes.push(name.clone());
            drift
                .node_lanes
                .insert(name.clone(), format!("fn:{}", function.name));
            drift.edges.push(yir_core::Edge {
                kind: yir_core::EdgeKind::Dep,
                from: name.clone(),
                to: original.name.clone(),
            });
            drift
                .nodes
                .iter_mut()
                .find(|n| n.name == original.name)
                .unwrap()
                .op
                .args[10] = name;
            let error = emit_registered(&drift, "counter").unwrap_err();
            assert!(
                error.contains("declared scalar kind"),
                "{zero_trip} {kind}: {error}"
            );
        }
    }
}

#[test]
fn compound_flat_branch_helpers_compose_with_typed_lifecycle() {
    assert_native_parity(include_str!("aggregate_compound_loops.ns"), true);
}

#[test]
fn registered_emission_rejects_deep_conditions_during_general_yir_verification() {
    let project = Project::with_source(&source(Fixture::default(), Logic::And));
    let mut module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let node = module
        .nodes
        .iter_mut()
        .find(|n| n.op.instruction == "loop_while_scalar_cond_chain")
        .unwrap();
    let seed = node.op.args[5].clone();
    let rhs = node.op.args[8].clone();
    node.op.args.truncate(5);
    node.op.args.push(seed);
    node.op.args.extend(std::iter::repeat_n("and".into(), 9999));
    for _ in 0..10000 {
        node.op.args.extend(["current_gt".into(), rhs.clone()]);
    }
    node.op.args.extend(["add_current".into(), "keep".into()]);
    let error = emit_registered(&module, "counter").unwrap_err();
    assert!(error.contains("condition depth limit"), "{error}");
}

#[test]
fn compound_reference_fuel_failure_retains_accepted_state_and_cleanup() {
    aggregate_conditional::assert_reference_fuel_failure(include_str!(
        "aggregate_compound_loops.ns"
    ));
}

use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_loop_probe::Invocation;

fn source(
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    compare: &str,
    reversed: bool,
) -> String {
    let mut source = aggregate_carried::source(shape, count, descending, op)
        .replace("seed: i64) ->", "seed: i64, pivot: i64) ->")
        .replace("divisor, seed)", "divisor, seed, pivot)");
    for slot in 0..count {
        if slot > 0 && slot % 2 == 0 {
            continue;
        }
        let rhs = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        let predicate = if reversed {
            format!("pivot {compare} {rhs}")
        } else {
            format!("{rhs} {compare} pivot")
        };
        let update = if slot % 3 == 2 { "*" } else { "+" };
        let original = format!("let carry{slot}: i64 = carry{slot} {update} {rhs};");
        let fallback = if slot == 0 {
            format!("carry{slot} * {rhs}")
        } else {
            format!("carry{slot}")
        };
        assert!(source.contains(&original));
        source = source.replace(
            &original,
            &format!(
                "if {predicate} {{ {original} }} else {{ let carry{slot}: i64 = {fallback}; }}"
            ),
        );
    }
    source
}

#[test]
fn guarded_conditional_carries_gain_native_preflight_and_existing_cond_chain_emission() {
    let project = Project::with_source(&source(Shape::Callee, 3, false, "/", ">", false));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("loop_while_scalar_cond_chain_body"));
    assert!(bridge.llvm_ir.contains("native_loop_preflight"));
    assert!(bridge.llvm_ir.contains("integer_divisor_invalid"));
}

#[derive(Clone, Copy)]
pub(super) struct Fixture {
    pub(super) shape: Shape,
    pub(super) count: usize,
    pub(super) descending: bool,
    pub(super) op: &'static str,
    pub(super) compare: &'static str,
    pub(super) reversed: bool,
    pub(super) keep_first_else: bool,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            shape: Shape::Callee,
            count: 3,
            descending: false,
            op: "/",
            compare: ">",
            reversed: false,
            keep_first_else: false,
        }
    }
}

impl Fixture {
    pub(super) fn source(self) -> String {
        let source = source(
            self.shape,
            self.count,
            self.descending,
            self.op,
            self.compare,
            self.reversed,
        );
        if self.keep_first_else {
            source.replace(
                "let carry0: i64 = carry0 * index;",
                "let carry0: i64 = carry0;",
            )
        } else {
            source
        }
    }

    pub(super) fn expected(self, case: Case, pivot: i64) -> Invocation {
        self.expected_with_predicate(case, pivot, &mut |state| {
            let (lhs, rhs) = if self.reversed {
                (pivot, state)
            } else {
                (state, pivot)
            };
            match self.compare {
                "==" => lhs == rhs,
                "!=" => lhs != rhs,
                "<" => lhs < rhs,
                "<=" => lhs <= rhs,
                ">" => lhs > rhs,
                ">=" => lhs >= rhs,
                _ => unreachable!(),
            }
        })
    }

    pub(super) fn expected_with_predicate(
        self,
        case: Case,
        pivot: i64,
        predicate: &mut impl FnMut(i64) -> bool,
    ) -> Invocation {
        let mut observation = Invocation {
            arguments: vec![
                Value::Bool(case.enabled),
                Value::Int(case.initial),
                Value::Int(case.limit),
                Value::Int(case.stride),
                Value::Int(case.divisor),
                Value::Int(case.seed),
                Value::Int(pivot),
            ],
            state: None,
            leaf: None,
            iterations: 0,
            reference_error: None,
        };
        let seeds = (0..self.count)
            .map(|slot| (i128::from(case.seed) + slot as i128) as i64)
            .collect::<Vec<_>>();
        let mut carries = seeds.clone();
        let mut index = case.initial;
        let mut trips = 0;
        if case.enabled || matches!(self.shape, Shape::Prefix) {
            // Deliberately independent of the native closed-form induction proof.
            while if self.descending {
                index > case.limit
            } else {
                index < case.limit
            } {
                if trips == 65536 || case.stride <= 0 {
                    return observation;
                }
                let next = if self.descending {
                    index.checked_sub(case.stride)
                } else {
                    index.checked_add(case.stride)
                };
                let Some(next) = next else {
                    return observation;
                };
                index = next;
                trips += 1;
                for slot in 0..self.count {
                    let rhs = if slot == 0 { index } else { carries[slot - 1] };
                    let selected = if slot == 0 || slot % 2 == 1 {
                        predicate(rhs)
                    } else {
                        true
                    };
                    let multiply = if slot == 0 && !selected {
                        if self.keep_first_else {
                            continue;
                        }
                        true
                    } else if slot % 2 == 1 && !selected {
                        continue;
                    } else {
                        slot % 3 == 2
                    };
                    carries[slot] = if multiply {
                        (i128::from(carries[slot]) * i128::from(rhs)) as i64
                    } else {
                        (i128::from(carries[slot]) + i128::from(rhs)) as i64
                    };
                }
            }
        }
        observation.iterations = trips;
        let value = if case.enabled || matches!(self.shape, Shape::Prefix | Shape::Suffix) {
            let value = carries[self.count - 1];
            observation.leaf = Some([value, case.divisor]);
            if !self.op.is_empty() && (case.divisor == 0 || (value, case.divisor) == (i64::MIN, -1))
            {
                observation.reference_error = Some(if case.divisor == 0 {
                    "zero"
                } else {
                    "overflow"
                });
                return observation;
            }
            match self.op {
                "/" => (i128::from(value) / i128::from(case.divisor)) as i64,
                "%" => (i128::from(value) % i128::from(case.divisor)) as i64,
                "" => value,
                _ => unreachable!(),
            }
        } else {
            case.seed
        };
        let mut state = if !case.enabled && !matches!(self.shape, Shape::Suffix) {
            let mut state = vec![case.seed, case.initial];
            state.extend(seeds);
            state
        } else {
            let mut state = vec![value, index];
            state.extend(carries);
            state
        };
        state.push(case.seed);
        observation.state = Some(state);
        observation
    }

    fn execute(self, cases: &[(Case, i64)]) {
        aggregate_loop_probe::execute(
            &self.source(),
            &cases
                .iter()
                .map(|&(case, pivot)| self.expected(case, pivot))
                .collect::<Vec<_>>(),
            "loop_while_scalar_cond_chain_body",
            !self.op.is_empty(),
        );
    }
}

pub(super) const BASE: Case = Case {
    enabled: true,
    initial: 0,
    limit: 4,
    stride: 1,
    divisor: 2,
    seed: 1,
};

#[test]
fn conditional_carries_match_native_reference_and_independent_ordered_oracle() {
    for shape in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix] {
        for descending in [false, true] {
            for op in ["/", "%"] {
                let fixture = Fixture {
                    shape,
                    descending,
                    op,
                    ..Fixture::default()
                };
                let mut cases = Vec::new();
                for enabled in [false, true] {
                    for trips in [0, 1, 7] {
                        for seed in [i64::MIN, -7, 0, i64::MAX] {
                            for pivot in [-2, 0, 2] {
                                cases.push((
                                    Case {
                                        enabled,
                                        limit: if descending { -trips } else { trips },
                                        seed,
                                        ..BASE
                                    },
                                    pivot,
                                ));
                            }
                        }
                    }
                }
                assert_eq!(cases.len(), 72);
                fixture.execute(&cases);
            }
        }
    }
}

#[test]
fn all_conditional_comparisons_reversed_operands_and_variable_widths_execute() {
    for (position, compare) in ["==", "!=", "<", "<=", ">", ">="].into_iter().enumerate() {
        for reversed in [false, true] {
            let fixture = Fixture {
                count: if position % 2 == 0 { 1 } else { 7 },
                descending: reversed,
                compare,
                reversed,
                op: "",
                ..Fixture::default()
            };
            let mut cases = Vec::new();
            for pivot in [i64::MIN, -2, 0, 2, i64::MAX] {
                for seed in [-1, 1, i64::MAX] {
                    cases.push((
                        Case {
                            limit: if reversed { -7 } else { 7 },
                            seed,
                            ..BASE
                        },
                        pivot,
                    ));
                }
            }
            fixture.execute(&cases);
        }
    }
}

#[test]
fn conditional_carry_preflight_traps_before_updates_and_skipped_calls_remain_skipped() {
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
    for shape in [Shape::Callee, Shape::Inline] {
        for op in ["/", "%"] {
            let fixture = Fixture {
                shape,
                op,
                ..Fixture::default()
            };
            let skipped = failures.map(|case| {
                (
                    Case {
                        enabled: false,
                        ..case
                    },
                    2,
                )
            });
            fixture.execute(&skipped);
            for case in failures {
                assert!(fixture.expected(case, 2).state.is_none());
                fixture.execute(&[(case, 2)]);
            }
        }
    }
    Fixture {
        shape: Shape::Prefix,
        ..Fixture::default()
    }
    .execute(&[(
        Case {
            enabled: false,
            ..failures[0]
        },
        0,
    )]);
    Fixture {
        op: "",
        ..Fixture::default()
    }
    .execute(&[(failures[0], 0)]);
    Fixture {
        op: "",
        ..Fixture::default()
    }
    .execute(&[(
        Case {
            limit: 65536,
            ..BASE
        },
        2,
    )]);
    Fixture {
        descending: true,
        op: "",
        ..Fixture::default()
    }
    .execute(&[(
        Case {
            initial: i64::MIN + 1,
            limit: i64::MIN,
            stride: 2,
            ..BASE
        },
        0,
    )]);
}

#[test]
fn conditional_flat_branch_helpers_compose_with_typed_lifecycle() {
    assert_native_parity(include_str!("aggregate_conditional_loops.ns"), true);
}

#[test]
fn conditional_native_admission_rejects_scalar_kind_drift_even_on_zero_trips() {
    for zero_trip in [false, true] {
        let mut source = source(Shape::Callee, 2, false, "/", ">", false);
        if zero_trip {
            source = source.replace("index < limit", "index < initial");
        }
        let project = Project::with_source(&source);
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        emit_registered(&module, "counter").unwrap();
        let original = module
            .nodes
            .iter()
            .find(|node| node.op.instruction == "loop_while_scalar_cond_chain")
            .unwrap();
        for slot in [0, 1, 2, 5, 7, 10, 12] {
            for (kind, literal) in [
                ("bool", "true"),
                ("i32", "1"),
                ("f32", "1.0"),
                ("f64", "1.0"),
            ] {
                let mut drift = module.clone();
                let name = format!("{}_type_drift", original.name);
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
                    .find(|node| node.name == original.name)
                    .unwrap()
                    .op
                    .args[slot] = name;
                let error = emit_registered(&drift, "counter").unwrap_err();
                assert!(
                    error.contains("declared scalar kind"),
                    "zero={zero_trip} slot={slot} kind={kind}: {error}"
                );
            }
        }
    }
}

#[test]
fn conditional_native_admission_rejects_missing_operands_and_hidden_branch_payloads() {
    let project = Project::with_source(&source(Shape::Callee, 2, false, "/", ">", false));
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let original = module
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_cond_chain")
        .unwrap();
    for (slot, value) in [
        (0, "missing"),
        (5, "missing"),
        (7, "missing"),
        (6, "carry0_gt"),
        (8, "add_carry1"),
        (9, "scoped_call"),
    ] {
        let mut drift = module.clone();
        drift
            .nodes
            .iter_mut()
            .find(|node| node.name == original.name)
            .unwrap()
            .op
            .args[slot] = value.into();
        assert!(
            emit_registered(&drift, "counter").is_err(),
            "{slot}: {value}"
        );
    }
}

#[test]
fn conditional_reference_fuel_failure_retains_accepted_state_and_cleanup() {
    assert_reference_fuel_failure(include_str!("aggregate_conditional_loops.ns"));
}

pub(super) fn assert_reference_fuel_failure(baseline: &str) {
    let source = baseline.replace("decompose(corrected, 3)", "decompose(corrected, 1000)");
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    emit_registered(&module, "counter").unwrap();
    let registry = yir_verify::default_registry();
    let baseline_project = Project::with_source(baseline);
    let baseline_module = nuisc::pipeline::compile_project(&baseline_project.0)
        .unwrap()
        .yir;
    let (mut baseline_session, _) = ApplicationSession::open_registered(
        &baseline_module,
        &registry,
        "counter",
        vec![
            Value::Int(10),
            Value::I32(-17),
            Value::Bool(true),
            Value::F32(1.5),
            Value::F64(-2.25),
        ],
    )
    .unwrap();
    // The same path fits this budget before increasing only conditional-loop work.
    baseline_session
        .event_budgeted(vec![Value::Int(3), Value::Bool(false)], 500)
        .unwrap();
    baseline_session.close(vec![Value::Int(0)]).unwrap();
    baseline_session.completion_status().unwrap();
    let (mut session, _) = ApplicationSession::open_registered(
        &module,
        &registry,
        "counter",
        vec![
            Value::Int(10),
            Value::I32(-17),
            Value::Bool(true),
            Value::F32(1.5),
            Value::F64(-2.25),
        ],
    )
    .unwrap();
    let accepted = session.state().clone();
    let error = session
        .event_budgeted(vec![Value::Int(3), Value::Bool(false)], 500)
        .unwrap_err();
    assert!(error.contains("budget"), "{error}");
    assert_eq!(session.state(), &accepted);
    session.close(vec![Value::Int(0)]).unwrap();
    assert!(session.completion_status().is_err());
}

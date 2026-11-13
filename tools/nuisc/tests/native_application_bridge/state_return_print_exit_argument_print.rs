#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_return_print_exit_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    sequence_prefixed(rows, cases::PREFIX_TAG)
}
fn sequence_prefixed(rows: &[cases::Case], tag: i64) -> probe::Sequence {
    let mut prior = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (state, trace) = row.execute_prefixed(prior, true, tag).unwrap();
                prior = Some(state);
                probe::Row {
                    input: row.words(),
                    state: state.to_vec(),
                    trace,
                }
            })
            .collect(),
    }
}
fn values(row: cases::Case) -> Vec<Value> {
    row.words()
        .into_iter()
        .enumerate()
        .map(|(i, word)| match i {
            0 | 25 | 26 if row.kind() == cases::Kind::F32 => {
                Value::F32(f32::from_bits(word as u32))
            }
            0 | 25 | 26 => Value::F64(f64::from_bits(word as u64)),
            1 => Value::Bool(word != 0),
            3 | 4 => Value::I32(word as i32),
            _ => Value::Int(word),
        })
        .collect()
}

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|event| {
            event.contains("cpu.print")
                || (event.contains("cpu.guard_print") && event.contains("if true then print"))
        })
        .map(|event| event.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn state_return_print_exit_requires_each_leaf_and_join_effect_site_grant() {
    use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};
    for kind in cases::KINDS {
        let project = Project::with_source(&cases::SOURCE.replace("f64", kind.ty()));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let sites = module
                .nodes
                .iter()
                .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
                .map(|n| n.name.clone())
                .collect::<Vec<_>>();
            let full = I64PrintPolicy::new(sites.clone()).unwrap();
            let bridge = emit_registered_with_i64_prints(
                &module,
                "counter",
                &full,
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            )
            .unwrap();
            assert_eq!(
                bridge
                    .callbacks
                    .iter()
                    .map(|c| c.arguments.len())
                    .collect::<Vec<_>>(),
                [48, 50, 2]
            );
            assert_eq!(bridge.state_fields.len(), 2);
            let joined = module
                .functions
                .iter()
                .filter(|f| {
                    module.nodes.iter().any(|n| {
                        f.body_nodes.contains(&n.name)
                            && matches!(n.op.instruction.as_str(), "print" | "guard_print")
                            && n.op.args.last().is_some_and(|arg| {
                                module.nodes.iter().any(|v| {
                                    v.name == *arg
                                        && v.op.instruction == "const_i64"
                                        && v.op.args == ["95"]
                                })
                            })
                    })
                })
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>();
            assert_eq!(joined.len(), 2);
            let prefixes = module
                .functions
                .iter()
                .filter(|f| {
                    module.nodes.iter().any(|n| {
                        f.body_nodes.contains(&n.name)
                            && matches!(n.op.instruction.as_str(), "print" | "guard_print")
                            && n.op.args.last().is_some_and(|arg| {
                                module.nodes.iter().any(|v| {
                                    v.name == *arg
                                        && v.op.instruction == "const_i64"
                                        && (v.op.args == ["97"] || v.op.args == ["99"])
                                })
                            })
                    })
                })
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>();
            assert_eq!(prefixes.len(), 4);
            for name in [
                "negative_update",
                "nonnegative_update",
                "negative_finish",
                "nonnegative_finish",
            ]
            .into_iter()
            .chain(joined)
            .chain(prefixes)
            {
                let leaf = module.functions.iter().find(|f| f.name == name).unwrap();
                let own = sites
                    .iter()
                    .filter(|s| leaf.body_nodes.contains(s))
                    .collect::<Vec<_>>();
                assert_eq!(own.len(), 3, "{name}");
                for missing in own {
                    let policy =
                        I64PrintPolicy::new(sites.iter().filter(|s| *s != missing).cloned())
                            .unwrap();
                    let error = emit_registered_with_i64_prints(
                        &module,
                        "counter",
                        &policy,
                        cases::LOOP_LIMIT,
                        cases::ENTRY_LIMIT,
                    )
                    .err()
                    .expect("selected leaf or join inherited another effect site's grant");
                    assert!(error.contains("print"), "{error}");
                }
            }
        }
    }
}

#[test]
fn state_return_print_exit_native_executes_only_selected_checks() {
    for kind in cases::KINDS {
        let prefix = cases::prefix(kind);
        let accepted = sequence(&prefix);
        let prior: cases::State = accepted
            .rows
            .last()
            .unwrap()
            .state
            .clone()
            .try_into()
            .unwrap();
        assert_eq!(prior[1], i64::MAX);
        let rows = cases::sequences(kind)
            .iter()
            .map(|r| sequence(r))
            .collect::<Vec<_>>();
        let failures = cases::failure_sequences(kind)
            .into_iter()
            .map(|(prefix, bad)| {
                let prior = prefix
                    .iter()
                    .fold(None, |p, r| Some(r.execute(p, true).unwrap().0));
                probe::SequenceFailure {
                    accepted: sequence(&prefix),
                    input: bad.words(),
                    before_failure: bad.execute(prior, true).unwrap_err(),
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_i64_prints(
            &cases::SOURCE.replace("f64", kind.ty()),
            &rows,
            &failures,
            &cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_return_print_exit_reference_keeps_complete_accepted_state() {
    for kind in cases::KINDS {
        let project = Project::with_source(&cases::SOURCE.replace("f64", kind.ty()));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for rows in cases::sequences(kind) {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(rows[0]),
                )
                .unwrap();
                let (mut prior, trace) = rows[0].execute(None, false).unwrap();
                assert_eq!(printed(&opened.events), trace);
                assert_eq!(state_words(session.state()), prior.map(|w| w as u64));
                for row in &rows[1..] {
                    let mut bad = values(*row);
                    bad[47] = Value::Bool(false);
                    assert!(session.event(bad).is_err());
                    assert_eq!(
                        session.phase(),
                        yir_runtime_host::ApplicationSessionPhase::Open
                    );
                    assert_eq!(state_words(session.state()), prior.map(|w| w as u64));
                    let (state, trace) = row.execute(Some(prior), false).unwrap();
                    let event = session.event(values(*row)).unwrap();
                    assert_eq!(printed(&event.events), trace);
                    assert_eq!(state_words(session.state()), state.map(|w| w as u64));
                    prior = state;
                }
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
                assert_eq!(state_words(session.state()), prior.map(|w| w as u64));
            }
            for (prefix, bad) in cases::failure_sequences(kind) {
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0]),
                )
                .unwrap();
                for row in &prefix[1..] {
                    session.event(values(*row)).unwrap();
                }
                let prior = prefix
                    .iter()
                    .fold(None, |p, r| Some(r.execute(p, false).unwrap().0))
                    .unwrap();
                assert!(session.event(values(bad)).is_err());
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert_eq!(state_words(session.state()), prior.map(|w| w as u64));
                session.close(vec![]).unwrap();
                assert_eq!(state_words(session.state()), prior.map(|w| w as u64));
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn state_return_print_exit_keeps_work_and_helper_entry_limits() {
    for kind in cases::KINDS {
        let rows = [sequence(&[cases::seed(kind)])];
        let source = cases::SOURCE.replace("f64", kind.ty());
        probe::check_i64_helper_entry_limit_rejection(
            &source,
            &rows,
            &cases::observers(kind),
            cases::LOOP_LIMIT,
            64,
        );
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &rows,
            &cases::observers(kind),
            1,
            cases::ENTRY_LIMIT,
            &[90, 70],
        );
    }
}

#[test]
fn state_return_print_exit_keeps_completed_leaf_effects_before_prefix_faults() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 25);
        assert_eq!(cases::failure_sequences(kind).len(), 51);
        assert_eq!(cases::cli_sequences(kind).len(), 4);
        assert_eq!(cases::delivery_failure_sequences(kind).len(), 12);
        for (input, result, completion) in
            [(-1, -4, 88), (0, 2, 94), (cases::TAG - 2, cases::TAG, 86)]
        {
            let mut row = cases::seed(kind);
            row.base.base.base.base.arguments.parents = 2;
            row.base.base.base.base.arguments.child.base.trips = 1;
            row.base.base.base.base.initial_control = input;
            let (state, trace) = row.execute(None, true).unwrap();
            assert_eq!(state[1], result);
            assert!(trace.windows(7).any(|w| w
                == [
                    completion,
                    if input == cases::TAG - 2 { 99 } else { 97 },
                    result ^ cases::PREFIX_TAG,
                    if input == cases::TAG - 2 { 100 } else { 98 },
                    95,
                    result ^ cases::PREFIX_TAG ^ cases::AFTER_TAG,
                    96
                ]));
            assert_eq!(row.words().len(), 48);
            assert_eq!(row.words()[..44], row.base.words());
            row.prefix_fault = 16;
            row.skip_prefix_fault = 16;
            row.base.joined_fault = 16;
            if input == cases::TAG - 2 {
                row.base.nonnegative_finish_fault = 16;
            }
            let trace = row.execute(None, true).unwrap_err();
            assert_eq!(trace.last(), Some(&(result ^ cases::PREFIX_TAG)));
            assert!(trace.windows(3).any(|w| w
                == [
                    completion,
                    if input == cases::TAG - 2 { 99 } else { 97 },
                    result ^ cases::PREFIX_TAG
                ]));
            row.print_fault = 16;
            row.skip_print_fault = 16;
            let trace = row.execute(None, true).unwrap_err();
            assert_eq!(
                trace.last(),
                Some(&(if input == cases::TAG - 2 { 99 } else { 97 }))
            );
            assert!(!trace.contains(&95));
        }
    }
}
#[test]
fn state_return_print_exit_consumes_full_width_arm_local_results() {
    assert_eq!(cases::SOURCE.matches("17179869184").count(), 6);
    for (tag, literal) in [
        (2199023255552, "2199023255552"),
        (i64::MIN, "(-9223372036854775807 - 1)"),
    ] {
        let source = cases::SOURCE.replace("17179869184", literal);
        for kind in cases::KINDS {
            let rows = [-1, 0, cases::TAG - 2, i64::MIN, i64::MAX].map(|control| {
                let mut row = cases::seed(kind);
                row.base.base.base.base.initial_control = control;
                sequence_prefixed(&[row, cases::seed(kind)], tag)
            });
            let failures = cases::prefix_failures(kind)
                .into_iter()
                .map(|(prefix, bad)| {
                    let prior = prefix.iter().fold(None, |p, row| {
                        Some(row.execute_prefixed(p, true, tag).unwrap().0)
                    });
                    probe::SequenceFailure {
                        accepted: sequence_prefixed(&prefix, tag),
                        input: bad.words(),
                        before_failure: bad.execute_prefixed(prior, true, tag).unwrap_err(),
                    }
                })
                .collect::<Vec<_>>();
            probe::check_sequences_with_i64_prints(
                &source.replace("f64", kind.ty()),
                &rows,
                &failures,
                &cases::observers(kind),
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

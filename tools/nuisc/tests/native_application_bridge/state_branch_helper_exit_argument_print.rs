#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_branch_helper_exit_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    sequence_masked(rows, 0)
}

fn sequence_masked(rows: &[cases::Case], mask: i64) -> probe::Sequence {
    let mut prior = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (state, trace) = row.execute_masked(prior, true, mask).unwrap();
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

#[test]
fn state_branch_helper_exit_consumes_returned_values_not_leaf_inputs() {
    let mask = 4294967296;
    let source = cases::SOURCE
        .replace(
            "return negative_update(value,",
            "return negative_update(value ^ 4294967296,",
        )
        .replace(
            "return nonnegative_update(value,",
            "return nonnegative_update(value ^ 4294967296,",
        )
        .replace("return value - 3;", "return (value ^ 4294967296) - 3;")
        .replace("return value + 2;", "return (value ^ 4294967296) + 2;");
    assert_eq!(source.matches("value ^ 4294967296").count(), 4);
    for kind in cases::KINDS {
        let rows = [i64::MIN, i64::MAX, -1, 0, 75, 617].map(|control| {
            let mut row = cases::seed(kind);
            row.base.base.initial_control = control;
            sequence_masked(&[row, cases::seed(kind)], mask)
        });
        let prefix = cases::prefix(kind);
        let prior = prefix
            .iter()
            .fold(None, |p, r| Some(r.execute(p, true).unwrap().0));
        let failures = cases::failures(kind)[6..10]
            .iter()
            .map(|bad| probe::SequenceFailure {
                accepted: sequence_masked(&prefix, mask),
                input: bad.words(),
                before_failure: bad.execute_masked(prior, true, mask).unwrap_err(),
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
fn state_branch_helper_exit_pins_selected_effects_and_fault_priority() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 16);
        assert_eq!(cases::cli_sequences(kind).len(), 8);
        assert_eq!(cases::failures(kind).len(), 13);
        for (initial, expected) in [
            (-1, -4),
            (0, 2),
            (i64::MIN, i64::MAX - 2),
            (i64::MAX, i64::MIN + 1),
            (4294967296, 4294967298),
        ] {
            let mut row = cases::seed(kind);
            row.base.base.arguments.parents = 2;
            row.base.base.arguments.child.base.trips = 1;
            row.base.base.initial_control = initial;
            let (entry, before, after) = if initial < 0 {
                (618, 83, 84)
            } else {
                (619, 85, 86)
            };
            row.negative_helper_fault = if initial < 0 { 64 } else { 16 };
            row.nonnegative_helper_fault = if initial < 0 { 16 } else { 64 };
            let (state, trace) = row.execute(None, true).unwrap();
            assert_eq!(state[1], expected);
            assert!(trace
                .windows(7)
                .any(|w| w == [entry, initial, 48, before, initial, after, 75]));
            assert_eq!(row.words()[..39], row.base.words());
            assert_eq!(row.words().len(), 41);
            row.negative_helper_fault = 16;
            row.nonnegative_helper_fault = 16;
            assert!(row
                .execute(None, true)
                .unwrap_err()
                .ends_with(&[entry, initial, 0, before, initial]));
            row.base.negative_fault = 16;
            row.base.nonnegative_fault = 16;
            assert!(row
                .execute(None, true)
                .unwrap_err()
                .ends_with(&[617, initial, 0, 0]));
            row.base.base.arguments.child.base.guards[2] = 16;
            assert!(row.execute(None, true).unwrap_err().ends_with(&[74]));
        }
    }
}

#[test]
fn state_branch_helper_exit_requires_each_leaf_effect_site_grant() {
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
                [41, 43, 2]
            );
            assert_eq!(bridge.state_fields.len(), 2);
            for name in ["negative_update", "nonnegative_update"] {
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
                    .expect("selected leaf inherited a caller or sibling effect grant");
                    assert!(error.contains("print"), "{error}");
                }
            }
        }
    }
}

#[test]
fn state_branch_helper_exit_native_executes_only_selected_checks() {
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
        let failures = cases::failures(kind)
            .into_iter()
            .map(|bad| probe::SequenceFailure {
                accepted: sequence(&prefix),
                input: bad.words(),
                before_failure: bad.execute(Some(prior), true).unwrap_err(),
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
fn state_branch_helper_exit_reference_keeps_complete_accepted_state() {
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
                    bad[40] = Value::Bool(false);
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
            for bad in cases::failures(kind) {
                let prefix = cases::prefix(kind);
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0]),
                )
                .unwrap();
                session.event(values(prefix[1])).unwrap();
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
fn state_branch_helper_exit_keeps_work_and_helper_entry_limits() {
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

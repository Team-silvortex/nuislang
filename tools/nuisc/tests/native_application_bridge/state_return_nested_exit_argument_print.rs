#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_return_nested_exit_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    sequence_prefixed(rows, cases::LEFT_TAG)
}
fn sequence_prefixed(rows: &[cases::Case], tag: i64) -> probe::Sequence {
    let mut prior = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (state, trace) = row.execute_tagged(prior, true, tag).unwrap();
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
            0 if row.kind() == cases::Kind::F32 => Value::F32(f32::from_bits(word as u32)),
            0 => Value::F64(f64::from_bits(word as u64)),
            2 | 3 => Value::Bool(word != 0),
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
fn state_return_nested_exit_requires_each_leaf_and_join_effect_site_grant() {
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
                [10, 12, 2]
            );
            assert_eq!(bridge.state_fields.len(), 2);
            assert!(sites.len() >= 16);
            for missing in &sites {
                let policy =
                    I64PrintPolicy::new(sites.iter().filter(|s| *s != missing).cloned()).unwrap();
                let error = emit_registered_with_i64_prints(
                    &module,
                    "counter",
                    &policy,
                    cases::LOOP_LIMIT,
                    cases::ENTRY_LIMIT,
                )
                .err()
                .expect("a nested site inherited another print grant");
                assert!(error.contains("print"), "{error}");
            }
        }
    }
}

#[test]
fn state_return_nested_exit_native_executes_only_selected_checks() {
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
        assert_ne!(
            prior,
            [cases::seed(kind).initial, cases::seed(kind).control]
        );
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
            &[],
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_return_nested_exit_reference_keeps_complete_accepted_state() {
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
                    bad[9] = Value::Bool(false);
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
fn state_return_nested_exit_keeps_work_and_helper_entry_limits() {
    for kind in cases::KINDS {
        let rows = [sequence(&[cases::seed(kind)])];
        let source = cases::SOURCE.replace("f64", kind.ty());
        probe::check_i64_helper_entry_limit_rejection(&source, &rows, &[], cases::LOOP_LIMIT, 1);
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &rows,
            &[],
            2,
            cases::ENTRY_LIMIT,
            &[90, 70],
        );
    }
}

#[test]
fn state_return_nested_exit_model_distinguishes_nested_results_and_first_failures() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 27);
        assert_eq!(cases::failure_sequences(kind).len(), 19);
        assert_eq!(cases::cli_sequences(kind).len(), 4);
        assert_eq!(cases::delivery_failure_sequences(kind).len(), 8);
        let mut row = cases::seed(kind);
        row.control = 7;
        row.trips = 1;
        let local = 8 ^ 4294967296i64;
        let staged = local ^ cases::LEFT_TAG;
        let joined = staged ^ 17179869184;
        let (state, trace) = row.execute(None, true).unwrap();
        assert_eq!(state, [(row.initial as u64 ^ kind.mask()) as i64, joined]);
        assert_eq!(
            trace,
            [90, 70, 71, 80, 8, 81, 101, local, 97, staged, 98, 95, joined, 96, 72, 73]
        );
        row.faults[0] = 0;
        row.faults[2] = 0;
        row.faults[4] = 0;
        assert_eq!(row.execute(None, true).unwrap_err(), [90, 70, 71, 80, 8]);
        row.faults[0] = 16;
        assert_eq!(row.execute(None, true).unwrap_err(), trace[..10]);
        row.faults[2] = 16;
        assert_eq!(row.execute(None, true).unwrap_err(), trace[..13]);
    }
}
#[test]
fn state_return_nested_exit_consumes_full_width_arm_local_results() {
    assert_eq!(cases::SOURCE.matches("34359738368").count(), 1);
    for (tag, literal) in [
        (2199023255552, "2199023255552"),
        (i64::MIN, "(-9223372036854775807 - 1)"),
    ] {
        let source = cases::SOURCE.replace("34359738368", literal);
        for kind in cases::KINDS {
            let rows = [-1, 0, 4294967296, i64::MIN, i64::MAX].map(|control| {
                let mut row = cases::seed(kind);
                row.control = control;
                sequence_prefixed(&[row, cases::seed(kind)], tag)
            });
            let failures = cases::failure_sequences(kind)
                .into_iter()
                .map(|(prefix, bad)| {
                    let prior = prefix.iter().fold(None, |p, row| {
                        Some(row.execute_tagged(p, true, tag).unwrap().0)
                    });
                    probe::SequenceFailure {
                        accepted: sequence_prefixed(&prefix, tag),
                        input: bad.words(),
                        before_failure: bad.execute_tagged(prior, true, tag).unwrap_err(),
                    }
                })
                .collect::<Vec<_>>();
            probe::check_sequences_with_i64_prints(
                &source.replace("f64", kind.ty()),
                &rows,
                &failures,
                &[],
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

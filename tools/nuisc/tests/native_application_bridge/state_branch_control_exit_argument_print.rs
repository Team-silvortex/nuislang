#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_branch_control_exit_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    let mut prior = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (state, trace) = row.execute(prior, true).unwrap();
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
fn state_branch_control_exit_native_executes_only_selected_checks() {
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
fn state_branch_control_exit_reference_keeps_complete_accepted_state() {
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
                    bad[38] = Value::Bool(false);
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
fn state_branch_control_exit_pins_branch_arithmetic_and_fault_priority() {
    for kind in cases::KINDS {
        let mut row = cases::seed(kind);
        row.base.arguments.parents = 2;
        row.base.arguments.child.base.trips = 1;
        for (initial, expected) in [
            (-1, -4),
            (0, 2),
            (i64::MIN, i64::MAX - 2),
            (i64::MAX, i64::MIN + 1),
            (4294967296, 4294967298),
        ] {
            row.base.initial_control = initial;
            row.negative_fault = if initial < 0 { 64 } else { 16 };
            row.nonnegative_fault = if initial < 0 { 16 } else { 64 };
            assert_eq!(row.words().len(), 39);
            assert_eq!(row.execute(None, true).unwrap().0[1], expected);
            if initial < 0 {
                row.negative_fault = 16;
            } else {
                row.nonnegative_fault = 16;
            }
            let trace = row.execute(None, true).unwrap_err();
            assert!(trace.ends_with(&[617, initial, 0, 0]));
            row.base.arguments.child.base.guards[2] = 16;
            assert!(row.execute(None, true).unwrap_err().ends_with(&[74]));
            row.base.arguments.child.base.guards[2] = 64;
        }
        for rows in cases::sequences(kind) {
            sequence(&rows);
        }
    }
}

#[test]
fn state_branch_control_exit_keeps_work_and_helper_entry_limits() {
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

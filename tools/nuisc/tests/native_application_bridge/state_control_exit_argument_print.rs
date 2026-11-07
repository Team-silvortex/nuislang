#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_control_exit_argument_print_cases.rs"]
mod cases;
#[path = "state_control_exit_preflight.rs"]
mod preflight;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    let mut previous = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (next, trace) = row.execute(previous, true).unwrap();
                previous = Some(next);
                probe::Row {
                    input: row.words(),
                    state: next.to_vec(),
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

fn check_calls(row: cases::Case, previous: Option<cases::State>, events: &[String]) {
    let plan = row.plan(previous);
    let children: i64 = plan.completed.iter().sum();
    let calls = |name: &str| {
        events
            .iter()
            .filter(|event| event.contains("cpu.call") && event.contains(&format!("] {name}(")))
            .count()
    };
    assert_eq!(
        calls("checked64"),
        children as usize * (1 + usize::from(row.arguments.child.base.selected)) + plan.suffixes
    );
    assert_eq!(calls("finish"), (row.arguments.parents + 1) as usize);
    assert_eq!(calls("child_count"), row.arguments.parents as usize);
    for (name, count) in [
        "continue_at",
        "confirm_continue",
        "break_at",
        "fallback_break",
        "rescue_continue",
        "confirm_break",
    ]
    .into_iter()
    .zip(plan.calls)
    {
        assert_eq!(calls(name), count, "{name}");
    }
    for (slot, exit) in ["continue", "break"].into_iter().enumerate() {
        for suffix in ["lhs", "rhs", "mode_value", "guard_value"] {
            assert_eq!(
                calls(&format!("{exit}_{suffix}")),
                plan.calls[if slot == 0 { 1 } else { 5 }]
            );
        }
    }
}

#[test]
fn state_control_exit_native_publishes_both_words_and_fault_prefixes() {
    for kind in cases::KINDS {
        let prefix = cases::prefix(kind);
        let previous = prefix.iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        let failures = cases::failures(kind)
            .into_iter()
            .map(|bad| probe::SequenceFailure {
                accepted: sequence(&prefix),
                input: bad.words(),
                before_failure: bad.execute(previous, true).unwrap_err(),
            })
            .collect::<Vec<_>>();
        let rows = cases::sequences(kind)
            .iter()
            .map(|rows| sequence(rows))
            .collect::<Vec<_>>();
        probe::check_sequences_with_i64_prints(
            &cases::SOURCE.replace("f64", kind.ty()),
            &rows,
            &failures,
            cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_control_exit_reference_retains_complete_state_after_rejection_and_faulted_close() {
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
                let (mut previous, trace) = rows[0].execute(None, false).unwrap();
                assert_eq!(printed(&opened.events), trace);
                check_calls(rows[0], None, &opened.events);
                assert_eq!(
                    state_words(session.state()),
                    previous.map(|word| word as u64)
                );
                for row in &rows[1..] {
                    let snapshot = state_words(session.state());
                    let mut rejected = values(*row);
                    rejected[35] = Value::Bool(false);
                    assert!(session.event(rejected).is_err());
                    assert_eq!(state_words(session.state()), snapshot);
                    assert_eq!(
                        session.phase(),
                        yir_runtime_host::ApplicationSessionPhase::Open
                    );
                    let (next, trace) = row.execute(Some(previous), false).unwrap();
                    let event = session.event(values(*row)).unwrap();
                    assert_eq!(printed(&event.events), trace);
                    check_calls(*row, Some(previous), &event.events);
                    assert_eq!(state_words(session.state()), next.map(|word| word as u64));
                    previous = next;
                }
                assert!(session.close(vec![Value::Int(0)]).is_err());
                assert_eq!(
                    state_words(session.state()),
                    previous.map(|word| word as u64)
                );
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
                assert_eq!(
                    state_words(session.state()),
                    previous.map(|word| word as u64)
                );
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
                let previous = prefix
                    .iter()
                    .fold(None, |previous, row| {
                        Some(row.execute(previous, false).unwrap().0)
                    })
                    .unwrap();
                assert!(session.event(values(bad)).is_err(), "{bad:?}");
                assert_eq!(
                    state_words(session.state()),
                    previous.map(|word| word as u64)
                );
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_eq!(
                    state_words(session.state()),
                    previous.map(|word| word as u64)
                );
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn state_control_exit_reference_close_failure_keeps_both_accepted_fields() {
    let stop = "fn stop(state: State) -> State { return state; }";
    let checked = "fn stop(state: State) -> State { let result_total = -state.total; let result_control = state.control + 1; let checked = 0 / state.control; return State { total: result_total, control: result_control }; }";
    assert_eq!(cases::SOURCE.matches(stop).count(), 1);
    for kind in cases::KINDS {
        let source = cases::SOURCE
            .replace(stop, checked)
            .replace("f64", kind.ty());
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let prefix = cases::prefix(kind);
            let (mut session, _) = ApplicationSession::open_registered(
                &module,
                &registry,
                "counter",
                values(prefix[0]),
            )
            .unwrap();
            session.event(values(prefix[1])).unwrap();
            let accepted = state_words(session.state());
            assert_eq!(accepted, [kind.initial() as u64, 0]);
            let error = session.close(vec![]).unwrap_err();
            assert_eq!(
                session.phase(),
                yir_runtime_host::ApplicationSessionPhase::Closed
            );
            assert_eq!(state_words(session.state()), accepted);
            assert_eq!(session.close(vec![]).unwrap_err(), error);
            assert_eq!(state_words(session.state()), accepted);
            assert!(session.completion_status().is_err());
            let mut healthy = cases::seed(kind);
            healthy.arguments.parents = 0;
            healthy.initial_control = 1;
            let (mut session, _) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(healthy))
                    .unwrap();
            session.close(vec![]).unwrap();
            assert_eq!(state_words(session.state()), [kind.last_state() as u64, 2]);
            session.completion_status().unwrap();
        }
    }
}

#[test]
fn state_control_exit_oracle_pins_persistent_updates_decoys_and_tail_order() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 24);
        let early = cases::early_sequences(kind).remove(0);
        let first = early[0].plan(None);
        assert_eq!(first.requested, [0, 3, 0, 4]);
        assert_eq!(first.completed, [0, 1, 0, 4]);
        assert_eq!(first.state, [kind.last_state(), 4]);
        let next = early[1].plan(Some(first.state));
        assert_eq!(next.completed, [0, 4, 0, 4]);
        assert_eq!(next.state, [kind.last_state(), 12]);
        assert_eq!(cases::cli_sequences(kind).len(), 10);
        assert_eq!(cases::failures(kind).len(), 30);
        for rows in cases::sequences(kind) {
            let mut previous = None;
            for row in rows {
                let planned = row.plan(previous);
                let (state, trace) = row.execute(previous, true).unwrap();
                assert_eq!(state, planned.state);
                assert_eq!(row.words()[..35], row.arguments.words());
                assert_eq!(
                    row.words()[35..],
                    [row.initial_control, row.control_tail_guard]
                );
                let initial = previous.unwrap_or(row.initial());
                let children: i64 = planned.completed.iter().sum();
                assert_eq!(state[1], initial[1].wrapping_add(children));
                assert_eq!(
                    state[0] as u64,
                    initial[0] as u64 ^ if children % 2 == 0 { 0 } else { kind.mask() }
                );
                assert_eq!(
                    trace
                        .windows(2)
                        .filter(|words| words[0] == 80)
                        .map(|words| words[1])
                        .collect::<Vec<_>>(),
                    planned.indices
                );
                for (marker, count) in [603, 605, 604, 606, 607, 608]
                    .into_iter()
                    .zip(planned.calls)
                {
                    assert_eq!(trace.iter().filter(|word| **word == marker).count(), count);
                }
                assert!(trace.ends_with(&[73, 82]));
                previous = Some(state);
            }
        }
        let prefix = cases::prefix(kind);
        let first = prefix[0].execute(None, true).unwrap().0;
        assert_eq!(first, [kind.last_state(), -1]);
        let accepted = prefix[1].execute(Some(first), true).unwrap().0;
        assert_eq!(accepted, [kind.initial(), 0]);
        let failures = cases::failures(kind);
        for (i, bad) in failures.into_iter().enumerate() {
            let trace = bad.execute(Some(accepted), true).unwrap_err();
            assert_ne!(bad.initial(), accepted);
            if i == 1 {
                assert!(!trace.contains(&75));
            }
            if i == 2 {
                assert!(trace.ends_with(&[73]));
                assert!(!trace.contains(&82));
            }
            if i == 3 {
                assert!(trace.ends_with(&[73, 82]));
            }
            if (4..28).contains(&i) {
                let index = i - 4;
                let slot = index / 12;
                let stage = index % 6;
                let late = index % 12 >= 6;
                let done = if late { 7 } else { 1 };
                let at = if late { 51 } else { 16 };
                assert_eq!(
                    trace.iter().filter(|word| **word == 75).count(),
                    done as usize
                );
                if stage == 3 {
                    assert!(trace.ends_with(&[
                        613 + slot as i64 * 2,
                        kind.last_state(),
                        1 ^ done,
                        0
                    ]));
                }
                if stage == 4 {
                    assert!(trace.ends_with(&[614 + slot as i64 * 2, kind.last_state(), 1, 0]));
                }
                if stage == 5 {
                    assert!(trace.ends_with(&[
                        if slot == 0 { 605 } else { 608 },
                        kind.last_state(),
                        done,
                        0
                    ]));
                }
                if stage < 3 {
                    assert!(!trace.contains(&(613 + slot as i64 * 2)));
                }
                if stage < 5 {
                    assert!(!trace.contains(&if slot == 0 { 605 } else { 608 }));
                }
                if stage == 2 {
                    assert!(trace.ends_with(&[610 + slot as i64 * 2, 0, 64 - at]));
                }
            }
        }
    }
}

#[test]
fn state_control_exit_native_distinguishes_event_control_decoy_from_accepted_state() {
    for kind in cases::KINDS {
        let mut open = cases::seed(kind);
        open.arguments.parents = 0;
        open.initial_control = 0;
        let mut event = cases::seed(kind);
        event.arguments.parents = 2;
        event.arguments.child.base.trips = 1;
        event.arguments.child.base.initial = kind.last_state();
        event.initial_control = 2;
        cases::controls(&mut event).flat.permit = 0;
        let normal = sequence(&[open, event]);
        assert!(normal.rows[1].trace.contains(&78));
        assert_eq!(normal.rows[1].state, [kind.last_state(), 1]);
        probe::check_sequences_with_i64_prints(
            &cases::SOURCE.replace("f64", kind.ty()),
            &[normal],
            &[],
            cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let mut replaced = sequence(&[open]);
        let (state, trace) = event
            .execute(Some([kind.initial(), event.initial_control]), true)
            .unwrap();
        assert!(!trace.contains(&78));
        assert_eq!(state, [kind.last_state(), 3]);
        replaced.rows.push(probe::Row {
            input: event.words(),
            state: state.to_vec(),
            trace,
        });
        let needle = "return work(state, selected,";
        assert_eq!(cases::SOURCE.matches(needle).count(), 1);
        let source = cases::SOURCE.replace(
            needle,
            "return work(State { total: state.total, control: initial_control }, selected,",
        );
        probe::check_sequences_with_i64_prints(
            &source.replace("f64", kind.ty()),
            &[replaced],
            &[],
            cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_control_exit_native_preserves_work_and_entry_limits_before_two_word_publication() {
    for kind in cases::KINDS {
        let source = cases::SOURCE.replace("f64", kind.ty());
        let seed = cases::seed(kind);
        let mut negative = seed;
        negative.arguments.child.base.initial = kind.last_state();
        let mut zero = seed;
        zero.arguments.child.base.initial = 0;
        for row in [seed, negative, zero] {
            let plan = row.plan(None);
            let work = row.arguments.parents + plan.requested.iter().sum::<i64>();
            let rows = [sequence(&[row])];
            probe::check_sequences_with_i64_prints(
                &source,
                &rows,
                &[],
                cases::observers(kind),
                work as u64,
                cases::ENTRY_LIMIT,
            );
            let trace = &rows[0].rows[0].trace;
            let at = trace.iter().rposition(|word| *word == 76).unwrap();
            probe::check_i64_loop_work_limit_rejection(
                &source,
                &rows,
                cases::observers(kind),
                work as u64 - 1,
                cases::ENTRY_LIMIT,
                &trace[..at + 6],
            );
        }
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &[sequence(&[seed])],
                cases::observers(kind),
                cases::LOOP_LIMIT,
                limit,
            );
        }
    }
}

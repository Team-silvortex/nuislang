#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "parent_reentry_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;
#[path = "state_break_argument_print.rs"]
mod state_break_argument_print;

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
                    state: vec![next],
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
            1 => Value::Bool(word != 0),
            3 | 4 => Value::I32(word as i32),
            _ => Value::Int(word),
        })
        .collect()
}

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

fn check_calls(row: cases::Case, previous: Option<i64>, events: &[String]) {
    let reserved = row.child_counts_from(previous);
    let completed = row.completed_child_counts_from(previous);
    assert_eq!(reserved.len(), completed.len());
    assert!(completed
        .iter()
        .zip(&reserved)
        .all(|(done, bound)| done <= bound));
    let children: i64 = completed.into_iter().sum();
    let calls = |name: &str| {
        events
            .iter()
            .filter(|event| event.contains("cpu.call") && event.contains(name))
            .count()
    };
    assert_eq!(
        calls("] checked64("),
        children as usize * (1 + usize::from(row.child.base.selected))
    );
    assert_eq!(calls("] finish("), (children + row.parents + 1) as usize);
    assert_eq!(
        calls("] child_count("),
        if matches!(
            row.clock,
            cases::GuardClock::ParentBound
                | cases::GuardClock::FallibleBound(_)
                | cases::GuardClock::StateBound(_)
                | cases::GuardClock::StateBreak(_, _)
        ) {
            row.parents as usize
        } else {
            0
        }
    );
}

#[test]
fn parent_reentry_argument_print_native_preserves_eager_initializers_and_fault_prefixes() {
    native_sequences(cases::PARENT);
}

fn native_sequences(matrix: cases::Matrix) {
    for kind in cases::KINDS {
        let prefix = (matrix.prefix)(kind);
        let previous = prefix.iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        let failures = (matrix.failures)(kind)
            .into_iter()
            .map(|bad| probe::SequenceFailure {
                accepted: sequence(&prefix),
                input: bad.words(),
                before_failure: bad.execute(previous, true).unwrap_err(),
            })
            .collect::<Vec<_>>();
        let rows = (matrix.sequences)(kind)
            .iter()
            .map(|rows| sequence(rows))
            .collect::<Vec<_>>();
        probe::check_sequences_with_i64_prints(
            &matrix.template.replace("f64", kind.ty()),
            &rows,
            &failures,
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn parent_reentry_argument_print_reference_preserves_counts_state_and_faulted_close() {
    reference_sequences(cases::PARENT);
}

fn reference_sequences(matrix: cases::Matrix) {
    for kind in cases::KINDS {
        let project = Project::with_source(&matrix.template.replace("f64", kind.ty()));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for rows in (matrix.sequences)(kind) {
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
                assert_eq!(state_words(session.state()), vec![previous as u64]);
                for row in &rows[1..] {
                    let (next, trace) = row.execute(Some(previous), false).unwrap();
                    let event = session.event(values(*row)).unwrap();
                    assert_eq!(printed(&event.events), trace);
                    assert_eq!(state_words(session.state()), vec![next as u64]);
                    check_calls(*row, Some(previous), &event.events);
                    previous = next;
                }
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
            }
            for bad in (matrix.failures)(kind) {
                let prefix = (matrix.prefix)(kind);
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0]),
                )
                .unwrap();
                let first = prefix[0].execute(None, false).unwrap().0;
                session.event(values(prefix[1])).unwrap();
                let previous = prefix[1].execute(Some(first), false).unwrap().0;
                assert!(session.event(values(bad)).is_err(), "{bad:?}");
                assert_eq!(state_words(session.state()), vec![previous as u64]);
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_eq!(state_words(session.state()), vec![previous as u64]);
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn parent_reentry_argument_print_native_keeps_entry_exhaustion_before_publication() {
    entry_exhaustion(cases::PARENT);
}

fn entry_exhaustion(matrix: cases::Matrix) {
    for kind in cases::KINDS {
        for (parents, trips, limit) in [(2, 2, 8), (3, 3, 64)] {
            let mut row = (matrix.seed)(kind);
            row.parents = parents;
            row.child.base.trips = trips;
            probe::check_i64_helper_entry_limit_rejection(
                &matrix.template.replace("f64", kind.ty()),
                &[sequence(&[row])],
                matrix.observers,
                cases::LOOP_LIMIT,
                limit,
            );
        }
    }
}

#[test]
fn parent_reentry_argument_print_oracle_pins_reseeding_skips_and_late_faults() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 22);
        assert_eq!(cases::cli_sequences(kind).len(), 6);
        assert_eq!(cases::failures(kind).len(), 6);
        let mut seed = cases::seed(kind);
        seed.child.base.left = -1;
        seed.child.base.right = 7;
        seed.child.base.mask = 0;
        let (state, trace) = seed.execute(None, true).unwrap();
        assert_eq!(state, seed.child.base.initial);
        let calls = trace
            .windows(3)
            .filter(|w| w[0] == 601)
            .map(|w| (w[1], w[2]))
            .collect::<Vec<_>>();
        // left, right within each child; left again on the next parent.
        assert_eq!(
            calls,
            vec![
                (4294967295, 4),
                (4294967295, 4),
                (4294967303, 4),
                (4294967303, 4),
                (4294967295, 3),
                (4294967295, 3),
                (4294967303, 3),
                (4294967303, 3),
            ]
        );
        for rows in cases::cli_sequences(kind) {
            for row in rows {
                assert!(row
                    .kind()
                    .decimal(row.child.base.initial)
                    .parse::<f64>()
                    .unwrap()
                    .is_finite());
            }
        }
        for rows in cases::cli_sequences(kind).into_iter().take(2) {
            let (state, trace) = rows[0].execute(None, true).unwrap();
            assert_eq!(state, rows[0].child.base.initial);
            assert!(!trace.contains(&601));
            assert_eq!(trace.last(), Some(&73));
        }
        for (i, bad) in cases::failures(kind).into_iter().enumerate() {
            let trace = bad.execute(None, true).unwrap_err();
            let completed = if i == 3 {
                2
            } else if i == 4 {
                3
            } else {
                1
            };
            assert_eq!(trace.iter().filter(|w| **w == 77).count(), completed);
            if i == 4 {
                assert_eq!(trace.last(), Some(&73));
            } else {
                assert!(!trace.contains(&73));
            }
            if i == 5 {
                assert!(trace.ends_with(&[601, 0, 0]));
            }
        }
    }
}

#[test]
fn child_progress_argument_print_native_preserves_completed_siblings_and_fault_prefixes() {
    native_sequences(cases::CHILD_PROGRESS);
}

#[test]
fn child_progress_argument_print_reference_preserves_counts_state_and_faulted_close() {
    reference_sequences(cases::CHILD_PROGRESS);
}

#[test]
fn child_progress_argument_print_native_keeps_entry_exhaustion_before_publication() {
    entry_exhaustion(cases::CHILD_PROGRESS);
}

#[test]
fn child_progress_argument_print_oracle_pins_late_children_and_lazy_faults() {
    let matrix = cases::CHILD_PROGRESS;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 22);
        assert_eq!((matrix.cli_sequences)(kind).len(), 6);
        assert_eq!((matrix.failures)(kind).len(), 9);
        let mut row = (matrix.seed)(kind);
        row.parents = 2;
        row.child.base.trips = 2;
        row.child.base.left = -1;
        row.child.base.right = 7;
        row.child.base.mask = 0;
        let mut parent = row;
        parent.clock = cases::GuardClock::Parent;
        assert_eq!(row.words(), parent.words());
        assert_eq!(row.args(), parent.args());
        assert_eq!(row.words().len(), 11);
        let (state, trace) = row.execute(None, true).unwrap();
        assert_eq!(state, row.child.base.initial);
        let calls = trace
            .windows(3)
            .filter(|w| w[0] == 601)
            .map(|w| (w[1], w[2]))
            .collect::<Vec<_>>();
        assert_eq!(
            calls,
            vec![
                (4294967295, 10),
                (4294967295, 10),
                (4294967303, 9),
                (4294967303, 9),
                (4294967295, 8),
                (4294967295, 8),
                (4294967303, 7),
                (4294967303, 7),
            ]
        );
        for (i, rows) in (matrix.cli_sequences)(kind).into_iter().enumerate() {
            for row in &rows {
                assert!(row
                    .kind()
                    .decimal(row.child.base.initial)
                    .parse::<f64>()
                    .unwrap()
                    .is_finite());
            }
            let (_, trace) = rows[0].execute(None, true).unwrap();
            if i < 2 {
                assert!(!trace.contains(&601));
            }
            if i == 4 {
                assert_eq!(trace.iter().filter(|w| **w == 601).count(), 6);
                assert_eq!(trace.iter().filter(|w| **w == 75).count(), 6);
            }
        }
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(None, true).unwrap_err();
            let completed = [4, 5, 4, 5, 4, 5, 7, 7, 9][i];
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), completed);
            assert_eq!(trace.iter().filter(|w| **w == 77).count(), completed / 3);
            assert_eq!(
                trace.iter().filter(|w| **w == 71).count(),
                completed + usize::from(i != 8)
            );
            if i == 8 {
                assert_eq!(trace.last(), Some(&73));
            } else {
                assert!(!trace.contains(&73));
            }
            if i == 4 || i == 5 {
                assert_eq!(trace.last(), Some(&74));
            }
            if i == 7 {
                assert!(trace.ends_with(&[601, 0, 0]));
            }
        }
    }
}

#[test]
fn parent_bound_argument_print_native_preserves_skip_resume_and_fault_prefixes() {
    native_sequences(cases::PARENT_BOUND);
}

#[test]
fn parent_bound_argument_print_reference_preserves_dynamic_counts_and_faulted_close() {
    reference_sequences(cases::PARENT_BOUND);
}

#[test]
fn parent_bound_argument_print_native_keeps_entry_and_exact_work_budgets() {
    let matrix = cases::PARENT_BOUND;
    for kind in cases::KINDS {
        let row = (matrix.seed)(kind);
        let source = matrix.template.replace("f64", kind.ty());
        let rows = [sequence(&[row])];
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &rows,
                matrix.observers,
                cases::LOOP_LIMIT,
                limit,
            );
        }
        assert_eq!(row.child_counts(), vec![0, 3, 0, 5]);
        assert_eq!(row.parents + row.child_counts().iter().sum::<i64>(), 12);
        let trace = &rows[0].rows[0].trace;
        let last_parent = trace.iter().rposition(|word| *word == 76).unwrap();
        let prefix = &trace[..last_parent + 4];
        assert!(prefix.ends_with(&[76, 602, 3, 3]));
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &rows,
            matrix.observers,
            11,
            cases::ENTRY_LIMIT,
            prefix,
        );
    }
}

#[test]
fn parent_bound_argument_print_oracle_pins_bounds_reseeding_and_skipped_poison() {
    let matrix = cases::PARENT_BOUND;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 24);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 9);
        let mut row = (matrix.seed)(kind);
        row.child.base.left = -1;
        row.child.base.right = 7;
        row.child.base.mask = 0;
        let (state, trace) = row.execute(None, true).unwrap();
        assert_eq!(state, row.child.base.initial);
        assert_eq!(row.child_counts(), vec![0, 3, 0, 5]);
        let bounds = trace
            .windows(3)
            .filter(|w| w[0] == 602)
            .map(|w| (w[1], w[2]))
            .collect::<Vec<_>>();
        assert_eq!(bounds, vec![(3, 0), (3, 1), (3, 2), (3, 3)]);
        let calls = trace
            .windows(3)
            .filter(|w| w[0] == 601)
            .map(|w| (w[1], w[2]))
            .collect::<Vec<_>>();
        let pins = [
            (4294967295, 48),
            (4294967303, 47),
            (4294967303, 46),
            (4294967295, 16),
            (4294967303, 15),
            (4294967303, 14),
            (4294967303, 13),
            (4294967303, 12),
        ];
        assert_eq!(
            calls,
            pins.into_iter()
                .flat_map(|pin| [pin, pin])
                .collect::<Vec<_>>()
        );
        for (i, rows) in (matrix.cli_sequences)(kind).into_iter().enumerate() {
            for row in &rows {
                assert!(row
                    .kind()
                    .decimal(row.child.base.initial)
                    .parse::<f64>()
                    .unwrap()
                    .is_finite());
            }
            let (_, trace) = rows[0].execute(None, true).unwrap();
            if i < 2 {
                assert!(!trace.contains(&601));
            }
            if i == 2 {
                assert_eq!(rows[0].child_counts(), vec![0, 0, 0, 2]);
            }
            if i == 3 {
                assert_eq!(rows[0].child_counts(), vec![0, 1, 0, 3]);
            }
            if i == 5 {
                assert_eq!(trace.iter().filter(|word| **word == 601).count(), 6);
            }
        }
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(None, true).unwrap_err();
            let completed = [4, 5, 4, 5, 4, 5, 7, 7, 8][i];
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), completed);
            assert_eq!(
                trace.iter().filter(|w| **w == 77).count(),
                if i == 8 { 4 } else { 3 }
            );
            assert_eq!(
                trace.iter().filter(|w| **w == 71).count(),
                completed + usize::from(i != 8)
            );
            if i == 8 {
                assert_eq!(trace.last(), Some(&73));
            } else {
                assert!(!trace.contains(&73));
            }
            if i == 4 || i == 5 {
                assert_eq!(trace.last(), Some(&74));
            }
            if i == 7 {
                assert!(trace.ends_with(&[601, 0, 0]));
            }
        }
    }
}

#[test]
fn fallible_bound_argument_print_native_preserves_producer_fault_prefixes() {
    native_sequences(cases::FALLIBLE_BOUND);
}

#[test]
fn fallible_bound_argument_print_reference_preserves_last_state_and_faulted_close() {
    reference_sequences(cases::FALLIBLE_BOUND);
}

#[test]
fn fallible_bound_argument_print_native_orders_producer_before_child_reservation() {
    let matrix = cases::FALLIBLE_BOUND;
    for kind in cases::KINDS {
        let row = (matrix.seed)(kind);
        let source = matrix.template.replace("f64", kind.ty());
        let rows = [sequence(&[row])];
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &rows,
                matrix.observers,
                cases::LOOP_LIMIT,
                limit,
            );
        }
        let trace = &rows[0].rows[0].trace;
        let last_parent = trace.iter().rposition(|word| *word == 76).unwrap();
        let prefix = &trace[..last_parent + 5];
        assert!(prefix.ends_with(&[76, 602, 3, 3, 2]));
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &rows,
            matrix.observers,
            11,
            cases::ENTRY_LIMIT,
            prefix,
        );

        let mut accepted = row;
        accepted.parents = 2;
        accepted.child.base.trips = 1;
        let previous = accepted.execute(None, true).unwrap().0;
        let failures = (matrix.failures)(kind)
            .into_iter()
            .take(5)
            .map(|bad| probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: bad.words(),
                before_failure: bad.execute(Some(previous), true).unwrap_err(),
            })
            .collect::<Vec<_>>();
        // The fourth producer must fault before the same 11-unit child reservation.
        // This probe also rejects either budget-exhaustion diagnostic on source faults.
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[accepted])],
            &failures,
            matrix.observers,
            11,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn fallible_bound_argument_print_oracle_pins_reached_zero_bounds_and_abi_isolation() {
    let matrix = cases::FALLIBLE_BOUND;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 24);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 14);
        let row = (matrix.seed)(kind);
        let mut old = row;
        old.clock = cases::GuardClock::ParentBound;
        assert_eq!(old.words().len(), 11);
        assert_eq!(row.words().len(), 12);
        assert_eq!(row.words()[..11], old.words());
        assert_eq!(row.args(), format!("{},5", old.args()));
        let (_, trace) = row.execute(None, true).unwrap();
        let calls = trace
            .windows(4)
            .filter(|w| w[0] == 602)
            .map(|w| (w[1], w[2], w[3]))
            .collect::<Vec<_>>();
        assert_eq!(calls, vec![(3, 0, 5), (3, 1, 4), (3, 2, 3), (3, 3, 2)]);
        let finite = (matrix.cli_sequences)(kind);
        let (_, absent) = finite[0][0].execute(None, true).unwrap();
        assert_eq!(absent, vec![90, 70, 73]);
        let (_, zero) = finite[1][0].execute(None, true).unwrap();
        assert_eq!(zero.iter().filter(|w| **w == 602).count(), 3);
        assert!(!zero.contains(&71));
        for (i, bad) in (matrix.failures)(kind).into_iter().take(5).enumerate() {
            let parent = [0, 1, 2, 3, 1][i];
            let completed = [0, 0, 3, 3, 0][i];
            let trace = bad.execute(None, true).unwrap_err();
            assert!(trace.ends_with(&[76, 602, bad.child.base.trips, parent, 0]));
            assert_eq!(
                trace.iter().filter(|w| **w == 602).count(),
                parent as usize + 1
            );
            assert_eq!(trace.iter().filter(|w| **w == 77).count(), parent as usize);
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), completed);
            assert_eq!(trace.iter().filter(|w| **w == 71).count(), completed);
            assert!(!trace.contains(&73));
            assert_eq!(
                bad.execute(Some(kind.initial()), false).unwrap_err().last(),
                Some(&76)
            );
        }
    }
}

#[test]
fn state_bound_argument_print_native_uses_changed_carry_and_accepted_state() {
    native_sequences(cases::STATE_BOUND);
}

#[test]
fn state_bound_argument_print_reference_preserves_state_dependent_counts_and_faults() {
    reference_sequences(cases::STATE_BOUND);
}

#[test]
fn state_bound_argument_print_native_keeps_state_dependent_work_and_fault_order() {
    let matrix = cases::STATE_BOUND;
    for kind in cases::KINDS {
        let row = (matrix.seed)(kind);
        let source = matrix.template.replace("f64", kind.ty());
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[row])],
            &[],
            matrix.observers,
            11,
            cases::ENTRY_LIMIT,
        );
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &[sequence(&[row])],
                matrix.observers,
                cases::LOOP_LIMIT,
                limit,
            );
        }
        let mut negative = row;
        negative.child.base.initial = kind.last_state();
        for (row, limit) in [(row, 10), (negative, 11)] {
            let rows = [sequence(&[row])];
            let trace = &rows[0].rows[0].trace;
            let last_parent = trace.iter().rposition(|word| *word == 76).unwrap();
            let prefix = &trace[..last_parent + 6];
            assert!(prefix.ends_with(&[76, 602, 3, 3, 2, 1]));
            probe::check_i64_loop_work_limit_rejection(
                &source,
                &rows,
                matrix.observers,
                limit,
                cases::ENTRY_LIMIT,
                prefix,
            );
        }
        let mut accepted = row;
        accepted.parents = 0;
        let mut bad = row;
        bad.clock = cases::GuardClock::StateBound(3);
        let failure = probe::SequenceFailure {
            accepted: sequence(&[accepted]),
            input: bad.words(),
            before_failure: bad.execute(Some(kind.initial()), true).unwrap_err(),
        };
        assert!(failure.before_failure.ends_with(&[76, 602, 3, 3, 0, 1]));
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[accepted])],
            &[failure],
            matrix.observers,
            10,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_bound_argument_print_oracle_pins_backedges_decoys_and_ieee_classes() {
    let matrix = cases::STATE_BOUND;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 27);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 14);
        let row = (matrix.seed)(kind);
        assert_eq!(row.child_counts_from(None), vec![0, 3, 0, 4]);
        assert_eq!(
            row.child_counts_from(Some(kind.last_state())),
            vec![0, 4, 0, 4]
        );
        let mut old = row;
        old.clock = cases::GuardClock::FallibleBound(5);
        assert_eq!(row.words(), old.words());
        assert_eq!(row.args(), old.args());
        let (accepted, trace) = row.execute(None, true).unwrap();
        assert_eq!(accepted, kind.last_state());
        let calls = |trace: &[i64]| {
            trace
                .windows(5)
                .filter(|w| w[0] == 602)
                .map(|w| (w[2], w[3], w[4]))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            calls(&trace),
            vec![(0, 5, 0), (1, 4, 0), (2, 3, 1), (3, 2, 1)]
        );
        let (next, trace) = row.execute(Some(accepted), true).unwrap();
        assert_eq!(next, accepted);
        assert_eq!(
            calls(&trace),
            vec![(0, 5, 1), (1, 4, 1), (2, 3, 1), (3, 2, 1)]
        );
        for (bits, count) in [(0, 3), (kind.nan(), 3), (1, 4)] {
            let previous = (bits as u64 ^ kind.mask()) as i64;
            assert_eq!(
                row.child_counts_from(Some(previous)),
                vec![0, count, 0, count]
            );
            assert_eq!(row.execute(Some(previous), true).unwrap().0, previous);
        }
        let prefix = (matrix.prefix)(kind);
        let previous = prefix.iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            let completed = [0, 0, 4, 4, 5, 6, 5, 6, 5, 6, 7, 7, 8, 1][i];
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), completed);
            if i < 4 {
                assert!(trace.ends_with(&[76, 602, 3, i as i64, 0, 1]));
            }
            if i == 12 {
                assert_eq!(trace.last(), Some(&73));
            } else {
                assert!(!trace.contains(&73));
            }
            if i == 13 {
                assert_eq!(bad.child_counts_from(previous), vec![0, 1, 0, 0]);
                assert!(trace.ends_with(&[76, 602, 0, 3, 0, 0]));
            }
        }
    }
}

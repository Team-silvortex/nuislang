use super::*;
use cases::state_grouped::controls as exits;

#[test]
fn state_grouped_argument_print_native_preserves_nested_calls_and_faults() {
    native_sequences(cases::STATE_GROUPED);
}

#[test]
fn state_grouped_argument_print_reference_counts_selected_inner_operands() {
    reference_sequences(cases::STATE_GROUPED);
}

#[test]
fn state_grouped_argument_print_native_observes_regrouped_faults() {
    let matrix = cases::STATE_GROUPED;
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty());
        let mut row = (matrix.cli_sequences)(kind)[5][0];
        row.parents = 2;
        exits(&mut row).rescue = 0;
        exits(&mut row).rescue_guard = 16;
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[row])],
            &[],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let mut accepted = row;
        accepted.parents = 0;
        let regrouped = source.replace("continue_at(child, cut, continue_guard - (outer * 16 + child)) && (confirm_continue(child, permit, confirm_guard - (outer * 16 + child)) || rescue_continue(child, rescue, rescue_guard - (outer * 16 + child)))", "(continue_at(child, cut, continue_guard - (outer * 16 + child)) && confirm_continue(child, permit, confirm_guard - (outer * 16 + child))) || rescue_continue(child, rescue, rescue_guard - (outer * 16 + child))");
        assert_ne!(regrouped, source);
        let trace = row.execute(Some(kind.initial()), true).unwrap().1;
        let at = trace.windows(4).position(|w| w == [604, 0, 0, 48]).unwrap();
        let mut failure = trace[..at].to_vec();
        failure.extend([607, 0, 0, 0]);
        probe::check_sequences_with_i64_prints(
            &regrouped,
            &[sequence(&[accepted])],
            &[probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: row.words(),
                before_failure: failure,
            }],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );

        let regrouped = source.replace("(break_at(child, stop_at, break_guard - (outer * 16 + child)) || fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child))) && confirm_break(child, seal, seal_guard - (outer * 16 + child))", "break_at(child, stop_at, break_guard - (outer * 16 + child)) || (fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child)) && confirm_break(child, seal, seal_guard - (outer * 16 + child)))");
        assert_ne!(regrouped, source);
        let mut poisoned = row;
        exits(&mut poisoned).seal_guard = 16;
        let fault = poisoned.execute(None, true).unwrap_err();
        assert!(fault.ends_with(&[608, 0, 0, 0]));
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[accepted])],
            &[probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: poisoned.words(),
                before_failure: poisoned.execute(Some(kind.initial()), true).unwrap_err(),
            }],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let mut skipped = sequence(&[row]);
        skipped.rows[0].input = poisoned.words();
        let at = skipped.rows[0]
            .trace
            .windows(4)
            .position(|w| w == [608, 0, 0, 48])
            .unwrap();
        skipped.rows[0].trace.drain(at..at + 4);
        probe::check_sequences_with_i64_prints(
            &regrouped,
            &[skipped],
            &[],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_grouped_argument_print_native_keeps_changed_work_and_fault_priority() {
    let matrix = cases::STATE_GROUPED;
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty());
        let seed = (matrix.seed)(kind);
        let mut negative = seed;
        negative.child.base.initial = kind.last_state();
        exits(&mut negative).flat.stop = 3;
        exits(&mut negative).seal = 3;
        let mut fallback = seed;
        exits(&mut fallback).flat.stop = 99;
        exits(&mut fallback).flat.fallback = 1;
        exits(&mut fallback).seal = 1;
        assert_eq!(fallback.child_counts(), [0, 3, 0, 3]);
        for (row, limit, negative_flag) in [(seed, 11, 1), (negative, 12, 1), (fallback, 10, 0)] {
            let rows = [sequence(&[row])];
            probe::check_sequences_with_i64_prints(
                &source,
                &rows,
                &[],
                matrix.observers,
                limit,
                cases::ENTRY_LIMIT,
            );
            let trace = &rows[0].rows[0].trace;
            let parent = trace.iter().rposition(|w| *w == 76).unwrap();
            let prefix = &trace[..parent + 6];
            assert!(prefix.ends_with(&[76, 602, 3, 3, 2, negative_flag]));
            probe::check_i64_loop_work_limit_rejection(
                &source,
                &rows,
                matrix.observers,
                limit - 1,
                cases::ENTRY_LIMIT,
                prefix,
            );
        }
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &[sequence(&[seed])],
                matrix.observers,
                cases::LOOP_LIMIT,
                limit,
            );
        }
        let mut accepted = seed;
        accepted.parents = 0;
        let mut bad = fallback;
        if let cases::GuardClock::StateGrouped { guard, exits, .. } = &mut bad.clock {
            *guard = 3;
            exits.flat.continue_guard = 48;
        }
        let failure = probe::SequenceFailure {
            accepted: sequence(&[accepted]),
            input: bad.words(),
            before_failure: bad.execute(Some(kind.initial()), true).unwrap_err(),
        };
        assert!(failure.before_failure.ends_with(&[76, 602, 3, 3, 0, 0]));
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[accepted])],
            &[failure],
            matrix.observers,
            9,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_grouped_argument_print_oracle_pins_truth_tables_and_abi() {
    let matrix = cases::STATE_GROUPED;
    for kind in cases::KINDS {
        assert_eq!((matrix.cli_sequences)(kind).len(), 12);
        assert_eq!((matrix.sequences)(kind).len(), 31);
        assert_eq!((matrix.failures)(kind).len(), 20);
        for (permit, rescue, stop, fallback, seal, counts) in [
            (0, -1, 2, -1, 2, [3, 1, 2, 1, 0, 1]),
            (99, 0, 2, -1, 2, [3, 1, 2, 1, 1, 1]),
            (99, 99, 0, -1, 0, [1, 1, 1, 0, 1, 1]),
            (0, -1, 1, 2, 2, [3, 1, 2, 1, 0, 2]),
        ] {
            let mut row = (matrix.seed)(kind);
            let gates = exits(&mut row);
            gates.flat.permit = permit;
            gates.rescue = rescue;
            gates.flat.stop = stop;
            gates.flat.fallback = fallback;
            gates.seal = seal;
            let gates = *gates;
            let completed = if stop == 0 { 1 } else { 3 };
            assert_eq!(gates.call_counts(&[completed]), counts);
            let (state, trace) = row.execute(None, true).unwrap();
            assert_eq!(state, kind.initial());
            let expected = counts.map(|n| n * 2);
            assert_eq!(row.grouped_calls_from(None), expected);
            for (marker, count) in [603, 605, 604, 606, 607, 608].into_iter().zip(expected) {
                assert_eq!(trace.iter().filter(|w| **w == marker).count(), count);
            }
            assert_eq!(
                trace
                    .windows(2)
                    .filter(|w| w[0] == 80)
                    .map(|w| w[1])
                    .collect::<Vec<_>>(),
                [0, completed - 1, 0, completed - 1]
            );
            let mut predecessor = row;
            predecessor.clock = cases::GuardClock::StateShortCircuit {
                guard: 5,
                suffix_guard: 64,
                exits: gates.flat,
            };
            assert_eq!(row.words().len(), 25);
            assert_eq!(row.words()[..21], predecessor.words());
            assert_eq!(
                row.args(),
                format!("{},{rescue},64,{seal},64", predecessor.args())
            );
        }
        let mut skipped = (matrix.seed)(kind);
        exits(&mut skipped).rescue_guard = 16;
        exits(&mut skipped).seal_guard = 17;
        assert!(skipped.execute(None, true).is_ok());
        exits(&mut skipped).flat.permit = 99;
        assert!(skipped
            .execute(None, true)
            .unwrap_err()
            .ends_with(&[607, 0, -1, 0]));
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        let completed = [0, 4, 8, 1, 1, 2, 0, 1, 2, 4, 5, 2, 1, 4, 1, 3, 6, 2, 3, 2];
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            assert_eq!(
                trace.iter().filter(|w| **w == 75).count(),
                completed[i],
                "fault {i}"
            );
            let tail = match i {
                12 | 13 => Some([607, 0, 0, 0]),
                14 => Some([607, 0, -1, 0]),
                15 | 16 => Some([608, 2, 2, 0]),
                17 => Some([608, 1, 1, 0]),
                18 => Some([608, 2, 99, 0]),
                19 => Some([606, 1, 1, 0]),
                _ => None,
            };
            if let Some(tail) = tail {
                assert!(trace.ends_with(&tail), "fault {i}");
            }
            if i == 6 {
                assert!(!trace.iter().any(|w| (603..=608).contains(w)));
            }
        }
    }
}

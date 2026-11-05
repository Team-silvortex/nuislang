use super::*;

fn exits(row: &mut cases::Case) -> &mut cases::ShortCircuitExits {
    let cases::GuardClock::StateShortCircuit { exits, .. } = &mut row.clock else {
        panic!("short-circuit profile");
    };
    exits
}

#[test]
fn state_short_circuit_argument_print_native_preserves_selected_calls_and_faults() {
    native_sequences(cases::STATE_SHORT_CIRCUIT);
}

#[test]
fn state_short_circuit_argument_print_reference_counts_each_operand_once() {
    reference_sequences(cases::STATE_SHORT_CIRCUIT);
}

#[test]
fn state_short_circuit_argument_print_native_observes_swapped_operand_faults() {
    let matrix = cases::STATE_SHORT_CIRCUIT;
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty());
        let mut row = (matrix.cli_sequences)(kind)[12][0];
        row.parents = 2;
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
        let trace = row.execute(Some(kind.initial()), true).unwrap().1;
        for (left, op, right, original, failed) in [
            (
                "continue_at(child, cut, continue_guard - (outer * 16 + child))",
                "&&",
                "confirm_continue(child, permit, confirm_guard - (outer * 16 + child))",
                [603, 0, 99, 48],
                [605, 0, 0, 0],
            ),
            (
                "break_at(child, stop_at, break_guard - (outer * 16 + child))",
                "||",
                "fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child))",
                [604, 0, 0, 48],
                [606, 0, 0, 0],
            ),
        ] {
            let swapped = source.replace(
                &format!("{left} {op} {right}"),
                &format!("{right} {op} {left}"),
            );
            assert_ne!(swapped, source);
            let at = trace
                .windows(4)
                .position(|words| words == original)
                .unwrap();
            let mut prefix = trace[..at].to_vec();
            prefix.extend(failed);
            probe::check_sequences_with_i64_prints(
                &swapped,
                &[sequence(&[accepted])],
                &[probe::SequenceFailure {
                    accepted: sequence(&[accepted]),
                    input: row.words(),
                    before_failure: prefix,
                }],
                matrix.observers,
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

#[test]
fn state_short_circuit_argument_print_native_keeps_changed_work_and_fault_priority() {
    let matrix = cases::STATE_SHORT_CIRCUIT;
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty());
        let seed = (matrix.seed)(kind);
        let mut negative = seed;
        negative.child.base.initial = kind.last_state();
        exits(&mut negative).stop = 3;
        let mut fallback = seed;
        exits(&mut fallback).stop = 99;
        exits(&mut fallback).fallback = 1;
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
        if let cases::GuardClock::StateShortCircuit { guard, exits, .. } = &mut bad.clock {
            *guard = 3;
            exits.continue_guard = 48;
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
fn state_short_circuit_argument_print_oracle_pins_lazy_rhs_and_changed_state() {
    let matrix = cases::STATE_SHORT_CIRCUIT;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 35);
        assert_eq!((matrix.cli_sequences)(kind).len(), 16);
        assert_eq!((matrix.failures)(kind).len(), 38);
        for (cut, permit, stop, fallback, counts, suffixes, indices, negative) in [
            (0, 0, 2, -1, [6, 2, 4, 2], 2, [0, 2, 0, 2], false),
            (0, 99, 0, -1, [2, 2, 2, 0], 0, [0, 0, 0, 0], false),
            (0, 0, 99, 1, [4, 2, 2, 2], 0, [0, 1, 0, 1], false),
            (99, 0, 0, 0, [2, 0, 2, 0], 0, [0, 0, 0, 0], false),
            (0, 0, 0, 0, [7, 2, 5, 5], 5, [0, 3, 0, 4], true),
            (0, 0, 1, 1, [4, 2, 2, 0], 0, [0, 1, 0, 1], false),
            (-1, -1, -1, -1, [7, 0, 7, 7], 7, [0, 3, 0, 4], true),
            (99, 99, 99, 99, [7, 0, 7, 7], 7, [0, 3, 0, 4], true),
        ] {
            let mut row = (matrix.seed)(kind);
            let gates = exits(&mut row);
            gates.cut = cut;
            gates.permit = permit;
            gates.stop = stop;
            gates.fallback = fallback;
            let (state, trace) = row.execute(None, true).unwrap();
            assert_eq!(
                state,
                if negative {
                    kind.last_state()
                } else {
                    kind.initial()
                }
            );
            assert_eq!(row.predicate_calls_from(None), (counts[0], counts[2]));
            assert_eq!(row.right_predicate_calls_from(None), (counts[1], counts[3]));
            for (marker, expected) in [603, 605, 604, 606].into_iter().zip(counts) {
                assert_eq!(trace.iter().filter(|w| **w == marker).count(), expected);
            }
            assert_eq!(row.suffix_calls_from(None), suffixes);
            assert_eq!(
                trace
                    .windows(2)
                    .filter(|w| w[0] == 80)
                    .map(|w| w[1])
                    .collect::<Vec<_>>(),
                indices
            );
            let mut predecessor = row;
            predecessor.clock = cases::GuardClock::StatePredicate {
                guard: 5,
                cut,
                suffix_guard: 64,
                stop,
                continue_guard: 64,
                break_guard: 64,
            };
            assert_eq!(row.words().len(), 21);
            assert_eq!(row.words()[..17], predecessor.words());
            assert_eq!(
                row.args(),
                format!("{},{permit},64,{fallback},64", predecessor.args())
            );
        }
        for rows in (matrix.cli_sequences)(kind) {
            let mut previous = None;
            for row in rows {
                previous = Some(row.execute(previous, true).unwrap().0);
            }
        }
        for confirm in [true, false] {
            let mut poison = (matrix.cli_sequences)(kind)[12][0];
            assert!(poison.execute(None, true).is_ok());
            if confirm {
                exits(&mut poison).cut = 0;
            } else {
                exits(&mut poison).stop = 99;
            }
            assert!(poison.execute(None, true).unwrap_err().ends_with(&[
                if confirm { 605 } else { 606 },
                0,
                0,
                0
            ]));
        }
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        let done = [
            0, 0, 4, 4, 5, 6, 5, 6, 5, 6, 7, 7, 8, 1, 6, 7, 1, 7, 4, 1, 2, 1, 1, 1, 2, 2, 4, 6, 0,
            2, 1, 2, 4, 5, 2, 1, 1, 2,
        ];
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            assert_eq!(
                trace.iter().filter(|w| **w == 75).count(),
                done[i],
                "fault {i}"
            );
            let tail = match i {
                30 | 32 => Some([605, 0, 0, 0]),
                31 | 33 => Some([606, 1, -1, 0]),
                34 => Some([606, 1, 1, 0]),
                35 => Some([605, 0, 99, 0]),
                36 => Some([603, 0, 0, 0]),
                37 => Some([604, 1, 2, 0]),
                _ => None,
            };
            if let Some(tail) = tail {
                assert!(trace.ends_with(&tail), "fault {i}");
            }
            if i == 28 {
                assert!(!trace.iter().any(|word| (603..=606).contains(word)));
            }
        }
    }
}

use super::*;
use cases::state_field_exits::{controls, matches, matrix, plan};

#[test]
fn state_field_exit_argument_print_native_uses_updated_bits_and_accepted_state() {
    for kind in cases::KINDS {
        native_sequences_for_kind(matrix(kind), kind);
    }
}

#[test]
fn state_field_exit_argument_print_reference_counts_dynamic_predicates() {
    reference_sequences(cases::STATE_FIELD_EXIT);
}

#[test]
fn state_field_exit_argument_print_native_preserves_ieee_operand_faults() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let source = matrix.template.replace("f64", kind.ty());
        let mut healthy = Vec::new();
        let mut failures = Vec::new();
        for (bits, mode) in [
            (0, 1),
            (kind.mask() as i64, 1),
            (kind.nan(), 2),
            ((kind.nan() as u64 | kind.mask()) as i64, 2),
        ] {
            let mut row = (matrix.seed)(kind);
            row.child.base.initial = bits;
            let gates = controls(&mut row);
            gates.flat.permit = mode;
            gates.seal = mode;
            gates.rescue_guard = 16;
            gates.flat.fallback_guard = 18;
            if let cases::GuardClock::StateFieldExit { suffix_guard, .. } = &mut row.clock {
                *suffix_guard = 16;
            }
            let updated = (bits as u64 ^ kind.mask()) as i64;
            let result = sequence(&[row]);
            assert_eq!(result.rows[0].state, [bits]);
            assert!(result.rows[0]
                .trace
                .windows(4)
                .any(|w| w == [605, updated, mode, 48]));
            healthy.push(result);
            let mut accepted = row;
            accepted.parents = 0;
            let mut bad = row;
            controls(&mut bad).seal_guard = 18;
            let trace = bad.execute(Some(bits), true).unwrap_err();
            assert!(trace.ends_with(&[608, updated, mode, 0]));
            failures.push(probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: bad.words(),
                before_failure: trace,
            });
        }
        probe::check_sequences_with_i64_prints(
            &source,
            &healthy,
            &failures,
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_field_exit_argument_print_native_keeps_state_driven_budgets() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let source = matrix.template.replace("f64", kind.ty());
        let seed = (matrix.seed)(kind);
        let mut negative = seed;
        negative.child.base.initial = kind.last_state();
        let mut zero = seed;
        zero.child.base.initial = 0;
        controls(&mut zero).flat.permit = 1;
        controls(&mut zero).flat.stop = 1;
        controls(&mut zero).seal = 1;
        for (row, limit, flag) in [(seed, 11, 1), (negative, 12, 1), (zero, 10, 0)] {
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
            let at = trace.iter().rposition(|w| *w == 76).unwrap();
            let prefix = &trace[..at + 6];
            assert!(prefix.ends_with(&[76, 602, 3, 3, 2, flag]));
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
        let mut bad = seed;
        if let cases::GuardClock::StateFieldExit { guard, exits, .. } = &mut bad.clock {
            *guard = 3;
            exits.flat.continue_guard = 48;
        }
        let trace = bad.execute(Some(kind.initial()), true).unwrap_err();
        assert!(trace.ends_with(&[76, 602, 3, 3, 0, 1]));
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[accepted])],
            &[probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: bad.words(),
                before_failure: trace,
            }],
            matrix.observers,
            10,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_field_exit_argument_print_native_charges_outlined_classifier_entries() {
    let total = "return (mode == 0 && value < 0.0) || (mode == 1 && value == 0.0) || (mode != 0 && mode != 1 && value != value);";
    let outlined = "if mode == 0 { return value < 0.0; } if mode == 1 { return value == 0.0; } return value != value;";
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        assert_eq!(matrix.template.matches(total).count(), 2);
        let source = matrix
            .template
            .replace(total, outlined)
            .replace("f64", kind.ty());
        let mut row = (matrix.seed)(kind);
        row.child.base.initial = kind.last_state();
        probe::check_i64_helper_entry_limit_rejection(
            &source,
            &[sequence(&[row])],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_field_exit_argument_print_oracle_pins_ieee_schedule_and_fault_order() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        assert_eq!((matrix.sequences)(kind).len(), 42);
        assert_eq!((matrix.cli_sequences)(kind).len(), 12);
        assert_eq!((matrix.failures)(kind).len(), 22);
        let infinity = if kind == cases::Kind::F32 {
            0x7f800000
        } else {
            0x7ff0000000000000
        };
        for (magnitude, zero, nan) in [
            (0, true, false),
            (1, false, false),
            (infinity, false, false),
            (kind.nan() as u64, false, true),
        ] {
            for sign in [0, kind.mask()] {
                let bits = (magnitude | sign) as i64;
                assert_eq!(matches(kind, bits, 0), sign != 0 && !zero && !nan);
                assert_eq!(matches(kind, bits, 1), zero);
                assert_eq!(matches(kind, bits, 2), nan);
            }
        }
        let seed = (matrix.seed)(kind);
        let schedule = plan(seed, None);
        assert_eq!(schedule.requested, [0, 3, 0, 4]);
        assert_eq!(schedule.completed, [0, 3, 0, 4]);
        assert_eq!(schedule.indices, [0, 2, 0, 4]);
        assert_eq!(schedule.calls, [7, 2, 6, 4, 1, 2]);
        assert_eq!(schedule.suffixes, 5);
        assert_eq!(schedule.state, kind.last_state());
        let subsequent = plan(seed, Some(kind.last_state()));
        assert_eq!(subsequent.completed, [0, 4, 0, 4]);
        assert_eq!(subsequent.indices, [0, 4, 0, 4]);
        assert_eq!(subsequent.calls, [8, 2, 8, 6, 2, 2]);
        assert_eq!(subsequent.suffixes, 8);
        assert_eq!(subsequent.state, kind.last_state());
        for rows in (matrix.sequences)(kind) {
            let mut previous = None;
            for row in rows {
                let expected = plan(row, previous);
                let (state, trace) = row.execute(previous, true).unwrap();
                assert_eq!(state, expected.state);
                assert_eq!(row.grouped_calls_from(previous), expected.calls);
                assert_eq!(row.suffix_calls_from(previous), expected.suffixes);
                for (marker, count) in [603, 605, 604, 606, 607, 608]
                    .into_iter()
                    .zip(expected.calls)
                {
                    assert_eq!(trace.iter().filter(|w| **w == marker).count(), count);
                }
                assert_eq!(
                    trace
                        .windows(2)
                        .filter(|w| w[0] == 80)
                        .map(|w| w[1])
                        .collect::<Vec<_>>(),
                    expected.indices
                );
                let cases::GuardClock::StateFieldExit {
                    guard,
                    suffix_guard,
                    exits,
                } = row.clock
                else {
                    unreachable!()
                };
                let mut predecessor = row;
                predecessor.clock = cases::GuardClock::StateGrouped {
                    guard,
                    suffix_guard,
                    exits,
                };
                assert_eq!(row.words().len(), 25);
                assert_eq!(row.words(), predecessor.words());
                assert_eq!(row.args(), predecessor.args());
                previous = Some(state);
            }
        }
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        let completed = [
            0, 0, 4, 8, 0, 1, 0, 4, 1, 5, 1, 5, 5, 2, 5, 2, 3, 7, 2, 5, 2, 2,
        ];
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            assert_eq!(
                trace.iter().filter(|w| **w == 75).count(),
                completed[i],
                "fault {i}"
            );
            let tail = match i {
                10 | 11 => Some([605, kind.initial(), 0, 0]),
                16 | 17 => Some([608, kind.initial(), 0, 0]),
                20 => Some([605, kind.last_state(), 0, 0]),
                21 => Some([608, kind.last_state(), 0, 0]),
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

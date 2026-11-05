use super::*;
use cases::state_division_exits::{controls, divisors, field, matrix, number, plan, quotient};

#[test]
fn state_division_exit_argument_print_native_preserves_quotients_and_fault_prefixes() {
    for kind in cases::KINDS {
        native_sequences_for_kind(matrix(kind), kind);
    }
}

#[test]
fn state_division_exit_argument_print_reference_preserves_counts_and_accepted_state() {
    reference_sequences(cases::STATE_DIVISION_EXIT);
}

#[test]
fn state_division_exit_argument_print_oracle_pins_ieee_schedule_and_argument_fault_order() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        assert_eq!((matrix.sequences)(kind).len(), 28);
        assert_eq!((matrix.cli_sequences)(kind).len(), 10);
        assert_eq!((matrix.failures)(kind).len(), 19);
        let one = number(kind, 1.0);
        let two = number(kind, 2.0);
        let infinity = number(kind, f64::INFINITY);
        let canonical_nan = if kind == cases::Kind::F32 {
            0x7fc00000
        } else {
            0x7ff8000000000000
        };
        let third = if kind == cases::Kind::F32 {
            0x3eaaaaab
        } else {
            0x3fd5555555555555
        };
        assert_eq!(quotient(kind, one, number(kind, 3.0)), Some(third));
        assert_eq!(
            quotient(kind, infinity - 1, number(kind, 0.5)),
            Some(infinity)
        );
        for sign in [0, kind.mask()] {
            let signed = |value: i64| (value as u64 | sign) as i64;
            assert_eq!(quotient(kind, signed(1), two), Some(sign as i64));
            assert_eq!(quotient(kind, signed(one), infinity), Some(sign as i64));
            assert_eq!(
                quotient(kind, signed(0), number(kind, -1.0)),
                Some((sign ^ kind.mask()) as i64)
            );
            assert_eq!(
                quotient(kind, signed(infinity), infinity),
                Some(canonical_nan)
            );
            assert_eq!(quotient(kind, signed(kind.nan()), two), Some(canonical_nan));
            assert_eq!(quotient(kind, signed(one), kind.nan()), Some(canonical_nan));
            assert_eq!(quotient(kind, one, sign as i64), None);
            assert_eq!(quotient(kind, kind.nan(), sign as i64), None);
        }
        let seed = (matrix.seed)(kind);
        let schedule = plan(seed, None);
        assert_eq!(schedule.requested, [0, 3, 0, 4]);
        assert_eq!(schedule.completed, [0, 3, 0, 4]);
        assert_eq!(schedule.calls, [7, 2, 6, 4, 1, 2]);
        for rows in (matrix.sequences)(kind) {
            let mut previous = None;
            for row in rows {
                let expected = plan(row, previous);
                let (state, trace) = row.execute(previous, true).unwrap();
                assert_eq!(state, expected.state);
                assert_eq!(row.grouped_calls_from(previous), expected.calls);
                assert_eq!(row.suffix_calls_from(previous), expected.suffixes);
                assert_eq!(row.words().len(), 27);
                assert_eq!(row.words()[..25], field(row).words());
                assert!(row.args().starts_with(&format!("{},", field(row).args())));
                for (marker, count) in [603, 605, 604, 606, 607, 608]
                    .into_iter()
                    .zip(expected.calls)
                {
                    assert_eq!(trace.iter().filter(|word| **word == marker).count(), count);
                }
                assert_eq!(
                    trace
                        .windows(2)
                        .filter(|words| words[0] == 80)
                        .map(|words| words[1])
                        .collect::<Vec<_>>(),
                    expected.indices
                );
                previous = Some(state);
            }
        }
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.initial()));
        for (index, row) in (matrix.failures)(kind).into_iter().enumerate() {
            assert_eq!(row.child.base.initial, kind.last_state());
            let trace = row.execute(previous, true).unwrap_err();
            if index < 9 {
                continue;
            }
            let (marker, target, guard, completed) = match index {
                9 | 10 => (603, 0, 48, 1),
                11 | 12 => (603, 3, 13, 7),
                13 | 14 => (604, 0, 48, 1),
                15 | 16 => (604, 3, 13, 7),
                17 | 18 => (606, 1, 47, 2),
                _ => unreachable!(),
            };
            assert!(
                trace.ends_with(&[marker, target, target, guard]),
                "fault {index}: {trace:?}"
            );
            assert_eq!(trace.iter().filter(|word| **word == 75).count(), completed);
            let confirmation = if marker == 603 { 605 } else { 608 };
            assert!(
                !trace.contains(&confirmation),
                "argument failed after callee entry"
            );
        }
    }
}

#[test]
fn state_division_exit_argument_print_native_distinguishes_operand_order() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let mut rows = Vec::new();
        for slot in 0..2 {
            let mut row = (matrix.seed)(kind);
            row.parents = 2;
            row.child.base.trips = 1;
            *divisors(&mut row) = [number(kind, 2.0); 2];
            let exits = controls(&mut row);
            exits.flat.cut = if slot == 0 { 0 } else { 99 };
            exits.flat.stop = 0;
            let mut result = sequence(&[row]);
            let marker = if slot == 0 { 605 } else { 608 };
            let at = result.rows[0]
                .trace
                .iter()
                .position(|word| *word == marker)
                .unwrap();
            let value = quotient(kind, kind.last_state(), number(kind, 2.0)).unwrap();
            let reversed = quotient(kind, number(kind, 2.0), kind.last_state()).unwrap();
            assert_ne!(value, reversed);
            assert_eq!(result.rows[0].trace[at + 1], value);
            result.rows[0].trace[at + 1] = reversed;
            rows.push(result);
        }
        let source = matrix
            .template
            .replace(
                "current.total / continue_divisor",
                "continue_divisor / current.total",
            )
            .replace(
                "current.total / break_divisor",
                "break_divisor / current.total",
            )
            .replace("f64", kind.ty());
        probe::check_sequences_with_i64_prints(
            &source,
            &rows,
            &[],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_division_exit_argument_print_native_keeps_reservations_and_entry_limits() {
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
            let at = trace.iter().rposition(|word| *word == 76).unwrap();
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
        let mut row = seed;
        controls(&mut row).flat.cut = 3;
        controls(&mut row).flat.stop = 99;
        controls(&mut row).flat.fallback = 99;
        let mut rejected = sequence(&[row]);
        divisors(&mut row)[0] = 0;
        rejected.rows[0].input = row.words();
        let trace = &rejected.rows[0].trace;
        let at = trace.iter().rposition(|word| *word == 76).unwrap();
        let prefix = trace[..at + 6].to_vec();
        assert!(prefix.ends_with(&[76, 602, 3, 3, 2, 1]));
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &[rejected],
            matrix.observers,
            10,
            cases::ENTRY_LIMIT,
            &prefix,
        );
    }
}

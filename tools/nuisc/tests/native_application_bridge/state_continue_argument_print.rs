use super::*;

#[test]
fn state_continue_argument_print_native_preserves_steps_suffixes_and_faults() {
    native_sequences(cases::STATE_CONTINUE);
}

#[test]
fn state_continue_argument_print_reference_counts_reached_suffix_calls() {
    reference_sequences(cases::STATE_CONTINUE);
}

#[test]
fn state_continue_argument_print_native_preserves_reservations_and_fault_priority() {
    let matrix = cases::STATE_CONTINUE;
    for kind in cases::KINDS {
        let row = (matrix.seed)(kind);
        let source = matrix.template.replace("f64", kind.ty());
        assert_eq!(row.child_counts(), vec![0, 3, 0, 4]);
        assert_eq!(row.completed_child_counts_from(None), row.child_counts());
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
        negative.clock = cases::GuardClock::StateContinue(5, 1, 64);
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
        bad.clock = cases::GuardClock::StateContinue(3, 0, 64);
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
fn state_continue_argument_print_oracle_pins_full_work_indices_and_skipped_poison() {
    let matrix = cases::STATE_CONTINUE;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 27);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 20);
        for (cut, skips) in [(-1, 0), (0, 2), (1, 2), (2, 2), (3, 1), (99, 0)] {
            let mut row = (matrix.seed)(kind);
            row.clock = cases::GuardClock::StateContinue(5, cut, 64);
            let (state, trace) = row.execute(None, true).unwrap();
            assert_eq!(state, kind.last_state());
            assert_eq!(row.child_counts(), vec![0, 3, 0, 4]);
            assert_eq!(row.completed_child_counts_from(None), row.child_counts());
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), 7);
            assert_eq!(trace.iter().filter(|w| **w == 78).count(), skips);
            assert_eq!(trace.iter().filter(|w| **w == 79).count(), 7 - skips);
            assert_eq!(row.suffix_calls_from(None), 7 - skips);
            assert_eq!(
                trace
                    .windows(2)
                    .filter(|w| w[0] == 80)
                    .map(|w| w[1])
                    .collect::<Vec<_>>(),
                [0, 3, 0, 4]
            );
            let mut predecessor = row;
            predecessor.clock = cases::GuardClock::StateBreak(5, cut);
            assert_eq!(row.words().len(), 14);
            assert_eq!(row.words()[..13], predecessor.words());
            assert_eq!(row.args(), format!("{},64", predecessor.args()));
        }
        for rows in (matrix.cli_sequences)(kind).into_iter().skip(2).take(4) {
            let mut row = rows[0];
            assert!(row.execute(None, true).is_ok());
            let (_, guard) = row.continue_control().unwrap();
            row.clock = cases::GuardClock::StateContinue(5, -1, guard);
            assert!(row
                .execute(None, true)
                .unwrap_err()
                .ends_with(&[79, 601, 0, 0]));
        }
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            let done = [0, 0, 4, 4, 5, 6, 5, 6, 5, 6, 7, 7, 8, 1, 6, 7, 1, 7, 4, 1][i];
            assert_eq!(
                trace.iter().filter(|w| **w == 75).count(),
                done,
                "fault row {i}"
            );
            assert_eq!(trace.contains(&73), i == 12);
            if (14..18).contains(&i) {
                assert!(trace.ends_with(&[79, 601, 0, 0]));
            }
            if i == 18 {
                assert_eq!(trace.last(), Some(&74));
            }
            if i == 19 {
                assert_eq!(trace.last(), Some(&0));
                assert_eq!(trace.iter().filter(|w| **w == 78).count(), 1);
                assert!(!trace.contains(&79));
            }
        }
    }
}

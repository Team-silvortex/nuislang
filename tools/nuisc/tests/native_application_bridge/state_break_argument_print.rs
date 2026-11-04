use super::*;

#[test]
fn state_break_argument_print_native_preserves_exits_indices_and_faults() {
    native_sequences(cases::STATE_BREAK);
}

#[test]
fn state_break_argument_print_reference_counts_completed_not_reserved_children() {
    reference_sequences(cases::STATE_BREAK);
}

#[test]
fn state_break_argument_print_native_reserves_before_early_exit_without_refund() {
    let matrix = cases::STATE_BREAK;
    for kind in cases::KINDS {
        let row = (matrix.seed)(kind);
        let source = matrix.template.replace("f64", kind.ty());
        assert_eq!(row.child_counts(), vec![0, 3, 0, 4]);
        assert_eq!(row.completed_child_counts_from(None), vec![0, 1, 0, 1]);
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[row])],
            &[],
            matrix.observers,
            11,
            cases::ENTRY_LIMIT,
        );
        let mut full = row;
        full.clock = cases::GuardClock::StateBreak(5, -1);
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &[sequence(&[full])],
                matrix.observers,
                cases::LOOP_LIMIT,
                limit,
            );
        }
        let mut negative = row;
        negative.child.base.initial = kind.last_state();
        negative.clock = cases::GuardClock::StateBreak(5, 1);
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
        bad.clock = cases::GuardClock::StateBreak(3, 0);
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
fn state_break_argument_print_oracle_pins_completion_indices_and_skipped_poison() {
    let matrix = cases::STATE_BREAK;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 27);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 15);
        for (cut, requested, completed, indices, exits) in [
            (-1, [0, 3, 0, 4], [0, 3, 0, 4], [0, 3, 0, 4], 0),
            (0, [0, 3, 0, 4], [0, 1, 0, 1], [0, 0, 0, 0], 2),
            (1, [0, 3, 0, 3], [0, 2, 0, 2], [0, 1, 0, 1], 2),
            (2, [0, 3, 0, 4], [0, 3, 0, 3], [0, 2, 0, 2], 2),
            (3, [0, 3, 0, 4], [0, 3, 0, 4], [0, 3, 0, 3], 1),
            (99, [0, 3, 0, 4], [0, 3, 0, 4], [0, 3, 0, 4], 0),
        ] {
            let mut row = (matrix.seed)(kind);
            row.clock = cases::GuardClock::StateBreak(5, cut);
            assert_eq!(row.child_counts(), requested);
            assert_eq!(row.completed_child_counts_from(None), completed);
            let (state, trace) = row.execute(None, true).unwrap();
            let done = completed.iter().sum::<i64>() as usize;
            assert_eq!(
                state,
                if done % 2 == 0 {
                    kind.initial()
                } else {
                    kind.last_state()
                }
            );
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), done);
            assert_eq!(trace.iter().filter(|w| **w == 78).count(), exits);
            assert_eq!(trace.iter().filter(|w| **w == 79).count(), done - exits);
            assert_eq!(
                trace
                    .windows(2)
                    .filter(|w| w[0] == 80)
                    .map(|w| w[1])
                    .collect::<Vec<_>>(),
                indices
            );
            let mut old = row;
            old.clock = cases::GuardClock::StateBound(5);
            assert_eq!(row.words().len(), 13);
            assert_eq!(row.words()[..12], old.words());
            assert_eq!(row.args(), format!("{},{cut}", old.args()));
        }
        let poisoned = (matrix.cli_sequences)(kind).remove(2)[0];
        assert_eq!(poisoned.child.base.guards, [49, 49, 49, 1]);
        assert!(poisoned.execute(None, true).is_ok());
        let prefix = (matrix.prefix)(kind);
        let previous = prefix.iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            let completed = [0, 0, 3, 3, 4, 5, 4, 5, 4, 5, 5, 5, 6, 1, 1][i];
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), completed);
            if i < 4 {
                assert!(trace.ends_with(&[76, 602, 3, i as i64, 0, i64::from(i < 2)]));
            }
            if i == 12 {
                assert_eq!(trace.last(), Some(&73));
            } else {
                assert!(!trace.contains(&73));
            }
            if i == 13 {
                assert_eq!(trace.last(), Some(&74));
                assert_eq!(trace.iter().filter(|w| **w == 78).count(), 1);
            }
            if i == 14 {
                assert!(trace.ends_with(&[76, 602, 0, 3, 0, 0]));
            }
        }
    }
}

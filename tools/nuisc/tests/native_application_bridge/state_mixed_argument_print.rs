use super::*;

#[test]
fn state_mixed_argument_print_native_preserves_exit_order_and_faults() {
    native_sequences(cases::STATE_MIXED);
    // Swapping the two source guards must reverse the colliding exit's priority.
    let continue_exit = "if child == cut { print(78); child = child + 1; continue; }";
    let break_exit = "if child == stop_at { print(81); break; }";
    for kind in cases::KINDS {
        let source = cases::STATE_MIXED
            .template
            .replace("f64", kind.ty())
            .replace(
                &format!("{continue_exit}\n                {break_exit}"),
                &format!("{break_exit}\n                {continue_exit}"),
            );
        let mut first = (cases::STATE_MIXED.seed)(kind);
        first.clock = cases::GuardClock::StateMixed {
            guard: 5,
            cut: -1,
            suffix_guard: 64,
            stop: 0,
        };
        let mut expected = sequence(&[first, first]);
        for row in &mut expected.rows {
            row.input[12] = 0;
        }
        probe::check_sequences_with_i64_prints(
            &source,
            &[expected],
            &[],
            cases::STATE_MIXED.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_mixed_argument_print_reference_counts_only_reached_helpers() {
    reference_sequences(cases::STATE_MIXED);
}

#[test]
fn state_mixed_argument_print_native_preserves_reserved_work() {
    let matrix = cases::STATE_MIXED;
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty());
        let seed = (matrix.seed)(kind);
        for (cut, stop) in [(0, 2), (1, 0), (0, 0)] {
            let mut row = seed;
            row.clock = cases::GuardClock::StateMixed {
                guard: 5,
                cut,
                suffix_guard: 64,
                stop,
            };
            assert_eq!(row.child_counts(), [0, 3, 0, 4]);
            probe::check_sequences_with_i64_prints(
                &source,
                &[sequence(&[row])],
                &[],
                matrix.observers,
                11,
                cases::ENTRY_LIMIT,
            );
            let rows = [sequence(&[row])];
            let trace = &rows[0].rows[0].trace;
            let last_parent = trace.iter().rposition(|word| *word == 76).unwrap();
            let prefix = &trace[..last_parent + 6];
            assert!(prefix.ends_with(&[76, 602, 3, 3, 2, 1]));
            probe::check_i64_loop_work_limit_rejection(
                &source,
                &rows,
                matrix.observers,
                10,
                cases::ENTRY_LIMIT,
                prefix,
            );
        }
        let mut negative = seed;
        negative.child.base.initial = kind.last_state();
        negative.clock = cases::GuardClock::StateMixed {
            guard: 5,
            cut: 0,
            suffix_guard: 64,
            stop: 3,
        };
        assert_eq!(negative.child_counts(), [0, 4, 0, 4]);
        probe::check_sequences_with_i64_prints(
            &source,
            &[sequence(&[negative])],
            &[],
            matrix.observers,
            12,
            cases::ENTRY_LIMIT,
        );
        let rows = [sequence(&[negative])];
        let trace = &rows[0].rows[0].trace;
        let last_parent = trace.iter().rposition(|word| *word == 76).unwrap();
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &rows,
            matrix.observers,
            11,
            cases::ENTRY_LIMIT,
            &trace[..last_parent + 6],
        );
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
        bad.clock = cases::GuardClock::StateMixed {
            guard: 3,
            cut: 0,
            suffix_guard: 64,
            stop: 2,
        };
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
fn state_mixed_argument_print_oracle_pins_collision_indices_and_accepted_state() {
    let matrix = cases::STATE_MIXED;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 31);
        assert_eq!((matrix.cli_sequences)(kind).len(), 12);
        assert_eq!((matrix.failures)(kind).len(), 22);
        for (cut, stop, done, continues, breaks, suffixes, indices, negative) in [
            (0, 2, 6, 2, 2, 2, [0, 2, 0, 2], false),
            (1, 0, 2, 0, 2, 0, [0, 0, 0, 0], false),
            (0, 0, 7, 2, 0, 5, [0, 3, 0, 4], true),
            (1, 1, 7, 2, 0, 5, [0, 3, 0, 4], true),
            (1, 3, 7, 2, 1, 4, [0, 3, 0, 3], true),
            (-1, -1, 7, 0, 0, 7, [0, 3, 0, 4], true),
            (99, 99, 7, 0, 0, 7, [0, 3, 0, 4], true),
        ] {
            let mut row = (matrix.seed)(kind);
            row.clock = cases::GuardClock::StateMixed {
                guard: 5,
                cut,
                suffix_guard: 64,
                stop,
            };
            let (state, trace) = row.execute(None, true).unwrap();
            assert_eq!(
                state,
                if negative {
                    kind.last_state()
                } else {
                    kind.initial()
                }
            );
            assert_eq!(trace.iter().filter(|w| **w == 75).count(), done);
            assert_eq!(trace.iter().filter(|w| **w == 78).count(), continues);
            assert_eq!(trace.iter().filter(|w| **w == 81).count(), breaks);
            assert_eq!(trace.iter().filter(|w| **w == 79).count(), suffixes);
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
            predecessor.clock = cases::GuardClock::StateContinue(5, cut, 64);
            assert_eq!(row.words().len(), 15);
            assert_eq!(row.words()[..14], predecessor.words());
            assert_eq!(row.args(), format!("{},{stop}", predecessor.args()));
        }
        for (i, rows) in (matrix.cli_sequences)(kind).into_iter().enumerate() {
            let mut previous = None;
            for row in rows {
                let accepted = row.execute(previous, true).unwrap().0;
                previous = Some(accepted);
            }
            assert!(previous.is_some(), "sequence {i}");
        }
        let seed = (matrix.seed)(kind);
        let accepted = seed.execute(None, true).unwrap().0;
        assert_eq!(accepted, kind.initial());
        let next = seed.execute(Some(kind.last_state()), true).unwrap().0;
        assert_eq!(next, kind.last_state());
        assert_eq!(
            seed.child_counts_from(Some(kind.last_state())),
            [0, 4, 0, 3]
        );
        for (i, remove_continue) in [(2, true), (3, false)] {
            let mut row = (matrix.cli_sequences)(kind)[i][0];
            assert!(row.execute(None, true).is_ok());
            if let cases::GuardClock::StateMixed { cut, stop, .. } = &mut row.clock {
                if remove_continue {
                    *cut = -1;
                } else {
                    *stop = -1;
                }
            }
            assert!(row
                .execute(None, true)
                .unwrap_err()
                .ends_with(&[79, 601, 0, 0]));
        }
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        let completed = [
            0, 0, 4, 4, 5, 6, 5, 6, 5, 6, 7, 7, 8, 1, 6, 7, 1, 7, 4, 1, 2, 1,
        ];
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            assert_eq!(
                trace.iter().filter(|w| **w == 75).count(),
                completed[i],
                "fault {i}"
            );
            if (14..18).contains(&i) {
                assert!(trace.ends_with(&[79, 601, 0, 0]));
            }
            if i == 20 {
                assert_eq!(trace.last(), Some(&74));
            }
            if i == 21 {
                assert!(trace.ends_with(&[76, 602, 3, 3, 0, 0]));
            }
        }
    }
}

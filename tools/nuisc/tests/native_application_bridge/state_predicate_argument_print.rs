use super::*;

#[test]
fn state_predicate_argument_print_native_preserves_checks_and_skipped_faults() {
    native_sequences(cases::STATE_PREDICATE);
}

#[test]
fn state_predicate_argument_print_reference_counts_reached_predicates_once() {
    reference_sequences(cases::STATE_PREDICATE);
}

#[test]
fn state_predicate_argument_print_native_reverses_collision_and_fault_priority() {
    let matrix = cases::STATE_PREDICATE;
    let first = "if continue_at(child, cut, continue_guard - (outer * 16 + child)) { print(78); child = child + 1; continue; }";
    let second =
        "if break_at(child, stop_at, break_guard - (outer * 16 + child)) { print(81); break; }";
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty()).replace(
            &format!("{first}\n                {second}"),
            &format!("{second}\n                {first}"),
        );
        assert_ne!(source, matrix.template.replace("f64", kind.ty()));
        let mut pure = (cases::STATE_MIXED.seed)(kind);
        pure.parents = 2;
        pure.clock = cases::GuardClock::StateMixed {
            guard: 5,
            cut: 1,
            suffix_guard: 64,
            stop: 0,
        };
        let mut expected = sequence(&[pure]);
        let row = &mut expected.rows[0];
        row.input[12] = 0;
        row.input.extend([16, 64]);
        assert_eq!(row.trace.iter().filter(|w| **w == 81).count(), 1);
        let exit = row.trace.iter().position(|w| *w == 81).unwrap();
        // Only parent 1 executes a child, and the reversed break is the first check.
        row.trace.splice(exit..exit, [604, 0, 0, 48]);
        assert_eq!(row.state, [kind.last_state()]);
        let mut accepted = (matrix.seed)(kind);
        accepted.parents = 0;
        let mut failed_input = expected.rows[0].input.clone();
        failed_input[16] = 16;
        let trace = pure.execute(Some(kind.initial()), true).unwrap().1;
        let exit = trace.iter().position(|w| *w == 81).unwrap();
        let mut prefix = trace[..exit].to_vec();
        prefix.extend([604, 0, 0, 0]);
        probe::check_sequences_with_i64_prints(
            &source,
            &[expected],
            &[probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: failed_input,
                before_failure: prefix,
            }],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_predicate_argument_print_native_keeps_reservation_and_entry_boundaries() {
    let matrix = cases::STATE_PREDICATE;
    for kind in cases::KINDS {
        let source = matrix.template.replace("f64", kind.ty());
        let seed = (matrix.seed)(kind);
        let mut negative = seed;
        negative.child.base.initial = kind.last_state();
        if let cases::GuardClock::StatePredicate { stop, .. } = &mut negative.clock {
            *stop = 3;
        }
        for (row, limit) in [(seed, 11), (negative, 12)] {
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
            assert!(prefix.ends_with(&[76, 602, 3, 3, 2, 1]));
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
        if let cases::GuardClock::StatePredicate {
            guard,
            continue_guard,
            ..
        } = &mut bad.clock
        {
            *guard = 3;
            *continue_guard = 48;
        }
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
fn state_predicate_argument_print_oracle_pins_reached_checks_and_fault_order() {
    let matrix = cases::STATE_PREDICATE;
    for kind in cases::KINDS {
        assert_eq!((matrix.sequences)(kind).len(), 31);
        assert_eq!((matrix.cli_sequences)(kind).len(), 12);
        assert_eq!((matrix.failures)(kind).len(), 30);
        for (cut, stop, first, second, suffixes) in [
            (0, 2, 6, 4, 2),
            (1, 0, 2, 2, 0),
            (0, 0, 7, 5, 5),
            (1, 1, 7, 5, 5),
            (1, 3, 7, 5, 4),
            (-1, -1, 7, 7, 7),
            (99, 99, 7, 7, 7),
        ] {
            let mut row = (matrix.seed)(kind);
            row.clock = cases::GuardClock::StatePredicate {
                guard: 5,
                cut,
                suffix_guard: 64,
                stop,
                continue_guard: 64,
                break_guard: 64,
            };
            let (_, trace) = row.execute(None, true).unwrap();
            assert_eq!(row.predicate_calls_from(None), (first, second));
            assert_eq!(trace.iter().filter(|w| **w == 603).count(), first);
            assert_eq!(trace.iter().filter(|w| **w == 604).count(), second);
            assert_eq!(row.suffix_calls_from(None), suffixes);
            let mut predecessor = row;
            predecessor.clock = cases::GuardClock::StateMixed {
                guard: 5,
                cut,
                suffix_guard: 64,
                stop,
            };
            assert_eq!(row.words().len(), 17);
            assert_eq!(row.words()[..15], predecessor.words());
            assert_eq!(row.args(), format!("{},64,64", predecessor.args()));
        }
        for rows in (matrix.cli_sequences)(kind) {
            let mut previous = None;
            for row in rows {
                previous = Some(row.execute(previous, true).unwrap().0);
            }
        }
        let mut poison = (matrix.cli_sequences)(kind)[6][0];
        assert!(poison.execute(None, true).is_ok());
        if let cases::GuardClock::StatePredicate { cut, .. } = &mut poison.clock {
            *cut = -1;
        }
        assert!(poison
            .execute(None, true)
            .unwrap_err()
            .ends_with(&[604, 0, 0, 0]));
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.last_state()));
        let done = [
            0, 0, 4, 4, 5, 6, 5, 6, 5, 6, 7, 7, 8, 1, 6, 7, 1, 7, 4, 1, 2, 1, 1, 1, 2, 2, 4, 6, 0,
            2,
        ];
        for (i, bad) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = bad.execute(previous, true).unwrap_err();
            assert_eq!(
                trace.iter().filter(|w| **w == 75).count(),
                done[i],
                "fault {i}"
            );
            let tail = match i {
                22 | 26 => Some([603, 0, 0, 0]),
                23 => Some([604, 0, 0, 0]),
                24 => Some([603, 1, 0, 0]),
                25 => Some([604, 1, 2, 0]),
                27 => Some([604, 2, 2, 0]),
                29 => Some([603, 1, 1, 0]),
                _ => None,
            };
            if let Some(tail) = tail {
                assert!(trace.ends_with(&tail), "fault {i}");
            }
            if i == 28 {
                assert_eq!(trace.last(), Some(&74));
                assert!(!trace.contains(&603));
                assert!(!trace.contains(&604));
            }
        }
    }
}

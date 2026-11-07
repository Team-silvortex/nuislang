use super::*;
use cases::state_parameter_exits::{call_counts, controls, divisors, faults, helper, matrix, plan};

#[test]
fn state_parameter_exit_argument_print_native_preserves_parameter_order_and_faults() {
    for kind in cases::KINDS {
        native_sequences_for_kind(matrix(kind), kind);
    }
}

#[test]
fn state_parameter_exit_argument_print_reference_counts_parameters_once() {
    reference_sequences(cases::STATE_PARAMETER_EXIT);
}

#[test]
fn state_parameter_exit_argument_print_oracle_pins_six_failure_stages_and_input_prefix() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        assert_eq!((matrix.sequences)(kind).len(), 31);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 34);
        for rows in (matrix.sequences)(kind) {
            let mut previous = None;
            for row in rows {
                let expected = plan(row, previous);
                let (state, trace) = row.execute(previous, true).unwrap();
                assert_eq!(state, expected.state);
                assert_eq!(row.words().len(), 35);
                assert_eq!(row.words()[..31], helper(row).words());
                assert_eq!(row.words()[31..], row.parameter_faults().unwrap());
                assert!(row.args().starts_with(&format!("{},", helper(row).args())));
                for (marker, count) in (613..=616).zip(call_counts(row, previous)) {
                    assert_eq!(trace.iter().filter(|word| **word == marker).count(), count);
                }
                for (at, word) in trace.iter().enumerate() {
                    if matches!(*word, 613 | 615) {
                        assert_eq!(trace[at - 3], if *word == 613 { 610 } else { 612 });
                    }
                    if matches!(*word, 614 | 616) {
                        assert_eq!(trace[at - 3], word - 1);
                    }
                    if matches!(*word, 605 | 608) {
                        assert_eq!(trace[at - 3], if *word == 605 { 614 } else { 616 });
                        assert_eq!(trace[at - 5], trace[at + 2]);
                        assert_eq!(trace[at - 2], trace[at + 3]);
                    }
                }
                previous = Some(state);
            }
        }
        let previous = (matrix.prefix)(kind).iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        assert_eq!(previous, Some(kind.initial()));
        for (i, row) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = row.execute(previous, true).unwrap_err();
            assert_eq!(row.child.base.initial, kind.last_state());
            if i == 1 {
                assert!(!trace.iter().any(|word| (613..=616).contains(word)));
            }
            if matches!(i, 2 | 3) {
                assert_eq!(trace[trace.len() - 4], if i == 2 { 605 } else { 608 });
                assert_eq!(trace[trace.len() - 7], if i == 2 { 614 } else { 616 });
                assert_eq!(trace.last(), Some(&0));
            }
            if (4..24).contains(&i) {
                let slot = if i < 14 {
                    usize::from(i >= 8)
                } else if i < 22 {
                    (i - 14) / 4
                } else {
                    1
                };
                assert!(!trace.contains(&(613 + slot as i64 * 2)), "fault {i}");
                assert!(!trace.contains(&(614 + slot as i64 * 2)), "fault {i}");
            }
            if i >= 24 {
                let (slot, position, completed) = if i < 32 {
                    let index = i - 24;
                    (index / 4, index / 2 % 2, if index % 2 == 0 { 1 } else { 7 })
                } else {
                    (1, i - 32, 2)
                };
                let marker = 613 + (slot * 2 + position) as i64;
                assert!(trace.ends_with(&[marker, 0, 0]), "fault {i}");
                assert_eq!(trace.iter().filter(|word| **word == 75).count(), completed);
                assert!(!trace.contains(&if slot == 0 { 605 } else { 608 }));
                if position == 0 {
                    assert!(!trace.contains(&(marker + 1)));
                } else {
                    assert_eq!(trace[trace.len() - 6], marker - 1);
                }
            }
        }
    }
}

#[test]
fn state_parameter_exit_argument_print_native_reorders_parameters_and_fault_priority() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let mut source = matrix.template.to_owned();
        for (exit, mode, guard) in [
            ("continue", "permit", "confirm_guard"),
            ("break", "seal", "seal_guard"),
        ] {
            let second =
                format!("{exit}_mode_value({mode}, {exit}_mode_fault - (outer * 16 + child))");
            let third = format!("{exit}_guard_value({guard} - (outer * 16 + child), {exit}_guard_fault - (outer * 16 + child))");
            let original = format!("{second}, {third}");
            assert_eq!(source.matches(&original).count(), 1);
            source = source.replace(&original, &format!("{third}, {second}"));
            let signature = format!("fn confirm_{exit}(value: f64, mode: i64, guard: i64)");
            assert_eq!(source.matches(&signature).count(), 1);
            source = source.replace(
                &signature,
                &format!("fn confirm_{exit}(value: f64, guard: i64, mode: i64)"),
            );
        }
        let mut rows = Vec::new();
        let mut failures = Vec::new();
        for slot in 0..2 {
            let mut row = (matrix.seed)(kind);
            row.parents = 2;
            row.child.base.trips = 1;
            controls(&mut row).flat.cut = if slot == 0 { 0 } else { 99 };
            controls(&mut row).flat.stop = 0;
            let marker = 613 + slot as i64 * 2;
            let mut result = sequence(&[row]);
            let trace = &mut result.rows[0].trace;
            let at = trace.iter().position(|word| *word == marker).unwrap();
            assert_eq!(trace[at + 3], marker + 1);
            assert_eq!(trace[at + 6], if slot == 0 { 605 } else { 608 });
            let mode = trace[at..at + 3].to_vec();
            let guard = trace[at + 3..at + 6].to_vec();
            trace[at..at + 3].copy_from_slice(&guard);
            trace[at + 3..at + 6].copy_from_slice(&mode);
            trace.swap(at + 8, at + 9);
            rows.push(result);
            let mut accepted = row;
            accepted.parents = 0;
            let mut bad = row;
            faults(&mut bad)[slot * 2..slot * 2 + 2].fill(16);
            if slot == 0 {
                controls(&mut bad).flat.confirm_guard = 16;
            } else {
                controls(&mut bad).seal_guard = 16;
            }
            let mut trace = bad.execute(Some(kind.initial()), true).unwrap_err();
            assert!(trace.ends_with(&[marker, 0, 0]));
            let end = trace.len();
            trace[end - 3..].copy_from_slice(&[marker + 1, 0, 0]);
            failures.push(probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: bad.words(),
                before_failure: trace,
            });
        }
        probe::check_sequences_with_i64_prints(
            &source.replace("f64", kind.ty()),
            &rows,
            &failures,
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

fn mask_parameter_inputs(trace: &mut [i64], position: usize) {
    for at in 0..trace.len() {
        if trace[at] == 613 + position as i64 || trace[at] == 615 + position as i64 {
            trace[at + 1] ^= 4294967296;
        }
    }
}

#[test]
fn state_parameter_exit_argument_print_native_consumes_full_width_parameter_returns() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        for (position, parameter) in ["mode", "guard"].into_iter().enumerate() {
            let mut source = matrix.template.to_owned();
            for (exit, mode, guard) in [
                ("continue", "permit", "confirm_guard"),
                ("break", "seal", "seal_guard"),
            ] {
                let name = format!("{exit}_{parameter}_value");
                let original = format!("fn {name}(value: i64, fault: i64) -> i64 {{ let checked = 0 / fault; return value; }}");
                assert_eq!(source.matches(&original).count(), 1);
                source = source.replace(
                    &original,
                    &original.replace("return value", "return value ^ 4294967296"),
                );
                let argument = if position == 0 {
                    mode.to_owned()
                } else {
                    format!("{guard} - (outer * 16 + child)")
                };
                let original = format!("{name}({argument},");
                assert_eq!(source.matches(&original).count(), 1);
                source = source.replace(&original, &format!("{name}(({argument}) ^ 4294967296,"));
            }
            let seed = (matrix.seed)(kind);
            let mut negative = seed;
            negative.child.base.initial = kind.last_state();
            let mut zero = seed;
            zero.child.base.initial = 0;
            controls(&mut zero).flat.permit = 1;
            controls(&mut zero).flat.stop = 1;
            controls(&mut zero).seal = 1;
            let mut nan = seed;
            nan.child.base.initial = kind.nan();
            controls(&mut nan).flat.permit = 4294967296;
            controls(&mut nan).seal = i64::MIN;
            let rows = [seed, negative, zero, nan]
                .into_iter()
                .map(|row| {
                    let mut result = sequence(&[row]);
                    mask_parameter_inputs(&mut result.rows[0].trace, position);
                    result
                })
                .collect::<Vec<_>>();
            let prefix = (matrix.prefix)(kind);
            let previous = prefix.iter().fold(None, |previous, row| {
                Some(row.execute(previous, true).unwrap().0)
            });
            let failures = (matrix.failures)(kind)[2..4]
                .iter()
                .map(|bad| {
                    let mut accepted = sequence(&prefix);
                    for row in &mut accepted.rows {
                        mask_parameter_inputs(&mut row.trace, position);
                    }
                    let mut trace = bad.execute(previous, true).unwrap_err();
                    mask_parameter_inputs(&mut trace, position);
                    probe::SequenceFailure {
                        accepted,
                        input: bad.words(),
                        before_failure: trace,
                    }
                })
                .collect::<Vec<_>>();
            probe::check_sequences_with_i64_prints(
                &source.replace("f64", kind.ty()),
                &rows,
                &failures,
                matrix.observers,
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

#[test]
fn state_parameter_exit_argument_print_native_preserves_reservations_and_entry_limits() {
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
        let inline_update = "let update_check = 0 / (ready_guard - (outer * 16 + child));\n                current = Pair { total: -current.total, code: right, flag: selected };";
        let wrapped_update = "current = Pair { total: finish(-current.total, ready_guard - (outer * 16 + child)), code: right, flag: selected };";
        assert_eq!(source.matches(inline_update).count(), 1);
        probe::check_i64_helper_entry_limit_rejection(
            &source.replace(inline_update, wrapped_update),
            &[sequence(&[negative])],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let mut row = seed;
        controls(&mut row).flat.cut = 3;
        controls(&mut row).flat.stop = 99;
        controls(&mut row).flat.fallback = 99;
        let mut rejected = sequence(&[row]);
        faults(&mut row)[0] = 51;
        divisors(&mut row)[0] = 0;
        rejected.rows[0].input = row.words();
        let trace = &rejected.rows[0].trace;
        let at = trace.iter().rposition(|word| *word == 76).unwrap();
        let prefix = trace[..at + 6].to_vec();
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

#[test]
fn state_parameter_exit_argument_print_native_requires_each_parameter_effect_grant() {
    use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};
    let names = [
        "continue_mode_value",
        "continue_guard_value",
        "break_mode_value",
        "break_guard_value",
    ];
    let mut source = cases::STATE_PARAMETER_EXIT.template.to_owned();
    for name in names {
        let start = format!("fn {name}(value: i64, fault: i64) -> i64 {{");
        assert_eq!(source.matches(&start).count(), 1);
        source = source.replace(&start, &format!("{start} print(82);"));
    }
    for kind in cases::KINDS {
        let project = Project::with_source(&source.replace("f64", kind.ty()));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let sites = module
                .nodes
                .iter()
                .filter(|node| matches!(node.op.instruction.as_str(), "print" | "guard_print"))
                .map(|node| node.name.clone())
                .collect::<Vec<_>>();
            let full = I64PrintPolicy::new(sites.clone()).unwrap();
            emit_registered_with_i64_prints(
                &module,
                "counter",
                &full,
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            )
            .unwrap();
            for name in names {
                let producer = module
                    .functions
                    .iter()
                    .find(|function| function.name == name)
                    .unwrap();
                assert_eq!(
                    sites
                        .iter()
                        .filter(|site| producer.body_nodes.contains(site))
                        .count(),
                    1
                );
                let policy = I64PrintPolicy::new(
                    sites
                        .iter()
                        .filter(|site| !producer.body_nodes.contains(site))
                        .cloned(),
                )
                .unwrap();
                let error = emit_registered_with_i64_prints(
                    &module,
                    "counter",
                    &policy,
                    cases::LOOP_LIMIT,
                    cases::ENTRY_LIMIT,
                )
                .err()
                .expect("parameter print inherited a caller grant");
                assert!(error.contains("print"), "{error}");
            }
        }
    }
}

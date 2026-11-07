use super::*;
use cases::state_division_exits::number;
use cases::state_helper_exits::{
    call_counts, controls, division, divisors, guards, matrix, plan, quotient,
};

#[test]
fn state_helper_exit_argument_print_native_preserves_producer_entries_and_faults() {
    for kind in cases::KINDS {
        native_sequences_for_kind(matrix(kind), kind);
    }
}

#[test]
fn state_helper_exit_argument_print_native_keeps_work_and_entry_limits() {
    let total = "let other = -value; if pick { return value; } return other;";
    let outlined = "if pick { return value; } return -value;";
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
        assert_eq!(source.matches(total).count(), 1);
        probe::check_i64_helper_entry_limit_rejection(
            &source.replace(total, outlined),
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
        guards(&mut row)[0] = 51;
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
fn state_helper_exit_argument_print_native_reverses_producer_order_and_fault_priority() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let mut source = matrix.template.to_owned();
        for exit in ["continue", "break"] {
            let lhs = format!("{exit}_lhs(current.total, {exit}_lhs_guard - (outer * 16 + child))");
            let rhs =
                format!("{exit}_rhs({exit}_divisor, {exit}_rhs_guard - (outer * 16 + child))");
            let original = format!("{lhs} / {rhs}");
            assert_eq!(source.matches(&original).count(), 1);
            source = source.replace(&original, &format!("{rhs} / {lhs}"));
        }
        let mut rows = Vec::new();
        let mut failures = Vec::new();
        for slot in 0..2 {
            let mut row = (matrix.seed)(kind);
            row.parents = 2;
            row.child.base.trips = 1;
            *divisors(&mut row) = [number(kind, 2.0); 2];
            controls(&mut row).flat.cut = if slot == 0 { 0 } else { 99 };
            controls(&mut row).flat.stop = 0;
            let marker = 609 + slot as i64 * 2;
            let mut result = sequence(&[row]);
            let trace = &mut result.rows[0].trace;
            let at = trace.iter().position(|word| *word == marker).unwrap();
            assert_eq!(trace[at + 3], marker + 1);
            assert_eq!(trace[at + 6], if slot == 0 { 605 } else { 608 });
            assert_eq!(
                trace[at + 7],
                quotient(kind, kind.last_state(), number(kind, 2.0)).unwrap()
            );
            let left = trace[at..at + 3].to_vec();
            let right = trace[at + 3..at + 6].to_vec();
            trace[at..at + 3].copy_from_slice(&right);
            trace[at + 3..at + 6].copy_from_slice(&left);
            trace[at + 7] = quotient(kind, number(kind, 2.0), kind.last_state()).unwrap();
            rows.push(result);
            let mut accepted = row;
            accepted.parents = 0;
            let mut bad = row;
            divisors(&mut bad)[slot] = 0;
            guards(&mut bad)[slot * 2..slot * 2 + 2].fill(16);
            if slot == 0 {
                controls(&mut bad).flat.confirm_guard = 16;
            } else {
                controls(&mut bad).seal_guard = 16;
            }
            let mut trace = bad.execute(Some(kind.initial()), true).unwrap_err();
            assert!(trace.ends_with(&[marker, kind.last_state(), 0]));
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

#[test]
fn state_helper_exit_argument_print_native_consumes_producer_return_values() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        for side in ["lhs", "rhs"] {
            let mut source = matrix.template.to_owned();
            for exit in ["continue", "break"] {
                let name = format!("{exit}_{side}");
                let original = format!("fn {name}(value: f64, guard: i64) -> f64 {{ let checked = 0 / guard; return value; }}");
                assert_eq!(source.matches(&original).count(), 1);
                source = source.replace(
                    &original,
                    &original.replace("return value", "return -value"),
                );
                let argument = if side == "lhs" {
                    "current.total".to_owned()
                } else {
                    format!("{exit}_divisor")
                };
                let call = format!("{name}({argument},");
                assert_eq!(source.matches(&call).count(), 1);
                source = source.replace(&call, &format!("{name}(-{argument},"));
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
            controls(&mut nan).flat.permit = 2;
            controls(&mut nan).seal = 2;
            let markers = if side == "lhs" {
                [609, 611]
            } else {
                [610, 612]
            };
            let rows = [seed, negative, zero, nan]
                .into_iter()
                .map(|row| {
                    let mut result = sequence(&[row]);
                    let trace = &mut result.rows[0].trace;
                    for at in 0..trace.len() {
                        if markers.contains(&trace[at]) {
                            trace[at + 1] = (trace[at + 1] as u64 ^ kind.mask()) as i64;
                        }
                    }
                    result
                })
                .collect::<Vec<_>>();
            probe::check_sequences_with_i64_prints(
                &source.replace("f64", kind.ty()),
                &rows,
                &[],
                matrix.observers,
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            );
        }
    }
}

#[test]
fn state_helper_exit_argument_print_native_requires_explicit_producer_print_grants() {
    use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};
    let names = ["continue_lhs", "continue_rhs", "break_lhs", "break_rhs"];
    let mut source = cases::STATE_HELPER_EXIT.template.to_owned();
    for name in names {
        let start = format!("fn {name}(value: f64, guard: i64) -> f64 {{");
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
                let omitted = sites
                    .iter()
                    .filter(|site| producer.body_nodes.contains(site))
                    .collect::<Vec<_>>();
                assert_eq!(omitted.len(), 1);
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
                .expect("producer print inherited a caller grant");
                assert!(error.contains("print"), "{error}");
            }
        }
    }
}

#[test]
fn state_helper_exit_argument_print_reference_counts_each_producer_once() {
    reference_sequences(cases::STATE_HELPER_EXIT);
}

#[test]
fn state_helper_exit_argument_print_oracle_pins_four_fault_stages_and_input_prefix() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        assert_eq!((matrix.sequences)(kind).len(), 28);
        assert_eq!((matrix.cli_sequences)(kind).len(), 8);
        assert_eq!((matrix.failures)(kind).len(), 24);
        for rows in (matrix.sequences)(kind) {
            let mut previous = None;
            for row in rows {
                let expected = plan(row, previous);
                let (state, trace) = row.execute(previous, true).unwrap();
                assert_eq!(state, expected.state);
                assert_eq!(row.words().len(), 31);
                assert_eq!(row.words()[..27], division(row).words());
                assert_eq!(row.words()[27..], row.helper_operands().unwrap());
                assert!(row
                    .args()
                    .starts_with(&format!("{},", division(row).args())));
                for (marker, count) in (609..=612).zip(call_counts(row, previous)) {
                    assert_eq!(trace.iter().filter(|word| **word == marker).count(), count);
                }
                for (at, word) in trace.iter().enumerate() {
                    if matches!(*word, 610 | 612) {
                        assert_eq!(trace[at - 3], word - 1);
                    }
                    if matches!(*word, 605 | 608) {
                        assert_eq!(trace[at - 3], if *word == 605 { 610 } else { 612 });
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
                assert!(!trace.iter().any(|word| (609..=612).contains(word)));
            }
            if matches!(i, 2 | 3) {
                assert_eq!(trace[trace.len() - 4], if i == 2 { 605 } else { 608 });
                assert_eq!(trace[trace.len() - 7], if i == 2 { 610 } else { 612 });
                assert_eq!(trace.last(), Some(&0));
            }
            if (4..14).contains(&i) {
                let slot = usize::from(i >= 8);
                let offset = if i >= 12 {
                    17
                } else if i % 4 >= 2 {
                    51
                } else {
                    16
                };
                let marker = 610 + slot as i64 * 2;
                let divisor = row.division_exits().unwrap().1[slot];
                assert!(
                    trace.ends_with(&[marker, divisor, 64 - offset]),
                    "fault {i}"
                );
                assert!(!trace.contains(&if slot == 0 { 605 } else { 608 }));
            }
            if i >= 14 {
                let (slot, side, completed) = if i < 22 {
                    let offset = i - 14;
                    (
                        offset / 4,
                        offset / 2 % 2,
                        if offset % 2 == 0 { 1 } else { 7 },
                    )
                } else {
                    (1, i - 22, 2)
                };
                let marker = 609 + (slot * 2 + side) as i64;
                let value = if side == 1 {
                    0
                } else if i >= 22 {
                    kind.initial()
                } else {
                    kind.last_state()
                };
                assert!(trace.ends_with(&[marker, value, 0]), "fault {i}");
                assert_eq!(trace.iter().filter(|word| **word == 75).count(), completed);
                assert!(!trace.contains(&if slot == 0 { 605 } else { 608 }));
                if side == 0 {
                    assert!(!trace.contains(&(marker + 1)));
                } else {
                    assert_eq!(trace[trace.len() - 6], marker - 1);
                }
            }
        }
    }
}

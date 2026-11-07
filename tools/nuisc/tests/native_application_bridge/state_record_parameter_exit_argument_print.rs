use super::*;
use cases::state_record_parameters::{
    call_counts, controls, divisors, faults, guard, matrix, mode, number, parameter, plan,
};

#[test]
fn state_record_parameter_exit_argument_print_native_uses_updated_fields_and_faults() {
    for kind in cases::KINDS {
        native_sequences_for_kind(matrix(kind), kind);
    }
}

#[test]
fn state_record_parameter_exit_argument_print_reference_preserves_dynamic_counts_and_state() {
    reference_sequences(cases::STATE_RECORD_PARAMETER_EXIT);
}

#[test]
fn state_record_parameter_exit_argument_print_native_distinguishes_stale_parent_fields() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let mut bad = (matrix.seed)(kind);
        bad.parents = 2;
        bad.child.base.trips = 1;
        controls(&mut bad).flat.stop = 0;
        controls(&mut bad).flat.fallback = 99;
        controls(&mut bad).flat.confirm_guard = 17;
        controls(&mut bad).seal_guard = 17;
        let mut accepted = bad;
        accepted.parents = 0;
        bad.child.base.initial = kind.last_state();
        let failure = bad.execute(Some(kind.initial()), true).unwrap_err();
        assert!(failure.ends_with(&[605, kind.last_state(), 0, 0]));
        probe::check_sequences_with_i64_prints(
            &matrix.template.replace("f64", kind.ty()),
            &[sequence(&[accepted])],
            &[probe::SequenceFailure {
                accepted: sequence(&[accepted]),
                input: bad.words(),
                before_failure: failure,
            }],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let mut source = matrix.template.to_owned();
        for name in [
            "continue_mode_value",
            "continue_guard_value",
            "break_mode_value",
            "break_guard_value",
        ] {
            let call = format!("{name}(current.total,");
            assert_eq!(source.matches(&call).count(), 1);
            source = source.replace(&call, &format!("{name}(total,"));
        }
        let mut stale = sequence(&[parameter(accepted), parameter(bad)]);
        for row in &mut stale.rows {
            row.trace = row
                .trace
                .iter()
                .flat_map(|word| {
                    if (613..=616).contains(word) {
                        vec![*word, kind.initial()]
                    } else {
                        vec![*word]
                    }
                })
                .collect();
        }
        assert!(stale.rows[1]
            .trace
            .windows(4)
            .any(|words| words == [605, kind.last_state(), 1, 1]));
        assert_eq!(stale.rows[1].state, [kind.last_state()]);
        probe::check_sequences_with_i64_prints(
            &source.replace("f64", kind.ty()),
            &[stale],
            &[],
            matrix.observers,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

fn mask_record_inputs(trace: &mut [i64], position: usize) {
    for at in 0..trace.len() {
        if trace[at] == 613 + position as i64 || trace[at] == 615 + position as i64 {
            trace[at + 2] ^= 4294967296;
        }
    }
}

#[test]
fn state_record_parameter_exit_argument_print_native_consumes_derived_full_width_returns() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        for (position, role) in ["mode", "guard"].into_iter().enumerate() {
            let mut source = matrix.template.to_owned();
            for (exit, mode_base, guard_base) in [
                ("continue", "permit", "confirm_guard"),
                ("break", "seal", "seal_guard"),
            ] {
                let name = format!("{exit}_{role}_value");
                let op = if position == 0 { "^" } else { "-" };
                let original = format!("fn {name}(value: f64, base: i64, fault: i64) -> i64 {{ let checked = 0 / fault; let alternate = base {op} 1; if value < 0.0 {{ return alternate; }} return base; }}");
                let replacement = format!("fn {name}(value: f64, base: i64, fault: i64) -> i64 {{ let checked = 0 / fault; let decoded = base ^ 4294967296; let alternate = decoded {op} 1; if value < 0.0 {{ return alternate; }} return decoded; }}");
                assert_eq!(source.matches(&original).count(), 1);
                source = source.replace(&original, &replacement);
                let argument = if position == 0 {
                    mode_base.to_owned()
                } else {
                    format!("{guard_base} - (outer * 16 + child)")
                };
                let original = format!("{name}(current.total, {argument},");
                assert_eq!(source.matches(&original).count(), 1);
                source = source.replace(
                    &original,
                    &format!("{name}(current.total, ({argument}) ^ 4294967296,"),
                );
            }
            let seed = (matrix.seed)(kind);
            let mut negative = seed;
            negative.child.base.initial = kind.last_state();
            let mut zero = seed;
            zero.child.base.initial = kind.mask() as i64;
            let mut nan = seed;
            nan.child.base.initial = kind.nan();
            controls(&mut nan).flat.permit = 4294967296;
            controls(&mut nan).seal = i64::MIN;
            let rows = [seed, negative, zero, nan]
                .into_iter()
                .map(|row| {
                    let mut result = sequence(&[row]);
                    mask_record_inputs(&mut result.rows[0].trace, position);
                    result
                })
                .collect::<Vec<_>>();
            let prefix = (matrix.prefix)(kind);
            let previous = prefix.iter().fold(None, |previous, row| {
                Some(row.execute(previous, true).unwrap().0)
            });
            let candidates = (matrix.failures)(kind);
            let failures = [8, 20]
                .into_iter()
                .map(|index| {
                    let bad = candidates[index];
                    let mut accepted = sequence(&prefix);
                    for row in &mut accepted.rows {
                        mask_record_inputs(&mut row.trace, position);
                    }
                    let mut trace = bad.execute(previous, true).unwrap_err();
                    mask_record_inputs(&mut trace, position);
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
fn state_record_parameter_exit_argument_print_native_preserves_state_driven_budgets() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        let source = matrix.template.replace("f64", kind.ty());
        let seed = (matrix.seed)(kind);
        let mut negative = seed;
        negative.child.base.initial = kind.last_state();
        let mut zero = seed;
        zero.child.base.initial = 0;
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
        let mut outlined = source.clone();
        for op in ["^", "-"] {
            let ready = format!(
                "let alternate = base {op} 1; if value < 0.0 {{ return alternate; }} return base;"
            );
            let nested = format!("if value < 0.0 {{ return base {op} 1; }} return base;");
            assert_eq!(outlined.matches(&ready).count(), 2);
            outlined = outlined.replace(&ready, &nested);
        }
        probe::check_i64_helper_entry_limit_rejection(
            &outlined,
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
fn state_record_parameter_exit_argument_print_native_keeps_individual_effect_grants() {
    use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};
    let names = [
        "continue_mode_value",
        "continue_guard_value",
        "break_mode_value",
        "break_guard_value",
    ];
    let mut source = cases::STATE_RECORD_PARAMETER_EXIT.template.to_owned();
    for name in names {
        let start = format!("fn {name}(value: f64, base: i64, fault: i64) -> i64 {{");
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
                .expect("record parameter print inherited a caller grant");
                assert!(error.contains("print"), "{error}");
            }
        }
    }
}

#[test]
fn state_record_parameter_exit_argument_print_oracle_pins_state_values_and_fault_stages() {
    for kind in cases::KINDS {
        let matrix = matrix(kind);
        assert_eq!((matrix.sequences)(kind).len(), 34);
        assert_eq!((matrix.cli_sequences)(kind).len(), 10);
        assert_eq!((matrix.failures)(kind).len(), 34);
        let infinity = number(kind, f64::INFINITY);
        for magnitude in [0, 1, kind.initial(), infinity, kind.nan()] {
            for sign in [0, kind.mask()] {
                let bits = (magnitude as u64 | sign) as i64;
                let negative = sign != 0 && magnitude != 0 && magnitude != kind.nan();
                for base in [0, 1, 4294967296, i64::MIN, i64::MAX] {
                    assert_eq!(mode(kind, bits, base), base ^ i64::from(negative));
                    assert_eq!(
                        guard(kind, bits, base),
                        base.wrapping_sub(i64::from(negative))
                    );
                }
            }
        }
        let seed = (matrix.seed)(kind);
        let mut wrapping = seed;
        controls(&mut wrapping).flat.confirm_guard = i64::MIN + 16;
        controls(&mut wrapping).seal_guard = i64::MIN + 18;
        let trace = wrapping.execute(None, true).unwrap().1;
        assert!(trace
            .windows(4)
            .any(|words| words == [614, kind.last_state(), i64::MIN, 48]));
        assert!(trace
            .windows(4)
            .any(|words| words == [616, kind.last_state(), i64::MIN, 46]));
        for marker in [605, 608] {
            assert!(trace
                .windows(4)
                .any(|words| words == [marker, kind.last_state(), 0, i64::MAX]));
        }
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
                assert_eq!(row.words().len(), 35);
                assert_eq!(row.words(), parameter(row).words());
                assert_eq!(row.args(), parameter(row).args());
                for (marker, count) in (613..=616).zip(call_counts(row, previous)) {
                    assert_eq!(trace.iter().filter(|word| **word == marker).count(), count);
                }
                for (at, word) in trace.iter().enumerate() {
                    if matches!(*word, 605 | 608) {
                        assert_eq!(trace[at - 8], if *word == 605 { 613 } else { 615 });
                        assert_eq!(trace[at - 4], if *word == 605 { 614 } else { 616 });
                        assert_eq!(trace[at - 7], trace[at - 3]);
                        assert_eq!(mode(kind, trace[at - 7], trace[at - 6]), trace[at + 2]);
                        assert_eq!(guard(kind, trace[at - 3], trace[at - 2]), trace[at + 3]);
                    }
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
        for (i, row) in (matrix.failures)(kind).into_iter().enumerate() {
            let trace = row.execute(previous, true).unwrap_err();
            assert_eq!(row.child.base.initial, kind.last_state());
            if i < 3 {
                continue;
            }
            let (slot, stage, at, bits, completed) = if i < 27 {
                let index = i - 3;
                let late = index % 12 >= 6;
                (
                    index / 12,
                    index % 6,
                    if late { 51 } else { 16 },
                    kind.last_state(),
                    if late { 7 } else { 1 },
                )
            } else if i < 31 {
                let index = i - 27;
                let late = index % 2 != 0;
                (
                    index / 2,
                    2,
                    if late { 51 } else { 16 },
                    kind.last_state(),
                    if late { 7 } else { 1 },
                )
            } else {
                (1, i - 28, 17, kind.initial(), 2)
            };
            let base_guard = i64::from(bits == kind.last_state());
            match stage {
                0 => assert!(trace.ends_with(&[609 + slot as i64 * 2, bits, 0])),
                1 => assert!(trace.ends_with(&[610 + slot as i64 * 2, 0, 0])),
                2 => assert!(trace.ends_with(&[
                    610 + slot as i64 * 2,
                    if i >= 27 { kind.mask() as i64 } else { 0 },
                    64 - at
                ])),
                3 => assert!(trace.ends_with(&[613 + slot as i64 * 2, bits, 1, 0])),
                4 => assert!(trace.ends_with(&[614 + slot as i64 * 2, bits, base_guard, 0])),
                5 => assert!(trace.ends_with(&[
                    if slot == 0 { 605 } else { 608 },
                    bits,
                    1 ^ base_guard,
                    0
                ])),
                _ => unreachable!(),
            }
            assert_eq!(
                trace.iter().filter(|word| **word == 75).count(),
                completed,
                "fault {i}"
            );
            if stage < 3 {
                assert!(!trace.contains(&(613 + slot as i64 * 2)));
            }
            if stage < 4 {
                assert!(!trace.contains(&(614 + slot as i64 * 2)));
            }
            if stage < 5 {
                assert!(!trace.contains(&if slot == 0 { 605 } else { 608 }));
            }
        }
    }
}

#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "state_checked_control_exit_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    let mut prior = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (next, trace) = row.execute(prior, true).unwrap();
                prior = Some(next);
                probe::Row {
                    input: row.words(),
                    state: next.to_vec(),
                    trace,
                }
            })
            .collect(),
    }
}

fn values(row: cases::Case) -> Vec<Value> {
    row.words()
        .into_iter()
        .enumerate()
        .map(|(i, word)| match i {
            0 | 25 | 26 if row.kind() == cases::Kind::F32 => {
                Value::F32(f32::from_bits(word as u32))
            }
            0 | 25 | 26 => Value::F64(f64::from_bits(word as u64)),
            1 => Value::Bool(word != 0),
            3 | 4 => Value::I32(word as i32),
            _ => Value::Int(word),
        })
        .collect()
}

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|event| {
            event.contains("cpu.print")
                || (event.contains("cpu.guard_print") && event.contains("if true then print"))
        })
        .map(|event| event.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

fn check_calls(row: cases::Case, prior: Option<cases::State>, events: &[String]) {
    let plan = row.plan(prior);
    let children: i64 = plan.completed.iter().sum();
    let calls = |name: &str| {
        events
            .iter()
            .filter(|event| event.contains("cpu.call") && event.contains(&format!("] {name}(")))
            .count()
    };
    assert_eq!(calls("advance_control"), children as usize);
    assert_eq!(calls("finish"), 1);
    assert_eq!(calls("child_count"), row.base.arguments.parents as usize);
    assert_eq!(
        calls("checked64"),
        children as usize * (1 + usize::from(row.base.arguments.child.base.selected))
            + plan.suffixes
    );
    for (name, count) in [
        "continue_at",
        "confirm_continue",
        "break_at",
        "fallback_break",
        "rescue_continue",
        "confirm_break",
    ]
    .into_iter()
    .zip(plan.calls)
    {
        assert_eq!(calls(name), count, "{name}");
    }
    for (slot, exit) in ["continue", "break"].into_iter().enumerate() {
        for suffix in ["lhs", "rhs", "mode_value", "guard_value"] {
            assert_eq!(
                calls(&format!("{exit}_{suffix}")),
                plan.calls[if slot == 0 { 1 } else { 5 }]
            );
        }
    }
}

#[test]
fn state_checked_control_exit_native_keeps_producer_order_and_complete_publication() {
    for kind in cases::KINDS {
        let prefix = cases::prefix(kind);
        let prior = prefix
            .iter()
            .fold(None, |prior, row| Some(row.execute(prior, true).unwrap().0));
        let rows = cases::sequences(kind)
            .iter()
            .map(|rows| sequence(rows))
            .collect::<Vec<_>>();
        let failures = cases::failures(kind)
            .into_iter()
            .map(|bad| probe::SequenceFailure {
                accepted: sequence(&prefix),
                input: bad.words(),
                before_failure: bad.execute(prior, true).unwrap_err(),
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_i64_prints(
            &cases::SOURCE.replace("f64", kind.ty()),
            &rows,
            &failures,
            &cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_checked_control_exit_reference_preserves_state_on_rejected_and_failed_events() {
    for kind in cases::KINDS {
        let project = Project::with_source(&cases::SOURCE.replace("f64", kind.ty()));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for rows in cases::sequences(kind) {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(rows[0]),
                )
                .unwrap();
                let (mut prior, trace) = rows[0].execute(None, false).unwrap();
                assert_eq!(printed(&opened.events), trace);
                check_calls(rows[0], None, &opened.events);
                assert_eq!(state_words(session.state()), prior.map(|word| word as u64));
                for row in &rows[1..] {
                    let mut rejected = values(*row);
                    rejected[37] = Value::Bool(false);
                    assert!(session.event(rejected).is_err());
                    assert_eq!(
                        session.phase(),
                        yir_runtime_host::ApplicationSessionPhase::Open
                    );
                    assert_eq!(state_words(session.state()), prior.map(|word| word as u64));
                    let (next, trace) = row.execute(Some(prior), false).unwrap();
                    let event = session.event(values(*row)).unwrap();
                    assert_eq!(printed(&event.events), trace);
                    check_calls(*row, Some(prior), &event.events);
                    assert_eq!(state_words(session.state()), next.map(|word| word as u64));
                    prior = next;
                }
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
                assert_eq!(state_words(session.state()), prior.map(|word| word as u64));
            }
            for bad in cases::failures(kind) {
                let prefix = cases::prefix(kind);
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(prefix[0]),
                )
                .unwrap();
                session.event(values(prefix[1])).unwrap();
                let prior = prefix
                    .iter()
                    .fold(None, |prior, row| {
                        Some(row.execute(prior, false).unwrap().0)
                    })
                    .unwrap();
                assert!(session.event(values(bad)).is_err(), "{bad:?}");
                assert_eq!(state_words(session.state()), prior.map(|word| word as u64));
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_eq!(state_words(session.state()), prior.map(|word| word as u64));
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn state_checked_control_exit_native_distinguishes_reordered_update_checks() {
    let check = "let update_check = 0 / (ready_guard - (outer * 16 + child));";
    let update = cases::SOURCE
        .lines()
        .find(|line| line.trim_start().starts_with("current = Pair"))
        .unwrap()
        .trim();
    assert_eq!(cases::SOURCE.matches(check).count(), 1);
    let source = cases::SOURCE
        .replace(check, "")
        .replace(update, &format!("{update} {check}"));
    for kind in cases::KINDS {
        let prefix = cases::prefix(kind);
        let prior = prefix
            .iter()
            .fold(None, |prior, row| Some(row.execute(prior, true).unwrap().0));
        let bad = cases::failures(kind);
        let failures = [15, 16]
            .into_iter()
            .map(|index| {
                let mut trace = bad[index].execute(prior, true).unwrap_err();
                assert!(trace.ends_with(&[74]));
                let done = cases::records(&trace, true)
                    .iter()
                    .filter(|entry| entry.1 == 75)
                    .count();
                trace.extend([617, done as i64, 0]);
                probe::SequenceFailure {
                    accepted: sequence(&prefix),
                    input: bad[index].words(),
                    before_failure: trace,
                }
            })
            .collect::<Vec<_>>();
        probe::check_sequences_with_i64_prints(
            &source.replace("f64", kind.ty()),
            &[sequence(&[cases::seed(kind)])],
            &failures,
            &cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

fn mask_control_inputs(trace: &mut [i64]) {
    for (at, marker) in cases::records(trace, true) {
        if marker == 617 {
            trace[at + 1] ^= 4294967296;
        }
    }
}

#[test]
fn state_checked_control_exit_native_consumes_full_width_update_returns() {
    let signature = "fn advance_control(value: i64, fault: i64) -> i64 { let checked = 0 / fault; return value + 1; }";
    assert_eq!(cases::SOURCE.matches(signature).count(), 1);
    let source = cases::SOURCE.replace(signature,
        "fn advance_control(value: i64, fault: i64) -> i64 { let checked = 0 / fault; return (value ^ 4294967296) + 1; }")
        .replace("advance_control(current.control,", "advance_control(current.control ^ 4294967296,");
    for kind in cases::KINDS {
        let rows = [i64::MIN, i64::MAX, -1, 71, 617]
            .into_iter()
            .map(|control| {
                let mut row = cases::seed(kind);
                row.base.initial_control = control;
                let mut result = sequence(&[row]);
                mask_control_inputs(&mut result.rows[0].trace);
                result
            })
            .collect::<Vec<_>>();
        let prefix = cases::prefix(kind);
        let prior = prefix
            .iter()
            .fold(None, |prior, row| Some(row.execute(prior, true).unwrap().0));
        let failures = cases::failures(kind)[8..15]
            .iter()
            .map(|bad| {
                let mut accepted = sequence(&prefix);
                for row in &mut accepted.rows {
                    mask_control_inputs(&mut row.trace);
                }
                let mut trace = bad.execute(prior, true).unwrap_err();
                mask_control_inputs(&mut trace);
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
            &cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn state_checked_control_exit_native_preserves_work_and_entry_boundaries() {
    let total = "let parity = round % 2; let active = parity * parity; let normal = base * active; let increased = (base + 1) * active; if negative { return increased; } return normal;";
    let outlined = "if round % 2 == 0 { return 0; } if negative { return base + 1; } return base;";
    for kind in cases::KINDS {
        let source = cases::SOURCE.replace("f64", kind.ty());
        let seed = cases::seed(kind);
        let mut negative = seed;
        negative.base.arguments.child.base.initial = kind.last_state();
        let mut zero = seed;
        zero.base.arguments.child.base.initial = 0;
        for row in [seed, negative, zero] {
            let plan = row.plan(None);
            let limit = (row.base.arguments.parents + plan.requested.iter().sum::<i64>()) as u64;
            let rows = [sequence(&[row])];
            probe::check_sequences_with_i64_prints(
                &source,
                &rows,
                &[],
                &cases::observers(kind),
                limit,
                cases::ENTRY_LIMIT,
            );
            let trace = &rows[0].rows[0].trace;
            let at = cases::records(trace, true)
                .into_iter()
                .rev()
                .find(|record| record.1 == 76)
                .unwrap()
                .0;
            probe::check_i64_loop_work_limit_rejection(
                &source,
                &rows,
                &cases::observers(kind),
                limit - 1,
                cases::ENTRY_LIMIT,
                &trace[..at + 6],
            );
        }
        for limit in [8, 64] {
            probe::check_i64_helper_entry_limit_rejection(
                &source,
                &[sequence(&[seed])],
                &cases::observers(kind),
                cases::LOOP_LIMIT,
                limit,
            );
        }
        assert_eq!(source.matches(total).count(), 1);
        let expensive = source
            .replace(total, outlined)
            .replace("total = -(-total);", "total = finish(total, 1);");
        probe::check_i64_helper_entry_limit_rejection(
            &expensive,
            &[sequence(&[negative])],
            &cases::observers(kind),
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
        let mut rejected = sequence(&[seed]);
        let mut poison = seed;
        poison.control_update_fault = 48;
        rejected.rows[0].input = poison.words();
        rejected.rows[0].trace = poison.execute(None, true).unwrap_err();
        let trace = &rejected.rows[0].trace;
        let at = cases::records(trace, true)
            .into_iter()
            .rev()
            .find(|record| record.1 == 76)
            .unwrap()
            .0;
        let prefix = trace[..at + 6].to_vec();
        probe::check_i64_loop_work_limit_rejection(
            &source,
            &[rejected],
            &cases::observers(kind),
            10,
            cases::ENTRY_LIMIT,
            &prefix,
        );
    }
}

#[test]
fn state_checked_control_exit_native_requires_an_explicit_update_effect_grant() {
    use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};
    let signature = "fn advance_control(value: i64, fault: i64) -> i64 {";
    assert_eq!(cases::SOURCE.matches(signature).count(), 1);
    let source = cases::SOURCE.replace(signature, &format!("{signature} print(83);"));
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
            let bridge = emit_registered_with_i64_prints(
                &module,
                "counter",
                &full,
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            )
            .unwrap();
            assert_eq!(
                bridge
                    .callbacks
                    .iter()
                    .map(|callback| callback.arguments.len())
                    .collect::<Vec<_>>(),
                [38, 40, 2]
            );
            assert_eq!(bridge.state_fields.len(), 2);
            let producer = module
                .functions
                .iter()
                .find(|function| function.name == "advance_control")
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
            .expect("control producer inherited an effect grant");
            assert!(error.contains("print"), "{error}");
        }
    }
}

#[test]
fn state_checked_control_exit_oracle_decodes_records_and_pins_fault_priority() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 33);
        assert_eq!(cases::cli_sequences(kind).len(), 11);
        assert_eq!(cases::failures(kind).len(), 19);
        for rows in cases::sequences(kind) {
            let mut prior = None;
            for row in rows {
                let (state, trace) = row.execute(prior, true).unwrap();
                let plan = row.plan(prior);
                assert_eq!(state, plan.state);
                assert_eq!(row.words().len(), 38);
                assert_eq!(row.words()[..37], row.base.words());
                assert_eq!(row.words()[37], row.control_update_fault);
                assert!(row.args().starts_with(&format!("{},", row.base.args())));
                let records = cases::records(&trace, true);
                let entries = records
                    .iter()
                    .filter(|row| row.1 == 617)
                    .collect::<Vec<_>>();
                assert_eq!(entries.len(), plan.completed.iter().sum::<i64>() as usize);
                let mut control = prior.unwrap_or(row.base.initial())[1];
                for &(at, _) in entries {
                    assert_eq!(trace[at + 1], control);
                    assert_ne!(trace[at + 2], 0);
                    assert_eq!(trace[at + 3], 75);
                    control = control.wrapping_add(1);
                }
                assert_eq!(state[1], control);
                prior = Some(state);
            }
        }
        let prefix = cases::prefix(kind);
        let prior = prefix
            .iter()
            .fold(None, |prior, row| Some(row.execute(prior, true).unwrap().0));
        assert_eq!(prior, Some([kind.initial(), 0]));
        let completed = [0, 1, 2, 3, 4, 5, 6];
        for (i, bad) in cases::failures(kind).into_iter().enumerate() {
            let trace = bad.execute(prior, true).unwrap_err();
            let records = cases::records(&trace, true);
            let updates = records.iter().filter(|record| record.1 == 75).count();
            let calls = records.iter().filter(|record| record.1 == 617).count();
            if (8..15).contains(&i) {
                assert_eq!(updates, completed[i - 8]);
                assert_eq!(calls, updates + 1);
                assert!(trace.ends_with(&[617, updates as i64, 0]));
            }
            if matches!(i, 15 | 16) {
                assert!(trace.ends_with(&[74]));
                assert_eq!(calls, updates);
                assert_eq!(updates, if i == 15 { 0 } else { 3 });
            }
            if i >= 17 {
                assert!(trace.ends_with(&[617, if i == 17 { 0 } else { 6 }, 0]));
                assert_eq!(calls, updates + 1);
            }
        }
    }
}

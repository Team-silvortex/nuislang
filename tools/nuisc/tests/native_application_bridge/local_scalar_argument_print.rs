#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "local_scalar_argument_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn sequence(rows: &[cases::Case]) -> probe::Sequence {
    let mut previous = None;
    probe::Sequence {
        rows: rows
            .iter()
            .map(|row| {
                let (next, trace) = row.execute(previous, true).unwrap();
                previous = Some(next);
                probe::Row {
                    input: row.words(),
                    state: vec![next],
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
            0 if row.kind == cases::Kind::F32 => Value::F32(f32::from_bits(word as u32)),
            0 => Value::F64(f64::from_bits(word as u64)),
            1 => Value::Bool(word != 0),
            3 | 4 => Value::I32(word as i32),
            _ => Value::Int(word),
        })
        .collect()
}

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn local_scalar_argument_print_native_preserves_eager_initializers_and_fault_prefixes() {
    for kind in cases::KINDS {
        let prefix = cases::prefix(kind);
        let previous = prefix.iter().fold(None, |previous, row| {
            Some(row.execute(previous, true).unwrap().0)
        });
        let failures = cases::failures(kind)
            .into_iter()
            .map(|bad| probe::SequenceFailure {
                accepted: sequence(&prefix),
                input: bad.words(),
                before_failure: bad.execute(previous, true).unwrap_err(),
            })
            .collect::<Vec<_>>();
        let rows = cases::sequences(kind)
            .iter()
            .map(|rows| sequence(rows))
            .collect::<Vec<_>>();
        probe::check_sequences_with_i64_prints(
            &cases::source(kind),
            &rows,
            &failures,
            cases::OBSERVERS,
            cases::LOOP_LIMIT,
            cases::ENTRY_LIMIT,
        );
    }
}

#[test]
fn local_scalar_argument_print_reference_preserves_counts_state_and_faulted_close() {
    for kind in cases::KINDS {
        let project = Project::with_source(&cases::source(kind));
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
                let (mut previous, trace) = rows[0].execute(None, false).unwrap();
                assert_eq!(printed(&opened.events), trace);
                assert_eq!(state_words(session.state()), vec![previous as u64]);
                for row in &rows[1..] {
                    let (next, trace) = row.execute(Some(previous), false).unwrap();
                    let event = session.event(values(*row)).unwrap();
                    assert_eq!(printed(&event.events), trace);
                    assert_eq!(state_words(session.state()), vec![next as u64]);
                    let expected_calls = row.trips as usize * (2 + usize::from(row.selected));
                    assert_eq!(
                        event
                            .events
                            .iter()
                            .filter(|e| e.contains("cpu.call") && e.contains("] checked64("))
                            .count(),
                        expected_calls
                    );
                    previous = next;
                }
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
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
                let first = prefix[0].execute(None, false).unwrap().0;
                session.event(values(prefix[1])).unwrap();
                let previous = prefix[1].execute(Some(first), false).unwrap().0;
                assert!(session.event(values(bad)).is_err(), "{bad:?}");
                assert_eq!(state_words(session.state()), vec![previous as u64]);
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert!(session.event(values(prefix[0])).is_err());
                session.close(vec![]).unwrap();
                assert_eq!(state_words(session.state()), vec![previous as u64]);
                assert!(session.completion_status().is_err());
            }
        }
    }
}

#[test]
fn local_scalar_argument_print_oracle_covers_full_width_and_skipped_faults() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 14);
        assert_eq!(cases::cli_sequences(kind).len(), 10);
        assert_eq!(cases::failures(kind).len(), 8);
        for rows in cases::cli_sequences(kind) {
            for row in rows {
                assert!(kind
                    .decimal(row.initial)
                    .parse::<f64>()
                    .unwrap()
                    .is_finite());
            }
        }
        for (left, right) in [(i64::MIN, i64::MAX), (-1, 0), (4294967296, -107)] {
            assert_eq!(cases::xor_word(left, right), left ^ right);
        }
        let mut skipped = cases::seed(kind);
        skipped.selected = false;
        skipped.guards[1] = 0;
        assert!(skipped.execute(None, false).is_ok());
        skipped.guards[0] = 0;
        assert!(
            skipped.execute(None, false).is_err(),
            "a later lazy gate must not skip a local initializer"
        );
    }
}

#[test]
fn local_scalar_argument_print_native_rejects_unproved_epoch_initializers() {
    for kind in cases::KINDS {
        let source = cases::source(kind);
        for bad in [
            source.replace(
                "i64_from_i32(left) + 4294967296",
                "i64_from_i32(current.code) + 4294967296",
            ),
            source.replace(
                "current = Pair { total: -current.total, code: right, flag: selected };",
                "let premature = checked64(i64_from_i32(current.code) ^ parameter_word, lazy_guard); print(premature);
                 current = Pair { total: -current.total, code: right, flag: selected };",
            ),
        ] {
            assert_ne!(bad, source);
            // These are resolved, well-typed sources, not frontend unknown-name controls.
            nuisc::frontend::parse_nuis_module(&bad).unwrap();
            let project = Project::with_source(&bad);
            let Ok(compiled) = nuisc::pipeline::compile_project(&project.0) else {
                continue;
            };
            let sites = compiled.yir.nodes.iter().filter(|node| {
                matches!(node.op.instruction.as_str(), "print" | "guard_print")
            }).map(|node| node.name.clone()).collect::<Vec<_>>();
            let policy = yir_lower_llvm::native_session::I64PrintPolicy::new(sites).unwrap();
            assert!(yir_lower_llvm::native_session::emit_registered_with_i64_prints(
                &compiled.yir, "counter", &policy, cases::LOOP_LIMIT, cases::ENTRY_LIMIT,
            ).is_err(), "unproved epoch-local source gained native admission");
        }
    }
}

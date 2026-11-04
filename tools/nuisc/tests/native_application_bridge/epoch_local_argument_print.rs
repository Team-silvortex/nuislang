#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "epoch_local_argument_print_cases.rs"]
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
            0 if row.kind() == cases::Kind::F32 => Value::F32(f32::from_bits(word as u32)),
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

const SOURCE: &str = include_str!("epoch_local_argument_print.ns");

#[test]
fn epoch_local_argument_print_native_preserves_eager_initializers_and_fault_prefixes() {
    native_sequences(SOURCE, cases::LOOP_LIMIT, cases::ENTRY_LIMIT);
}

pub(super) fn native_sequences(template: &str, loop_limit: u64, entry_limit: u64) {
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
            &template.replace("f64", kind.ty()),
            &rows,
            &failures,
            cases::OBSERVERS,
            loop_limit,
            entry_limit,
        );
    }
}

#[test]
fn epoch_local_argument_print_reference_preserves_counts_state_and_faulted_close() {
    reference_sequences(SOURCE, 0);
}

pub(super) fn reference_sequences(template: &str, extra_entry_finishes: usize) {
    for kind in cases::KINDS {
        let project = Project::with_source(&template.replace("f64", kind.ty()));
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
                    let expected_calls =
                        row.base.trips as usize * (1 + usize::from(row.base.selected));
                    assert_eq!(
                        event
                            .events
                            .iter()
                            .filter(|e| e.contains("cpu.call") && e.contains("] checked64("))
                            .count(),
                        expected_calls
                    );
                    assert_eq!(
                        event
                            .events
                            .iter()
                            .filter(|e| e.contains("cpu.call") && e.contains("] finish("))
                            .count(),
                        row.base.trips as usize + 1 + extra_entry_finishes
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
fn epoch_local_argument_print_native_keeps_entry_exhaustion_before_publication() {
    entry_exhaustion(SOURCE, cases::LOOP_LIMIT);
}

pub(super) fn entry_exhaustion(template: &str, loop_limit: u64) {
    for kind in cases::KINDS {
        let mut row = cases::seed(kind);
        row.base.trips = 2;
        probe::check_i64_helper_entry_limit_rejection(
            &template.replace("f64", kind.ty()),
            &[sequence(&[row])],
            cases::OBSERVERS,
            loop_limit,
            8,
        );
    }
}

#[test]
fn epoch_local_argument_print_oracle_pins_zero_words_finite_cli_and_matrix_sizes() {
    for kind in cases::KINDS {
        assert_eq!(cases::sequences(kind).len(), 16);
        assert_eq!(cases::cli_sequences(kind).len(), 12);
        assert_eq!(cases::failures(kind).len(), 9);
        for rows in cases::cli_sequences(kind) {
            for row in rows {
                assert!(row
                    .kind()
                    .decimal(row.base.initial)
                    .parse::<f64>()
                    .unwrap()
                    .is_finite());
            }
        }
        let mut row = cases::seed(kind);
        row.base.selected = false;
        row.base.guards[1] = 0;
        assert!(row.execute(None, false).is_ok());
        row.base.guards[0] = 0;
        assert!(row.execute(None, false).is_err());
    }
}

#[test]
fn epoch_local_argument_print_native_rejects_escape_duplicate_and_nonroot_scopes() {
    reject_unproved_uses(SOURCE, cases::LOOP_LIMIT, cases::ENTRY_LIMIT);
}

pub(super) fn reject_unproved_uses(template: &str, loop_limit: u64, entry_limit: u64) {
    for kind in cases::KINDS {
        let source = template.replace("f64", kind.ty());
        for bad in [
            source.replace("print(74);", "print(epoch_word);"),
            source.replace(
                "current.flag && (checked64(epoch_word, lazy_guard) != 0)",
                "checked64(epoch_word, lazy_guard) != 0",
            ),
            source.replace(
                "checked64(epoch_word, lazy_guard)",
                "checked64(epoch_word ^ epoch_word, lazy_guard)",
            ),
            source
                .replace(
                    "let epoch_word: i64 =",
                    "if selected { let epoch_word: i64 =",
                )
                .replace("print(74);", "} print(74);"),
        ] {
            assert_ne!(bad, source);
            nuisc::frontend::parse_nuis_module(&bad).unwrap();
            let project = Project::with_source(&bad);
            let Ok(compiled) = nuisc::pipeline::compile_project(&project.0) else {
                continue;
            };
            let sites = compiled
                .yir
                .nodes
                .iter()
                .filter(|node| matches!(node.op.instruction.as_str(), "print" | "guard_print"))
                .map(|node| node.name.clone())
                .collect::<Vec<_>>();
            let policy = yir_lower_llvm::native_session::I64PrintPolicy::new(sites).unwrap();
            assert!(
                yir_lower_llvm::native_session::emit_registered_with_i64_prints(
                    &compiled.yir,
                    "counter",
                    &policy,
                    loop_limit,
                    entry_limit
                )
                .is_err()
            );
        }
    }
}

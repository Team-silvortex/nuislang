#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[allow(dead_code)]
#[path = "return_plain_const_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn values(case: cases::Case, bad: Option<usize>) -> Vec<Value> {
    let mut words = case.words();
    if let Some(slot) = bad {
        words[slot] = 0;
    }
    words
        .into_iter()
        .enumerate()
        .map(|(i, n)| {
            if (1..=3).contains(&i) {
                Value::Bool(n != 0)
            } else {
                Value::Int(n)
            }
        })
        .collect()
}

#[test]
fn return_plain_const_reference_all_leaf_kinds_and_atomic_failed_event_publication() {
    for source in cases::source_variants() {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for case in cases::cases() {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(case, None),
                )
                .unwrap();
                let printed = |events: &[String]| {
                    events
                        .iter()
                        .filter(|s| {
                            s.contains("cpu.print")
                                || (s.contains("cpu.guard_print")
                                    && s.contains("if true then print"))
                        })
                        .map(|s| s.rsplit_once(' ').unwrap().1.parse::<i64>().unwrap())
                        .collect::<Vec<_>>()
                };
                assert_eq!(printed(&opened.events), case.trace(false));
                assert_eq!(state_words(session.state()), vec![case.result() as u64]);
                let event = session.event(values(case.next(), None)).unwrap();
                assert_eq!(printed(&event.events), case.next().trace(false));
                assert_eq!(
                    state_words(session.state()),
                    vec![case.next().result() as u64]
                );
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
            }
            for (case, slot) in cases::failures() {
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(case.next(), None),
                )
                .unwrap();
                let last = session.state().clone();
                assert!(session
                    .event(values(case, Some(slot)))
                    .unwrap_err()
                    .contains("zero"));
                assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
                assert_eq!(session.state(), &last);
                assert!(session.event(values(case, None)).is_err());
                session.close(vec![]).unwrap();
                assert!(session.completion_status().is_err());
            }
        }
    }
}

fn row(case: cases::Case) -> probe::Row {
    probe::Row {
        input: case.words(),
        state: vec![case.result()],
        trace: case.trace(true),
    }
}

#[test]
fn return_plain_const_native_selected_only_callee_inputs_canaries_and_eight_traps() {
    let rows = cases::cases()
        .into_iter()
        .map(|c| (row(c), row(c.next())))
        .collect::<Vec<_>>();
    let failures = cases::failures()
        .into_iter()
        .map(|(c, slot)| {
            let mut input = c.words();
            input[slot] = 0;
            probe::Failure {
                accepted: row(c.next()),
                input,
                before_failure: c.before_failure(slot),
            }
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check(&source, &rows, &failures, cases::OBSERVER);
    }
}

use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[allow(dead_code)]
#[path = "return_typed_exit_prefix_cases.rs"]
mod cases;
use cases::Case;
#[path = "word_session_probe.rs"]
mod probe;

fn values(case: Case, bad: Option<usize>) -> Vec<Value> {
    let mut words = case.words();
    if let Some(slot) = bad {
        words[slot] = 0;
    }
    words
        .into_iter()
        .enumerate()
        .map(|(i, n)| match i {
            1 => Value::I32(n as i32),
            2 => Value::F32(f32::from_bits(n as u32)),
            3 => Value::F64(f64::from_bits(n as u64)),
            4..=8 => Value::Bool(n != 0),
            _ => Value::Int(n),
        })
        .collect()
}

fn trace(events: &[String]) -> Vec<i64> {
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
fn return_typed_exit_prefix_reference_five_current_fields_order_and_fault_publication() {
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
                assert_eq!(trace(&opened.events), case.trace(false));
                assert_eq!(state_words(session.state()), vec![case.result() as u64]);
                let event = session.event(values(case.next(), None)).unwrap();
                assert_eq!(trace(&event.events), case.next().trace(false));
                assert_eq!(
                    state_words(session.state()),
                    vec![case.next().result() as u64]
                );
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
            }
            for (case, slot, _) in cases::failures() {
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

fn row(case: Case) -> probe::Row {
    probe::Row {
        input: case.words(),
        state: vec![case.result()],
        trace: case.trace(true),
    }
}

#[test]
fn return_typed_exit_prefix_native_raw_ieee_current_fields_canaries_and_twelve_traps() {
    let rows = cases::cases()
        .into_iter()
        .map(|c| (row(c), row(c.next())))
        .collect::<Vec<_>>();
    let failures = cases::failures()
        .into_iter()
        .map(|(c, slot, stage)| {
            let mut input = c.words();
            input[slot] = 0;
            probe::Failure {
                accepted: row(c.next()),
                input,
                before_failure: c.before_failure(stage),
            }
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check(&source, &rows, &failures, cases::OBSERVER);
    }
}

#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
use std::collections::BTreeSet;
use yir_runtime_host::ApplicationSessionPhase;

#[allow(dead_code)]
#[path = "return_stateful_bool_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn values(case: cases::Case, bad: Option<usize>) -> Vec<Value> {
    let mut words = case.words();
    if let Some(slot) = bad {
        assert_eq!(words[slot], 1);
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

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|s| {
            s.contains("cpu.print")
                || (s.contains("cpu.guard_print") && s.contains("if true then print"))
        })
        .map(|s| s.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn return_stateful_bool_oracle_distinguishes_open_stale_ignored_and_repeated_input_results() {
    assert_eq!(cases::cases().len(), 16);
    assert_eq!(
        cases::source_variants()
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        4
    );
    for case in cases::cases() {
        let open = case.result(case.flag);
        let events = case.events();
        let mut previous = open;
        let mut results = vec![];
        let mut ignored = false;
        let mut ignores_false = false;
        let mut ignores_true = false;
        for (index, event) in events.iter().enumerate() {
            let result = event.result(previous);
            ignored |= result != event.result(event.flag);
            ignores_false |= result != event.result(false);
            ignores_true |= result != event.result(true);
            if index == 1 {
                assert_ne!(previous, open);
                assert_ne!(result, event.result(open));
            }
            results.push(result);
            previous = result;
        }
        assert!(ignored && ignores_false && ignores_true, "{case:?}");
        assert_eq!(events[1], events[2]);
        assert_eq!(events[2], events[4]);
        assert!(
            results[1] != results[2] || results[2] != results[4],
            "{case:?}"
        );
        assert_ne!(case.result(false), case.result(true));
        if case.gate && case.first {
            assert_ne!(case.result(false), case.seed == 9);
        }
    }
}

#[test]
fn return_stateful_bool_reference_consumes_latest_state_and_retains_failed_event_publication() {
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
                assert_eq!(printed(&opened.events), case.trace(case.flag, false));
                let mut previous = case.result(case.flag);
                assert_eq!(state_words(session.state()), vec![u64::from(previous)]);
                for event in case.events() {
                    let result = session.event(values(event, None)).unwrap();
                    assert_eq!(printed(&result.events), event.trace(previous, false));
                    previous = event.result(previous);
                    assert_eq!(state_words(session.state()), vec![u64::from(previous)]);
                }
                session.close(vec![]).unwrap();
                assert_eq!(state_words(session.state()), vec![u64::from(previous)]);
                session.completion_status().unwrap();
            }
            for (bad, slot) in cases::failures() {
                let [open, first, second] = cases::failure_prefix();
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(open, None),
                )
                .unwrap();
                let opened = session.state().clone();
                session.event(values(first, None)).unwrap();
                session.event(values(second, None)).unwrap();
                let last = session.state().clone();
                assert_ne!(last, opened);
                assert_eq!(state_words(&last), vec![1]);
                assert!(session
                    .event(values(bad, Some(slot)))
                    .unwrap_err()
                    .contains("zero"));
                assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
                assert_eq!(session.state(), &last);
                assert!(session.event(values(bad, None)).is_err());
                session.close(vec![]).unwrap();
                assert_eq!(session.state(), &last);
                assert!(session.completion_status().is_err());
            }
        }
    }
}

fn sequence(rows: impl IntoIterator<Item = cases::Case>) -> probe::Sequence {
    let mut previous = None;
    let rows = rows
        .into_iter()
        .map(|case| {
            let input_state = previous.unwrap_or(case.flag);
            let result = case.result(input_state);
            previous = Some(result);
            probe::Row {
                input: case.words(),
                state: vec![i64::from(result)],
                trace: case.trace(input_state, true),
            }
        })
        .collect();
    probe::Sequence { rows }
}

#[test]
fn return_stateful_bool_native_reuses_real_outputs_across_five_events_and_eight_traps() {
    let rows = cases::cases()
        .into_iter()
        .map(|case| sequence(std::iter::once(case).chain(case.events())))
        .collect::<Vec<_>>();
    let failures = cases::failures()
        .into_iter()
        .map(|(bad, slot)| {
            let accepted = sequence(cases::failure_prefix());
            assert_eq!(accepted.rows[0].state, vec![0]);
            assert_eq!(accepted.rows.last().unwrap().state, vec![1]);
            let mut input = bad.words();
            assert_eq!(input[slot], 1);
            input[slot] = 0;
            probe::SequenceFailure {
                accepted,
                input,
                before_failure: bad.before_failure(slot),
            }
        })
        .collect::<Vec<_>>();
    for source in cases::source_variants() {
        probe::check_sequences(&source, &rows, &failures, cases::OBSERVER);
    }
}

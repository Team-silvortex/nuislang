use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_nested_in_fields_oracle.rs"]
mod oracle;
use oracle::{Case, SOURCE, STAGES};

fn arguments(case: &Case, bad: Option<usize>, open: bool) -> Vec<Value> {
    let mut args = if open {
        vec![
            Value::Int(case.seed),
            Value::I32(case.small_seed),
            Value::F32(f32::from_bits(case.narrow_seed as u32)),
            Value::F64(f64::from_bits(case.wide_seed as u64)),
            Value::Bool(case.bool_seed),
            Value::Bool(case.peer_seed),
        ]
    } else {
        vec![]
    };
    let values = bad.map_or_else(|| case.values(), |index| case.bad_values(index));
    args.extend(values.iter().enumerate().map(|(index, value)| {
        if index < 2 {
            Value::Bool(value == "true")
        } else {
            Value::Int(value.parse().unwrap())
        }
    }));
    args
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
fn state(session: &ApplicationSession<'_>, case: &Case) {
    assert_eq!(
        state_words(session.state())
            .into_iter()
            .map(|w| w as i64)
            .collect::<Vec<_>>(),
        case.state_words()
    );
    assert_eq!(
        format!("accepted:{}", session.state()),
        case.state_text("accepted")
    );
}

#[test]
fn return_nested_in_fields_reference_values_order_and_last_publication() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for opened in oracle::cases() {
                assert_eq!(opened.open_args().split(',').count(), 21);
                let previous = opened.previous();
                let (mut session, trace) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    arguments(&opened, None, true),
                )
                .unwrap();
                assert_eq!(printed(&trace.events), opened.trace);
                state(&session, &opened);
                let trace = session.event(arguments(&previous, None, false)).unwrap();
                assert_eq!(printed(&trace.events), previous.trace);
                state(&session, &previous);
                session.close(vec![]).unwrap();
                state(&session, &previous);
                session.completion_status().unwrap();
            }
            for (index, opened) in oracle::selected().iter().enumerate() {
                let previous = opened.previous();
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    arguments(opened, None, true),
                )
                .unwrap();
                session.event(arguments(&previous, None, false)).unwrap();
                let last = session.state().clone();
                let error = session
                    .event(arguments(opened, Some(index), false))
                    .unwrap_err();
                assert!(error.contains("zero"), "stage={}: {error}", STAGES[index]);
                assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
                assert_eq!(session.state(), &last);
                assert!(session.event(arguments(opened, None, false)).is_err());
                session.close(vec![]).unwrap();
                assert_eq!(session.state(), &last);
                assert!(session.completion_status().is_err());
            }
        }
    }
}
#[test]
fn return_nested_in_fields_reference_prefix_and_joined_observations_and_dead_work() {
    assert_eq!(oracle::cases().len(), 80);
    assert_eq!(oracle::cli_cases().len(), 16);
    let cases = oracle::cli_cases();
    let exited = cases.iter().filter(|c| c.exited).collect::<Vec<_>>();
    assert!(exited
        .iter()
        .all(|c| c.first && c.second && c.fields.is_none()));
    for c in exited {
        for index in [5, 7, 8, 10, 11] {
            assert_eq!(c.words()[index + 2], 0);
        }
        assert!(!c.trace.contains(&96));
        assert!(c.prefix_fields.is_some());
    }
    let silent = cases
        .iter()
        .find(|c| c.exited && !c.trace.contains(&97))
        .unwrap();
    assert_eq!(silent.words()[14], 0);
    let retained = cases
        .iter()
        .find(|c| c.first && !c.second && c.bool_seed != c.peer_seed)
        .unwrap();
    assert!(!retained.exited);
    assert_eq!(retained.words()[11], 0);
    assert!(retained.fields.is_some());
    let continuing = cases
        .iter()
        .filter(|c| c.first && c.second && !c.exited)
        .collect::<Vec<_>>();
    assert_eq!(continuing.len(), 2);
    for c in continuing {
        let (_, a, b, before_wide, d, before_peer) = c.prefix_fields.unwrap();
        let (_, after_a, after_b, after_wide, after_d, after_peer) = c.fields.unwrap();
        assert_eq!((after_a, after_b, after_d), (a, b, d));
        assert!(c.trace.contains(&75));
        assert!(c.trace.contains(&78));
        if d != 0 {
            assert_eq!(after_wide, before_wide ^ i64::MIN);
            assert_eq!(after_peer, before_peer);
            assert!(c.trace.contains(&74));
        } else {
            assert_eq!(after_wide, before_wide);
            assert_eq!(after_peer, 1);
            assert!(!c.trace.contains(&74));
            assert_eq!(c.words()[7], 0);
        }
    }
    for profile in [false, true] {
        let marker = oracle::native_selected(profile).pop().unwrap();
        assert_eq!(marker.prefix_fields, Some((70001, -70002, 76, 96, 1, 1)));
        assert_eq!(
            marker.fields,
            Some((70001, -70002, 76, i64::MIN | 96, 1, 1))
        );
        assert_eq!(marker.before_failure(12).last(), Some(&95));
        assert_eq!(marker.before_failure(11).last(), Some(&95));
        if profile {
            assert!(marker.before_failure(11).contains(&96));
        }
    }
}
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_nested_in_fields_native.rs"]
mod native;

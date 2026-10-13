use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_tail_in_fields_oracle.rs"]
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
fn return_tail_in_fields_reference_values_order_and_last_publication() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for opened in oracle::cases() {
                assert_eq!(opened.open_args().split(',').count(), 31);
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
fn return_tail_in_fields_reference_readonly_fields_retention_and_dead_choices() {
    assert_eq!(oracle::cases().len(), 80);
    let cases = oracle::cli_cases();
    for c in cases.iter().filter(|c| c.exited || !c.first) {
        for index in [20, 21, 22] {
            assert_eq!(c.words()[index + 2], 0);
        }
        assert!(!c.trace.contains(&66));
    }
    for c in cases.iter().filter(|c| c.first && !c.exited) {
        let (owner, a, b, wide, flag, peer) = c.fields.unwrap();
        let parity = (a < 0)
            ^ (f32::from_bits(b as u32) < 0.0)
            ^ (f64::from_bits(wide as u64) < 0.0)
            ^ (flag != 0)
            ^ (peer != 0);
        assert!(c.trace.contains(&66));
        assert_eq!(c.trace.contains(&65), parity && flag != 0);
        assert_eq!(c.trace.contains(&64), !parity);
        assert_eq!(c.owner, if parity && flag == 0 { !owner } else { owner });
    }
    assert!(cases.iter().any(|c| c.trace.contains(&65)));
    assert!(cases.iter().any(|c| c.trace.contains(&64)));
    assert!(cases
        .iter()
        .any(|c| c.trace.contains(&66) && !c.trace.contains(&65) && !c.trace.contains(&64)));
    for profile in [false, true] {
        let marker = oracle::native_selected(profile).pop().unwrap();
        assert_eq!(marker.before_failure(22).last(), Some(&66));
    }
}
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_tail_in_fields_native.rs"]
mod native;

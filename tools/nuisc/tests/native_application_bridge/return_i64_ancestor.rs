use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_i64_ancestor_oracle.rs"]
mod oracle;
use oracle::{Case, SOURCE, STAGES};

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
const OWNER_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n";

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_i32_ancestor_probe.rs"]
mod probe;

fn arguments(case: &Case, bad: Option<usize>, open: bool) -> Vec<Value> {
    let mut args = if open {
        vec![Value::Int(i64::from(case.seed))]
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
fn return_i64_ancestor_private_bool_reference_values_order_and_last_publication() {
    let project = Project::with_source(SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        for opened in oracle::cases() {
            assert_eq!(opened.open_args().split(',').count(), 12);
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

#[test]
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
fn return_i64_ancestor_private_bool_native_exact_values_and_order() {
    probe::check(SOURCE, &oracle::cases(), false, None);
}

#[test]
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
fn return_i64_ancestor_private_bool_native_real_event_traps() {
    probe::check(SOURCE, &oracle::selected(), true, None);
}

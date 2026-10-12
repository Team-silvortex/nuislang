use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_i64_i32_oracle.rs"]
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

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
const PAIR_PROBE: &str =
    "  call void @nuis_debug_print_i64(i64 %arg0)\n  %observed_pair_child = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_pair_child)\n";

fn arguments(case: &Case, bad: Option<usize>, open: bool) -> Vec<Value> {
    let mut args = if open {
        vec![Value::Int(case.seed), Value::I32(case.child_seed)]
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
fn return_i64_i32_private_pair_reference_values_order_and_last_publication() {
    let project = Project::with_source(SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        for opened in oracle::cases() {
            assert_eq!(opened.open_args().split(',').count(), 13);
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
fn return_i64_i32_private_pair_independent_extrema_and_marker_fault_prefixes() {
    let cases = oracle::cases();
    assert_eq!(cases.len(), 13 * 11 * 4);
    assert_eq!(cases.iter().filter(|c| c.pair.is_some()).count(), 143);
    assert!(cases
        .iter()
        .any(|c| c.pair.is_some_and(|(a, _)| a != c.owner)));
    assert!(cases.iter().any(|c| c.observed().contains(&-70002)));
    let case = Case::new(70001, -97, true, true);
    assert_eq!(case.pair, Some((-70002, 96)));
    assert_eq!(oracle::selected()[7].pair, case.pair);
    assert_eq!(oracle::selected()[8].pair, case.pair);
    assert_eq!(case.before_failure(6), [99, 90, 87, 91, 92, 95, 93]);
    assert_eq!(
        case.before_failure(7),
        [99, 90, 87, 91, 92, 95, 93, -70002, 96, 96]
    );
    assert_eq!(
        case.before_failure(8),
        [99, 90, 87, 91, 92, 95, 93, -70002, 96, 96, 83]
    );
}

#[test]
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
fn return_i64_i32_private_pair_native_exact_values_and_order() {
    probe::check_with_open_words(
        SOURCE,
        &oracle::cases(),
        false,
        Some(("after_work", PAIR_PROBE)),
        Case::open_words,
    );
}

#[test]
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
fn return_i64_i32_private_pair_native_real_event_traps() {
    probe::check_with_open_words(
        SOURCE,
        &oracle::selected(),
        true,
        Some(("after_work", PAIR_PROBE)),
        Case::open_words,
    );
}

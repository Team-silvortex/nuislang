use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "private_event_cases.rs"]
mod cases;
use cases::{Case, Family};

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "private_event_failure_probe.rs"]
mod probe;

fn arguments(case: &Case, values: &[String], open: bool) -> Vec<Value> {
    let mut args = Vec::new();
    if open {
        args.push(match case.family {
            Family::Bool => Value::Bool(case.seed_word != 0),
            Family::I32 | Family::I64 => Value::Int(case.seed_word as i64),
            Family::F32 => Value::F32(f32::from_bits(case.seed_word as u32)),
            Family::F64 => Value::F64(f64::from_bits(case.seed_word)),
        });
    }
    args.extend(values.iter().enumerate().map(|(slot, value)| {
        if slot < case.flags.len() {
            Value::Bool(value == "true")
        } else {
            Value::Int(value.parse().unwrap())
        }
    }));
    args
}

fn printed(events: &[String]) -> Vec<i32> {
    events
        .iter()
        .filter(|event| {
            event.contains("cpu.print")
                || (event.contains("cpu.guard_print") && event.contains("if true then print"))
        })
        .map(|event| event.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

fn reference(family: Family) {
    let project = Project::with_source(&family.source());
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        for (index, opened) in family.selected().iter().enumerate() {
            let previous = opened.previous();
            let (mut session, trace) = ApplicationSession::open_registered(
                &module,
                &registry,
                "counter",
                arguments(opened, &opened.values, true),
            )
            .unwrap_or_else(|error| panic!("{}: {error}", opened.open_args()));
            assert_eq!(printed(&trace.events), opened.trace);
            assert_eq!(state_words(session.state()), opened.state_words());
            assert_eq!(
                opened.state_text("open"),
                format!("open:{}", session.state())
            );
            let trace = session
                .event(arguments(&previous, &previous.values, false))
                .unwrap();
            assert_eq!(printed(&trace.events), previous.trace);
            assert_eq!(state_words(session.state()), previous.state_words());
            let last = session.state().clone();
            assert_ne!(opened.state_words(), previous.state_words());
            let error = session
                .event(arguments(opened, &opened.bad_values(index), false))
                .unwrap_err();
            assert!(error.contains("zero"), "{family:?} stage={index}: {error}");
            assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
            assert_eq!(session.state(), &last);
            assert!(session
                .event(arguments(opened, &opened.values, false))
                .is_err());
            assert_eq!(session.state(), &last);
            // Reference errors are recoverable: one explicit close is permitted
            // with the last accepted state, but cannot erase the original failure.
            let closed = session.close(vec![]).unwrap().unwrap();
            assert!(printed(&closed.events).is_empty());
            assert_eq!(session.state(), &last);
            assert!(session.close(vec![]).unwrap().is_none());
            assert!(session.completion_status().is_err());
        }
    }
}

macro_rules! family_tests {
    ($native:ident, $reference:ident, $family:ident) => {
        #[test]
        #[cfg(all(
            any(target_os = "macos", target_os = "linux"),
            target_pointer_width = "64"
        ))]
        fn $native() {
            probe::check(Family::$family);
        }
        #[test]
        fn $reference() {
            reference(Family::$family);
        }
    };
}
family_tests!(
    private_event_failure_bool_native_order,
    private_event_failure_bool_reference_atomicity,
    Bool
);
family_tests!(
    private_event_failure_i32_native_order,
    private_event_failure_i32_reference_atomicity,
    I32
);
family_tests!(
    private_event_failure_i64_native_order,
    private_event_failure_i64_reference_atomicity,
    I64
);
family_tests!(
    private_event_failure_f32_native_order,
    private_event_failure_f32_reference_atomicity,
    F32
);
family_tests!(
    private_event_failure_f64_native_order,
    private_event_failure_f64_reference_atomicity,
    F64
);

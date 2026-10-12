use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_i64_float_oracle.rs"]
mod oracle;
use oracle::{Case, SOURCE, STAGES};

fn arguments(case: &Case, bad: Option<usize>, open: bool) -> Vec<Value> {
    let mut args = if open {
        vec![if case.ty == "f32" {
            Value::F32(f32::from_bits(case.seed as u32))
        } else {
            Value::F64(f64::from_bits(case.seed as u64))
        }]
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
fn return_i64_float_private_pair_reference_values_order_and_last_publication() {
    for ty in ["f32", "f64"] {
        let project = Project::with_source(&SOURCE.replace("f64", ty));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for opened in oracle::cli_cases(ty) {
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
            for (index, opened) in oracle::selected(ty).iter().enumerate() {
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
fn return_i64_float_private_pair_observation_prefix_keeps_marker_values_before_real_late_faults() {
    for ty in ["f32", "f64"] {
        let case = Case::new(ty, 0, true, true);
        assert_eq!(case.pair, Some((96, 0)));
        assert_eq!(oracle::selected(ty)[7].pair, Some((96, 0)));
        assert_eq!(oracle::selected(ty)[8].pair, Some((96, 0)));
        assert_eq!(case.before_failure(6), [99, 90, 87, 91, 92, 93, 95, 95, 93]);
        assert_eq!(
            case.before_failure(7),
            [99, 90, 87, 91, 92, 93, 95, 95, 93, 96, 0, 96]
        );
        assert_eq!(
            case.before_failure(8),
            [99, 90, 87, 91, 92, 93, 95, 95, 93, 96, 0, 96, 83]
        );
    }
}

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
const OWNER_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n";
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_i64_float_native_f32.rs"]
mod native_f32;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_i64_float_native_f64.rs"]
mod native_f64;

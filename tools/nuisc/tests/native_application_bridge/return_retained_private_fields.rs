use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_retained_private_fields_oracle.rs"]
mod oracle;
use oracle::{Case, SOURCE, STAGES};

fn arguments(case: &Case, bad: Option<usize>, open: bool) -> Vec<Value> {
    let mut args = if open {
        vec![
            Value::Int(case.seed),
            Value::I32(case.small_seed),
            if case.ty == "f32" {
                Value::F32(f32::from_bits(case.float_seed as u32))
            } else {
                Value::F64(f64::from_bits(case.float_seed as u64))
            },
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
fn return_retained_private_fields_reference_values_order_and_last_publication() {
    for ty in ["f32", "f64"] {
        let project = Project::with_source(&SOURCE.replace("f64", ty));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for opened in oracle::cases(ty) {
                assert_eq!(opened.open_args().split(',').count(), 16);
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
fn return_retained_private_fields_independent_extrema_and_marker_fault_prefixes() {
    for ty in ["f32", "f64"] {
        let cases = oracle::cases(ty);
        assert_eq!(cases.len(), 3 * 4 * 12 * 4);
        assert_eq!(cases.iter().filter(|c| c.fields.is_some()).count(), 144);
        assert!(cases
            .iter()
            .any(|c| c.fields.is_some_and(|(owner, _, _)| owner != c.owner)));
        let tiny = Case::retained(ty, i64::MIN, i32::MIN, 1, false, false);
        assert_eq!(tiny.state_words()[2], 1);
        if ty == "f32" {
            assert_ne!(tiny.decimal(), tiny.state_decimal());
        }
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        let marker = Case::retained(ty, 70001, 70001, sign | 96, true, true);
        assert_eq!(marker.fields, Some((70001, -70002, 96)));
        assert_eq!(marker.before_failure(8), [99, 90, 87, 88, 91, 92, 94, 93]);
        assert_eq!(
            marker.before_failure(9),
            [99, 90, 87, 88, 91, 92, 94, 93, 70001, -70002, 96, 96, 95, 95]
        );
        assert_eq!(
            marker.before_failure(10),
            [99, 90, 87, 88, 91, 92, 94, 93, 70001, -70002, 96, 96, 95, 95, 83]
        );
        for c in oracle::native_selected(ty).iter().skip(9) {
            assert_eq!(c.fields, marker.fields);
        }
        let cli = oracle::cli_cases(ty);
        assert_eq!(cli.len(), 24);
        assert!(cli.iter().any(|c| c.small_seed == 0
            && c.first
            && c.second
            && c.trace.iter().filter(|&&n| n == 94).count() > 1));
    }
}

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
const OWNER_PROBE: &str = "  %observed_owner = add i64 %arg0, 0\n  call void @nuis_debug_print_i64(i64 %observed_owner)\n";
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_retained_private_fields_native_f32.rs"]
mod native_f32;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_retained_private_fields_native_f64.rs"]
mod native_f64;

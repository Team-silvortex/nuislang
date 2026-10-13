use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_multi_float_field_vector_oracle.rs"]
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
fn return_multi_float_field_vector_reference_values_order_and_last_publication() {
    let project = Project::with_source(SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        for opened in oracle::cases() {
            assert_eq!(opened.open_args().split(',').count(), 25);
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
fn return_multi_float_field_vector_current_narrow_reads_and_masked_wide_bits() {
    assert_eq!(oracle::cases().len(), 12 * 2 * 2 * 2 * 4 * 4 + 8 + 4);
    assert_eq!(oracle::cli_cases().len(), 18);
    for current in oracle::masked_cases() {
        let stale = current.stale_narrow();
        assert_ne!(current.fields, stale.fields);
        assert_eq!(
            current.owner, stale.owner,
            "NaN/zero comparisons hide a stale wide sign"
        );
        assert_eq!(current.result, stale.result);
        assert_eq!(current.trace, stale.trace);
    }
    for value in [-2.5, 1.5] {
        let current = Case::new(
            i64::MAX,
            i32::MIN,
            oracle::narrow_bits(value),
            oracle::wide_bits(4.75),
            false,
            true,
            true,
            true,
        );
        let stale = current.stale_narrow();
        assert_ne!(current.fields, stale.fields);
        assert_ne!(current.owner, stale.owner);
        assert_ne!(current.result, stale.result);
        assert_eq!(current.trace, stale.trace);
        let source = SOURCE.replace("wide_work(c, b,", "wide_work(c, narrow_seed,");
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let (mut session, trace) = ApplicationSession::open_registered(
                &module,
                &registry,
                "counter",
                arguments(&current, None, true),
            )
            .unwrap();
            assert_eq!(printed(&trace.events), current.trace);
            state(&session, &stale);
            session.close(vec![]).unwrap();
        }
    }
    let marker = oracle::native_selected().pop().unwrap();
    assert_eq!(marker.fields, Some((70001, -70002, 93, 96, 1, 1)));
    assert_eq!(marker.owner, -70002);
    assert!(!marker.before_failure(14).ends_with(&[96]));
    assert_eq!(
        &marker.before_failure(15)[14..20],
        &[70001, -70002, 93, 96, 1, 1]
    );
    assert_eq!(marker.before_failure(16).last(), Some(&83));
    assert!(oracle::cli_cases()
        .iter()
        .any(|c| c.fields.is_some() && c.words()[14] == 0));
    assert!(oracle::cli_cases()
        .iter()
        .any(|c| c.fields.is_some() && c.words()[15] == 0));
}
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_multi_float_field_vector_native.rs"]
mod native;

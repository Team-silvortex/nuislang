use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_cross_width_field_vector_oracle.rs"]
mod oracle;
use oracle::{Case, STAGES};

fn arguments(case: &Case, bad: Option<usize>, open: bool) -> Vec<Value> {
    let mut args = if open {
        vec![
            if case.ty == "f32" {
                Value::F32(f32::from_bits(case.seed as u32))
            } else {
                Value::F64(f64::from_bits(case.seed as u64))
            },
            Value::I32(case.small_seed),
            if case.child_ty == "f32" {
                Value::F32(f32::from_bits(case.float_seed as u32))
            } else {
                Value::F64(f64::from_bits(case.float_seed as u64))
            },
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
fn return_cross_width_field_vector_reference_values_order_and_last_publication() {
    for ty in ["f32", "f64"] {
        let project = Project::with_source(&oracle::source(ty));
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for opened in oracle::cases(ty) {
                assert_eq!(opened.open_args().split(',').count(), 22);
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
fn return_cross_width_field_vector_detects_equality_masked_owner_and_stale_siblings() {
    for ty in ["f32", "f64"] {
        assert_eq!(oracle::cases(ty).len(), 12 * 2 * 4 * 4 * 4 + 8);
        assert_eq!(oracle::cli_cases(ty).len(), 18);
        for current in oracle::masked_cases(ty) {
            let stale = current.stale_sibling();
            assert_ne!(current.fields, stale.fields);
            assert_ne!(current.owner, stale.owner);
            assert_eq!(
                current.result, stale.result,
                "equality masks wrong owner bits"
            );
            assert_eq!(current.trace, stale.trace);
        }
        for value in [-2.5, 1.5] {
            let current = Case::cross(
                ty,
                oracle::child_type(ty),
                oracle::bits(ty, 3.25),
                i32::MIN,
                oracle::bits(oracle::child_type(ty), value),
                false,
                true,
                true,
                true,
            );
            let stale = current.stale_sibling();
            assert_ne!(current.owner, stale.owner);
            assert_ne!(current.result, stale.result);
            assert_eq!(current.trace, stale.trace);
            let source = oracle::source(ty)
                .replace("d && peer_work(c,", "d && peer_work(bool_seed,")
                .replace("c || bool_work(d,", "c || bool_work(peer_seed,");
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
        let marker = oracle::native_selected(ty).pop().unwrap();
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        assert_eq!(marker.fields, Some((sign | 95, -70002, 96, 1, 1)));
        assert_eq!(marker.owner, 95);
        assert_eq!(
            marker.before_failure(12),
            [99, 90, 87, 88, 91, 82, 84, 92, 94, 93, 81, 78]
        );
        for index in [13, 14] {
            let prefix = marker.before_failure(index);
            assert_eq!(&prefix[12..17], &[sign | 95, -70002, 96, 1, 1]);
            assert_eq!(*prefix.last().unwrap(), if index == 13 { 95 } else { 83 });
        }
        let cli = oracle::cli_cases(ty);
        assert!(cli.iter().any(|c| c.fields.is_some() && c.words()[12] == 0));
        assert!(cli.iter().any(|c| c.fields.is_some() && c.words()[13] == 0));
        assert_eq!(marker.previous().seed, marker.seed);
    }
}

#[test]
fn return_cross_width_field_vector_oracle_keeps_widths_and_raw_words_independent() {
    for ty in ["f32", "f64"] {
        let child = oracle::child_type(ty);
        assert_ne!(ty, child);
        for case in oracle::cases(ty) {
            assert_eq!(case.child_ty, child);
            assert_eq!(case.previous().child_ty, child);
            assert_eq!(case.stale_sibling().child_ty, child);
            assert_eq!(case.open_words()[0], case.seed);
            assert_eq!(case.open_words()[2], case.float_seed);
            assert!(case.state_text("state").contains(&format!(
                "float_seed: {}{child}",
                if case.child_ty == "f32" {
                    f64::from(f32::from_bits(case.float_seed as u32)).to_string()
                } else {
                    f64::from_bits(case.float_seed as u64).to_string()
                }
            )));
            if ty == "f32" {
                assert!(u32::try_from(case.seed).is_ok());
            }
            if child == "f32" {
                assert!(u32::try_from(case.float_seed).is_ok());
            }
        }
        let same = Case::new(
            ty,
            oracle::bits(ty, 3.25),
            i32::MIN,
            oracle::bits(ty, -2.5),
            false,
            true,
            true,
            true,
        );
        let cross = Case::cross(
            ty,
            child,
            same.seed,
            same.small_seed,
            oracle::bits(child, -2.5),
            same.bool_seed,
            same.peer_seed,
            true,
            true,
        );
        assert_eq!(same.owner, cross.owner);
        assert_eq!(same.trace, cross.trace);
        assert_ne!(same.float_seed, cross.float_seed);
        assert_ne!(same.state_text("state"), cross.state_text("state"));
    }
}

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_cross_width_field_vector_native_f32.rs"]
mod native_f32;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_cross_width_field_vector_native_f64.rs"]
mod native_f64;

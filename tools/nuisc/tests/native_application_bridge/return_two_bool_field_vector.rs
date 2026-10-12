use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_two_bool_field_vector_oracle.rs"]
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
fn return_two_bool_field_vector_reference_values_order_and_last_publication() {
    for ty in ["f32", "f64"] {
        let project = Project::with_source(&SOURCE.replace("f64", ty));
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
fn return_two_bool_field_vector_independent_seeds_stale_mutants_and_marker_prefixes() {
    for ty in ["f32", "f64"] {
        let cases = oracle::cases(ty);
        assert_eq!(cases.len(), 3 * 4 * 12 * 4 * 4);
        assert_eq!(cases.iter().filter(|c| c.fields.is_some()).count(), 576);
        let flags = cases
            .iter()
            .filter_map(|c| c.fields.map(|(_, _, _, a, b)| (a, b)))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            flags,
            std::collections::BTreeSet::from([(0, 0), (0, 1), (1, 0), (1, 1)])
        );
        // Independent changes to either boolean seed must affect actual computation.
        let base = Case::new(
            ty,
            i64::MAX,
            i32::MIN,
            oracle::bits(ty, -2.5),
            false,
            false,
            true,
            true,
        );
        let changed_c = Case::new(
            ty,
            base.seed,
            base.small_seed,
            base.float_seed,
            true,
            false,
            true,
            true,
        );
        let changed_d = Case::new(
            ty,
            base.seed,
            base.small_seed,
            base.float_seed,
            false,
            true,
            true,
            true,
        );
        assert_ne!(base.owner, changed_c.owner);
        assert_ne!(base.owner, changed_d.owner);
        for value in [-2.5, 1.5] {
            let current = Case::new(
                ty,
                i64::MAX,
                i32::MIN,
                oracle::bits(ty, value),
                false,
                true,
                true,
                true,
            );
            let stale = current.stale_sibling();
            assert_ne!(
                current.fields, stale.fields,
                "updated sibling must be observable"
            );
            assert_ne!(current.owner, stale.owner);
            assert_ne!(current.result, stale.result);
            assert_eq!(
                current.trace, stale.trace,
                "mutant differs in data, not selected calls"
            );
            let source = SOURCE
                .replace("d && peer_work(c,", "d && peer_work(bool_seed,")
                .replace("c || bool_work(d,", "c || bool_work(peer_seed,")
                .replace("f64", ty);
            let project = Project::with_source(&source);
            let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
            for reversed in [false, true] {
                let module = typed_effect_returns::reordered(&compiled.yir, reversed);
                let registry = yir_verify::default_registry();
                let (mut session, trace) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    arguments(&current, None, true),
                )
                .unwrap();
                assert_eq!(printed(&trace.events), current.trace);
                state(&session, &stale);
                assert_ne!(
                    state_words(session.state())
                        .into_iter()
                        .map(|w| w as i64)
                        .collect::<Vec<_>>(),
                    current.state_words()
                );
                session.close(vec![]).unwrap();
            }
        }
        let sign = if ty == "f32" { 1 << 31 } else { i64::MIN };
        let marker = Case::new(ty, 70001, 70001, sign | 96, false, true, true, true);
        assert_eq!(marker.fields, Some((70001, -70002, 96, 1, 1)));
        assert_eq!(marker.owner, -70002);
        assert_eq!(
            marker.before_failure(12),
            [99, 90, 87, 88, 91, 82, 84, 92, 94, 93, 81, 78]
        );
        assert_eq!(
            marker.before_failure(13),
            [
                99, 90, 87, 88, 91, 82, 84, 92, 94, 93, 81, 78, 70001, -70002, 96, 1, 1, 96, 95,
                78, 95
            ]
        );
        assert_eq!(
            marker.before_failure(14),
            [
                99, 90, 87, 88, 91, 82, 84, 92, 94, 93, 81, 78, 70001, -70002, 96, 1, 1, 96, 95,
                78, 95, 83
            ]
        );
        let cli = oracle::cli_cases(ty);
        assert_eq!(cli.len(), 34);
        let and_skipped = cli
            .iter()
            .find(|c| c.first && c.second && c.float_seed == oracle::bits(ty, -2.5) && !c.peer_seed)
            .unwrap();
        assert_eq!(
            and_skipped.words()[13],
            0,
            "inactive AND sibling RHS must stay poisoned"
        );
        let or_skipped = cli
            .iter()
            .find(|c| c.first && c.second && c.float_seed == oracle::bits(ty, 1.5) && c.bool_seed)
            .or_else(|| {
                cli.iter()
                    .find(|c| c.first && c.second && c.bool_seed && !c.trace.contains(&81))
            })
            .unwrap();
        assert_eq!(
            or_skipped.words()[12],
            0,
            "inactive OR sibling RHS must stay poisoned"
        );
        assert!(cli
            .iter()
            .any(|c| c.first && c.second && c.trace.iter().filter(|&&s| s == 78).count() > 1));
        assert_eq!(marker.previous().bool_seed, marker.bool_seed);
        assert_eq!(marker.previous().peer_seed, marker.peer_seed);
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
#[path = "return_two_bool_field_vector_native_f32.rs"]
mod native_f32;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_two_bool_field_vector_native_f64.rs"]
mod native_f64;

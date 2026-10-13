use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_pair_tail_in_fields_oracle.rs"]
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
fn return_pair_tail_in_fields_reference_values_order_and_last_publication() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for opened in oracle::cases() {
                assert_eq!(opened.open_args().split(',').count(), 32);
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
fn return_pair_tail_in_fields_reference_current_pair_and_stale_dependency() {
    assert_eq!(oracle::cases().len(), 80);
    for c in oracle::cases() {
        if c.exited || !c.first {
            for index in [20, 21, 22, 23] {
                assert_eq!(c.words()[index + 2], 0);
            }
            assert!(c.before_tail.is_none() && c.fields.is_none());
            continue;
        }
        let (before, a, b, wide, flag, peer) = c.before_tail.unwrap();
        let parity = (a < 0)
            ^ (f32::from_bits(b as u32) < 0.0)
            ^ (f64::from_bits(wide as u64) < 0.0)
            ^ (flag != 0)
            ^ (peer != 0);
        let changed = parity && flag != 0;
        let (owner, after_a, after_b, after_wide, after_flag, after_peer) = c.fields.unwrap();
        assert_eq!((after_a, after_b, after_peer), (a, b, peer));
        let expected_flag = (flag != 0) ^ (changed && f64::from_bits(after_wide as u64) < 0.0);
        assert_eq!(after_flag, i64::from(expected_flag));
        assert_eq!(c.trace.contains(&62), changed);
        assert_eq!(after_wide, wide ^ if changed { i64::MIN } else { 0 });
        assert_eq!(owner, if changed { !before } else { before });
        assert_eq!(c.trace.contains(&63), changed);
        assert_eq!(c.trace.contains(&65), changed);
        assert_eq!(c.trace.contains(&64), !parity);
        assert!(
            c.trace.iter().position(|v| *v == 66).unwrap()
                < c.trace.iter().position(|v| *v == 96).unwrap()
        );
    }
    let witness = oracle::cli_cases()
        .into_iter()
        .find(|c| c.first && !c.second && c.bool_seed && !c.peer_seed)
        .unwrap();
    let before = witness.before_tail.unwrap();
    let after = witness.fields.unwrap();
    assert!(witness.trace.contains(&63));
    assert_ne!(before.3, after.3);
    assert_eq!(before.4, 1);
    assert_eq!(after.4, 0);
    assert_eq!(witness.owner, !witness.seed);
    assert!(!witness.result);
    // Feeding the stale pre-negation c to the second update leaves d true.
    // Later current-c/old-d computation would wrongly retain owner == seed.
    let stale_second = (before.4 != 0) ^ (f64::from_bits(before.3 as u64) < 0.0);
    assert!(stale_second);
    assert_eq!(after.0, witness.seed);
    assert_eq!(after.5, 0);
    assert!(after.0 == witness.seed || after.5 != 0);
}
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_pair_tail_in_fields_native.rs"]
mod native;

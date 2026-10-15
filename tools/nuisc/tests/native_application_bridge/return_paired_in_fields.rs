use super::*;
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_paired_in_fields_oracle.rs"]
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
fn return_paired_in_fields_reference_values_order_and_last_publication() {
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
fn return_paired_in_fields_reference_two_exits_use_current_prefix_and_skip_all_tail_work() {
    let cases = oracle::cli_cases();
    for second in [false, true] {
        let exited = cases
            .iter()
            .filter(|c| c.first && c.second == second && c.exited)
            .collect::<Vec<_>>();
        assert!(exited.iter().any(|c| c.result));
        assert!(exited.iter().any(|c| !c.result));
    }
    for c in oracle::cases() {
        assert_eq!(c.trace.contains(&76), c.first && c.second);
        assert_eq!(c.trace.contains(&58), c.first && !c.second);
        if c.exited {
            assert!(c.fields.is_none() && c.before_tail.is_none());
            for index in [5, 7, 8, 10, 18, 20, 21, 22, 23] {
                assert_eq!(c.words()[index + 2], 0);
            }
            assert!(!c.trace.contains(&67));
        }
    }
    let witness = cases
        .iter()
        .find(|c| c.first && !c.second && !c.bool_seed && c.peer_seed)
        .unwrap();
    let prefix = witness.prefix_fields.unwrap();
    assert_eq!(prefix.1, i64::from(i32::MAX));
    assert_eq!(prefix.4, 1);
    assert!(witness.exited && !witness.result);
    let stale_small_condition = (witness.small_seed < 0) ^ (prefix.4 != 0);
    assert!(!stale_small_condition);
    // This false source return must still terminate the source arm.
    assert!(witness.trace.contains(&58));
    assert!(!witness.trace.contains(&67) && !witness.trace.contains(&66));
    let parity = |a: i64, b: i64, c: i64, d: i64, e: i64| {
        (a < 0)
            ^ (f32::from_bits(b as u32) < 0.0)
            ^ (f64::from_bits(c as u64) < 0.0)
            ^ (d != 0)
            ^ (e != 0)
    };
    // Counterfactual arithmetic assumes healthy continuing proof operands.
    // The real exited cases also leave those unused operands poisoned at zero.
    let (mut value, mut a, mut b, mut c, mut d, mut e) = prefix;
    if parity(a, b, c, d, e) {
        value = !value;
    }
    if parity(a, b, c, d, e) && d != 0 {
        c ^= i64::MIN;
        d = i64::from((d != 0) ^ (f64::from_bits(c as u64) < 0.0));
        if d == 0 {
            a = i64::from(!(a as i32));
        }
        if a >= 0 {
            b ^= 1 << 31;
        }
        e = i64::from((e != 0) ^ !(f32::from_bits(b as u32) < 0.0));
        value = !value;
    }
    if parity(a, b, c, d, e) {
        value = !value;
    }
    let missed_exit_result = value == witness.seed && e != 0;
    assert!(missed_exit_result);
    assert_ne!(missed_exit_result, witness.result);
    let continued = cases
        .iter()
        .find(|c| c.first && !c.second && c.bool_seed && !c.peer_seed)
        .unwrap();
    assert!(!continued.exited && continued.result);
    assert!(continued.trace.contains(&59));
}
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[path = "return_paired_in_fields_native.rs"]
mod native;

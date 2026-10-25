#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
#[allow(dead_code)]
#[path = "computed_iteration_print_cases.rs"]
mod cases;
#[path = "word_session_probe.rs"]
mod probe;

fn calls(events: &[String], callee: &str) -> usize {
    events
        .iter()
        .filter(|event| event.contains("cpu.call") && event.contains(&format!("] {callee}(")))
        .count()
}

fn values(case: cases::Case) -> Vec<Value> {
    case.words()
        .into_iter()
        .enumerate()
        .map(|(i, value)| {
            if matches!(i, 1 | 3 | 4 | 13) {
                Value::Bool(value != 0)
            } else {
                Value::Int(value)
            }
        })
        .collect()
}

fn sequence(rows: Vec<cases::Case>) -> probe::Sequence {
    let mut previous = None;
    probe::Sequence {
        rows: rows
            .into_iter()
            .map(|case| {
                let (next, trace) = case.execute_native(previous).unwrap();
                previous = Some(next);
                probe::Row {
                    input: case.words(),
                    state: vec![next],
                    trace,
                }
            })
            .collect(),
    }
}

#[test]
fn computed_iteration_print_native_keeps_one_predicate_and_selected_fault_prefixes() {
    let failures = cases::native_failures()
        .into_iter()
        .map(|bad| probe::SequenceFailure {
            accepted: sequence(cases::prefix()),
            input: bad.words(),
            before_failure: bad.execute_native(Some(cases::LAST_STATE)).unwrap_err(),
        })
        .collect::<Vec<_>>();
    probe::check_sequences_with_i64_prints(
        cases::SOURCE,
        &[sequence(cases::healthy())],
        &failures,
        cases::OBSERVERS,
        cases::LOOP_LIMIT,
        cases::ENTRY_LIMIT,
    );
}

#[test]
fn computed_iteration_print_reference_preserves_predicate_counts_and_last_state() {
    let project = Project::with_source(cases::SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(compiled.yir.functions.len() <= 64);
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        let rows = cases::healthy();
        let (mut session, opened) =
            ApplicationSession::open_registered(&module, &registry, "counter", values(rows[0]))
                .unwrap();
        let (mut previous, trace) = rows[0].execute(None).unwrap();
        let printed = |events: &[String]| {
            events
                .iter()
                .filter(|s| {
                    s.contains("cpu.print")
                        || (s.contains("cpu.guard_print") && s.contains("if true then print"))
                })
                .map(|s| s.rsplit_once(' ').unwrap().1.parse::<i64>().unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(printed(&opened.events), trace);
        for (callee, _) in cases::OBSERVERS {
            assert_eq!(calls(&opened.events, callee), 0);
        }
        for row in &rows[1..] {
            let (next, trace) = row.execute(Some(previous)).unwrap();
            let event = session.event(values(*row)).unwrap();
            assert_eq!(printed(&event.events), trace);
            let (_, observed) = row.execute_native(Some(previous)).unwrap();
            for (index, (callee, _)) in cases::OBSERVERS.iter().enumerate() {
                assert_eq!(
                    calls(&event.events, callee),
                    observed
                        .iter()
                        .filter(|v| **v == cases::ENTRY_MARKERS[index])
                        .count(),
                    "{callee} {row:?}"
                );
            }
            assert_eq!(state_words(session.state()), vec![next as u64]);
            previous = next;
        }
        session.close(vec![]).unwrap();
        session.completion_status().unwrap();
        for bad in cases::native_failures() {
            let rows = cases::prefix();
            let (mut session, _) =
                ApplicationSession::open_registered(&module, &registry, "counter", values(rows[0]))
                    .unwrap();
            for row in &rows[1..] {
                session.event(values(*row)).unwrap();
            }
            assert!(session.event(values(bad)).is_err(), "{bad:?}");
            assert_eq!(state_words(session.state()), vec![cases::LAST_STATE as u64]);
            assert_eq!(
                session.phase(),
                yir_runtime_host::ApplicationSessionPhase::Faulted
            );
            assert!(session.event(values(rows[1])).is_err());
            session.close(vec![]).unwrap();
            assert_eq!(state_words(session.state()), vec![cases::LAST_STATE as u64]);
            assert!(session.completion_status().is_err());
        }
    }
}

#[test]
fn computed_iteration_print_requires_distinct_exact_native_policy_and_bound_identity() {
    use yir_lower_llvm::native_session::*;
    let project = Project::with_source(cases::SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let sites = compiled
        .yir
        .nodes
        .iter()
        .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
        .map(|n| n.name.clone())
        .collect::<Vec<_>>();
    assert!(emit_registered_with_work_limits(&compiled.yir, "counter", 4, 64).is_err());
    let literal = LiteralPrintPolicy::new(sites.clone()).unwrap();
    assert!(
        emit_registered_with_literal_prints(&compiled.yir, "counter", &literal, 4, 64)
            .unwrap_err()
            .contains("direct i64 constant")
    );
    let policy = I64PrintPolicy::new(sites.clone()).unwrap();
    let bridge = emit_registered_with_i64_prints(&compiled.yir, "counter", &policy, 4, 64).unwrap();
    assert!(bridge.literal_print_sites.is_empty());
    assert_eq!(bridge.i64_print_sites.len(), sites.len());
    assert_eq!(bridge.max_literal_prints_per_invocation(), 0);
    assert_eq!(bridge.max_prints_per_invocation(), sites.len() as u128 * 64);
    let build = LiteralPrintBuildPolicy::new_i64(sites.clone(), 4, 64).unwrap();
    assert!(build.token().starts_with("i64-v1.4.64."));
    assert_eq!(
        LiteralPrintBuildPolicy::parse(&build.token())
            .unwrap()
            .emit(&compiled.yir, "counter")
            .unwrap()
            .llvm_ir,
        bridge.llvm_ir
    );
    assert!(build
        .bundle_claims()
        .iter()
        .any(|(k, v)| *k == "native_session_policy_contract" && v == I64_PRINT_BUILD_CONTRACT));
    assert!(!build
        .bundle_claims()
        .iter()
        .any(|(k, _)| *k == "native_session_literal_print_bound"));
}

#[test]
fn computed_iteration_print_policy_rejects_wrong_kinds_unselected_sites_and_unordered_work() {
    use yir_lower_llvm::native_session::*;
    let project = Project::with_source(cases::SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let sites = compiled
        .yir
        .nodes
        .iter()
        .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
        .map(|n| n.name.clone())
        .collect::<Vec<_>>();
    assert!(I64PrintPolicy::new(["", "x"]).is_err());
    assert!(I64PrintPolicy::new(["x", "x"]).is_err());
    for grants in [
        sites[1..].to_vec(),
        sites
            .iter()
            .cloned()
            .chain(["unknown_site".into()])
            .collect(),
    ] {
        assert!(emit_registered_with_i64_prints(
            &compiled.yir,
            "counter",
            &I64PrintPolicy::new(grants).unwrap(),
            4,
            64
        )
        .is_err());
    }
    let policy = I64PrintPolicy::new(sites.clone()).unwrap();
    let mut unordered = compiled.yir.clone();
    unordered.edges.clear();
    assert!(emit_registered_with_i64_prints(&unordered, "counter", &policy, 4, 64).is_err());
    let target =
        compiled
            .yir
            .nodes
            .iter()
            .find(|n| {
                matches!(n.op.instruction.as_str(), "print" | "guard_print")
                    && compiled.yir.nodes.iter().any(|p| {
                        p.name == *n.op.args.last().unwrap() && p.op.instruction == "call_i64"
                    })
            })
            .unwrap();
    let mut wrong = compiled.yir.clone();
    let producer = wrong
        .nodes
        .iter_mut()
        .find(|n| n.name == *target.op.args.last().unwrap())
        .unwrap();
    producer.op.instruction = "const_bool".into();
    producer.op.args = vec!["true".into()];
    let error = emit_registered_with_i64_prints(&wrong, "counter", &policy, 4, 64).unwrap_err();
    assert!(error.contains(&target.name), "{error}");
    assert!(error.contains("declared scalar kind"), "{error}");
}

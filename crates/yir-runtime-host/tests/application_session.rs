use yir_core::{
    Value, YirFunction, YirFunctionParameter, YirFunctionResult, YirFunctionRole, YirModule,
    YirValueOwnership,
};
use yir_runtime_host::{ApplicationSession, ApplicationSessionEntries, ApplicationSessionPhase};

fn fixture() -> YirModule {
    let mut module = yir_syntax::parse_module(
        r#"
yir 0.1
resource cpu0 cpu.arm64
cpu.const_i64 global cpu0 100
cpu.print global_print cpu0 global
cpu.param_i64 seed cpu0 0
cpu.struct opened cpu0 Counter count=seed
cpu.param_i64 count cpu0 0
cpu.param_i64 delta cpu0 1
cpu.add next cpu0 count delta
cpu.struct updated cpu0 Counter count=next
cpu.param_i64 final_count cpu0 0
cpu.param_i64 divisor cpu0 1
cpu.print close_print cpu0 final_count
cpu.div quotient cpu0 final_count divisor
cpu.struct closed cpu0 Counter count=quotient
cpu.print main_print cpu0 global
edge dep global global_print
edge dep seed opened
edge dep count next
edge dep delta next
edge dep next updated
edge dep final_count close_print
edge dep close_print quotient
edge dep final_count quotient
edge dep divisor quotient
edge dep quotient closed
edge dep global main_print
"#,
    )
    .unwrap();
    let helper = |name: &str, params: &[(&str, &str)], result: &str, body: &[&str]| YirFunction {
        name: name.to_owned(),
        domain: "cpu".to_owned(),
        role: YirFunctionRole::Helper,
        parameters: params
            .iter()
            .map(|(name, node)| YirFunctionParameter {
                name: (*name).to_owned(),
                ty: "i64".to_owned(),
                ownership: YirValueOwnership::Value,
                node: (*node).to_owned(),
            })
            .collect(),
        result: Some(YirFunctionResult {
            ty: "Counter".to_owned(),
            ownership: YirValueOwnership::Owned,
            node: result.to_owned(),
        }),
        body_nodes: body.iter().map(|name| (*name).to_owned()).collect(),
    };
    module.functions = vec![
        helper("open", &[("seed", "seed")], "opened", &["seed", "opened"]),
        helper(
            "update",
            &[("state.count", "count"), ("delta", "delta")],
            "updated",
            &["count", "delta", "next", "updated"],
        ),
        helper(
            "close",
            &[("state.count", "final_count"), ("divisor", "divisor")],
            "closed",
            &[
                "final_count",
                "divisor",
                "close_print",
                "quotient",
                "closed",
            ],
        ),
        YirFunction {
            name: "main".to_owned(),
            domain: "cpu".to_owned(),
            role: YirFunctionRole::Entry,
            parameters: vec![],
            result: None,
            body_nodes: vec!["main_print".to_owned()],
        },
    ];
    module
}

fn entries() -> ApplicationSessionEntries<'static> {
    ApplicationSessionEntries {
        open: "open",
        event: "update",
        close: "close",
        state_parameter: "state",
    }
}

fn count(session: &ApplicationSession<'_>) -> i64 {
    let Value::Struct(value) = session.state() else {
        panic!("not a struct")
    };
    let Value::Int(count) = value.fields[0].1 else {
        panic!("not a scalar")
    };
    count
}

#[test]
fn state_and_global_initialization_survive_many_separate_deliveries() {
    let module = fixture();
    let registry = yir_verify::default_registry();
    let (mut session, opened) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    assert!(opened
        .lane_steps
        .values()
        .flatten()
        .any(|step| step.ends_with("-> global_print")));
    assert!(!opened
        .lane_steps
        .values()
        .flatten()
        .any(|step| step.ends_with("-> main_print")));
    for index in 1..=100 {
        let trace = session.event(vec![Value::Int(2)]).unwrap();
        assert_eq!(count(&session), 10 + 2 * index);
        assert_eq!(trace.lane_steps.values().map(Vec::len).sum::<usize>(), 2);
        assert!(trace.values.is_empty());
    }
    let closed = session.close(vec![Value::Int(1)]).unwrap().unwrap();
    assert_eq!(
        closed
            .lane_steps
            .values()
            .flatten()
            .filter(|step| step.ends_with("-> close_print"))
            .count(),
        1
    );
    assert!(session.close(vec![Value::Int(0)]).unwrap().is_none());
    session.completion_status().unwrap();
    assert!(session.event(vec![Value::Int(1)]).is_err());
}

#[test]
fn invalid_ingress_has_no_effect_and_can_be_corrected() {
    let module = fixture();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    for arguments in [
        vec![],
        vec![Value::Bool(true)],
        vec![Value::Pointer(Some(1))],
        vec![Value::Int(1), Value::Int(2)],
    ] {
        assert!(session.event(arguments).is_err());
        assert_eq!(session.phase(), ApplicationSessionPhase::Open);
        assert_eq!(count(&session), 10);
        assert_eq!(
            session.failure_kind(),
            yir_core::ApplicationFailureKind::None
        );
    }
    assert!(session.close(vec![]).is_err());
    assert_eq!(session.phase(), ApplicationSessionPhase::Open);
    session.event(vec![Value::Int(2)]).unwrap();
    assert_eq!(count(&session), 12);
    session.close(vec![Value::Int(1)]).unwrap();
}

#[test]
fn failed_close_is_terminal_and_is_not_retried() {
    let module = fixture();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    let error = session.close(vec![Value::Int(0)]).unwrap_err();
    assert_eq!(
        session.failure_kind(),
        yir_core::ApplicationFailureKind::Callback
    );
    assert!(session.completion_status().is_err());
    assert!(error.contains("zero"), "{error}");
    assert_eq!(session.phase(), ApplicationSessionPhase::Closed);
    assert_eq!(session.close(vec![Value::Int(1)]).unwrap_err(), error);
    assert_eq!(count(&session), 10);
}

#[test]
fn failed_event_stops_delivery_but_allows_one_explicit_close() {
    let mut module = fixture();
    module
        .nodes
        .iter_mut()
        .find(|node| node.name == "next")
        .unwrap()
        .op
        .instruction = "div".to_owned();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    assert!(session
        .event(vec![Value::Int(0)])
        .unwrap_err()
        .contains("zero"));
    assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
    assert_eq!(
        session.failure_kind(),
        yir_core::ApplicationFailureKind::Callback
    );
    assert!(session.event(vec![Value::Int(2)]).is_err());
    assert_eq!(count(&session), 10);
    let trace = session.close(vec![Value::Int(1)]).unwrap().unwrap();
    assert!(!trace
        .lane_steps
        .values()
        .flatten()
        .any(|step| step.ends_with("-> next")));
    assert!(session.close(vec![Value::Int(1)]).unwrap().is_none());
    assert!(session.completion_status().unwrap_err().contains("zero"));
    assert_eq!(
        session.failure_kind(),
        yir_core::ApplicationFailureKind::Callback
    );
}

#[test]
fn rejects_stale_or_capability_signatures_before_running_open() {
    let registry = yir_verify::default_registry();
    for case in 0..7 {
        let mut module = fixture();
        match case {
            0 => module.functions[2].parameters[0].name = "state.other".to_owned(),
            1 => module.functions[1].parameters[0].ownership = YirValueOwnership::Borrowed,
            2 => module.functions[1].parameters[0].ty = "Ptr<i64>".to_owned(),
            3 => module.functions[2].result.as_mut().unwrap().ty = "Other".to_owned(),
            4 => module.functions[1].role = YirFunctionRole::Entry,
            5 => module.functions[0].result = None,
            _ => module.functions[1].parameters[1].node = "count".to_owned(),
        }
        assert!(
            ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(1)]).is_err(),
            "accepted case {case}"
        );
    }
    assert!(
        ApplicationSession::open(&fixture(), &registry, entries(), vec![Value::Pointer(None)])
            .is_err()
    );
}

#[test]
fn rejects_returned_state_layout_drift() {
    let mut module = fixture();
    module
        .nodes
        .iter_mut()
        .find(|node| node.name == "updated")
        .unwrap()
        .op
        .args[1] = "renamed=next".to_owned();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    assert!(session
        .event(vec![Value::Int(1)])
        .unwrap_err()
        .contains("state fields"));
    assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
    assert_eq!(count(&session), 10);
    session.close(vec![Value::Int(1)]).unwrap();
}

#[test]
fn recursive_scalar_state_uses_registered_field_paths() {
    let mut module = fixture();
    for (index, inner) in ["opened", "updated", "closed"].into_iter().enumerate() {
        let outer = format!("{inner}_outer");
        module
            .nodes
            .iter_mut()
            .find(|node| node.name == inner)
            .unwrap()
            .op
            .args[0] = "Inner".to_owned();
        module.nodes.push(yir_core::Node {
            name: outer.clone(),
            resource: "cpu0".to_owned(),
            op: yir_core::Operation::parse(
                "cpu.struct",
                vec!["Counter".to_owned(), format!("inner={inner}")],
            )
            .unwrap(),
        });
        module.edges.push(yir_core::Edge {
            kind: yir_core::EdgeKind::Dep,
            from: inner.to_owned(),
            to: outer.clone(),
        });
        module.functions[index].body_nodes.push(outer.clone());
        module.functions[index].result.as_mut().unwrap().node = outer;
        if index > 0 {
            module.functions[index].parameters[0].name = "state.inner.count".to_owned();
        }
    }
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    session.event(vec![Value::Int(3)]).unwrap();
    let Value::Struct(outer) = session.state() else {
        panic!("missing outer state")
    };
    let Value::Struct(inner) = &outer.fields[0].1 else {
        panic!("missing inner state")
    };
    assert_eq!(inner.fields, vec![("count".to_owned(), Value::Int(13))]);
    session.close(vec![Value::Int(1)]).unwrap();
}

#[test]
fn generic_function_session_checks_scalar_boundaries_and_excludes_entry_calls() {
    let mut module = fixture();
    module.functions[0].result = Some(YirFunctionResult {
        ty: "i64".to_owned(),
        ownership: YirValueOwnership::Value,
        node: "seed".to_owned(),
    });
    let registry = yir_verify::default_registry();
    let mut execution = yir_exec::FunctionSession::new(&module, &registry).unwrap();
    assert!(execution.invoke("missing", vec![]).is_err());
    assert!(execution
        .invoke("main", vec![])
        .unwrap_err()
        .contains("helper"));
    assert!(execution.invoke("open", vec![Value::Bool(true)]).is_err());
    assert_eq!(
        execution
            .invoke("open", vec![Value::Int(12)])
            .unwrap()
            .value,
        Value::Int(12)
    );
    assert_eq!(
        execution
            .invoke("open", vec![Value::Int(24)])
            .unwrap()
            .value,
        Value::Int(24)
    );
}

#[test]
fn event_fuel_exhaustion_faults_without_changing_accepted_state_and_allows_cleanup() {
    let module = fixture();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    // One function entry and two nodes, not a budget for the entire session.
    session.event_budgeted(vec![Value::Int(2)], 3).unwrap();
    assert_eq!(count(&session), 12);
    assert!(session.event_budgeted(vec![Value::Bool(true)], 0).is_err());
    assert_eq!(session.phase(), ApplicationSessionPhase::Open);
    let error = session.event_budgeted(vec![Value::Int(7)], 2).unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
    assert_eq!(count(&session), 12);
    assert!(session.event_budgeted(vec![Value::Int(7)], 100).is_err());
    session.close(vec![Value::Int(1)]).unwrap();
    assert_eq!(count(&session), 12);
    assert!(session.completion_status().is_err());
}

#[test]
fn scoped_fuel_is_shared_by_nested_calls_and_does_not_leak_to_later_invocations() {
    let module = yir_syntax::parse_module(
        r#"
yir 0.1
resource cpu0 cpu.arm64
function outer cpu helper
function-param outer value i64 value input
function-result outer i64 value called
function-node outer input
function-node outer called
function inner cpu helper
function-param inner value i64 value forwarded
function-result inner i64 value forwarded
function-node inner forwarded
cpu.param_i64 input cpu0 0
cpu.param_i64 forwarded cpu0 0
cpu.call_i64 called cpu0 inner input
edge dep input called
"#,
    )
    .unwrap();
    let registry = yir_verify::default_registry();
    let mut execution = yir_exec::FunctionSession::new(&module, &registry).unwrap();
    for steps in 0..3 {
        let error = execution
            .invoke_budgeted("outer", vec![Value::Int(7)], steps)
            .unwrap_err();
        assert!(error.contains("step budget exhausted"), "{steps}: {error}");
    }
    for value in [11, 19] {
        let result = execution
            .invoke_budgeted("outer", vec![Value::Int(value)], 3)
            .unwrap();
        assert_eq!(result.value, Value::Int(value));
        assert_eq!(
            result
                .trace
                .lane_steps
                .values()
                .map(Vec::len)
                .sum::<usize>(),
            1
        );
    }
    assert_eq!(
        execution
            .invoke("outer", vec![Value::Int(23)])
            .unwrap()
            .value,
        Value::Int(23)
    );
    let mut recursive = module.clone();
    recursive
        .nodes
        .iter_mut()
        .find(|node| node.name == "called")
        .unwrap()
        .op
        .args[0] = "outer".to_owned();
    let mut execution = yir_exec::FunctionSession::new(&recursive, &registry).unwrap();
    let error = execution
        .invoke_budgeted("outer", vec![Value::Int(1)], 20)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
}

#[test]
fn exhausted_invocations_drain_partial_traces_and_do_not_repeat_global_initialization() {
    let module = fixture();
    let registry = yir_verify::default_registry();
    let mut execution = yir_exec::FunctionSession::new(&module, &registry).unwrap();
    execution.invoke("open", vec![Value::Int(10)]).unwrap();
    // Entry + print executes, then fuel runs out before div/struct. These
    // effects are not rolled back, but their trace must not appear on retry.
    assert!(execution
        .invoke_budgeted("close", vec![Value::Int(10), Value::Int(1)], 2)
        .is_err());
    let result = execution
        .invoke_budgeted("open", vec![Value::Int(20)], 2)
        .unwrap();
    assert_eq!(
        result
            .trace
            .lane_steps
            .values()
            .map(Vec::len)
            .sum::<usize>(),
        1
    );
    assert!(result.trace.events.is_empty());
}

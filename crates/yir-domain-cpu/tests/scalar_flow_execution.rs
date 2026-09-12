use yir_core::{
    ExecutionState, Node, Operation, RegisteredExecutionStep, RegisteredMod, Resource,
    ResourceKind, Value,
};
use yir_domain_cpu::CpuMod;

fn fixture() -> (Node, Resource, ExecutionState) {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            [
                "initial",
                "limit",
                "step",
                "lt",
                "add",
                "and",
                "current_gt",
                "zero",
                "current_lt",
                "threshold",
                "break",
            ]
            .map(str::to_owned)
            .to_vec(),
        )
        .unwrap(),
    };
    let resource = Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    };
    let mut state = ExecutionState::default();
    for (name, value) in [
        ("initial", 0),
        ("limit", 6),
        ("step", 1),
        ("zero", 0),
        ("threshold", 4),
    ] {
        state.values.insert(name.to_owned(), Value::Int(value));
    }
    (node, resource, state)
}

#[test]
fn scalar_flow_direct_execution_returns_state_not_a_trace_only_unit() {
    let (node, resource, mut state) = fixture();
    let Value::Struct(result) = CpuMod.execute(&node, &resource, &mut state).unwrap() else {
        panic!("scalar flow did not return loop state")
    };
    assert_eq!(result.fields, [("current".to_owned(), Value::Int(1))]);
    assert!(state
        .events
        .iter()
        .any(|event| event.contains("iterations=1 final=1")));
}

#[test]
fn scalar_flow_wraps_induction_like_native_i64_arithmetic() {
    let (mut node, resource, mut state) = fixture();
    state
        .values
        .insert("initial".to_owned(), Value::Int(i64::MAX - 1));
    state
        .values
        .insert("limit".to_owned(), Value::Int(i64::MAX));
    node.op.args[3] = "le".to_owned();
    node.op.args[6] = "current_lt".to_owned();
    let Value::Struct(result) = CpuMod.execute(&node, &resource, &mut state).unwrap() else {
        panic!("missing loop state")
    };
    assert_eq!(result.fields[0].1, Value::Int(i64::MIN));
}

#[test]
fn scalar_flow_rejects_incomplete_and_unavailable_control_metadata() {
    let (mut node, resource, mut state) = fixture();
    node.op.args[6] = "carry9_gt".to_owned();
    assert!(CpuMod
        .execute(&node, &resource, &mut state)
        .unwrap_err()
        .contains("unavailable scalar loop source"));
    node.op.args.pop();
    assert!(CpuMod
        .execute(&node, &resource, &mut state)
        .unwrap_err()
        .contains("missing flow control action"));
    assert!(
        state.events.is_empty(),
        "failed execution must not publish completion"
    );
}

#[test]
fn scalar_flow_zero_step_is_bounded_without_fabricating_completion() {
    let (node, resource, mut state) = fixture();
    state.values.insert("step".to_owned(), Value::Int(0));
    let error = CpuMod.execute(&node, &resource, &mut state).unwrap_err();
    assert!(error.contains("exceeded 100000 reference steps"), "{error}");
    assert!(state.events.is_empty());
}

#[test]
fn scalar_flow_async_step_requires_a_typed_function_result() {
    let (mut node, resource, mut state) = fixture();
    node.op.instruction = "loop_while_scalar_async_flow_cond_chain".to_owned();
    node.op.args[2] = "step_function".to_owned();
    node.op.args.remove(4);
    let mut execution = CpuMod
        .begin_execution(&node, &resource, &state)
        .unwrap()
        .unwrap();
    let RegisteredExecutionStep::Call {
        function,
        arguments,
    } = execution.resume(&mut state, None).unwrap()
    else {
        panic!("missing step request")
    };
    assert_eq!(function, "step_function");
    assert_eq!(arguments, [Value::Int(0)]);
    let error = execution
        .resume(&mut state, Some(Value::Bool(true)))
        .err()
        .unwrap();
    assert!(error.contains("requires an i64 step result"), "{error}");
    let error = CpuMod.execute(&node, &resource, &mut state).unwrap_err();
    assert!(
        error.contains("requires a registered function execution context"),
        "{error}"
    );
    assert!(state.events.is_empty());
}

#[test]
fn plain_scalar_chain_is_cooperative_and_returns_source_ordered_carries() {
    for instruction in ["loop_while_scalar_chain", "loop_while_i64_chain"] {
        let (mut node, resource, mut state) = fixture();
        node.op.instruction = instruction.to_owned();
        node.op.args.truncate(5);
        node.op
            .args
            .extend(["product", "mul_current", "sum", "add_carry0"].map(str::to_owned));
        state.values.insert("limit".to_owned(), Value::Int(3));
        state.values.insert("product".to_owned(), Value::Int(1));
        state.values.insert("sum".to_owned(), Value::Int(2));
        let mut execution = CpuMod
            .begin_execution(&node, &resource, &state)
            .unwrap()
            .unwrap();
        for _ in 0..3 {
            assert!(matches!(
                execution.resume(&mut state, None).unwrap(),
                RegisteredExecutionStep::Continue
            ));
            assert!(
                state.events.is_empty(),
                "unfinished loop cannot publish completion"
            );
        }
        let RegisteredExecutionStep::Complete(Value::Struct(result)) =
            execution.resume(&mut state, None).unwrap()
        else {
            panic!("plain loop did not return its state");
        };
        assert_eq!(
            result.fields,
            [
                ("current".to_owned(), Value::Int(3)),
                ("carry0".to_owned(), Value::Int(6)),
                ("carry1".to_owned(), Value::Int(11)),
            ]
        );
        assert_eq!(state.events.len(), 1);

        state.values.insert("initial".to_owned(), Value::Int(3));
        let Value::Struct(result) = CpuMod.execute(&node, &resource, &mut state).unwrap() else {
            panic!("zero-trip loop did not return its seeds");
        };
        assert_eq!(result.fields[1].1, Value::Int(1));
        assert_eq!(result.fields[2].1, Value::Int(2));
    }
}

#[test]
fn plain_scalar_chain_carry_overflow_wraps_and_invalid_kinds_fail_closed() {
    let (mut node, resource, mut state) = fixture();
    node.op.instruction = "loop_while_scalar_chain".to_owned();
    node.op.args.truncate(5);
    node.op
        .args
        .extend(["seed", "mul_current"].map(str::to_owned));
    state.values.insert("limit".to_owned(), Value::Int(2));
    state.values.insert("seed".to_owned(), Value::Int(i64::MAX));
    let Value::Struct(result) = CpuMod.execute(&node, &resource, &mut state).unwrap() else {
        panic!("missing scalar loop state");
    };
    assert_eq!(result.fields[1].1, Value::Int(-2));
    state.events.clear();
    node.op.args[6] = "mul_carry9".to_owned();
    assert!(CpuMod
        .execute(&node, &resource, &mut state)
        .unwrap_err()
        .contains("unavailable scalar loop source"));
    state.values.insert("seed".to_owned(), Value::F64(1.0));
    assert!(CpuMod.execute(&node, &resource, &mut state).is_err());
    assert!(state.events.is_empty());
}

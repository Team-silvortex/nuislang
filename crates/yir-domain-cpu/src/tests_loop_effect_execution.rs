use super::*;
use yir_core::{Operation, ResourceKind};

fn resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn scoped_loop(captured: &str) -> Node {
    Node {
        name: "update_loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_i64_effect",
            [
                "initial",
                "limit",
                "step",
                "lt",
                "add",
                "cpu",
                "scoped_call",
                "3",
                "render_showcase_frame",
                "$current",
                captured,
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    }
}

#[test]
fn scoped_loop_execution_requires_a_real_function_context() {
    let mut state = ExecutionState::default();
    state.bind_value("initial", Value::Int(0));
    state.bind_value("limit", Value::Int(3));
    state.bind_value("step", Value::Int(1));
    state.bind_value("captured", Value::Bool(true));

    let error = CpuMod
        .execute(&scoped_loop("captured"), &resource(), &mut state)
        .expect_err("logging a call is not execution");
    assert!(
        error.contains("requires a registered function execution context"),
        "{error}"
    );
    assert!(state.events.is_empty());
}

#[test]
fn plain_descending_loop_execution_returns_its_exact_exit_value() {
    let mut state = ExecutionState::default();
    state.bind_value("initial", Value::Int(7));
    state.bind_value("limit", Value::Int(0));
    state.bind_value("step", Value::Int(2));
    let node = Node {
        name: "descending".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_i64",
            ["initial", "limit", "step", "gt", "sub"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        )
        .unwrap(),
    };

    assert_eq!(
        CpuMod.execute(&node, &resource(), &mut state).unwrap(),
        Value::Int(-1)
    );
}

#[test]
fn scoped_loop_execution_still_rejects_a_missing_captured_value() {
    let mut state = ExecutionState::default();
    state.bind_value("initial", Value::Int(0));
    state.bind_value("limit", Value::Int(3));
    state.bind_value("step", Value::Int(1));

    let error = CpuMod
        .execute(&scoped_loop("missing_capture"), &resource(), &mut state)
        .expect_err("missing runtime captures must remain fail-closed");

    assert!(
        error.contains("missing value for `missing_capture`"),
        "{error}"
    );
    assert!(!error.contains("missing value for `render_showcase_frame`"));
}

fn loop_state(initial: i64, limit: i64, step: i64) -> ExecutionState {
    let mut state = ExecutionState::default();
    state.bind_value("initial", Value::Int(initial));
    state.bind_value("limit", Value::Int(limit));
    state.bind_value("step", Value::Int(step));
    state.bind_value("captured", Value::Bool(true));
    state
}

#[test]
fn scoped_loop_dispatches_each_induction_value_before_advancing() {
    use yir_core::RegisteredExecutionStep as Step;
    let mut state = loop_state(0, 3, 1);
    let mut execution = CpuMod
        .begin_execution(&scoped_loop("captured"), &resource(), &state)
        .unwrap()
        .unwrap();
    for current in 0..3 {
        let result = (current > 0).then_some(Value::Int(99));
        let Step::Call {
            function,
            arguments,
        } = execution.resume(&mut state, result).unwrap()
        else {
            panic!("expected a real helper call")
        };
        assert_eq!(function, "render_showcase_frame");
        assert_eq!(arguments, vec![Value::Int(current), Value::Bool(true)]);
        assert!(
            state.events.is_empty(),
            "cannot report completion before the last result"
        );
    }
    assert!(matches!(
        execution.resume(&mut state, Some(Value::Int(99))).unwrap(),
        Step::Complete(Value::Int(3))
    ));
    assert!(state
        .events
        .last()
        .unwrap()
        .contains("iterations=3 final=3"));
}

#[test]
fn zero_trip_scoped_loop_does_not_require_a_callee_execution() {
    let mut state = loop_state(3, 3, 1);
    assert_eq!(
        CpuMod
            .execute(&scoped_loop("captured"), &resource(), &mut state)
            .unwrap(),
        Value::Int(3)
    );
    assert!(state.events.last().unwrap().contains("iterations=0"));
}

#[test]
fn scoped_loop_rejects_invalid_metadata_and_missing_or_unsolicited_results() {
    let mut state = loop_state(0, 3, 1);
    for (index, value) in [
        (7, usize::MAX.to_string()),
        (7, "0".to_owned()),
        (3, "invalid".to_owned()),
        (4, "invalid".to_owned()),
    ] {
        let mut node = scoped_loop("captured");
        node.op.args[index] = value;
        assert!(CpuMod.describe(&node, &resource()).is_err());
        assert!(CpuMod.begin_execution(&node, &resource(), &state).is_err());
    }
    let mut execution = CpuMod
        .begin_execution(&scoped_loop("captured"), &resource(), &state)
        .unwrap()
        .unwrap();
    assert!(execution
        .resume(&mut state, Some(Value::Int(1)))
        .err()
        .unwrap()
        .contains("unsolicited"));
    execution.resume(&mut state, None).unwrap();
    assert!(execution
        .resume(&mut state, None)
        .err()
        .unwrap()
        .contains("requires a scalar"));
    assert!(state.events.is_empty());
}

#[test]
fn scoped_copy_snapshot_is_refreshed_each_iteration_and_moves_are_not_repeated() {
    use yir_core::RegisteredExecutionStep as Step;
    let mut state = loop_state(0, 2, 1);
    let pointer = state.alloc_heap_buffer(1, 4);
    state.bind_value("buffer", Value::Pointer(Some(pointer)));
    let mut execution = CpuMod
        .begin_execution(&scoped_loop("copy_owned:buffer"), &resource(), &state)
        .unwrap()
        .unwrap();
    let Step::Call { arguments, .. } = execution.resume(&mut state, None).unwrap() else {
        panic!("missing call")
    };
    assert_eq!(arguments[1], Value::OwnedBytes(vec![4]));
    state.write_heap_buffer_at(Some(pointer), 0, 7).unwrap();
    let Step::Call { arguments, .. } = execution.resume(&mut state, Some(Value::Int(0))).unwrap()
    else {
        panic!("missing call")
    };
    assert_eq!(arguments[1], Value::OwnedBytes(vec![7]));

    state.bind_value("owned", Value::OwnedBytes(vec![42]));
    let mut execution = CpuMod
        .begin_execution(&scoped_loop("move_owned:owned"), &resource(), &state)
        .unwrap()
        .unwrap();
    let Step::Call { arguments, .. } = execution.resume(&mut state, None).unwrap() else {
        panic!("missing move call")
    };
    assert_eq!(arguments[1], Value::OwnedBytes(vec![42]));
    assert!(!state.values.contains_key("owned"));
    assert!(execution
        .resume(&mut state, Some(Value::Int(0)))
        .err()
        .unwrap()
        .contains("cannot repeat"));
}

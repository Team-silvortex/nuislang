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
fn scoped_loop_execution_treats_callee_and_current_as_action_metadata() {
    let mut state = ExecutionState::default();
    state.bind_value("initial", Value::Int(0));
    state.bind_value("limit", Value::Int(3));
    state.bind_value("step", Value::Int(1));
    state.bind_value("captured", Value::Bool(true));

    let result = CpuMod
        .execute(&scoped_loop("captured"), &resource(), &mut state)
        .expect("scoped loop action should resolve only its runtime operands");

    assert_eq!(result, Value::Int(3));
    let event = state.events.last().expect("loop execution event");
    assert!(event.contains("cpu.scoped_call render_showcase_frame"));
    assert!(event.contains("Int(0)"));
    assert!(event.contains("Bool(true)"));
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

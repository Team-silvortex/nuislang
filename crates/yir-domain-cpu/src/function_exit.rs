use yir_core::{ExecutionState, Node, Value};

pub(super) fn selected_return(
    node: &Node,
    state: &ExecutionState,
) -> Result<Option<Value>, String> {
    let (minimum, maximum, then_result, else_result) = match node.op.instruction.as_str() {
        "guard_return" => (2, 3, 1, None),
        "guard_drop_owned_bytes_return" => (3, 3, 2, None),
        "branch_drop_owned_bytes_return" => (5, 5, 2, Some(4)),
        _ => return Ok(None),
    };
    if !(minimum..=maximum).contains(&node.op.args.len()) {
        return Err(format!(
            "{} `{}` has an invalid return arity",
            node.op.full_name(),
            node.name
        ));
    }
    let taken = match state.expect_value(&node.op.args[0])? {
        Value::Bool(value) => *value,
        Value::Int(value) => *value != 0,
        _ => {
            return Err(format!(
                "guard `{}` requires a bool or i64 condition",
                node.name
            ))
        }
    };
    let result = if taken {
        Some(then_result)
    } else {
        else_result
    };
    result
        .map(|index| state.expect_value(&node.op.args[index]).cloned())
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::Operation;

    fn node(instruction: &str, args: &[&str]) -> Node {
        Node {
            name: "exit".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(
                &format!("cpu.{instruction}"),
                args.iter().map(|s| (*s).to_owned()).collect(),
            )
            .unwrap(),
        }
    }

    #[test]
    fn guards_read_only_the_selected_result_and_accept_boolean_or_integer_conditions() {
        for node in [
            node("guard_return", &["ready", "returned"]),
            node("guard_return", &["ready", "returned", "Layout{value:i64}"]),
            node(
                "guard_drop_owned_bytes_return",
                &["ready", "bytes", "returned"],
            ),
        ] {
            for condition in [Value::Bool(false), Value::Int(0)] {
                let mut state = ExecutionState::default();
                state.values.insert("ready".to_owned(), condition);
                assert_eq!(selected_return(&node, &state).unwrap(), None);
            }
            for condition in [Value::Bool(true), Value::Int(-3)] {
                let mut state = ExecutionState::default();
                state.values.insert("ready".to_owned(), condition);
                assert!(selected_return(&node, &state).is_err());
                state.values.insert("returned".to_owned(), Value::Int(42));
                assert_eq!(
                    selected_return(&node, &state).unwrap(),
                    Some(Value::Int(42))
                );
            }
        }
    }

    #[test]
    fn terminal_cleanup_returns_the_selected_arm() {
        let node = node(
            "branch_drop_owned_bytes_return",
            &["ready", "a", "left", "b", "right"],
        );
        for (condition, selected, value) in [(true, "left", 17), (false, "right", 19)] {
            let mut state = ExecutionState::default();
            state
                .values
                .insert("ready".to_owned(), Value::Bool(condition));
            assert!(selected_return(&node, &state).is_err());
            state.values.insert(selected.to_owned(), Value::Int(value));
            assert_eq!(
                selected_return(&node, &state).unwrap(),
                Some(Value::Int(value))
            );
        }
    }

    #[test]
    fn malformed_exit_contracts_fail_without_panicking() {
        for (instruction, arity) in [
            ("guard_return", 2),
            ("guard_drop_owned_bytes_return", 3),
            ("branch_drop_owned_bytes_return", 5),
        ] {
            for len in [0, arity - 1, arity + 2] {
                let node = node(instruction, &vec!["ready"; len]);
                assert!(selected_return(&node, &ExecutionState::default())
                    .unwrap_err()
                    .contains("arity"));
            }
            let node = node(instruction, &vec!["ready"; arity]);
            let mut state = ExecutionState::default();
            assert!(selected_return(&node, &state).is_err());
            state.values.insert("ready".to_owned(), Value::Unit);
            assert!(selected_return(&node, &state)
                .unwrap_err()
                .contains("bool or i64"));
        }
        assert_eq!(
            selected_return(&node("print", &[]), &ExecutionState::default()).unwrap(),
            None
        );
    }
}

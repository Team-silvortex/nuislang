use super::*;
use yir_core::{Operation, ResourceKind};

fn cpu_resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn flow_node(args: Vec<&str>) -> Node {
    Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_chain",
            args.into_iter().map(str::to_owned).collect(),
        )
        .unwrap(),
    }
}

#[test]
fn flow_chain_without_carries_has_bounded_semantics() {
    let node = flow_node(vec![
        "initial",
        "limit",
        "step",
        "lt",
        "add",
        "current_ge",
        "control_rhs",
        "break",
    ]);
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert!(semantics.has_effect);
    assert_eq!(
        semantics.dependencies,
        ["initial", "limit", "step", "control_rhs"]
    );
}

#[test]
fn flow_chain_collects_each_carry_initial() {
    let node = flow_node(vec![
        "initial",
        "limit",
        "step",
        "lt",
        "add",
        "current_ge",
        "control_rhs",
        "continue",
        "carry_initial",
        "add_current",
    ]);
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert_eq!(
        semantics.dependencies,
        ["initial", "limit", "step", "control_rhs", "carry_initial"]
    );
}

#[test]
fn post_flow_chain_accepts_an_unbounded_entry_condition() {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_chain",
            [
                "initial",
                "unused_limit",
                "step",
                "always",
                "add",
                "carry0_ge",
                "control_rhs",
                "break",
                "carry_initial",
                "add_current",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert!(semantics.has_effect);
    assert_eq!(
        semantics.dependencies,
        [
            "initial",
            "unused_limit",
            "step",
            "control_rhs",
            "carry_initial",
        ]
    );
}

#[test]
fn post_flow_chain_accepts_an_invariant_pattern_gate() {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_chain",
            [
                "initial",
                "pattern_condition",
                "step",
                "invariant_true",
                "add",
                "current_ge",
                "control_rhs",
                "break",
                "carry_initial",
                "add_current",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert!(semantics.has_effect);
    assert!(semantics
        .dependencies
        .iter()
        .any(|dependency| dependency == "pattern_condition"));
}

#[test]
fn post_flow_chain_accepts_a_terminal_pattern_transition() {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_chain",
            [
                "initial",
                "pattern_condition",
                "step",
                "pattern_exit",
                "add",
                "current_gt",
                "control_rhs",
                "continue",
                "carry_initial",
                "add_current",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();

    assert_eq!(
        semantics.dependencies,
        [
            "initial",
            "pattern_condition",
            "step",
            "control_rhs",
            "carry_initial",
        ]
    );
}

#[test]
fn post_flow_cond_chain_accepts_a_dynamic_pattern_carry_gate() {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_cond_chain",
            [
                "initial",
                "unused_limit",
                "step",
                "pattern_carry0",
                "add",
                "prev_carry1_gt",
                "control_rhs",
                "break",
                "active",
                "prev_carry1_gt",
                "threshold",
                "keep",
                "add_invariant",
                "minus_one",
                "payload",
                "prev_carry1_gt",
                "threshold",
                "add_invariant",
                "minus_one",
                "keep",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();

    assert!(semantics.has_effect);
    for dependency in ["active", "payload", "threshold", "minus_one"] {
        assert!(semantics
            .dependencies
            .iter()
            .any(|candidate| candidate == dependency));
    }
}

#[test]
fn previous_carry_flow_control_remains_post_flow_only() {
    let node = flow_node(vec![
        "initial",
        "limit",
        "step",
        "lt",
        "add",
        "prev_carry0_gt",
        "control_rhs",
        "break",
        "carry_initial",
        "add_current",
    ]);

    let error = describe_cpu_node(&node, &cpu_resource()).unwrap_err();
    assert!(error.contains("invalid flow control kind"), "{error}");
}

#[test]
fn execution_path_routes_previous_state_validation_to_async_post_flow_only() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    for (name, value) in [("initial", 4), ("limit", 0), ("rhs", 3)] {
        state.values.insert(name.to_owned(), Value::Int(value));
    }
    let node = |instruction: &str| Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            instruction,
            [
                "initial",
                "limit",
                "step",
                "gt",
                "prev_current_gt",
                "rhs",
                "break",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };

    assert_eq!(
        cpu.execute(
            &node("cpu.loop_while_scalar_async_post_flow_cond_chain"),
            &resource,
            &mut state,
        )
        .unwrap(),
        Value::Unit
    );
    let error = cpu
        .execute(
            &node("cpu.loop_while_scalar_async_flow_cond_chain"),
            &resource,
            &mut state,
        )
        .unwrap_err();
    assert!(error.contains("invalid flow control kind"), "{error}");
}

#[test]
fn guarded_loop_continue_effects_preserve_their_inputs() {
    for (instruction, args, expected) in [
        (
            "cpu.guard_loop_continue",
            vec!["condition"],
            vec!["condition"],
        ),
        (
            "cpu.guard_loop_print_continue",
            vec!["condition", "shown"],
            vec!["condition", "shown"],
        ),
    ] {
        let node = Node {
            name: "repeat".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, args.into_iter().map(str::to_owned).collect())
                .unwrap(),
        };
        let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
        assert!(semantics.has_effect);
        assert_eq!(semantics.dependencies, expected);
    }
}

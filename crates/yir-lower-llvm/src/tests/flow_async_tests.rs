use super::support::*;

#[test]
fn emits_mixed_break_continue_async_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("rhs0", "3"),
        ("rhs1", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_continue".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_sum".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_sum".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_sum", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_async_flow_action"));
    assert!(llvm_ir.contains("loop_async_flow_rhs"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_flow_cond_chain_cond"));
}

#[test]
fn emits_mixed_break_continue_async_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "8"),
        ("limit", "0"),
        ("rhs0", "5"),
        ("rhs1", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_continue".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_diff".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.sub",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_diff".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_diff".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_diff", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_async_post_flow_action"));
    assert!(llvm_ir.contains("loop_async_post_flow_rhs"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_flow_or_async_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "10"),
        ("limit", "0"),
        ("rhs0", "8"),
        ("rhs1", "4"),
        ("rhs2", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
                "flow_break".to_owned(),
                "current_eq".to_owned(),
                "rhs2".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_diff".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.sub",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_diff".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_diff".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_diff", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_rhs") >= 2);
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_boolean_flow_or_async_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "12"),
        ("limit", "0"),
        ("rhs0", "9"),
        ("rhs1", "6"),
        ("rhs2", "3"),
        ("rhs3", "2"),
        ("rhs4", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "or".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs3".to_owned(),
                "current_ne".to_owned(),
                "rhs4".to_owned(),
                "flow_break".to_owned(),
                "current_eq".to_owned(),
                "rhs4".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1", "rhs2", "rhs3", "rhs4"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_diff".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.sub",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_diff".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_diff".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_diff", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_rhs") >= 2);
    assert!(count_occurrences(&llvm_ir, " = and i1 ") >= 2);
    assert!(count_occurrences(&llvm_ir, " = or i1 ") >= 1);
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_cond"));
}

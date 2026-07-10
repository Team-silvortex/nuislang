use super::support::*;

#[test]
fn emits_recursive_boolean_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("step", "1"),
        ("carry0", "0"),
        ("rhs0", "2"),
        ("rhs1", "6"),
        ("rhs2", "4"),
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
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "or".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "continue".to_owned(),
                "carry0".to_owned(),
                "current_eq".to_owned(),
                "rhs0".to_owned(),
                "keep".to_owned(),
                "keep".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "carry0", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp sgt i64"));
    assert!(llvm_ir.contains("icmp slt i64"));
    assert!(llvm_ir.contains("icmp ne i64"));
    assert!(llvm_ir.contains(" = or i1 "));
    assert!(llvm_ir.contains(" = and i1 "));
}

#[test]
fn emits_recursive_boolean_sync_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "10"),
        ("limit", "0"),
        ("step", "1"),
        ("carry0", "0"),
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
            "cpu.loop_while_scalar_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "sub".to_owned(),
                "or".to_owned(),
                "current_eq".to_owned(),
                "rhs0".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "break".to_owned(),
                "carry0".to_owned(),
                "current_eq".to_owned(),
                "rhs0".to_owned(),
                "keep".to_owned(),
                "keep".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "carry0", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("icmp sgt i64"));
    assert!(llvm_ir.contains("icmp ne i64"));
    assert!(llvm_ir.contains(" = and i1 "));
    assert!(llvm_ir.contains(" = or i1 "));
}

#[test]
fn emits_mixed_break_continue_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("step", "1"),
        ("rhs0", "2"),
        ("rhs1", "1"),
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
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
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
    for from in ["initial", "limit", "step", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_flow_action"));
    assert!(llvm_ir.contains("loop_flow_rhs"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_flow_or_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "12"),
        ("step", "1"),
        ("rhs0", "7"),
        ("rhs1", "2"),
        ("rhs2", "5"),
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
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
                "flow_break".to_owned(),
                "current_eq".to_owned(),
                "rhs2".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_flow_rhs") >= 2);
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_boolean_flow_or_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "16"),
        ("step", "1"),
        ("rhs0", "10"),
        ("rhs1", "3"),
        ("rhs2", "7"),
        ("rhs3", "1"),
        ("rhs4", "5"),
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
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "or".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "or".to_owned(),
                "current_eq".to_owned(),
                "rhs3".to_owned(),
                "current_gt".to_owned(),
                "rhs4".to_owned(),
                "flow_break".to_owned(),
                "current_lt".to_owned(),
                "rhs4".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in [
        "initial", "limit", "step", "rhs0", "rhs1", "rhs2", "rhs3", "rhs4",
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_flow_rhs") >= 2);
    assert!(count_occurrences(&llvm_ir, " = and i1 ") >= 1);
    assert!(count_occurrences(&llvm_ir, " = or i1 ") >= 2);
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_cond"));
}

#[test]
fn emits_mixed_break_continue_sync_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "8"),
        ("limit", "0"),
        ("step", "1"),
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
            "cpu.loop_while_scalar_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "sub".to_owned(),
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
    for from in ["initial", "limit", "step", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_post_flow_action"));
    assert!(llvm_ir.contains("loop_post_flow_rhs"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_post_flow_cond_chain_cond"));
}

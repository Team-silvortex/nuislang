use super::support::*;

#[test]
fn rejects_flow_and_in_sync_flow_cond_chain() {
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
                "flow_and".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    assert_emit_module_error(&module, "is missing flow control action");
}

#[test]
fn rejects_missing_flow_action_in_sync_flow_cond_chain() {
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
                "current_gt".to_owned(),
                "rhs0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    assert_emit_module_error(&module, "is missing flow control action");
}

#[test]
fn rejects_missing_flow_rhs_in_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [("initial", "0"), ("limit", "8"), ("step", "1")] {
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
                "flow_break".to_owned(),
                "current_gt".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    assert_emit_module_error(&module, "is missing flow control rhs");
}

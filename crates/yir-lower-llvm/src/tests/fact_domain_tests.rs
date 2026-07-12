use super::support::*;

#[test]
fn folds_known_variant_is_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "ok_payload", "3");
    push_cpu_node(
        &mut module,
        "ok_variant",
        "cpu.struct",
        vec!["Result.Ok", "value=ok_payload"],
    );
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.variant_is",
        vec!["ok_variant", "Result.Ok"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[("ok_payload", "ok_variant"), ("ok_variant", "enabled")],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("zext i1 true to i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_network_state_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    module.resources.push(Resource {
        name: "network0".to_owned(),
        kind: ResourceKind::parse("network.main"),
    });
    push_cpu_const_i64(&mut module, "stream_window", "64");
    push_cpu_const_i64(&mut module, "send_window", "32");
    push_cpu_const_i64(&mut module, "remote_port", "443");
    push_cpu_node(
        &mut module,
        "send_probe",
        "cpu.extern_call_i64",
        vec![
            "c",
            "host_network_send_probe",
            "stream_window",
            "send_window",
            "remote_port",
        ],
    );
    module.nodes.push(Node {
        name: "network_result".to_owned(),
        resource: "network0".to_owned(),
        op: Operation::parse(
            "network.observe",
            vec!["send_probe".to_owned(), "send_ready".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "enabled".to_owned(),
        resource: "network0".to_owned(),
        op: Operation::parse("network.is_send_ready", vec!["network_result".to_owned()]).unwrap(),
    });
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("stream_window", "send_probe"),
            ("send_window", "send_probe"),
            ("remote_port", "send_probe"),
        ],
    );
    module
        .edges
        .retain(|edge| !(edge.from == "enabled" && edge.to == "selected"));
    module.edges.push(Edge {
        kind: EdgeKind::CrossDomainExchange,
        from: "send_probe".to_owned(),
        to: "network_result".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::CrossDomainExchange,
        from: "enabled".to_owned(),
        to: "selected".to_owned(),
    });
    push_dep(&mut module, "network_result", "enabled");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("zext i1 true to i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_task_result_state_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "task_payload", "11");
    push_cpu_node(
        &mut module,
        "task",
        "cpu.spawn_task",
        vec!["task_handle", "task_payload"],
    );
    push_cpu_node(&mut module, "task_result", "cpu.join_result", vec!["task"]);
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.task_completed",
        vec!["task_result"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("task_payload", "task"),
            ("task", "task_result"),
            ("task_result", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("zext i1 true to i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

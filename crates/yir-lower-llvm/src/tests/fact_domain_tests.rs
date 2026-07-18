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
fn folds_known_variant_field_value_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "ok_payload", "23");
    push_cpu_node(
        &mut module,
        "ok_variant",
        "cpu.struct",
        vec!["Result.Ok", "value=ok_payload"],
    );
    push_cpu_node(
        &mut module,
        "actual",
        "cpu.variant_field",
        vec!["ok_variant", "Result.Ok", "value"],
    );
    push_cpu_const_i64(&mut module, "expected", "23");
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["actual", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("ok_payload", "ok_variant"),
            ("ok_variant", "actual"),
            ("actual", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
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
fn lowers_task_result_state_through_scheduler_runtime() {
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
    push_deps(
        &mut module,
        &[
            ("task_payload", "task"),
            ("task", "task_result"),
            ("task_result", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("call i64 @nuis_scheduler_task_spawn_i64_v1"));
    assert!(llvm_ir.contains("call i64 @nuis_scheduler_task_join_state_v1"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.spawn_task `task`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.join_result `task_result`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.task_completed `enabled`"));
}

#[test]
fn folds_known_task_value_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "task_payload", "11");
    push_cpu_node(
        &mut module,
        "task",
        "cpu.spawn_task",
        vec!["task_handle", "task_payload"],
    );
    push_cpu_node(&mut module, "task_result", "cpu.join_result", vec!["task"]);
    push_cpu_node(&mut module, "actual", "cpu.task_value", vec!["task_result"]);
    push_cpu_const_i64(&mut module, "expected", "11");
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["actual", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("task_payload", "task"),
            ("task", "task_result"),
            ("task_result", "actual"),
            ("actual", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn lowers_positive_timeout_task_value_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "task_payload", "11");
    push_cpu_const_i64(&mut module, "timeout_ns", "16");
    push_cpu_node(
        &mut module,
        "task",
        "cpu.spawn_task",
        vec!["task_handle", "task_payload"],
    );
    push_cpu_node(
        &mut module,
        "bounded_task",
        "cpu.timeout",
        vec!["task", "timeout_ns"],
    );
    push_cpu_node(
        &mut module,
        "task_result",
        "cpu.join_result",
        vec!["bounded_task"],
    );
    push_cpu_node(&mut module, "actual", "cpu.task_value", vec!["task_result"]);
    push_cpu_const_i64(&mut module, "expected", "11");
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["actual", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("task_payload", "task"),
            ("task", "bounded_task"),
            ("timeout_ns", "bounded_task"),
            ("bounded_task", "task_result"),
            ("task_result", "actual"),
            ("actual", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.timeout `bounded_task`"));
    assert!(llvm_ir.contains("call void @nuis_scheduler_task_timeout_v1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.join_result `task_result`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.task_value `actual`"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn lowers_cancelled_task_state_through_scheduler_abi() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "task_payload", "11");
    push_cpu_node(
        &mut module,
        "task",
        "cpu.spawn_task",
        vec!["task_handle", "task_payload"],
    );
    push_cpu_node(&mut module, "cancelled_task", "cpu.cancel", vec!["task"]);
    push_cpu_node(
        &mut module,
        "task_result",
        "cpu.join_result",
        vec!["cancelled_task"],
    );
    push_cpu_node(
        &mut module,
        "cancelled",
        "cpu.task_cancelled",
        vec!["task_result"],
    );
    push_deps(
        &mut module,
        &[
            ("task_payload", "task"),
            ("task", "cancelled_task"),
            ("cancelled_task", "task_result"),
            ("task_result", "cancelled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("call void @nuis_scheduler_task_cancel_v1"));
    assert!(llvm_ir.contains("call i64 @nuis_scheduler_task_join_state_v1"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.cancel `cancelled_task`"));
}

#[test]
fn lowers_ready_delay_before_timeout_and_join() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "task_payload", "11");
    push_cpu_const_i64(&mut module, "ready_delay", "4");
    push_cpu_const_i64(&mut module, "timeout_limit", "3");
    push_cpu_node(
        &mut module,
        "task",
        "cpu.spawn_task",
        vec!["task_handle", "task_payload"],
    );
    push_cpu_node(
        &mut module,
        "delayed_task",
        "cpu.ready_after",
        vec!["task", "ready_delay"],
    );
    push_cpu_node(
        &mut module,
        "bounded_task",
        "cpu.timeout",
        vec!["delayed_task", "timeout_limit"],
    );
    push_cpu_node(
        &mut module,
        "task_result",
        "cpu.join_result",
        vec!["bounded_task"],
    );
    push_cpu_node(
        &mut module,
        "timed_out",
        "cpu.task_timed_out",
        vec!["task_result"],
    );
    push_deps(
        &mut module,
        &[
            ("task_payload", "task"),
            ("task", "delayed_task"),
            ("ready_delay", "delayed_task"),
            ("delayed_task", "bounded_task"),
            ("timeout_limit", "bounded_task"),
            ("bounded_task", "task_result"),
            ("task_result", "timed_out"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    let ready = llvm_ir
        .find("call void @nuis_scheduler_task_ready_after_v1")
        .expect("ready-delay ABI call");
    let timeout = llvm_ir
        .find("call void @nuis_scheduler_task_timeout_v1")
        .expect("timeout ABI call");
    let join = llvm_ir
        .find("call i64 @nuis_scheduler_task_join_state_v1")
        .expect("join ABI call");
    assert!(ready < timeout && timeout < join);
    assert!(!llvm_ir.contains("deferred lowering for cpu.ready_after `delayed_task`"));
}

#[test]
fn folds_known_mutex_value_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "shared_payload", "17");
    push_cpu_node(
        &mut module,
        "mutex",
        "cpu.mutex_new",
        vec!["shared_payload"],
    );
    push_cpu_node(&mut module, "guard", "cpu.mutex_lock", vec!["mutex"]);
    push_cpu_node(&mut module, "actual", "cpu.mutex_value", vec!["guard"]);
    push_cpu_const_i64(&mut module, "expected", "17");
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["actual", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("shared_payload", "mutex"),
            ("mutex", "guard"),
            ("guard", "actual"),
            ("actual", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

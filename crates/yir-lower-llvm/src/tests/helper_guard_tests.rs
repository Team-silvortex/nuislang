use super::support::*;

#[test]
fn emits_helper_function_lanes_and_cpu_call_i64() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "seed".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["6".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "invoke".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.call_i64", vec!["inc".to_owned(), "seed".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "seed".to_owned(),
        to: "invoke".to_owned(),
    });

    module.nodes.push(Node {
        name: "inc_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "inc_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "inc_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["inc_param_0".to_owned(), "inc_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "inc_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["inc_sum".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "inc_param_0".to_owned(),
        to: "inc_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "inc_one".to_owned(),
        to: "inc_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "inc_sum".to_owned(),
        to: "inc_ret".to_owned(),
    });
    for name in ["inc_param_0", "inc_one", "inc_sum", "inc_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:inc".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("define i64 @nuis_fn_inc(i64 %arg0)"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_inc(i64"));
}

#[test]
fn defers_spawned_i64_helper_call_until_scheduler_poll() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "seed", "6");
    push_cpu_node(
        &mut module,
        "schedule",
        "cpu.async_call",
        vec!["inc", "seed"],
    );
    push_cpu_node(&mut module, "invoke", "cpu.call_i64", vec!["inc", "seed"]);
    push_cpu_node(&mut module, "task", "cpu.spawn_task", vec!["inc", "invoke"]);
    push_cpu_node(&mut module, "result", "cpu.join_result", vec!["task"]);
    push_cpu_node(&mut module, "value", "cpu.task_value", vec!["result"]);
    push_cpu_node(&mut module, "inc_param_0", "cpu.param_i64", vec!["0"]);
    push_cpu_const_i64(&mut module, "inc_one", "1");
    push_cpu_node(
        &mut module,
        "inc_sum",
        "cpu.add",
        vec!["inc_param_0", "inc_one"],
    );
    push_cpu_node(&mut module, "inc_ret", "cpu.return_i64", vec!["inc_sum"]);
    push_deps(
        &mut module,
        &[
            ("seed", "schedule"),
            ("seed", "invoke"),
            ("invoke", "task"),
            ("task", "result"),
            ("result", "value"),
            ("inc_param_0", "inc_sum"),
            ("inc_one", "inc_sum"),
            ("inc_sum", "inc_ret"),
        ],
    );
    module.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: "schedule".to_owned(),
        to: "invoke".to_owned(),
    });
    for name in ["inc_param_0", "inc_one", "inc_sum", "inc_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:inc".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("define i64 @nuis_fn_inc(i64 %arg0)"));
    assert!(llvm_ir.contains("call i64 @nuis_scheduler_task_spawn_thunk_i64_v1(ptr @nuis_fn_inc"));
    assert!(!llvm_ir.contains("call i64 @nuis_fn_inc(i64"));
    assert!(llvm_ir.contains("call i64 @nuis_scheduler_task_value_i64_v1"));
}

#[test]
fn emits_guard_return_as_real_branch() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "cond".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "early".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["64".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_return",
            vec!["cond".to_owned(), "early".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "later".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["7".to_owned()]).unwrap(),
    });
    for from in ["cond", "early"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("br i1"));
    assert!(llvm_ir.contains("guard_return_then."));
    assert!(llvm_ir.contains("ret i64 %"));
    assert!(llvm_ir.contains("guard_return_cont."));
    assert_eq!(llvm_ir.matches("ret i64 ").count(), 2);
    assert!(llvm_ir.find("guard_return_cont.").unwrap() < llvm_ir.find("= add i64 0, 7").unwrap());
}

#[test]
fn resolves_structural_guard_return_through_fieldwise_selection() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "cond", "cpu.const_bool", vec!["true"]);
    push_cpu_const_i64(&mut module, "score", "64");
    push_cpu_node(
        &mut module,
        "summary",
        "cpu.struct",
        vec!["Summary", "score=score"],
    );
    push_cpu_node(
        &mut module,
        "guard",
        "cpu.guard_return",
        vec!["cond", "summary"],
    );
    push_cpu_const_i64(&mut module, "later", "7");
    push_deps(
        &mut module,
        &[
            ("score", "summary"),
            ("cond", "guard"),
            ("summary", "guard"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("structural cpu.guard_return `guard`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.guard_return `guard`"));
    assert!(llvm_ir.contains("= add i64 0, 7"));
}

#[test]
fn emits_guard_print_return_as_real_branch() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "cond".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "msg".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["usage".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "early".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["64".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_print_return",
            vec!["cond".to_owned(), "msg".to_owned(), "early".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "later".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["7".to_owned()]).unwrap(),
    });
    for from in ["cond", "msg", "early"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("br i1"));
    assert!(llvm_ir.contains("guard_print_return_then."));
    assert!(llvm_ir.contains("call i32 @puts(ptr"));
    assert_eq!(llvm_ir.matches("ret i64 ").count(), 2);
    assert!(llvm_ir.contains("guard_print_return_cont."));
}

#[test]
fn emits_branch_print_return_as_real_two_way_branch() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "cond".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "then_msg".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["ok".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "then_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "else_msg".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["fail".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "else_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["64".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "branch".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.branch_print_return",
            vec![
                "cond".to_owned(),
                "then_msg".to_owned(),
                "then_ret".to_owned(),
                "else_msg".to_owned(),
                "else_ret".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["cond", "then_msg", "then_ret", "else_msg", "else_ret"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "branch".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("branch_print_return_then."));
    assert!(llvm_ir.contains("branch_print_return_else."));
    assert!(llvm_ir.matches("call i32 @puts(ptr").count() >= 2);
    assert!(llvm_ir.matches("ret i64 ").count() >= 2);
}

#[test]
fn emits_i32_helper_returns_with_i32_ret_in_recursive_helpers() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "step_param".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i32", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "next".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.sub", vec!["step_param".to_owned(), "one".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i32", vec!["next".to_owned()]).unwrap(),
    });
    for from in ["step_param", "one", "next"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "step_ret".to_owned(),
        });
    }

    let ordered_node_names = vec![
        "step_param".to_owned(),
        "one".to_owned(),
        "next".to_owned(),
        "step_ret".to_owned(),
    ];
    let mut param_bindings = BTreeMap::new();
    param_bindings.insert(
        "step_param".to_owned(),
        cpu_param_binding(CpuCallScalarKind::I32, 0),
    );
    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<_, _>>();
    let mut global_counter = 0;

    let emitted = emit_cpu_function(
        &module,
        &resources,
        &ordered_node_names,
        &param_bindings,
        &BTreeMap::new(),
        CpuCallScalarKind::I32,
        &mut global_counter,
    )
    .expect("i32 helper lowering should succeed");

    assert!(emitted.body.contains("sub i32 "));
    assert!(emitted.body.contains("ret i32 "));
    assert!(!emitted.body.contains("ret i64 %"));
}

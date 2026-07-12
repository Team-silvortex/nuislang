use super::support::*;

#[test]
fn emits_module_with_contract_metadata_nodes_on_cpu_without_fake_cycles() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "seed".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["7".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "print_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.print", vec!["seed".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "lowering_cpu_target_config".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.target_config",
            vec![
                "arm64".to_owned(),
                "cpu.arm64.apple_aapcs64".to_owned(),
                "128".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "lowering_cpu_target_contract_type".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.text",
            vec![
                "arch=symbol:arm64;abi=symbol:cpu.arm64.apple_aapcs64;vector_bits=i64:128"
                    .to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "project_abi_cpu_selection_entry".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.text",
            vec!["mode=symbol:auto;abi=symbol:cpu.arm64.apple_aapcs64;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:llvm".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "project_abi_cpu_selection_summary_type".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.text",
            vec!["mode=symbol:auto;abi=symbol:cpu.arm64.apple_aapcs64;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:llvm".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "seed".to_owned(),
        to: "print_0".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "lowering_cpu_target_contract_type".to_owned(),
        to: "lowering_cpu_target_config".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "project_abi_cpu_selection_summary_type".to_owned(),
        to: "project_abi_cpu_selection_entry".to_owned(),
    });
    for name in [
        "lowering_cpu_target_config",
        "lowering_cpu_target_contract_type",
        "project_abi_cpu_selection_entry",
        "project_abi_cpu_selection_summary_type",
    ] {
        module
            .node_lanes
            .insert(name.to_owned(), "contract".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.target_config"));
}

#[test]
fn emits_static_aot_tick_i64_values() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "tick".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.tick_i64", vec!["4".to_owned(), "3".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "bias".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["10".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.add", vec!["tick".to_owned(), "bias".to_owned()]).unwrap(),
    });
    for from in ["tick", "bias"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "sum".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("static AOT lowering freezes cpu.tick_i64"));
    assert!(llvm_ir.contains("add i64 4, 3"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.tick_i64"));
}

#[test]
fn emits_three_arg_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [("arg0", "1"), ("arg1", "2"), ("arg2", "3")] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "spawn_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_subprocess_spawn".to_owned(),
                "arg0".to_owned(),
                "arg1".to_owned(),
                "arg2".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["arg0", "arg1", "arg2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "spawn_call".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_subprocess_spawn(i64, i64, i64)"));
    assert!(llvm_ir.contains("call i64 @host_subprocess_spawn(i64"));
}

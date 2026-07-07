use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

#[test]
fn emits_builtin_text_handle_extern_calls_as_i64_handles() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "text".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "write".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_stdout_write".to_owned(),
                "text".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "text".to_owned(),
        to: "write".to_owned(),
    });

    let llvm_ir = yir_lower_llvm::emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_stdout_write(i64)"));
    assert!(llvm_ir.contains("call i64 @host_stdout_write(i64"));
    assert!(!llvm_ir.contains("@host_stdout_write(ptr"));
}

#[test]
fn emits_guard_host_call_return_as_branch_local_host_call() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, op) in [
        (
            "cond",
            Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
        ),
        (
            "message",
            Operation::parse("cpu.text", vec!["usage\n".to_owned()]).unwrap(),
        ),
        (
            "code",
            Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
        ),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op,
        });
    }
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_host_call_return",
            vec![
                "cond".to_owned(),
                "code".to_owned(),
                "1".to_owned(),
                "c".to_owned(),
                "host_stdout_write".to_owned(),
                "1".to_owned(),
                "message".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["cond", "code", "message"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = yir_lower_llvm::emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("guard_host_call_return_then."));
    assert!(llvm_ir.contains("call i64 @host_stdout_write(i64"));
    assert!(!llvm_ir.contains("@host_stdout_write(ptr"));
}

#[test]
fn emits_guard_host_call_return_chain_in_branch_order() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, op) in [
        (
            "cond",
            Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
        ),
        (
            "message",
            Operation::parse("cpu.text", vec!["usage\n".to_owned()]).unwrap(),
        ),
        (
            "code",
            Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
        ),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op,
        });
    }
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_host_call_return",
            vec![
                "cond".to_owned(),
                "code".to_owned(),
                "2".to_owned(),
                "c".to_owned(),
                "host_stdout_write".to_owned(),
                "1".to_owned(),
                "message".to_owned(),
                "c".to_owned(),
                "host_stdout_flush".to_owned(),
                "0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["cond", "code", "message"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = yir_lower_llvm::emit_module(&module).expect("LLVM lowering should succeed");
    let write_at = llvm_ir.find("@host_stdout_write").expect("write call");
    let flush_at = llvm_ir.find("@host_stdout_flush").expect("flush call");
    assert!(write_at < flush_at);
}

#[test]
fn emits_guard_host_call_write_flush_exit_code_from_call_results() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, op) in [
        (
            "cond",
            Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
        ),
        (
            "message",
            Operation::parse("cpu.text", vec!["usage\n".to_owned()]).unwrap(),
        ),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op,
        });
    }
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_host_call_return",
            vec![
                "cond".to_owned(),
                "write_flush_exit_code".to_owned(),
                "3".to_owned(),
                "wrote".to_owned(),
                "flushed".to_owned(),
                "0".to_owned(),
                "2".to_owned(),
                "wrote".to_owned(),
                "c".to_owned(),
                "host_stdout_write".to_owned(),
                "1".to_owned(),
                "message".to_owned(),
                "flushed".to_owned(),
                "c".to_owned(),
                "host_stdout_flush".to_owned(),
                "0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["cond", "message"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = yir_lower_llvm::emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp sge i64"));
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("ret i64"));
}

#[test]
fn emits_branch_host_call_return_as_two_real_host_call_blocks() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, op) in [
        (
            "cond",
            Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
        ),
        (
            "message",
            Operation::parse("cpu.text", vec!["ok\n".to_owned()]).unwrap(),
        ),
        (
            "error",
            Operation::parse("cpu.text", vec!["err\n".to_owned()]).unwrap(),
        ),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op,
        });
    }
    module.nodes.push(Node {
        name: "branch".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.branch_host_call_return",
            vec![
                "cond".to_owned(),
                "write_flush_exit_code".to_owned(),
                "3".to_owned(),
                "wrote".to_owned(),
                "flushed".to_owned(),
                "0".to_owned(),
                "2".to_owned(),
                "wrote".to_owned(),
                "c".to_owned(),
                "host_stdout_write".to_owned(),
                "1".to_owned(),
                "message".to_owned(),
                "flushed".to_owned(),
                "c".to_owned(),
                "host_stdout_flush".to_owned(),
                "0".to_owned(),
                "write_flush_exit_code".to_owned(),
                "3".to_owned(),
                "err_wrote".to_owned(),
                "err_flushed".to_owned(),
                "1".to_owned(),
                "2".to_owned(),
                "err_wrote".to_owned(),
                "c".to_owned(),
                "host_stderr_write".to_owned(),
                "1".to_owned(),
                "error".to_owned(),
                "err_flushed".to_owned(),
                "c".to_owned(),
                "host_stderr_flush".to_owned(),
                "0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["cond", "message", "error"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "branch".to_owned(),
        });
    }

    let llvm_ir = yir_lower_llvm::emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("branch_host_call_return_then."));
    assert!(llvm_ir.contains("branch_host_call_return_else."));
    assert!(llvm_ir.contains("call i64 @host_stdout_write(i64"));
    assert!(llvm_ir.contains("call i64 @host_stderr_write(i64"));
}

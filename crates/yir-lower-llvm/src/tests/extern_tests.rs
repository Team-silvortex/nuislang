use super::support::*;

#[test]
fn emits_dynamic_declare_for_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "arg0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["100000".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "sleep_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec!["c".to_owned(), "usleep".to_owned(), "arg0".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "arg0".to_owned(),
        to: "sleep_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @usleep(i64)"));
    assert!(llvm_ir.contains("call i64 @usleep(i64"));
}

#[test]
fn emits_dynamic_declare_for_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "arg0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["7".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "curve_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec![
                "c".to_owned(),
                "host_i32_curve".to_owned(),
                "arg0".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "arg0".to_owned(),
        to: "curve_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @host_i32_curve(i32)"));
    assert!(llvm_ir.contains("call i32 @host_i32_curve(i32"));
}

#[test]
fn emits_dynamic_declare_for_libc_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "usec".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["100000".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "sleep_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "usleep".to_owned(), "usec".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "usec".to_owned(),
        to: "sleep_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @usleep(i32)"));
    assert!(llvm_ir.contains("call i32 @usleep(i32"));
}

#[test]
fn emits_dynamic_declare_for_libc_close_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "fd".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["-1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "close_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "close".to_owned(), "fd".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "fd".to_owned(),
        to: "close_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @close(i32)"));
    assert!(llvm_ir.contains("call i32 @close(i32"));
}

#[test]
fn lowers_libc_puts_text_argument_as_ptr() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello libc".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "puts_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "puts".to_owned(), "message".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "message".to_owned(),
        to: "puts_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @puts(ptr)"));
    assert!(llvm_ir.contains("call i32 @puts(ptr"));
    assert!(!llvm_ir.contains("call i32 @puts(i32"));
}

#[test]
fn lowers_libc_strlen_text_argument_as_ptr() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["nuis".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "strlen_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec!["libc".to_owned(), "strlen".to_owned(), "message".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "message".to_owned(),
        to: "strlen_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @strlen(ptr)"));
    assert!(llvm_ir.contains("call i64 @strlen(ptr"));
    assert!(!llvm_ir.contains("call i64 @strlen(i64"));
}

#[test]
fn lowers_libc_write_mixed_arguments_with_text_as_ptr() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "fd".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello write".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "len".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["11".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "write_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "libc".to_owned(),
                "write".to_owned(),
                "fd".to_owned(),
                "message".to_owned(),
                "len".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["fd", "message", "len"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "write_call".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @write(i32, ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @write(i32"));
    assert!(llvm_ir.contains(", ptr "));
    assert!(!llvm_ir.contains("@write(i32, i64, i64)"));
}

#[test]
fn emits_dynamic_declare_for_libc_no_arg_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "pid_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "getpid".to_owned()],
        )
        .unwrap(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @getpid()"));
    assert!(llvm_ir.contains("call i32 @getpid()"));
}

#[test]
fn emits_dynamic_declare_for_text_ptr_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "text_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_accept_text_ptr".to_owned(),
                "message".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "message".to_owned(),
        to: "text_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_accept_text_ptr(ptr)"));
    assert!(llvm_ir.contains("call i64 @host_accept_text_ptr(ptr"));
}

#[test]
fn emits_dynamic_declare_for_buffer_ptr_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "len".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["8".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "fill".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "buffer".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.alloc_buffer",
            vec!["len".to_owned(), "fill".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "buffer_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_fill_buffer_ptr".to_owned(),
                "buffer".to_owned(),
                "len".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("len", "buffer"),
        ("fill", "buffer"),
        ("buffer", "buffer_call"),
        ("len", "buffer_call"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_fill_buffer_ptr(ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @host_fill_buffer_ptr(ptr"));
}

#[test]
fn emits_dynamic_declare_for_libc_read_ref_buffer_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("fd", "cpu.const_i32", "-1"),
        ("len", "cpu.const_i64", "8"),
        ("fill", "cpu.const_i64", "0"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "scratch".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.alloc_buffer",
            vec!["len".to_owned(), "fill".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "read_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "libc".to_owned(),
                "read".to_owned(),
                "fd".to_owned(),
                "scratch".to_owned(),
                "len".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("len", "scratch"),
        ("fill", "scratch"),
        ("fd", "read_call"),
        ("scratch", "read_call"),
        ("len", "read_call"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @read(i32, ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @read(i32"));
}

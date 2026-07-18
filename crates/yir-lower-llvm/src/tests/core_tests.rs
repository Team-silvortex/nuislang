use super::support::*;

#[test]
fn lowers_buffer_copy_to_glm_owned_blob() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "len", "3");
    push_cpu_const_i64(&mut module, "fill", "7");
    push_cpu_node(
        &mut module,
        "buffer",
        "cpu.alloc_buffer",
        vec!["len", "fill"],
    );
    push_cpu_node(
        &mut module,
        "bytes",
        "cpu.copy_buffer_owned",
        vec!["buffer"],
    );
    push_cpu_node(
        &mut module,
        "byte_len",
        "cpu.owned_bytes_len",
        vec!["bytes"],
    );
    push_cpu_node(
        &mut module,
        "drop_bytes",
        "cpu.drop_owned_bytes",
        vec!["bytes"],
    );
    push_cpu_node(&mut module, "release", "cpu.free", vec!["buffer"]);
    push_cpu_const_i64(&mut module, "status", "0");
    push_deps(
        &mut module,
        &[
            ("len", "buffer"),
            ("fill", "buffer"),
            ("buffer", "bytes"),
            ("bytes", "byte_len"),
            ("bytes", "drop_bytes"),
            ("byte_len", "drop_bytes"),
            ("buffer", "release"),
            ("drop_bytes", "release"),
            ("release", "status"),
        ],
    );
    module.edges.push(Edge {
        kind: EdgeKind::Lifetime,
        from: "buffer".to_owned(),
        to: "release".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Lifetime,
        from: "bytes".to_owned(),
        to: "drop_bytes".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("owned buffer copy lowering should succeed");
    assert!(llvm_ir.contains("mul i64 3, 8"));
    assert!(llvm_ir.contains("call ptr @nuis_scheduler_owned_blob_copy_v1(ptr"));
    assert!(llvm_ir.contains("call i64 @nuis_scheduler_owned_blob_len_v1(ptr"));
    assert!(llvm_ir.contains("call void @nuis_scheduler_owned_blob_drop_v1(ptr"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.copy_buffer_owned `bytes`"));
}

#[test]
fn lowers_cpu_select_between_structs_then_field_access() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("enabled", "true"),
        ("then_score", "42"),
        ("then_aux", "7"),
        ("else_score", "-1"),
        ("else_aux", "3"),
    ] {
        let instruction = if value == "true" {
            "cpu.const_bool"
        } else {
            "cpu.const_i64"
        };
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "then_pair".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec![
                "Pair".to_owned(),
                "score=then_score".to_owned(),
                "aux=then_aux".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "else_pair".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec![
                "Pair".to_owned(),
                "score=else_score".to_owned(),
                "aux=else_aux".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "then_pair".to_owned(),
                "else_pair".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_score".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.field", vec!["selected".to_owned(), "score".to_owned()]).unwrap(),
    });
    for (from, to) in [
        ("then_score", "then_pair"),
        ("then_aux", "then_pair"),
        ("else_score", "else_pair"),
        ("else_aux", "else_pair"),
        ("enabled", "selected"),
        ("then_pair", "selected"),
        ("else_pair", "selected"),
        ("selected", "selected_score"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.field `selected_score`"));
}

#[test]
fn lowers_cpu_select_between_bool_and_i64_condition_values_as_bool() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("enabled", "cpu.const_bool", "true"),
        ("explicit_false", "cpu.const_bool", "false"),
        ("lhs", "cpu.const_i64", "8"),
        ("rhs", "cpu.const_i64", "3"),
        ("ok_value", "cpu.const_i64", "0"),
        ("err_value", "cpu.const_i64", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "computed_bool_i64".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.gt", vec!["lhs".to_owned(), "rhs".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_bool".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "explicit_false".to_owned(),
                "computed_bool_i64".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "status".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "selected_bool".to_owned(),
                "ok_value".to_owned(),
                "err_value".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("lhs", "computed_bool_i64"),
        ("rhs", "computed_bool_i64"),
        ("enabled", "selected_bool"),
        ("explicit_false", "selected_bool"),
        ("computed_bool_i64", "selected_bool"),
        ("selected_bool", "status"),
        ("ok_value", "status"),
        ("err_value", "status"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected_bool`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `status`"));
}

#[test]
fn lowers_cpu_select_between_enum_variants_as_tagged_union() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("enabled", "cpu.const_bool", "true"),
        ("ok_payload", "cpu.const_i64", "7"),
        ("err_payload", "cpu.const_i64", "99"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "ok_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Ok".to_owned(), "value=ok_payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "err_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Err".to_owned(), "value=err_payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "ok_variant".to_owned(),
                "err_variant".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_is_err".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_is",
            vec!["selected".to_owned(), "Result.Err".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_ok_value".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_field",
            vec![
                "selected".to_owned(),
                "Result.Ok".to_owned(),
                "value".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("ok_payload", "ok_variant"),
        ("err_payload", "err_variant"),
        ("enabled", "selected"),
        ("ok_variant", "selected"),
        ("err_variant", "selected"),
        ("selected", "selected_is_err"),
        ("selected", "selected_ok_value"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_is `selected_is_err`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_field `selected_ok_value`"));
}

#[test]
fn lowers_const_select_around_unselected_wrong_variant_payload_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("enabled", "cpu.const_bool", "false"),
        ("payload", "cpu.const_i64", "41"),
        ("one", "cpu.const_i64", "1"),
        ("fallback", "cpu.const_i64", "7"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "err_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Err".to_owned(), "value=payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "wrong_payload".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_field",
            vec![
                "err_variant".to_owned(),
                "Result.Ok".to_owned(),
                "value".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "bad_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["wrong_payload".to_owned(), "one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "bad_result".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Ok".to_owned(), "value=bad_sum".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "bad_result".to_owned(),
                "fallback".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("payload", "err_variant"),
        ("err_variant", "wrong_payload"),
        ("wrong_payload", "bad_sum"),
        ("one", "bad_sum"),
        ("bad_sum", "bad_result"),
        ("enabled", "selected"),
        ("bad_result", "selected"),
        ("fallback", "selected"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_field `wrong_payload`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.add `bad_sum`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.struct `bad_result`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
}

#[test]
fn defers_dynamic_select_around_wrong_variant_payload_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("lhs_bias", "3"),
        ("rhs", "3"),
        ("payload", "41"),
        ("one", "1"),
        ("fallback", "7"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "lhs_seed".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.tick_i64", vec!["5".to_owned(), "0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "lhs".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["lhs_seed".to_owned(), "lhs_bias".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "enabled".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.gt", vec!["lhs".to_owned(), "rhs".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "err_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Err".to_owned(), "value=payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "wrong_payload".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_field",
            vec![
                "err_variant".to_owned(),
                "Result.Ok".to_owned(),
                "value".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "bad_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["wrong_payload".to_owned(), "one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "bad_result".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Ok".to_owned(), "value=bad_sum".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "bad_result".to_owned(),
                "fallback".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("lhs_seed", "lhs"),
        ("lhs_bias", "lhs"),
        ("lhs", "enabled"),
        ("rhs", "enabled"),
        ("payload", "err_variant"),
        ("err_variant", "wrong_payload"),
        ("wrong_payload", "bad_sum"),
        ("one", "bad_sum"),
        ("bad_sum", "bad_result"),
        ("enabled", "selected"),
        ("bad_result", "selected"),
        ("fallback", "selected"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("ret i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert!(llvm_ir.contains("then `bad_result`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_field `wrong_payload`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.add `bad_sum`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.struct `bad_result`"));
}

#[test]
fn defers_const_select_when_selected_branch_is_wrong_variant_payload_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("enabled", "cpu.const_bool", "true"),
        ("payload", "cpu.const_i64", "41"),
        ("one", "cpu.const_i64", "1"),
        ("fallback", "cpu.const_i64", "7"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "err_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Err".to_owned(), "value=payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "wrong_payload".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_field",
            vec![
                "err_variant".to_owned(),
                "Result.Ok".to_owned(),
                "value".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "bad_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["wrong_payload".to_owned(), "one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "bad_result".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Ok".to_owned(), "value=bad_sum".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "bad_result".to_owned(),
                "fallback".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("payload", "err_variant"),
        ("err_variant", "wrong_payload"),
        ("wrong_payload", "bad_sum"),
        ("one", "bad_sum"),
        ("bad_sum", "bad_result"),
        ("enabled", "selected"),
        ("bad_result", "selected"),
        ("fallback", "selected"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("selected branch `bad_result` is delayed"));
    assert!(llvm_ir.contains("depends on delayed `wrong_payload`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_field `wrong_payload`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.add `bad_sum`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.struct `bad_result`"));
}

use super::support::*;

#[test]
fn folds_known_i64_condition_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "enabled", "1");
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_static_input_default_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "input", "cpu.input_i64", vec!["limit", "42"]);
    push_cpu_const_i64(&mut module, "expected", "42");
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["input", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[("input", "enabled"), ("expected", "enabled")],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("static AOT lowering freezes cpu.input_i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_buffer_len_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "len", "4");
    push_cpu_const_i64(&mut module, "fill", "0");
    push_cpu_node(
        &mut module,
        "buffer",
        "cpu.alloc_buffer",
        vec!["len", "fill"],
    );
    push_cpu_node(&mut module, "actual_len", "cpu.buffer_len", vec!["buffer"]);
    push_cpu_const_i64(&mut module, "expected", "4");
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.eq",
        vec!["actual_len", "expected"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("len", "buffer"),
            ("fill", "buffer"),
            ("buffer", "actual_len"),
            ("actual_len", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("call ptr @malloc"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_null_check_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "ptr", "cpu.null", vec![]);
    push_cpu_node(&mut module, "enabled", "cpu.is_null", vec!["ptr"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_dep(&mut module, "ptr", "enabled");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq ptr null, null"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_struct_field_fact_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "score", "13");
    push_cpu_node(
        &mut module,
        "packet",
        "cpu.struct",
        vec!["Packet", "score=score"],
    );
    push_cpu_node(&mut module, "actual", "cpu.field", vec!["packet", "score"]);
    push_cpu_const_i64(&mut module, "expected", "13");
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["actual", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("score", "packet"),
            ("score", "actual"),
            ("packet", "actual"),
            ("actual", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.field `actual`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

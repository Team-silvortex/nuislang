use super::support::*;

fn push_i64_compare_select_fixture(module: &mut YirModule, op: &str, lhs: &str, rhs: &str) {
    push_cpu_const_i64(module, "lhs", lhs);
    push_cpu_const_i64(module, "rhs", rhs);
    push_cpu_node(module, "enabled", op, vec!["lhs", "rhs"]);
    push_wrong_variant_payload_select_fixture(module, "enabled", "fallback", "bad_result");
    push_deps(module, &[("lhs", "enabled"), ("rhs", "enabled")]);
}

#[test]
fn folds_known_i64_madd_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_madd_i64_condition_fixture(&mut module, "2", "3", "1", "7");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("mul i64"));
    assert!(llvm_ir.contains("add i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_bitwise_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    for (name, value) in [("lhs", "6"), ("mask", "2"), ("expected", "2")] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(&mut module, "masked", "cpu.and", vec!["lhs", "mask"]);
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["masked", "expected"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("lhs", "masked"),
            ("mask", "masked"),
            ("masked", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("and i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn does_not_fold_oversized_i64_shift_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    for (name, value) in [("lhs", "1"), ("shift", "64"), ("expected", "0")] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(&mut module, "shifted", "cpu.shl", vec!["lhs", "shift"]);
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.eq",
        vec!["shifted", "expected"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("lhs", "shifted"),
            ("shift", "shifted"),
            ("shifted", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("shl i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn does_not_fold_overflowing_i64_madd_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_madd_i64_condition_fixture(&mut module, "9223372036854775807", "2", "0", "0");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("mul i64"));
    assert!(llvm_ir.contains("add i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_comparison_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_i64_compare_select_fixture(&mut module, "cpu.lt", "2", "3");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp slt i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_arithmetic_chain_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_binary_i64_condition_fixture(&mut module, "5", "3", "3", "cpu.add", "cpu.gt");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("add i64"));
    assert!(llvm_ir.contains("icmp sgt i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn propagates_known_i64_through_select_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "choose_lhs", "cpu.const_bool", vec!["true"]);
    push_cpu_const_i64(&mut module, "lhs", "5");
    push_cpu_const_i64(&mut module, "rhs", "9");
    push_cpu_node(
        &mut module,
        "selected_i64",
        "cpu.select",
        vec!["choose_lhs", "lhs", "rhs"],
    );
    push_cpu_const_i64(&mut module, "expected", "5");
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.eq",
        vec!["selected_i64", "expected"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("choose_lhs", "selected_i64"),
            ("lhs", "selected_i64"),
            ("rhs", "selected_i64"),
            ("selected_i64", "enabled"),
            ("expected", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn propagates_known_bool_through_select_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "choose_true", "cpu.const_bool", vec!["true"]);
    push_cpu_node(&mut module, "truthy", "cpu.const_bool", vec!["true"]);
    push_cpu_node(&mut module, "falsy", "cpu.const_bool", vec!["false"]);
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.select",
        vec!["choose_true", "truthy", "falsy"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("choose_true", "enabled"),
            ("truthy", "enabled"),
            ("falsy", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn does_not_fold_overflowing_i64_arithmetic_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_binary_i64_condition_fixture(
        &mut module,
        "9223372036854775807",
        "1",
        "0",
        "cpu.add",
        "cpu.gt",
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("add i64"));
    assert!(llvm_ir.contains("icmp sgt i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_division_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    for (name, value) in [("lhs_seed", "9"), ("lhs_divisor", "3"), ("rhs", "3")] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "lhs",
        "cpu.div",
        vec!["lhs_seed", "lhs_divisor"],
    );
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["lhs", "rhs"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("lhs_seed", "lhs"),
            ("lhs_divisor", "lhs"),
            ("lhs", "enabled"),
            ("rhs", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sdiv i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn does_not_fold_division_by_zero_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    for (name, value) in [("lhs_seed", "9"), ("lhs_divisor", "0"), ("rhs", "3")] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "lhs",
        "cpu.div",
        vec!["lhs_seed", "lhs_divisor"],
    );
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["lhs", "rhs"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("lhs_seed", "lhs"),
            ("lhs_divisor", "lhs"),
            ("lhs", "enabled"),
            ("rhs", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sdiv i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_remainder_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    for (name, value) in [("lhs_seed", "10"), ("lhs_divisor", "4"), ("rhs", "2")] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "lhs",
        "cpu.rem",
        vec!["lhs_seed", "lhs_divisor"],
    );
    push_cpu_node(&mut module, "enabled", "cpu.eq", vec!["lhs", "rhs"]);
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_deps(
        &mut module,
        &[
            ("lhs_seed", "lhs"),
            ("lhs_divisor", "lhs"),
            ("lhs", "enabled"),
            ("rhs", "enabled"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("srem i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_equality_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_i64_compare_select_fixture(&mut module, "cpu.eq", "2", "2");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

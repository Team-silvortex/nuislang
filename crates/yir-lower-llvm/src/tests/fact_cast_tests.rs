use super::support::*;

fn push_eq_expected_lazy_select_fixture(
    module: &mut YirModule,
    actual: &str,
    expected_value: &str,
    deps_to_actual: &[(&str, &str)],
) {
    push_cpu_const_i64(module, "expected", expected_value);
    push_cpu_node(module, "enabled", "cpu.eq", vec![actual, "expected"]);
    push_wrong_variant_payload_select_fixture(module, "enabled", "fallback", "bad_result");
    let mut deps = deps_to_actual.to_vec();
    deps.push((actual, "enabled"));
    deps.push(("expected", "enabled"));
    push_deps(module, &deps);
}

#[test]
fn folds_known_i64_to_bool_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "truthy", "1");
    push_cpu_node(
        &mut module,
        "enabled",
        "cpu.cast_i64_to_bool",
        vec!["truthy"],
    );
    push_wrong_variant_payload_select_fixture(&mut module, "enabled", "fallback", "bad_result");
    push_dep(&mut module, "truthy", "enabled");

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp ne i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_bool_to_i64_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "flag", "cpu.const_bool", vec!["true"]);
    push_cpu_node(
        &mut module,
        "flag_i64",
        "cpu.cast_bool_to_i64",
        vec!["flag"],
    );
    push_eq_expected_lazy_select_fixture(&mut module, "flag_i64", "1", &[("flag", "flag_i64")]);

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("zext i1 true to i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i32_to_i64_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "narrow", "cpu.const_i32", vec!["21"]);
    push_cpu_node(&mut module, "wide", "cpu.cast_i32_to_i64", vec!["narrow"]);
    push_eq_expected_lazy_select_fixture(&mut module, "wide", "21", &[("narrow", "wide")]);

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sext i32"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i64_to_i32_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "wide", "21");
    push_cpu_node(&mut module, "narrow", "cpu.cast_i64_to_i32", vec!["wide"]);
    push_eq_expected_lazy_select_fixture(&mut module, "narrow", "21", &[("wide", "narrow")]);

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("trunc i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn does_not_fold_out_of_range_i64_to_i32_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "wide", "2147483648");
    push_cpu_node(&mut module, "narrow", "cpu.cast_i64_to_i32", vec!["wide"]);
    push_eq_expected_lazy_select_fixture(
        &mut module,
        "narrow",
        "2147483648",
        &[("wide", "narrow")],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("trunc i64"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_i32_to_f64_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "narrow", "cpu.const_i32", vec!["21"]);
    push_cpu_node(
        &mut module,
        "wide_float",
        "cpu.cast_i32_to_f64",
        vec!["narrow"],
    );
    push_eq_expected_lazy_select_fixture(
        &mut module,
        "wide_float",
        "21",
        &[("narrow", "wide_float")],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sitofp i32"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_safe_known_i32_to_f32_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "narrow", "cpu.const_i32", vec!["16777216"]);
    push_cpu_node(&mut module, "float", "cpu.cast_i32_to_f32", vec!["narrow"]);
    push_eq_expected_lazy_select_fixture(&mut module, "float", "16777216", &[("narrow", "float")]);

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sitofp i32"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn does_not_fold_unsafe_i32_to_f32_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "narrow", "cpu.const_i32", vec!["16777217"]);
    push_cpu_node(&mut module, "float", "cpu.cast_i32_to_f32", vec!["narrow"]);
    push_eq_expected_lazy_select_fixture(&mut module, "float", "16777217", &[("narrow", "float")]);

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sitofp i32"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(llvm_ir.contains("delayed branch lowering requires a compile-time constant condition"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_known_f32_to_f64_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "narrow", "cpu.const_i32", vec!["21"]);
    push_cpu_node(&mut module, "float", "cpu.cast_i32_to_f32", vec!["narrow"]);
    push_cpu_node(
        &mut module,
        "wide_float",
        "cpu.cast_f32_to_f64",
        vec!["float"],
    );
    push_eq_expected_lazy_select_fixture(
        &mut module,
        "wide_float",
        "21",
        &[("narrow", "float"), ("float", "wide_float")],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("fpext float"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn folds_safe_known_f64_to_f32_cast_for_lazy_const_select() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "narrow", "cpu.const_i32", vec!["21"]);
    push_cpu_node(
        &mut module,
        "wide_float",
        "cpu.cast_i32_to_f64",
        vec!["narrow"],
    );
    push_cpu_node(
        &mut module,
        "float",
        "cpu.cast_f64_to_f32",
        vec!["wide_float"],
    );
    push_eq_expected_lazy_select_fixture(
        &mut module,
        "float",
        "21",
        &[("narrow", "wide_float"), ("wide_float", "float")],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("fptrunc double"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(!llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert_wrong_variant_chain_not_deferred(&llvm_ir);
}

#[test]
fn lowers_i64_float_roundtrip_cast_instructions() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "integer", "21");
    push_cpu_node(
        &mut module,
        "float32",
        "cpu.cast_i64_to_f32",
        vec!["integer"],
    );
    push_cpu_node(
        &mut module,
        "from_float32",
        "cpu.cast_f32_to_i64",
        vec!["float32"],
    );
    push_cpu_node(
        &mut module,
        "float64",
        "cpu.cast_i64_to_f64",
        vec!["from_float32"],
    );
    push_cpu_node(
        &mut module,
        "from_float64",
        "cpu.cast_f64_to_i64",
        vec!["float64"],
    );
    push_deps(
        &mut module,
        &[
            ("integer", "float32"),
            ("float32", "from_float32"),
            ("from_float32", "float64"),
            ("float64", "from_float64"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("sitofp i64"));
    assert!(llvm_ir.contains("fptosi float"));
    assert!(llvm_ir.contains("fptosi double"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.cast_"));
}

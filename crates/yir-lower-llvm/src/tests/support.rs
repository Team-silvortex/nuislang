pub(super) use std::collections::BTreeMap;

pub(super) use super::super::{
    cpu_param_binding, emit_cpu_function, emit_module, CpuCallScalarKind,
};
pub(super) use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

pub(super) fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

pub(super) fn assert_emit_module_error(module: &YirModule, expected_fragment: &str) {
    let error = emit_module(module).expect_err("LLVM lowering should fail");
    assert!(
        error.contains(expected_fragment),
        "expected error to contain `{expected_fragment}`, got `{error}`"
    );
}

pub(super) fn module_with_cpu0() -> YirModule {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module
}

pub(super) fn push_cpu_node(
    module: &mut YirModule,
    name: &str,
    instruction: &str,
    args: Vec<&str>,
) {
    module.nodes.push(Node {
        name: name.to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            instruction,
            args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
        )
        .unwrap(),
    });
}

pub(super) fn push_cpu_const_i64(module: &mut YirModule, name: &str, value: &str) {
    push_cpu_node(module, name, "cpu.const_i64", vec![value]);
}

pub(super) fn push_dep(module: &mut YirModule, from: &str, to: &str) {
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

pub(super) fn push_deps(module: &mut YirModule, deps: &[(&str, &str)]) {
    for (from, to) in deps {
        push_dep(module, from, to);
    }
}

pub(super) fn push_wrong_variant_payload_select_fixture(
    module: &mut YirModule,
    condition: &str,
    then_branch: &str,
    else_branch: &str,
) {
    push_cpu_const_i64(module, "payload", "41");
    push_cpu_const_i64(module, "one", "1");
    push_cpu_const_i64(module, "fallback", "7");
    push_cpu_node(
        module,
        "err_variant",
        "cpu.struct",
        vec!["Result.Err", "value=payload"],
    );
    push_cpu_node(
        module,
        "wrong_payload",
        "cpu.variant_field",
        vec!["err_variant", "Result.Ok", "value"],
    );
    push_cpu_node(module, "bad_sum", "cpu.add", vec!["wrong_payload", "one"]);
    push_cpu_node(
        module,
        "bad_result",
        "cpu.struct",
        vec!["Result.Ok", "value=bad_sum"],
    );
    push_cpu_node(
        module,
        "selected",
        "cpu.select",
        vec![condition, then_branch, else_branch],
    );
    push_deps(
        module,
        &[
            ("payload", "err_variant"),
            ("err_variant", "wrong_payload"),
            ("wrong_payload", "bad_sum"),
            ("one", "bad_sum"),
            ("bad_sum", "bad_result"),
            (condition, "selected"),
            (then_branch, "selected"),
            (else_branch, "selected"),
        ],
    );
}

pub(super) fn push_binary_i64_condition_fixture(
    module: &mut YirModule,
    lhs_seed: &str,
    lhs_rhs: &str,
    rhs: &str,
    binary_op: &str,
    compare_op: &str,
) {
    push_cpu_const_i64(module, "lhs_seed", lhs_seed);
    push_cpu_const_i64(module, "lhs_rhs", lhs_rhs);
    push_cpu_const_i64(module, "rhs", rhs);
    push_cpu_node(module, "lhs", binary_op, vec!["lhs_seed", "lhs_rhs"]);
    push_cpu_node(module, "enabled", compare_op, vec!["lhs", "rhs"]);
    push_wrong_variant_payload_select_fixture(module, "enabled", "fallback", "bad_result");
    push_deps(
        module,
        &[
            ("lhs_seed", "lhs"),
            ("lhs_rhs", "lhs"),
            ("lhs", "enabled"),
            ("rhs", "enabled"),
        ],
    );
}

pub(super) fn push_madd_i64_condition_fixture(
    module: &mut YirModule,
    lhs: &str,
    rhs_mul: &str,
    acc: &str,
    expected: &str,
) {
    for (name, value) in [
        ("lhs", lhs),
        ("rhs_mul", rhs_mul),
        ("acc", acc),
        ("expected", expected),
    ] {
        push_cpu_const_i64(module, name, value);
    }
    push_cpu_node(
        module,
        "madd_value",
        "cpu.madd",
        vec!["lhs", "rhs_mul", "acc"],
    );
    push_cpu_node(module, "enabled", "cpu.eq", vec!["madd_value", "expected"]);
    push_wrong_variant_payload_select_fixture(module, "enabled", "fallback", "bad_result");
    push_deps(
        module,
        &[
            ("lhs", "madd_value"),
            ("rhs_mul", "madd_value"),
            ("acc", "madd_value"),
            ("madd_value", "enabled"),
            ("expected", "enabled"),
        ],
    );
}

pub(super) fn assert_wrong_variant_chain_not_deferred(llvm_ir: &str) {
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_field `wrong_payload`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.add `bad_sum`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.struct `bad_result`"));
}

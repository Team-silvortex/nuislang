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

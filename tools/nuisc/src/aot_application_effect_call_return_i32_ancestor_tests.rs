use super::*;

const SOURCE: &str =
    include_str!("../tests/native_application_bridge/return_i32_ancestor_private_bool.ns");

#[test]
fn native_return_i32_ancestor_private_bool_keeps_typed_single_export_effect_order_and_policy_vetoes(
) {
    let (_, mut module, mode) = compiled(SOURCE);
    let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
    assert!(bridge.state_layout.source().ends_with(";result:bool}"));
    assert!(module
        .functions
        .iter()
        .any(|f| f.name.starts_with("__nuis_effect_call_")
            && f.result.as_ref().is_some_and(|result| result.ty == "i32")));
    for op in ["cast_i32_to_i64", "cast_i64_to_i32"] {
        assert!(module.nodes.iter().any(|n| n.op.instruction == op), "{op}");
    }
    assert!(!bridge
        .llvm_ir
        .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    module
        .edges
        .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
    let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
    assert!(error.contains("lacks dependency order"), "{error}");
}

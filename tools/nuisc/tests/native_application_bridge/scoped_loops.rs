use super::*;

const SCOPED: &str = include_str!("scoped_loops.ns");

#[test]
fn scoped_scalar_loop_calls_keep_native_reference_and_typed_state_parity() {
    let project = Project::with_source(SCOPED);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let module = &compiled.yir;
    let scoped = module
        .nodes
        .iter()
        .filter(|node| node.op.instruction == "loop_while_i64_effect")
        .collect::<Vec<_>>();
    assert_eq!(scoped.len(), 4, "{scoped:?}");
    for action in ["scoped_call", "scoped_call_i64_carry"] {
        assert!(scoped.iter().any(|node| node.op.args[6] == action));
    }
    let bridge = emit_registered(module, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("native_loop_preflight"));
    for node in scoped {
        assert!(bridge
            .llvm_ir
            .contains(&format!("@nuis_fn_{}(", node.op.args[8])));
    }
    assert_native_parity(SCOPED, true);
}

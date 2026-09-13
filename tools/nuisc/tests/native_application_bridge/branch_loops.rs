use super::*;

const BRANCHES: &str = include_str!("branch_loops.ns");

#[test]
fn guarded_multi_state_exits_preserve_native_reference_and_typed_session_parity() {
    let project = Project::with_source(BRANCHES);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let calls = module
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "call_owned_struct")
        .collect::<Vec<_>>();
    assert!(
        !calls.is_empty(),
        "source must retain aggregate branch helpers"
    );
    let bridge = emit_registered(&module, "counter").unwrap();
    for node in calls {
        assert!(bridge
            .llvm_ir
            .contains(&format!(" = call i64 @nuis_fn_{}(", node.op.args[0])));
    }
    assert!(module
        .nodes
        .iter()
        .any(|n| n.op.instruction == "guard_return"));
    assert_native_parity(BRANCHES, true);
}

use super::*;

const DYNAMIC: &str = include_str!("dynamic_loops.ns");

#[test]
fn dynamic_scalar_loops_preserve_native_reference_and_typed_helper_parity() {
    let project = Project::with_source(DYNAMIC);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(!compiled.llvm_ir.contains("native_loop_preflight"));
    let module = compiled.yir;
    let bridge = emit_registered(&module, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("native_loop_preflight"));
    assert!(bridge.llvm_ir.contains("udiv i64"));
    assert!(!bridge.llvm_ir.contains("div i128"));
    assert_native_parity(DYNAMIC, true);
}

#[test]
fn dynamic_bound_may_share_a_carry_seed_without_duplicate_effect_edges() {
    let source = include_str!("loops.ns").replacen("index < 4", "index < value", 1);
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    assert!(emit_registered(&module, "counter")
        .unwrap()
        .llvm_ir
        .contains("native_loop_preflight"));
}

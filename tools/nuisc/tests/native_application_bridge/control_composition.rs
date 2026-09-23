use super::*;

const COMPOSED: &str = include_str!("control_composition.ns");

#[test]
fn composed_counted_returns_fit_native_graph_without_relaxing_limits() {
    let project = Project::with_source(COMPOSED);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    let functions = bridge
        .llvm_ir
        .lines()
        .filter(|line| line.starts_with("define ") && line.contains(" @nuis_fn_"))
        .count();
    // This source exceeded the unchanged 64-function admission limit before
    // ready-value elision reached 57, terminal folding 54 and statement folding 51.
    // The added same-name record/scalar scope keeps that bound through hygiene.
    assert!(functions <= 51, "{functions} functions");
    assert_native_parity(COMPOSED, true);
}

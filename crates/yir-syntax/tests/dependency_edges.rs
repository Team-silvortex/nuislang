use yir_syntax::parse_module;

#[test]
fn does_not_infer_dependency_against_explicit_result_edge() {
    let module = parse_module(
        r#"
resource cpu0 cpu.arm64

cpu.const_i64 seed cpu0 1
cpu.add producer cpu0 seed result
cpu.field result cpu0 producer value
edge dep producer result
"#,
    )
    .unwrap();

    assert!(module
        .edges
        .iter()
        .any(|edge| edge.from == "producer" && edge.to == "result"));
    assert!(!module
        .edges
        .iter()
        .any(|edge| edge.from == "result" && edge.to == "producer"));
}

#[test]
fn still_infers_mutual_argument_dependencies_without_explicit_edges() {
    let module = parse_module(
        r#"
resource cpu0 cpu.arm64

cpu.add left cpu0 right right
cpu.add right cpu0 left left
"#,
    )
    .unwrap();

    assert!(module
        .edges
        .iter()
        .any(|edge| edge.from == "right" && edge.to == "left"));
    assert!(module
        .edges
        .iter()
        .any(|edge| edge.from == "left" && edge.to == "right"));
}

use super::*;
use yir_core::{Node, Operation};

fn node(name: &str, instruction: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            instruction,
            args.iter().map(|value| (*value).to_owned()).collect(),
        )
        .unwrap(),
    }
}

#[test]
fn dynamic_counts_retain_profile_value_provenance() {
    let nodes = [
        node("profile.vertices", "cpu.const", &["4"]),
        node("profile.instances", "cpu.const", &["1"]),
        node("delta", "cpu.const", &["1"]),
        node("vertices", "cpu.sub", &["profile.vertices", "delta"]),
        node("instances", "cpu.add", &["profile.instances", "delta"]),
        node(
            "draw",
            "shader.draw_instanced",
            &["pass", "packet", "vertices", "instances"],
        ),
    ];
    let map = nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect();
    assert!(draw_count_uses_profile(
        &map,
        &nodes[5],
        2,
        "profile.vertices"
    ));
    assert!(draw_count_uses_profile(
        &map,
        &nodes[5],
        3,
        "profile.instances"
    ));
    assert!(!draw_count_uses_profile(
        &map,
        &nodes[5],
        2,
        "profile.instances"
    ));
}

#[test]
fn unrelated_profile_operands_and_cycles_do_not_authorize_draw_counts() {
    let nodes = [
        node("profile", "cpu.const", &["4"]),
        node("constant", "cpu.const", &["3"]),
        node("cycle", "cpu.add", &["cycle", "constant"]),
        node(
            "draw",
            "shader.draw_instanced",
            &["profile", "profile", "constant", "cycle"],
        ),
    ];
    let map = nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect();
    assert!(!draw_count_uses_profile(&map, &nodes[3], 2, "profile"));
    assert!(!draw_count_uses_profile(&map, &nodes[3], 3, "profile"));
    assert!(!draw_count_uses_profile(&map, &nodes[3], 4, "profile"));
}

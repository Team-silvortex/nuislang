use super::*;
use crate::graph::topological_order;

#[test]
fn topological_order_keeps_descending_ready_node_priority() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: Vec::new(),
        functions: Vec::new(),
        nodes: vec![
            node("a", "cpu0", "cpu.const", &["1"]),
            node("b", "cpu0", "cpu.const", &["2"]),
            node("z", "cpu0", "cpu.identity", &["b"]),
        ],
        edges: vec![dep("b", "z")],
        node_lanes: BTreeMap::new(),
    };

    assert_eq!(topological_order(&module).unwrap(), ["b", "z", "a"]);
}

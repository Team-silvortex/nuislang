use super::*;
use crate::verify_glm_protocol;

fn glm_module(nodes: Vec<Node>, edges: Vec<Edge>) -> YirModule {
    YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        functions: Vec::new(),
        nodes,
        edges,
        node_lanes: BTreeMap::new(),
    }
}

#[test]
fn glm_reports_the_first_missing_dependency_edge() {
    let module = glm_module(
        vec![
            node("value", "cpu0", "cpu.const", &["1"]),
            node("consume", "cpu0", "cpu.identity", &["value"]),
        ],
        Vec::new(),
    );

    assert_eq!(
        verify_glm_protocol(&module).unwrap_err(),
        "GLM: node `consume` uses `value` as val Read without dep/xfer edge"
    );
}

#[test]
fn glm_rejects_an_unordered_consumer_before_owned_consume() {
    let module = glm_module(
        vec![
            node("len", "cpu0", "cpu.const", &["1"]),
            node("fill", "cpu0", "cpu.const", &["0"]),
            node("buffer", "cpu0", "cpu.alloc_buffer", &["len", "fill"]),
            node("drop_buffer", "cpu0", "cpu.free", &["buffer"]),
            node("read_buffer", "cpu0", "cpu.buffer_len", &["buffer"]),
        ],
        vec![
            dep("len", "buffer"),
            dep("fill", "buffer"),
            dep("buffer", "drop_buffer"),
            lifetime("buffer", "drop_buffer"),
            dep("buffer", "read_buffer"),
        ],
    );

    assert_eq!(
        verify_glm_protocol(&module).unwrap_err(),
        "GLM: node `drop_buffer` consumes res `buffer` with Own, but `read_buffer` is not ordered before that consume"
    );
}

#[test]
fn glm_accepts_a_consumer_ordered_before_owned_consume() {
    let module = glm_module(
        vec![
            node("len", "cpu0", "cpu.const", &["1"]),
            node("fill", "cpu0", "cpu.const", &["0"]),
            node("buffer", "cpu0", "cpu.alloc_buffer", &["len", "fill"]),
            node("read_buffer", "cpu0", "cpu.buffer_len", &["buffer"]),
            node("drop_buffer", "cpu0", "cpu.free", &["buffer"]),
        ],
        vec![
            dep("len", "buffer"),
            dep("fill", "buffer"),
            dep("buffer", "read_buffer"),
            dep("buffer", "drop_buffer"),
            effect("read_buffer", "drop_buffer"),
            lifetime("buffer", "drop_buffer"),
        ],
    );

    verify_glm_protocol(&module).unwrap();
}

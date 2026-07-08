use super::*;

#[test]
fn owner_write_after_last_borrow_use_is_allowed() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("nil", "cpu0", "cpu.null", &[]),
            node("v1", "cpu0", "cpu.const", &["10"]),
            node("v2", "cpu0", "cpu.const", &["99"]),
            node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "nil"]),
            node("head", "cpu0", "cpu.move_ptr", &["head_raw"]),
            node("head_ref", "cpu0", "cpu.borrow", &["head"]),
            node("read_head", "cpu0", "cpu.load_value", &["head_ref"]),
            node("write_head", "cpu0", "cpu.store_value", &["head", "v2"]),
        ],
        edges: vec![
            dep("v1", "head_raw"),
            dep("nil", "head_raw"),
            dep("head_raw", "head"),
            lifetime("head_raw", "head"),
            dep("head", "head_ref"),
            dep("head_ref", "read_head"),
            effect("head_ref", "read_head"),
            dep("head", "write_head"),
            dep("v2", "write_head"),
            effect("read_head", "write_head"),
            lifetime("head", "write_head"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn owner_free_after_last_borrow_use_is_allowed() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("len", "cpu0", "cpu.const", &["4"]),
            node("fill", "cpu0", "cpu.const", &["7"]),
            node("idx1", "cpu0", "cpu.const", &["1"]),
            node("buf_raw", "cpu0", "cpu.alloc_buffer", &["len", "fill"]),
            node("buf", "cpu0", "cpu.move_ptr", &["buf_raw"]),
            node("buf_ref", "cpu0", "cpu.borrow", &["buf"]),
            node("read_slot", "cpu0", "cpu.load_at", &["buf_ref", "idx1"]),
            node("drop_buf", "cpu0", "cpu.free", &["buf"]),
        ],
        edges: vec![
            dep("len", "buf_raw"),
            dep("fill", "buf_raw"),
            dep("buf_raw", "buf"),
            lifetime("buf_raw", "buf"),
            dep("buf", "buf_ref"),
            dep("buf_ref", "read_slot"),
            dep("idx1", "read_slot"),
            effect("buf_ref", "read_slot"),
            dep("buf", "drop_buf"),
            effect("read_slot", "drop_buf"),
            lifetime("buf", "drop_buf"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn explicit_borrow_end_allows_owner_write() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("nil", "cpu0", "cpu.null", &[]),
            node("v1", "cpu0", "cpu.const", &["10"]),
            node("v2", "cpu0", "cpu.const", &["99"]),
            node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "nil"]),
            node("head", "cpu0", "cpu.move_ptr", &["head_raw"]),
            node("head_ref", "cpu0", "cpu.borrow", &["head"]),
            node("end_ref", "cpu0", "cpu.borrow_end", &["head_ref"]),
            node("write_head", "cpu0", "cpu.store_value", &["head", "v2"]),
        ],
        edges: vec![
            dep("v1", "head_raw"),
            dep("nil", "head_raw"),
            dep("head_raw", "head"),
            lifetime("head_raw", "head"),
            dep("head", "head_ref"),
            dep("head_ref", "end_ref"),
            effect("head_ref", "end_ref"),
            dep("head", "write_head"),
            dep("v2", "write_head"),
            effect("end_ref", "write_head"),
            lifetime("head", "write_head"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn alloc_node_with_borrowed_next_is_rejected() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("nil", "cpu0", "cpu.null", &[]),
            node("v1", "cpu0", "cpu.const", &["10"]),
            node("v2", "cpu0", "cpu.const", &["20"]),
            node("tail_raw", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
            node("tail", "cpu0", "cpu.move_ptr", &["tail_raw"]),
            node("tail_ref", "cpu0", "cpu.borrow", &["tail"]),
            node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "tail_ref"]),
        ],
        edges: vec![
            dep("v2", "tail_raw"),
            dep("nil", "tail_raw"),
            dep("tail_raw", "tail"),
            lifetime("tail_raw", "tail"),
            dep("tail", "tail_ref"),
            dep("v1", "head_raw"),
            dep("tail_ref", "head_raw"),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("cannot capture borrowed pointer"));
}

#[test]
fn store_next_with_borrowed_pointer_is_rejected() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("nil", "cpu0", "cpu.null", &[]),
            node("v1", "cpu0", "cpu.const", &["10"]),
            node("v2", "cpu0", "cpu.const", &["20"]),
            node("tail_raw", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
            node("tail", "cpu0", "cpu.move_ptr", &["tail_raw"]),
            node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "nil"]),
            node("head", "cpu0", "cpu.move_ptr", &["head_raw"]),
            node("tail_ref", "cpu0", "cpu.borrow", &["tail"]),
            node("link_tail", "cpu0", "cpu.store_next", &["head", "tail_ref"]),
        ],
        edges: vec![
            dep("v2", "tail_raw"),
            dep("nil", "tail_raw"),
            dep("tail_raw", "tail"),
            lifetime("tail_raw", "tail"),
            dep("v1", "head_raw"),
            dep("nil", "head_raw"),
            dep("head_raw", "head"),
            lifetime("head_raw", "head"),
            dep("tail", "tail_ref"),
            dep("head", "link_tail"),
            dep("tail_ref", "link_tail"),
            lifetime("head", "link_tail"),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("cannot write borrowed pointer"));
}

#[test]
fn freeing_live_link_target_is_rejected() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("nil", "cpu0", "cpu.null", &[]),
            node("v2", "cpu0", "cpu.const", &["20"]),
            node("v1", "cpu0", "cpu.const", &["10"]),
            node("tail", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
            node("head", "cpu0", "cpu.alloc_node", &["v1", "tail"]),
            node("drop_tail", "cpu0", "cpu.free", &["tail"]),
        ],
        edges: vec![
            dep("v2", "tail"),
            dep("nil", "tail"),
            dep("v1", "head"),
            dep("tail", "head"),
            dep("tail", "drop_tail"),
            effect("head", "drop_tail"),
            lifetime("tail", "drop_tail"),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("still links to it"));
}

#[test]
fn freeing_detached_link_target_is_allowed() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("nil", "cpu0", "cpu.null", &[]),
            node("v2", "cpu0", "cpu.const", &["20"]),
            node("v1", "cpu0", "cpu.const", &["10"]),
            node("tail", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
            node("head", "cpu0", "cpu.alloc_node", &["v1", "tail"]),
            node("detach_tail", "cpu0", "cpu.store_next", &["head", "nil"]),
            node("drop_tail", "cpu0", "cpu.free", &["tail"]),
        ],
        edges: vec![
            dep("v2", "tail"),
            dep("nil", "tail"),
            dep("v1", "head"),
            dep("tail", "head"),
            dep("head", "detach_tail"),
            dep("nil", "detach_tail"),
            effect("head", "detach_tail"),
            lifetime("head", "detach_tail"),
            dep("tail", "drop_tail"),
            effect("detach_tail", "drop_tail"),
            lifetime("tail", "drop_tail"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

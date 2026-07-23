use crate::{
    DataFabricPrimitive, DataMod, DataWindow, ExecutionState, Node, Operation, RegisteredMod,
    Resource, ResourceKind, Value,
};

#[test]
fn freeing_live_link_target_is_rejected_in_execution_state() {
    let mut state = ExecutionState::default();
    let tail = state.alloc_heap_node(20, None);
    let _head = state.alloc_heap_node(10, Some(tail));

    let error = state.free_heap_node(Some(tail)).unwrap_err();
    assert!(error.contains("still links to it"));
}

#[test]
fn data_mod_rejects_nested_window_payloads() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let data_mod = DataMod;
    let mut state = ExecutionState::default();

    state.values.insert("base".to_owned(), Value::Int(7));
    let first = data_mod
        .execute(
            &Node {
                name: "window0".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.immutable_window",
                    vec!["base".to_owned(), "0".to_owned(), "1".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();
    state.values.insert("window0".to_owned(), first);

    let error = data_mod
        .execute(
            &Node {
                name: "window1".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.copy_window",
                    vec!["window0".to_owned(), "0".to_owned(), "1".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap_err();
    assert!(error.contains("cannot wrap non-window-compatible payload"));
}

#[test]
fn data_mod_rejects_mutable_window_payloads_for_output_pipe() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let data_mod = DataMod;
    let mut state = ExecutionState::default();

    state.values.insert(
        "window0".to_owned(),
        Value::DataWindow(DataWindow {
            base: Box::new(Value::Int(7)),
            offset: 0,
            len: 1,
            immutable: false,
        }),
    );

    let error = data_mod
        .execute(
            &Node {
                name: "pipe".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse("data.output_pipe", vec!["window0".to_owned()]).unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap_err();
    assert!(error.contains("illegal pipe payload"));
}

#[test]
fn data_mod_freeze_window_converts_mutable_window_to_immutable() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let data_mod = DataMod;
    let mut state = ExecutionState::default();

    state.values.insert(
        "window0".to_owned(),
        Value::DataWindow(DataWindow {
            base: Box::new(Value::Int(7)),
            offset: 0,
            len: 1,
            immutable: false,
        }),
    );

    let value = data_mod
        .execute(
            &Node {
                name: "frozen".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse("data.freeze_window", vec!["window0".to_owned()]).unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();

    let Value::DataWindow(window) = value else {
        panic!("expected frozen data window");
    };
    assert!(window.immutable);
}

#[test]
fn data_mod_write_window_updates_single_slot_mutable_window() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let data_mod = DataMod;
    let mut state = ExecutionState::default();

    state.values.insert(
        "window0".to_owned(),
        Value::DataWindow(DataWindow {
            base: Box::new(Value::Int(7)),
            offset: 0,
            len: 1,
            immutable: false,
        }),
    );
    state.values.insert("value0".to_owned(), Value::Int(11));

    let value = data_mod
        .execute(
            &Node {
                name: "updated".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.write_window",
                    vec!["window0".to_owned(), "0".to_owned(), "value0".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();

    let Value::DataWindow(window) = value else {
        panic!("expected mutable data window");
    };
    assert!(!window.immutable);
    assert_eq!(*window.base, Value::Int(11));
}

#[test]
fn data_mod_write_window_updates_buffer_backed_window_storage() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let data_mod = DataMod;
    let mut state = ExecutionState::default();
    let buffer = state.alloc_heap_buffer(4, 0);

    state
        .values
        .insert("buffer0".to_owned(), Value::Pointer(Some(buffer)));
    state.values.insert("value0".to_owned(), Value::Int(33));

    let window = data_mod
        .execute(
            &Node {
                name: "window0".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.copy_window",
                    vec!["buffer0".to_owned(), "1".to_owned(), "2".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();
    state.values.insert("window0".to_owned(), window);

    let updated = data_mod
        .execute(
            &Node {
                name: "updated".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.write_window",
                    vec!["window0".to_owned(), "0".to_owned(), "value0".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();

    let Value::DataWindow(window) = updated else {
        panic!("expected buffer-backed data window");
    };
    assert_eq!(state.read_heap_buffer_at(Some(buffer), 1).unwrap(), 33);
    assert_eq!(window.offset, 1);
    assert_eq!(window.len, 2);
}

#[test]
fn data_mod_read_window_reads_buffer_backed_window_storage() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let data_mod = DataMod;
    let mut state = ExecutionState::default();
    let buffer = state.alloc_heap_buffer(4, 0);
    state.write_heap_buffer_at(Some(buffer), 2, 55).unwrap();
    state
        .values
        .insert("buffer0".to_owned(), Value::Pointer(Some(buffer)));

    let window = data_mod
        .execute(
            &Node {
                name: "window0".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.copy_window",
                    vec!["buffer0".to_owned(), "1".to_owned(), "2".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();
    state.values.insert("window0".to_owned(), window);

    let value = data_mod
        .execute(
            &Node {
                name: "value".to_owned(),
                resource: "fabric0".to_owned(),
                op: Operation::parse(
                    "data.read_window",
                    vec!["window0".to_owned(), "1".to_owned()],
                )
                .unwrap(),
            },
            &resource,
            &mut state,
        )
        .unwrap();

    assert_eq!(value, Value::Int(55));
}

#[test]
fn classifies_data_fabric_primitives_into_eight_families() {
    let cases = [
        ("data.bind_core", Some(DataFabricPrimitive::Bind)),
        ("data.handle_table", Some(DataFabricPrimitive::Handle)),
        (
            "data.provider_request_ingress",
            Some(DataFabricPrimitive::Ingress),
        ),
        ("data.marker", Some(DataFabricPrimitive::Marker)),
        ("data.move", Some(DataFabricPrimitive::Move)),
        ("data.copy_window", Some(DataFabricPrimitive::Window)),
        ("data.read_window", Some(DataFabricPrimitive::Window)),
        ("data.write_window", Some(DataFabricPrimitive::Window)),
        ("data.freeze_window", Some(DataFabricPrimitive::Window)),
        ("data.output_pipe", Some(DataFabricPrimitive::Pipe)),
        ("data.input_pipe", Some(DataFabricPrimitive::Pipe)),
        ("data.observe", Some(DataFabricPrimitive::Observe)),
        ("data.is_ready", Some(DataFabricPrimitive::Observe)),
        ("data.value", Some(DataFabricPrimitive::Observe)),
        ("cpu.const", None),
    ];

    for (op, expected) in cases {
        let parsed = Operation::parse(op, Vec::new()).unwrap();
        assert_eq!(parsed.data_fabric_primitive(), expected, "op={op}");
    }
}

#[test]
fn data_mod_imports_provider_request_as_opaque_handle() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let node = Node {
        name: "request".to_owned(),
        resource: "fabric0".to_owned(),
        op: Operation::parse(
            "data.provider_request_ingress",
            vec![
                "request_handle",
                "descriptor_table",
                "count",
                "provider",
                "capability",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let data_mod = DataMod;
    let semantics = data_mod
        .describe(&node, &resource)
        .expect("describe ingress");
    assert_eq!(semantics.dependencies.len(), 5);
    assert!(semantics.has_effect);

    let mut state = ExecutionState::default();
    for (name, value) in [
        ("request_handle", 101),
        ("descriptor_table", 501),
        ("count", 2),
        ("provider", 20),
        ("capability", 2020),
    ] {
        state.values.insert(name.to_owned(), Value::Int(value));
    }
    let value = data_mod
        .execute(&node, &resource, &mut state)
        .expect("execute ingress");
    assert_eq!(value, Value::Int(101));
}

#[test]
fn data_mod_imports_capsule_request_with_eight_dependencies() {
    let resource = Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    };
    let names = [
        "request_handle",
        "descriptor_table",
        "count",
        "provider",
        "capability",
        "capsule",
        "input_roles",
        "output_roles",
    ];
    let node = Node {
        name: "capsule_request".to_owned(),
        resource: "fabric0".to_owned(),
        op: Operation::parse(
            "data.provider_request_ingress",
            names.into_iter().map(str::to_owned).collect(),
        )
        .unwrap(),
    };
    let data_mod = DataMod;
    let semantics = data_mod
        .describe(&node, &resource)
        .expect("describe capsule ingress");
    assert_eq!(semantics.dependencies.len(), 8);
    assert!(semantics.has_effect);

    let mut state = ExecutionState::default();
    for (index, name) in names.into_iter().enumerate() {
        state
            .values
            .insert(name.to_owned(), Value::Int(index as i64 + 1));
    }
    assert_eq!(
        data_mod.execute(&node, &resource, &mut state).unwrap(),
        Value::Int(1)
    );
    assert!(state
        .events
        .iter()
        .any(|event| event.contains("capsule capsule")));
}

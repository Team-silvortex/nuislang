use super::*;
use yir_core::{
    DataWindow, IndexBuffer, Operation, RegisteredMod, RenderPass, RenderPipeline, ResourceKind,
    ShaderBinding, ShaderBindingSet, SurfaceTarget, VertexBuffer, VertexLayout, Viewport,
};

fn fixture() -> (Node, Resource, ExecutionState) {
    let node = Node {
        name: "draw".to_owned(),
        resource: "gpu".to_owned(),
        op: Operation::parse(
            "shader.draw_instanced",
            ["pass", "packet", "4", "1"].map(str::to_owned).to_vec(),
        )
        .unwrap(),
    };
    let resource = Resource {
        name: "gpu".to_owned(),
        kind: ResourceKind::parse("shader.reference"),
    };
    let mut state = ExecutionState::default();
    state.bind_value(
        "pass",
        Value::RenderPass(RenderPass {
            target: SurfaceTarget {
                format: "rgba8_unorm".to_owned(),
                width: 16,
                height: 8,
            },
            pipeline: RenderPipeline {
                shading_model: "ball".to_owned(),
                topology: "triangle_strip".to_owned(),
            },
            viewport: Viewport {
                width: 8,
                height: 8,
            },
            shader_module: None,
        }),
    );
    state.bind_value("packet", Value::Tuple(vec![Value::Int(1), Value::Int(2)]));
    (node, resource, state)
}

fn pass(state: &mut ExecutionState) -> &mut RenderPass {
    let Value::RenderPass(pass) = state.values.get_mut("pass").unwrap() else {
        unreachable!()
    };
    pass
}

fn assert_shared_rejection(node: &Node, resource: &Resource, state: &mut ExecutionState) {
    let error = ShaderMod
        .validate_draw_instanced(node, resource, state)
        .unwrap_err();
    assert_eq!(ShaderMod.execute(node, resource, state).unwrap_err(), error);
    assert!(state.events.is_empty());
    assert!(state.lane_events.is_empty());
}

#[test]
fn descriptor_uses_pass_extent_without_reference_rasterization_or_effects() {
    let (mut node, resource, mut state) = fixture();
    node.op.args[2] = "vertices".to_owned();
    node.op.args[3] = "instances".to_owned();
    state.bind_value("vertices", Value::I32(4));
    state.bind_value("instances", Value::Bool(true));
    let packet = state.values.remove("packet").unwrap();
    state.bind_value(
        "packet",
        Value::DataWindow(DataWindow {
            base: Box::new(packet),
            offset: 0,
            len: 2,
            immutable: true,
        }),
    );
    pass(&mut state).viewport = Viewport {
        width: 1,
        height: 2,
    };
    pass(&mut state).pipeline.shading_model = "control_panel".to_owned();
    let descriptor = ShaderMod
        .validate_draw_instanced(&node, &resource, &state)
        .unwrap();
    assert_eq!((descriptor.width(), descriptor.height()), (1, 2));
    assert_eq!(descriptor.rgba8_byte_length(), 8);
    assert_eq!(
        ShaderDrawArguments::from_dispatch(&descriptor.provider_arguments().unwrap()).unwrap(),
        ShaderDrawArguments {
            width: 1,
            height: 2,
            vertex_count: 4,
            instance_count: 1,
        }
    );
    assert!(state.events.is_empty());
    assert!(!state
        .values
        .values()
        .any(|value| matches!(value, Value::Frame(_))));
}

#[test]
fn valid_reference_bindings_are_not_silently_ignored_by_provider_projection() {
    let (mut node, resource, mut state) = fixture();
    state.bind_value("bindings", Value::BindingSet(bindings()));
    node.op.args.push("bindings".to_owned());
    let descriptor = ShaderMod
        .validate_draw_instanced(&node, &resource, &state)
        .unwrap();
    assert!(descriptor
        .provider_arguments()
        .unwrap_err()
        .contains("resource bindings"));
    assert!(ShaderMod.execute(&node, &resource, &mut state).is_ok());
}

#[test]
fn shader_runtime_arguments_reject_unknown_missing_and_nonpositive_fields() {
    let draw = ShaderDrawArguments {
        width: 8,
        height: 8,
        vertex_count: 3,
        instance_count: 2,
    };
    assert_eq!(
        ShaderDrawArguments::from_dispatch(&draw.to_dispatch()).unwrap(),
        draw
    );
    for case in 0..5 {
        let mut arguments = draw.to_dispatch();
        match case {
            0 => arguments.contract = "unknown".to_owned(),
            1 => {
                arguments.scalars.remove("height");
            }
            2 => {
                arguments.scalars.insert("binding".to_owned(), 1);
            }
            3 => {
                arguments.scalars.insert("vertex_count".to_owned(), 0);
            }
            _ => {
                arguments.scalars.insert("width".to_owned(), u64::MAX);
            }
        }
        assert!(ShaderDrawArguments::from_dispatch(&arguments).is_err());
    }
}

#[test]
fn reference_path_retains_pixels_and_emits_one_event() {
    let (node, resource, mut state) = fixture();
    let expected = crate::sphere_render::draw_sphere_surface_with_size(
        state.expect_value("packet").unwrap(),
        8,
        8,
    )
    .unwrap();
    let value = ShaderMod.execute(&node, &resource, &mut state).unwrap();
    assert_eq!(value, Value::Frame(expected));
    assert_eq!(state.events.len(), 1);
    assert!(state.events[0]
        .starts_with("effect shader.draw_instanced @gpu [shader.reference]: frame[8x8]"));
}

#[test]
fn malformed_counts_values_and_node_shape_share_the_reference_validation() {
    for case in 0..8 {
        let (mut node, mut resource, mut state) = fixture();
        match case {
            0 => node.op.args[2] = "0".to_owned(),
            1 => node.op.args[3] = "-1".to_owned(),
            2 => node.op.args[2] = "missing".to_owned(),
            3 => node.op.args[2] = "packet".to_owned(),
            4 => state.bind_value("pass", Value::Int(1)),
            5 => state.bind_value("packet", Value::Unit),
            6 => node.op.args.clear(),
            _ => resource.kind = ResourceKind::parse("cpu.main"),
        }
        assert_shared_rejection(&node, &resource, &mut state);
    }
}

fn bindings() -> ShaderBindingSet {
    ShaderBindingSet {
        pipeline: RenderPipeline {
            shading_model: "ball".to_owned(),
            topology: "triangle_strip".to_owned(),
        },
        bindings: vec![
            ShaderBinding {
                kind: "vertex_layout_binding".to_owned(),
                slot: 0,
                value: Box::new(Value::VertexLayout(VertexLayout {
                    stride: 2,
                    attributes: vec!["pos2f".to_owned()],
                })),
            },
            ShaderBinding {
                kind: "vertex_binding".to_owned(),
                slot: 1,
                value: Box::new(Value::VertexBuffer(VertexBuffer {
                    vertex_count: 4,
                    elements: vec![-1, -1, 1, -1, -1, 1, 1, 1],
                })),
            },
        ],
    }
}

#[test]
fn binding_validation_borrows_inputs_and_checks_geometry_bounds() {
    let original = bindings();
    let borrowed = crate::geometry_overlay::resolve_geometry_inputs(&original).unwrap();
    let Value::VertexBuffer(buffer) = original.bindings[1].value.as_ref() else {
        unreachable!()
    };
    assert!(std::ptr::eq(borrowed.vertex_buffer, buffer));
    for case in 0..6 {
        let (mut node, resource, mut state) = fixture();
        let mut bindings = original.clone();
        match case {
            0 => bindings.bindings.clear(),
            1 => *bindings.bindings[0].value = Value::Unit,
            2 => {
                let Value::VertexBuffer(buffer) = bindings.bindings[1].value.as_mut() else {
                    unreachable!()
                };
                buffer.elements.pop();
            }
            3 => node.op.args[2] = "5".to_owned(),
            4 => bindings.bindings.push(ShaderBinding {
                kind: "index_binding".to_owned(),
                slot: 2,
                value: Box::new(Value::IndexBuffer(IndexBuffer { indices: vec![0] })),
            }),
            _ => {
                let Value::VertexLayout(layout) = bindings.bindings[0].value.as_mut() else {
                    unreachable!()
                };
                layout.stride = usize::MAX;
            }
        }
        state.bind_value("bindings", Value::BindingSet(bindings));
        node.op.args.push("bindings".to_owned());
        assert_shared_rejection(&node, &resource, &mut state);
    }
}

#[test]
fn empty_and_overflowing_extents_fail_before_any_rasterization() {
    for (width, height) in [(0, 8), (8, 0), (usize::MAX, 2)] {
        let (node, resource, mut state) = fixture();
        let pass = pass(&mut state);
        pass.target.width = width;
        pass.target.height = height;
        pass.viewport = Viewport { width, height };
        assert_shared_rejection(&node, &resource, &mut state);
    }
}

#[test]
fn typed_uniform_is_an_owned_f32_snapshot_without_reference_rasterization() {
    let (mut node, resource, mut state) = fixture();
    let binding = ShaderBinding {
        kind: "uniform_binding".to_owned(),
        slot: 2,
        value: Box::new(Value::Tuple(vec![Value::F32(1.0); 4])),
    };
    let pipeline = pass(&mut state).pipeline.clone();
    state.bind_value(
        "bindings",
        Value::BindingSet(ShaderBindingSet {
            pipeline,
            bindings: vec![binding],
        }),
    );
    node.op.args.push("bindings".to_owned());
    let arguments = ShaderMod
        .validate_draw_instanced(&node, &resource, &state)
        .unwrap()
        .provider_arguments()
        .unwrap();
    let uniform = crate::ShaderFragmentUniform::from_dispatch(&arguments)
        .unwrap()
        .unwrap();
    assert_eq!(uniform.slot, 2);
    assert_eq!(
        uniform.bytes,
        [1.0f32; 4]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>()
            .as_slice()
    );
    let Value::BindingSet(bindings) = state.values.get_mut("bindings").unwrap() else {
        unreachable!()
    };
    *bindings.bindings[0].value = Value::Unit;
    assert!(ShaderMod
        .validate_draw_instanced(&node, &resource, &state)
        .is_err());
    assert_eq!(
        crate::ShaderFragmentUniform::from_dispatch(&arguments).unwrap(),
        Some(uniform)
    );
    assert!(state.events.is_empty());
}

#[test]
fn uniform_shape_type_slot_and_pipeline_fail_before_draw() {
    for case in 0..6 {
        let (mut node, resource, mut state) = fixture();
        let mut bindings = ShaderBindingSet {
            pipeline: pass(&mut state).pipeline.clone(),
            bindings: vec![ShaderBinding {
                kind: "uniform_binding".to_owned(),
                slot: 2,
                value: Box::new(Value::Tuple(vec![Value::F32(1.0); 4])),
            }],
        };
        match case {
            0 => bindings.bindings[0].slot = 31,
            1 => *bindings.bindings[0].value = Value::Tuple(vec![Value::F32(1.0); 3]),
            2 => *bindings.bindings[0].value = Value::Tuple(vec![Value::Int(1); 4]),
            3 => *bindings.bindings[0].value = Value::Tuple(vec![Value::F32(f32::NAN); 4]),
            4 => bindings.bindings.push(bindings.bindings[0].clone()),
            _ => bindings.pipeline.topology = "other".to_owned(),
        }
        state.bind_value("bindings", Value::BindingSet(bindings));
        node.op.args.push("bindings".to_owned());
        assert_shared_rejection(&node, &resource, &mut state);
    }
}

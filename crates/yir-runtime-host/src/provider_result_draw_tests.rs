use super::*;
use yir_core::{
    provider_runtime_ipc::{DispatchFrame, DispatchTarget},
    Operation, RenderPass, RenderPipeline, ResourceKind, SurfaceTarget, Viewport,
};

fn fixture() -> (ProviderResultShaderMod, Node, Resource, ExecutionState) {
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
        kind: ResourceKind::parse("shader.test"),
    };
    let mut state = ExecutionState::default();
    state.bind_value(
        "pass",
        Value::RenderPass(RenderPass {
            target: SurfaceTarget {
                format: "rgba8_unorm".to_owned(),
                width: 1,
                height: 1,
            },
            pipeline: RenderPipeline {
                shading_model: "ball".to_owned(),
                topology: "triangle_strip".to_owned(),
            },
            viewport: Viewport {
                width: 1,
                height: 1,
            },
            shader_module: None,
        }),
    );
    state.bind_value("packet", Value::Tuple(vec![Value::Int(1), Value::Int(2)]));
    let target = DispatchTarget {
        source_yir_fnv1a64: fnv1a64_hex(b"test"),
        module: "shader".to_owned(),
        instruction: "draw_instanced".to_owned(),
        node: node.name.clone(),
        resource: resource.name.clone(),
    };
    let frame = ProviderResultFrame::from_ipc(
        &target,
        DispatchFrame {
            sequence: 0,
            arguments: yir_domain_shader::ShaderDrawArguments {
                width: 1,
                height: 1,
                vertex_count: 4,
                instance_count: 1,
            }
            .to_dispatch(),
            request_id: "render".to_owned(),
            provider_family: "test:device".to_owned(),
            element_type: "u8".to_owned(),
            layout: "image-2d-row-major:pixel-format=rgba8".to_owned(),
            shape: vec![1, 1],
            row_stride_bytes: 4,
            payload: vec![1, 2, 3, 255],
            completion_wire: ProviderPhysicalCompletion::new(
                "shader.clock.frame.v1",
                "test.clock",
                "test.fence",
                1,
            )
            .unwrap()
            .to_wire(),
        },
    )
    .unwrap();
    let source = ProviderResultSource::Replay(ProviderResultQueue::new(vec![frame]).unwrap());
    (
        ProviderResultShaderMod {
            state: Arc::new(Mutex::new(source)),
        },
        node,
        resource,
        state,
    )
}

fn remaining(adapter: &ProviderResultShaderMod) -> usize {
    let source = adapter.state.lock().unwrap();
    let ProviderResultSource::Replay(queue) = &*source else {
        unreachable!()
    };
    queue.frames.len()
}

#[test]
fn provider_frame_uses_pass_extent_not_ascii_preview_minimum_and_records_actual_pixels() {
    for model in ["ball", "control_panel", "custom"] {
        let (adapter, node, resource, mut state) = fixture();
        let Value::RenderPass(pass) = state.values.get_mut("pass").unwrap() else {
            unreachable!()
        };
        pass.pipeline.shading_model = model.to_owned();
        state.begin_registered_provider_completion(&node).unwrap();
        let value = adapter.execute(&node, &resource, &mut state).unwrap();
        let Value::Frame(frame) = value else {
            panic!("expected provider frame")
        };
        assert_eq!((frame.width, frame.height), (1, 1));
        assert!(frame.rows.is_empty());
        assert_eq!(frame.rgba8, Some(vec![1, 2, 3, 255]));
        assert_eq!(remaining(&adapter), 0);
        assert_eq!(
            state.events,
            ["effect shader.draw_instanced @gpu [shader.test]: frame[1x1; rgba8_bytes=4]"]
        );
        state
            .finish_registered_provider_completion(
                adapter.provider_completion_registration(&node).unwrap(),
                &node,
            )
            .unwrap();
    }
}

#[test]
fn invalid_request_fails_before_consuming_provider_result_or_emitting_draw() {
    for case in 0..4 {
        let (adapter, mut node, resource, mut state) = fixture();
        match case {
            0 => node.op.args[2] = "0".to_owned(),
            1 => state.bind_value("packet", Value::Unit),
            2 => node.op.args.push("packet".to_owned()),
            _ => node.op.args.clear(),
        }
        assert!(adapter.execute(&node, &resource, &mut state).is_err());
        assert_eq!(remaining(&adapter), 1);
        assert!(state.events.is_empty());
        assert!(state.lane_events.is_empty());
    }
}

#[test]
fn returned_dimension_drift_does_not_stage_completion_or_emit_draw() {
    let (adapter, node, resource, mut state) = fixture();
    let Value::RenderPass(pass) = state.values.get_mut("pass").unwrap() else {
        unreachable!()
    };
    pass.target.width = 2;
    pass.viewport.width = 2;
    state.begin_registered_provider_completion(&node).unwrap();
    assert!(adapter
        .execute(&node, &resource, &mut state)
        .unwrap_err()
        .contains("arguments mismatch"));
    assert_eq!(remaining(&adapter), 1);
    assert!(state.events.is_empty());
    assert!(state
        .finish_registered_provider_completion(
            adapter.provider_completion_registration(&node).unwrap(),
            &node,
        )
        .is_err());
}

#[test]
fn replay_rejects_changed_runtime_counts_before_consuming_frame() {
    for index in [2, 3] {
        let (adapter, mut node, resource, mut state) = fixture();
        node.op.args[index] = "3".to_owned();
        assert!(adapter
            .execute(&node, &resource, &mut state)
            .unwrap_err()
            .contains("arguments mismatch"));
        assert_eq!(remaining(&adapter), 1);
        assert!(state.events.is_empty());
    }
}

#[test]
fn unsupported_pass_projection_rejects_before_consuming_frame() {
    for case in 0..2 {
        let (adapter, node, resource, mut state) = fixture();
        let Value::RenderPass(pass) = state.values.get_mut("pass").unwrap() else {
            unreachable!()
        };
        if case == 0 {
            pass.target.format = "bgra8_unorm".to_owned();
        } else {
            pass.pipeline.topology = "triangle_list".to_owned();
        }
        assert!(adapter
            .execute(&node, &resource, &mut state)
            .unwrap_err()
            .contains("unsupported"));
        assert_eq!(remaining(&adapter), 1);
        assert!(state.events.is_empty());
    }
}

#[test]
fn unbound_draw_keeps_reference_execution_and_does_not_consume_provider_frame() {
    let (adapter, mut node, resource, mut state) = fixture();
    node.name = "reference.draw".to_owned();
    let Value::Frame(frame) = adapter.execute(&node, &resource, &mut state).unwrap() else {
        panic!("expected reference frame")
    };
    assert!(frame.rgba8.is_none());
    assert!(!frame.rows.is_empty());
    assert_eq!(remaining(&adapter), 1);
    assert_eq!(state.events.len(), 1);
    assert!(!state.events[0].contains("rgba8_bytes"));
}

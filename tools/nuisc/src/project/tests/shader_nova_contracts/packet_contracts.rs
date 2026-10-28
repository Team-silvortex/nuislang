use super::*;

pub(super) fn validates_shader_packet_contract_from_cpu_usage() {
    let surface_shader = standard_surface_shader_profile_module(false);
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                let packet: SurfaceShaderPacket =
                  shader_profile_packet("SurfaceShader", 1, 2, 3);
              }
            }
            "#,
        ),
        ("surface_shader.ns", surface_shader.as_str()),
    ]);

    validate_shader_packet_contract(&project, "SurfaceShader").unwrap();
}

pub(super) fn validates_nova_panel_contract_from_struct_literal_usage() {
    let surface_shader = standard_nova_surface_shader_profile_module();
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                let header: NovaHeaderPacket = nova_header_packet(8);
                let slider_color: NovaSliderPacket = nova_slider_packet(1);
                let slider_speed: NovaSliderPacket = nova_slider_packet(2);
                let slider_radius: NovaSliderPacket = nova_slider_packet(3);
                let sliders: NovaSliderGroupPacket =
                  nova_slider_group_packet(slider_color, slider_speed, slider_radius);
                let toggle: NovaTogglePacket = nova_toggle_packet(1);
                let progress: NovaProgressPacket = nova_progress_packet(2);
                let meter: NovaMeterPacket = nova_meter_packet(3);
                let button: NovaButtonPacket = nova_button_packet(1, 8);
                let text_input: NovaTextInputPacket = nova_text_input_packet(4, 1);
                let select: NovaSelectPacket = nova_select_packet(0, 8);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 8);
                let radio: NovaRadioPacket = nova_radio_packet(1, 4, 8);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(0, 4, 8);
                let list: NovaListPacket = nova_list_packet(1, 5, 8);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 8);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 8);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 8);
                let theme: NovaThemePacket = nova_theme_packet(8, 3, 1, 2);
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
                let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
                let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
                let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
                let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
                let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
                let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
                let scene_link: NovaSceneLinkPacket = nova_scene_link_packet(2, 12, 9, 8, 1, 1);
                let instance: NovaInstancePacket = nova_instance_packet(2, 3, 2, 1, 8, 1);
                let scene_graph: NovaSceneGraphPacket = nova_scene_graph_packet(2, 6, 3, 3, 1);
                let scene_node: NovaSceneNodePacket = nova_scene_node_packet(2, 4, 5, 3, 1);
                let instance_group: NovaInstanceGroupPacket = nova_instance_group_packet(3, 4, 3, 1, 8);
                let scene_cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(2, 6, 3, 8, 1);
                let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
                let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
                let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
                let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
                let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
                let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
                let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
                let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
                let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
                let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
                let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
                let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
                let frame_pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
                let frame_variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
                let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
                let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
                let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
                let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
                let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
                let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
                let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
                let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
                let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
                let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
                let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
                let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
                let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
                let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
                let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
                let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 1);
                let event: NovaEventPacket = nova_event_packet(1, 2, 3, 1);
                let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 1);
                let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 1);
                let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 1);
                let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 1);
                let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 1);
                let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 1);
                let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 1);
                let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 1);
                let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 1);
                let focus: NovaFocusPacket = nova_focus_packet(2);
                let panel: NovaPanelPacket = nova_panel_from_parts(
                  header,
                  sliders,
                  toggle,
                  progress,
                  meter,
                  button,
                  text_input,
                  select,
                  checkbox,
                  radio,
                  textarea,
                  tabs,
                  list,
                  table,
                  tree,
                  inspector,
                  outline,
                  theme,
                  surface,
                  viewport,
                  layer,
                  scene,
                  camera,
                  material,
                  light,
                  mesh,
                  transform,
                  node,
                  scene_link,
                  instance,
                  scene_graph,
                  scene_node,
                  instance_group,
                  scene_cluster,
                  visibility,
                  cull,
                  lod,
                  streaming,
                  residency,
                  eviction,
                  prefetch,
                  budget,
                  pressure,
                  thermal,
                  power,
                  latency,
                  frame_pacing,
                  frame_variance,
                  jank,
                  pass,
                  frame,
                  target,
                  frame_graph,
                  attachment,
                  pass_chain,
                  barrier,
                  resource_set,
                  schedule,
                  submission,
                  queue,
                  semaphore,
                  timeline,
                  fence,
                  signal,
                  event,
                  dispatch,
                  feedback,
                  intent,
                  reaction,
                  outcome,
                  resolution,
                  commit,
                  snapshot,
                  checkpoint,
                  focus
                );
                print(panel);
              }
            }
            "#,
        ),
        ("surface_shader.ns", surface_shader.as_str()),
    ]);

    validate_shader_packet_contract(&project, "SurfaceShader").unwrap();
}

pub(super) fn lowers_real_shader_packet_profile_project_with_expected_packet_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_packet_profile_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "color_slot",
        "speed_slot",
        "radius_slot",
        "packet_tag",
        "material_mode",
        "pass_kind",
        "packet_field_count",
        "panel_color",
        "panel_speed",
        "panel_radius",
        "render_controls",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
}

pub(super) fn lowers_real_shader_packet_bridge_project_with_expected_bridge_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_packet_bridge_demo");
    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    let binding = |name: &str| {
        main.body
            .iter()
            .find_map(|stmt| match stmt {
                NirStmt::Let {
                    name: candidate,
                    value,
                    ..
                } if candidate == name => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing binding {name}"))
    };
    assert!(matches!(
        binding("frame_packet"),
        NirExpr::DataProfileSendUplink { unit, input }
            if unit == "FabricPlane" && matches!(input.as_ref(), NirExpr::Var(name) if name == "render_controls")
    ));
    let NirExpr::ShaderResult { value, state, .. } = binding("frame_result") else {
        panic!("frame_result must wrap the direct profile render");
    };
    assert_eq!(
        *state,
        nuis_semantics::model::NirShaderFlowState::FrameReady
    );
    assert!(matches!(
        value.as_ref(),
        NirExpr::ShaderProfileRender { unit, packet }
            if unit == "SurfaceShader" && matches!(packet.as_ref(), NirExpr::Var(name) if name == "frame_packet")
    ));
    assert!(matches!(
        binding("frame_ready"),
        NirExpr::ShaderFrameReady(result)
            if matches!(result.as_ref(), NirExpr::Var(name) if name == "frame_result")
    ));
    let NirExpr::DataProfileSendDownlink { unit, input } = binding("host_frame") else {
        panic!("host_frame must be the fabric downlink");
    };
    assert_eq!(unit, "FabricPlane");
    assert!(matches!(
        input.as_ref(), NirExpr::ShaderValue(result)
            if matches!(result.as_ref(), NirExpr::Var(name) if name == "frame_result")
    ));
    let present = main
        .body
        .iter()
        .position(|stmt| {
            matches!(
                stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(frame))
                    if matches!(frame.as_ref(), NirExpr::Var(name) if name == "host_frame")
            )
        })
        .expect("present the downlinked frame");
    let mut order = ["frame_packet", "frame_result", "frame_ready", "host_frame"]
        .into_iter()
        .map(|name| {
            main.body
                .iter()
                .position(|stmt| {
                    matches!(
                        stmt, NirStmt::Let { name: candidate, .. } if candidate == name
                    )
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    order.push(present);
    assert!(
        order.windows(2).all(|pair| pair[0] < pair[1]),
        "preserve the upload/render/readiness/download/present order"
    );
}

pub(super) fn rejects_shader_packet_field_count_mismatch() {
    let surface_shader = standard_surface_shader_profile_module_with_packet_field_count(false, 4);
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                let packet: SurfaceShaderPacket =
                  shader_profile_packet("SurfaceShader", 1, 2, 3);
              }
            }
            "#,
        ),
        ("surface_shader.ns", surface_shader.as_str()),
    ]);

    let error = validate_shader_packet_contract(&project, "SurfaceShader").unwrap_err();
    assert!(error.contains("packet_field_count = 3"));
}

pub(super) fn nova_panel_packet_requires_nova_support_surface() {
    let contract = ShaderPacketContract {
        type_name: "NovaPanelPacket".to_owned(),
        field_count: 6,
    };
    assert_eq!(
        shader_packet_support_surface_contract(&contract),
        ["shader.profile.packet.nova.v1"]
    );
}

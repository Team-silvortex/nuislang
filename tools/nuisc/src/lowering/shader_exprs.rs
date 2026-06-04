use super::*;

pub(super) fn lower_shader_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => Some(
            lower_shader_profile_color_seed(unit, base, delta, state, bindings),
        ),
        NirExpr::ShaderProfileSpeedSeed {
            unit,
            delta,
            scale,
            base,
        } => Some(lower_shader_profile_speed_seed(
            unit, delta, scale, base, state, bindings,
        )),
        NirExpr::ShaderProfileRadiusSeed { unit, base, delta } => Some(
            lower_shader_profile_radius_seed(unit, base, delta, state, bindings),
        ),
        NirExpr::ShaderProfileRender { unit, packet } => {
            Some(lower_shader_profile_render(unit, packet, state, bindings))
        }
        NirExpr::ShaderProfileTargetRef { unit } => {
            Some(lower_project_profile_ref(state, "shader", unit, "target"))
        }
        NirExpr::ShaderProfileViewportRef { unit } => {
            Some(lower_project_profile_ref(state, "shader", unit, "viewport"))
        }
        NirExpr::ShaderProfilePipelineRef { unit } => {
            Some(lower_project_profile_ref(state, "shader", unit, "pipeline"))
        }
        NirExpr::ShaderProfileVertexCountRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "vertex_count",
        )),
        NirExpr::ShaderProfileInstanceCountRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "instance_count",
        )),
        NirExpr::ShaderProfilePacketColorSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_color_slot",
        )),
        NirExpr::ShaderProfilePacketSpeedSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_speed_slot",
        )),
        NirExpr::ShaderProfilePacketRadiusSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_radius_slot",
        )),
        NirExpr::ShaderProfilePacketTagRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_tag",
        )),
        NirExpr::ShaderProfileMaterialModeRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "material_mode",
        )),
        NirExpr::ShaderProfilePassKindRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "pass_kind",
        )),
        NirExpr::ShaderProfilePacketFieldCountRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_field_count",
        )),
        NirExpr::ShaderTarget {
            format,
            width,
            height,
        } => Some(Ok(lower_shader_target(format, *width, *height, state))),
        NirExpr::ShaderViewport { width, height } => {
            Some(Ok(lower_shader_viewport(*width, *height, state)))
        }
        NirExpr::ShaderPipeline {
            name: pipe_name,
            topology,
        } => Some(Ok(lower_shader_pipeline(pipe_name, topology, state))),
        NirExpr::ShaderInlineWgsl { entry, source } => {
            Some(Ok(lower_shader_inline_wgsl(entry, source, state)))
        }
        NirExpr::ShaderResult { value, state: flow } => Some(lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            value,
            "shader_result",
            flow.render(),
        )),
        NirExpr::ShaderPassReady(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_pass_ready",
            "is_pass_ready",
        )),
        NirExpr::ShaderFrameReady(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_frame_ready",
            "is_frame_ready",
        )),
        NirExpr::ShaderValue(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_value",
            "value",
        )),
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => Some(lower_shader_begin_pass(
            target, pipeline, viewport, state, bindings,
        )),
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => Some(lower_shader_draw_instanced(
            pass,
            packet,
            vertex_count,
            instance_count,
            state,
            bindings,
        )),
        _ => None,
    }
}

fn lower_shader_profile_color_seed(
    unit: &str,
    base: &NirExpr,
    delta: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(base.clone()),
                rhs: Box::new(delta.clone()),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePacketColorSlotRef {
                unit: unit.to_owned(),
            }),
        }),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::ShaderProfileMaterialModeRef {
                unit: unit.to_owned(),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePassKindRef {
                unit: unit.to_owned(),
            }),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

fn lower_shader_profile_speed_seed(
    unit: &str,
    delta: &NirExpr,
    scale: &NirExpr,
    base: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Mul,
                        lhs: Box::new(delta.clone()),
                        rhs: Box::new(scale.clone()),
                    }),
                    rhs: Box::new(base.clone()),
                }),
                rhs: Box::new(NirExpr::ShaderProfileInstanceCountRef {
                    unit: unit.to_owned(),
                }),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePacketSpeedSlotRef {
                unit: unit.to_owned(),
            }),
        }),
        rhs: Box::new(NirExpr::ShaderProfilePacketTagRef {
            unit: unit.to_owned(),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

fn lower_shader_profile_radius_seed(
    unit: &str,
    base: &NirExpr,
    delta: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(base.clone()),
                    rhs: Box::new(delta.clone()),
                }),
                rhs: Box::new(NirExpr::ShaderProfileVertexCountRef {
                    unit: unit.to_owned(),
                }),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePacketRadiusSlotRef {
                unit: unit.to_owned(),
            }),
        }),
        rhs: Box::new(NirExpr::ShaderProfilePacketFieldCountRef {
            unit: unit.to_owned(),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

fn lower_shader_profile_render(
    unit: &str,
    packet: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::ShaderDrawInstanced {
        pass: Box::new(NirExpr::ShaderBeginPass {
            target: Box::new(NirExpr::ShaderProfileTargetRef {
                unit: unit.to_owned(),
            }),
            pipeline: Box::new(NirExpr::ShaderProfilePipelineRef {
                unit: unit.to_owned(),
            }),
            viewport: Box::new(NirExpr::ShaderProfileViewportRef {
                unit: unit.to_owned(),
            }),
        }),
        packet: Box::new(packet.clone()),
        vertex_count: Box::new(NirExpr::ShaderProfileVertexCountRef {
            unit: unit.to_owned(),
        }),
        instance_count: Box::new(NirExpr::ShaderProfileInstanceCountRef {
            unit: unit.to_owned(),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

fn lower_shader_target(
    format: &str,
    width: i64,
    height: i64,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_target");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "target".to_owned(),
            args: vec![format.to_owned(), width.to_string(), height.to_string()],
        },
    });
    name
}

fn lower_shader_viewport(width: i64, height: i64, state: &mut LoweringState<'_>) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_viewport");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "viewport".to_owned(),
            args: vec![width.to_string(), height.to_string()],
        },
    });
    name
}

fn lower_shader_pipeline(pipe_name: &str, topology: &str, state: &mut LoweringState<'_>) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_pipeline");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "pipeline".to_owned(),
            args: vec![pipe_name.to_owned(), topology.to_owned()],
        },
    });
    name
}

fn lower_shader_inline_wgsl(entry: &str, source: &str, state: &mut LoweringState<'_>) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_inline_wgsl");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "inline_wgsl".to_owned(),
            args: vec![entry.to_owned(), source.to_owned()],
        },
    });
    name
}

fn lower_shader_begin_pass(
    target: &NirExpr,
    pipeline: &NirExpr,
    viewport: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let target_name = lower_expr(target, state, bindings)?;
    let pipeline_name = lower_expr(pipeline, state, bindings)?;
    let viewport_name = lower_expr(viewport, state, bindings)?;
    let name = next_name(state, "shader_begin_pass");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "begin_pass".to_owned(),
            args: vec![
                target_name.clone(),
                pipeline_name.clone(),
                viewport_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &target_name, &name);
    push_dep_edges(state, &pipeline_name, &name);
    push_dep_edges(state, &viewport_name, &name);
    Ok(name)
}

fn lower_shader_draw_instanced(
    pass: &NirExpr,
    packet: &NirExpr,
    vertex_count: &NirExpr,
    instance_count: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let pass_name = lower_expr(pass, state, bindings)?;
    let packet_name = lower_expr(packet, state, bindings)?;
    let vertex_count_name = lower_expr(vertex_count, state, bindings)?;
    let instance_count_name = lower_expr(instance_count, state, bindings)?;
    let name = next_name(state, "shader_draw_instanced");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "draw_instanced".to_owned(),
            args: vec![
                pass_name.clone(),
                packet_name.clone(),
                vertex_count_name.clone(),
                instance_count_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &pass_name, &name);
    push_xfer_edge(state, &packet_name, &name);
    push_xfer_edge(state, &vertex_count_name, &name);
    push_xfer_edge(state, &instance_count_name, &name);
    Ok(name)
}

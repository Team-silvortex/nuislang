use super::*;
use nuis_semantics::model::{NirShaderSampleMode, NirShaderSampleUvMode};

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
        NirExpr::ShaderProfileSliderColorSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "slider_color_slot",
        )),
        NirExpr::ShaderProfileSliderSpeedSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "slider_speed_slot",
        )),
        NirExpr::ShaderProfileSliderRadiusSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "slider_radius_slot",
        )),
        NirExpr::ShaderProfileHeaderAccentSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "header_accent_slot",
        )),
        NirExpr::ShaderProfileToggleLiveSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "toggle_live_slot",
        )),
        NirExpr::ShaderProfileFocusSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "focus_slot",
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
        NirExpr::ShaderTexture2d {
            format,
            width,
            height,
            texels,
        } => Some(Ok(lower_shader_texture2d(
            format, *width, *height, texels, state,
        ))),
        NirExpr::ShaderSampler {
            filter,
            address_mode,
        } => Some(Ok(lower_shader_sampler(filter, address_mode, state))),
        NirExpr::ShaderUv { u, v } => Some(Ok(lower_shader_uv(*u, *v, state))),
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            mode,
        } => Some(lower_shader_sample(texture, sampler, x, y, *mode, state, bindings)),
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            mode,
        } => Some(lower_shader_sample_uv(texture, sampler, uv, *mode, state, bindings)),
        NirExpr::ShaderBinding {
            kind,
            slot,
            layout,
            profile_contract,
            value,
        } => Some(lower_shader_binding(
            kind,
            *slot,
            layout.as_deref(),
            profile_contract.as_deref(),
            value,
            state,
            bindings,
        )),
        NirExpr::ShaderBindSet { pipeline, bindings: set_bindings } => {
            Some(lower_shader_bind_set(pipeline, set_bindings, state, bindings))
        }
        NirExpr::ShaderInlineWgsl { entry, source } => {
            Some(lower_shader_inline_wgsl(entry, source, state))
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

fn lower_shader_texture2d(
    format: &str,
    width: i64,
    height: i64,
    texels: &str,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_texture2d");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "texture2d".to_owned(),
            args: vec![
                format.to_owned(),
                width.to_string(),
                height.to_string(),
                texels.to_owned(),
            ],
        },
    });
    name
}

fn lower_shader_sampler(filter: &str, address_mode: &str, state: &mut LoweringState<'_>) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_sampler");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "sampler".to_owned(),
            args: vec![filter.to_owned(), address_mode.to_owned()],
        },
    });
    name
}

fn lower_shader_uv(u: i64, v: i64, state: &mut LoweringState<'_>) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_uv");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "uv".to_owned(),
            args: vec![u.to_string(), v.to_string()],
        },
    });
    name
}

fn lower_shader_inline_wgsl(
    entry: &str,
    source: &str,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_inline_wgsl");
    let normalized = crate::shader_source::normalize_inline_wgsl_source(source)?;
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "inline_wgsl".to_owned(),
            args: vec![entry.to_owned(), normalized],
        },
    });
    Ok(name)
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

fn lower_shader_sample(
    texture: &NirExpr,
    sampler: &NirExpr,
    x: &NirExpr,
    y: &NirExpr,
    mode: NirShaderSampleMode,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let texture_name = lower_expr(texture, state, bindings)?;
    let sampler_name = lower_expr(sampler, state, bindings)?;
    let x_name = lower_expr(x, state, bindings)?;
    let y_name = lower_expr(y, state, bindings)?;
    let name = next_name(state, "shader_sample");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: mode.render().to_owned(),
            args: vec![
                texture_name.clone(),
                sampler_name.clone(),
                x_name.clone(),
                y_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &texture_name, &name);
    push_dep_edges(state, &sampler_name, &name);
    push_dep_edges(state, &x_name, &name);
    push_dep_edges(state, &y_name, &name);
    Ok(name)
}

fn lower_shader_sample_uv(
    texture: &NirExpr,
    sampler: &NirExpr,
    uv: &NirExpr,
    mode: NirShaderSampleUvMode,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let texture_name = lower_expr(texture, state, bindings)?;
    let sampler_name = lower_expr(sampler, state, bindings)?;
    let uv_name = lower_expr(uv, state, bindings)?;
    let name = next_name(state, "shader_sample_uv");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: mode.render().to_owned(),
            args: vec![texture_name.clone(), sampler_name.clone(), uv_name.clone()],
        },
    });
    push_dep_edges(state, &texture_name, &name);
    push_dep_edges(state, &sampler_name, &name);
    push_dep_edges(state, &uv_name, &name);
    Ok(name)
}

fn lower_shader_binding(
    kind: &str,
    slot: i64,
    layout: Option<&str>,
    profile_contract: Option<&str>,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let value_name = lower_expr(value, state, bindings)?;
    let name = next_name(state, "shader_binding");
    let mut args = vec![slot.to_string()];
    if let Some(layout) = layout {
        args.push(layout.to_owned());
    }
    if let Some(profile_contract) = profile_contract {
        args.push(profile_contract.to_owned());
    }
    args.push(value_name.clone());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: kind.to_owned(),
            args,
        },
    });
    push_dep_edges(state, &value_name, &name);
    Ok(name)
}

fn lower_shader_bind_set(
    pipeline: &NirExpr,
    set_bindings: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let pipeline_name = lower_expr(pipeline, state, bindings)?;
    let mut binding_names = Vec::with_capacity(set_bindings.len());
    for binding in set_bindings {
        let binding_name = lower_expr(binding, state, bindings)?;
        binding_names.push(binding_name);
    }
    let name = next_name(state, "shader_bind_set");
    let mut args = vec![pipeline_name.clone()];
    args.extend(binding_names.iter().cloned());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "bind_set".to_owned(),
            args,
        },
    });
    push_dep_edges(state, &pipeline_name, &name);
    for binding_name in &binding_names {
        push_dep_edges(state, binding_name, &name);
    }
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

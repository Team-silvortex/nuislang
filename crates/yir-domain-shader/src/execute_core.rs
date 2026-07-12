use super::{
    parse_shader_flow_state,
    texture_sampling::{
        expect_sampler_value, expect_texture_value, expect_uv_value, normalized_uv_to_texel,
        parse_bool_flag, parse_csv_indices, parse_csv_ints, parse_texture_literal,
        sample_texture_by_filter, sample_texture_linear, sample_texture_nearest,
        sample_texture_uv_by_filter,
    },
};
use yir_core::{
    BlendState, DepthState, ExecutionState, IndexBuffer, Node, RasterState, RenderPass,
    RenderPipeline, RenderStateSet, Resource, SamplerState, ShaderBinding, ShaderBindingSet,
    ShaderFlowState, ShaderResultHandle, StructValue, SurfaceTarget, Value, VertexBuffer,
    VertexLayout, Viewport,
};

pub(crate) fn execute_shader_core_node(
    node: &Node,
    _resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Option<Value>, String> {
    let result = match node.op.instruction.as_str() {
        "target_config" => execute_target_config(node),
        "const" => execute_const(node),
        "const_bool" => execute_const_bool(node),
        "const_i32" => execute_const_i32(node),
        "const_i64" => execute_const_i64(node),
        "const_f32" => execute_const_f32(node),
        "const_f64" => execute_const_f64(node),
        "add" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? + state.expect_int(&node.op.args[1])?,
        )),
        "sub" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? - state.expect_int(&node.op.args[1])?,
        )),
        "mul" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?,
        )),
        "add_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? + state.expect_i32(&node.op.args[1])?,
        )),
        "mul_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? * state.expect_i32(&node.op.args[1])?,
        )),
        "add_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? + state.expect_f32(&node.op.args[1])?,
        )),
        "mul_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? * state.expect_f32(&node.op.args[1])?,
        )),
        "add_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
        )),
        "mul_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
        )),
        "target" => execute_target(node),
        "viewport" => execute_viewport(node),
        "pipeline" => Ok(Value::Pipeline(RenderPipeline {
            shading_model: node.op.args[0].clone(),
            topology: node.op.args[1].clone(),
        })),
        "inline_wgsl" => Ok(Value::Struct(StructValue {
            type_name: "ShaderInlineWgsl".to_owned(),
            fields: vec![
                ("entry".to_owned(), Value::Symbol(node.op.args[0].clone())),
                ("source".to_owned(), Value::Symbol(node.op.args[1].clone())),
            ],
        })),
        "vertex_layout" => execute_vertex_layout(node),
        "vertex_buffer" => execute_vertex_buffer(node),
        "index_buffer" => Ok(Value::IndexBuffer(IndexBuffer {
            indices: parse_csv_indices(node, &node.op.args[0])?,
        })),
        "blend_state" => Ok(Value::Blend(BlendState {
            enabled: parse_bool_flag(node, 0, "blend enabled")?,
            mode: node.op.args[1].clone(),
        })),
        "depth_state" => Ok(Value::Depth(DepthState {
            test_enabled: parse_bool_flag(node, 0, "depth test")?,
            write_enabled: parse_bool_flag(node, 1, "depth write")?,
            compare: node.op.args[2].clone(),
        })),
        "raster_state" => Ok(Value::Raster(RasterState {
            cull_mode: node.op.args[0].clone(),
            front_face: node.op.args[1].clone(),
        })),
        "render_state" => execute_render_state(node, state),
        "uv" => execute_uv(node),
        "texture2d" => Ok(Value::Texture(parse_texture_literal(node)?)),
        "sampler" => Ok(Value::Sampler(SamplerState {
            filter: node.op.args[0].clone(),
            address_mode: node.op.args[1].clone(),
        })),
        "uniform" | "storage" | "attachment" => execute_generic_binding(node, state),
        "texture_binding"
        | "sampler_binding"
        | "vertex_layout_binding"
        | "vertex_binding"
        | "index_binding" => execute_typed_binding(node, state),
        "bind_set" => execute_bind_set(node, state),
        "pack_ball_state" => {
            let color = state.expect_value(&node.op.args[0])?.clone();
            let speed = state.expect_value(&node.op.args[1])?.clone();
            Ok(Value::Tuple(vec![color, speed]))
        }
        "begin_pass" => execute_begin_pass(node, state),
        "observe" => execute_observe(node, state),
        "is_pass_ready" => {
            let result = state.expect_shader_result(&node.op.args[0])?;
            Ok(Value::Bool(matches!(
                result.state,
                ShaderFlowState::PassReady
            )))
        }
        "is_frame_ready" => {
            let result = state.expect_shader_result(&node.op.args[0])?;
            Ok(Value::Bool(matches!(
                result.state,
                ShaderFlowState::FrameReady
            )))
        }
        "value" => {
            let result = state.expect_shader_result(&node.op.args[0])?;
            Ok((*result.value).clone())
        }
        "sample" | "sample_nearest" => execute_sample(node, state),
        "sample_uv" | "sample_uv_nearest" | "sample_uv_linear" => execute_sample_uv(node, state),
        _ => return Ok(None),
    };
    result.map(Some)
}

fn execute_target_config(node: &Node) -> Result<Value, String> {
    let mut values = vec![
        Value::Symbol(node.op.args[0].clone()),
        Value::Symbol(node.op.args[1].clone()),
        Value::Int(node.op.args[2].parse::<i64>().map_err(|_| {
            format!(
                "node `{}` has invalid lane width `{}`",
                node.name, node.op.args[2]
            )
        })?),
    ];
    if let Some(features) = node.op.args.get(3) {
        values.push(Value::Symbol(features.clone()));
    }
    Ok(Value::Tuple(values))
}

fn execute_const(node: &Node) -> Result<Value, String> {
    Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
        |_| {
            format!(
                "node `{}` has invalid integer literal `{}`",
                node.name, node.op.args[0]
            )
        },
    )?))
}

fn execute_const_bool(node: &Node) -> Result<Value, String> {
    Ok(Value::Bool(match node.op.args[0].as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(format!(
                "node `{}` has invalid bool literal `{other}`",
                node.name
            ))
        }
    }))
}

fn execute_const_i32(node: &Node) -> Result<Value, String> {
    Ok(Value::I32(node.op.args[0].parse::<i32>().map_err(
        |_| {
            format!(
                "node `{}` has invalid i32 literal `{}`",
                node.name, node.op.args[0]
            )
        },
    )?))
}

fn execute_const_i64(node: &Node) -> Result<Value, String> {
    Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
        |_| {
            format!(
                "node `{}` has invalid i64 literal `{}`",
                node.name, node.op.args[0]
            )
        },
    )?))
}

fn execute_const_f32(node: &Node) -> Result<Value, String> {
    Ok(Value::F32(node.op.args[0].parse::<f32>().map_err(
        |_| {
            format!(
                "node `{}` has invalid f32 literal `{}`",
                node.name, node.op.args[0]
            )
        },
    )?))
}

fn execute_const_f64(node: &Node) -> Result<Value, String> {
    Ok(Value::F64(node.op.args[0].parse::<f64>().map_err(
        |_| {
            format!(
                "node `{}` has invalid f64 literal `{}`",
                node.name, node.op.args[0]
            )
        },
    )?))
}

fn execute_target(node: &Node) -> Result<Value, String> {
    let width = node.op.args[1].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid width `{}`",
            node.name, node.op.args[1]
        )
    })? as usize;
    let height = node.op.args[2].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid height `{}`",
            node.name, node.op.args[2]
        )
    })? as usize;
    Ok(Value::Target(SurfaceTarget {
        format: node.op.args[0].clone(),
        width,
        height,
    }))
}

fn execute_viewport(node: &Node) -> Result<Value, String> {
    let width = node.op.args[0].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid width `{}`",
            node.name, node.op.args[0]
        )
    })? as usize;
    let height = node.op.args[1].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid height `{}`",
            node.name, node.op.args[1]
        )
    })? as usize;
    Ok(Value::Viewport(Viewport { width, height }))
}

fn execute_vertex_layout(node: &Node) -> Result<Value, String> {
    Ok(Value::VertexLayout(VertexLayout {
        stride: node.op.args[0].parse::<usize>().map_err(|_| {
            format!(
                "node `{}` has invalid vertex stride `{}`",
                node.name, node.op.args[0]
            )
        })?,
        attributes: node.op.args[1]
            .split(',')
            .map(|attr| attr.trim().to_owned())
            .filter(|attr| !attr.is_empty())
            .collect(),
    }))
}

fn execute_vertex_buffer(node: &Node) -> Result<Value, String> {
    Ok(Value::VertexBuffer(VertexBuffer {
        vertex_count: node.op.args[0].parse::<usize>().map_err(|_| {
            format!(
                "node `{}` has invalid vertex count `{}`",
                node.name, node.op.args[0]
            )
        })?,
        elements: parse_csv_ints(node, &node.op.args[1], "vertex element")?,
    }))
}

fn execute_render_state(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let pipeline = match state.expect_value(&node.op.args[0])?.clone() {
        Value::Pipeline(pipeline) => pipeline,
        other => {
            return Err(format!(
                "shader.render_state expects pipeline value, got {}",
                other
            ))
        }
    };
    let blend = match state.expect_value(&node.op.args[1])?.clone() {
        Value::Blend(blend) => blend,
        other => {
            return Err(format!(
                "shader.render_state expects blend state, got {}",
                other
            ))
        }
    };
    let depth = match state.expect_value(&node.op.args[2])?.clone() {
        Value::Depth(depth) => depth,
        other => {
            return Err(format!(
                "shader.render_state expects depth state, got {}",
                other
            ))
        }
    };
    let raster = match state.expect_value(&node.op.args[3])?.clone() {
        Value::Raster(raster) => raster,
        other => {
            return Err(format!(
                "shader.render_state expects raster state, got {}",
                other
            ))
        }
    };
    Ok(Value::RenderState(RenderStateSet {
        pipeline,
        blend,
        depth,
        raster,
    }))
}

fn execute_uv(node: &Node) -> Result<Value, String> {
    Ok(Value::Tuple(vec![
        Value::Int(node.op.args[0].parse::<i64>().map_err(|_| {
            format!(
                "node `{}` has invalid u coord `{}`",
                node.name, node.op.args[0]
            )
        })?),
        Value::Int(node.op.args[1].parse::<i64>().map_err(|_| {
            format!(
                "node `{}` has invalid v coord `{}`",
                node.name, node.op.args[1]
            )
        })?),
    ]))
}

fn execute_generic_binding(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let slot = node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid binding slot `{}`",
            node.name, node.op.args[0]
        )
    })?;
    let value = state.expect_value(&node.op.args[1])?.clone();
    if node.op.instruction == "attachment" && !matches!(value, Value::Target(_)) {
        return Err(format!(
            "shader.attachment expects target value, got {}",
            value
        ));
    }
    Ok(Value::Binding(ShaderBinding {
        kind: node.op.instruction.clone(),
        slot,
        value: Box::new(value),
    }))
}

fn execute_typed_binding(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let slot = node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid binding slot `{}`",
            node.name, node.op.args[0]
        )
    })?;
    let value = state.expect_value(&node.op.args[1])?.clone();
    let expected = match node.op.instruction.as_str() {
        "texture_binding" => "texture",
        "sampler_binding" => "sampler",
        "vertex_layout_binding" => "vertex_layout",
        "vertex_binding" => "vertex_buffer",
        _ => "index_buffer",
    };
    match (expected, &value) {
        ("texture", Value::Texture(_)) | ("sampler", Value::Sampler(_)) => {}
        ("vertex_layout", Value::VertexLayout(_)) => {}
        ("vertex_buffer", Value::VertexBuffer(_)) | ("index_buffer", Value::IndexBuffer(_)) => {}
        _ => {
            return Err(format!(
                "shader.{} expects {} value, got {}",
                node.op.instruction, expected, value
            ))
        }
    }
    Ok(Value::Binding(ShaderBinding {
        kind: node.op.instruction.clone(),
        slot,
        value: Box::new(value),
    }))
}

fn execute_bind_set(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let pipeline = match state.expect_value(&node.op.args[0])?.clone() {
        Value::Pipeline(pipeline) => pipeline,
        other => {
            return Err(format!(
                "shader.bind_set expects pipeline value, got {}",
                other
            ))
        }
    };

    let mut bindings = Vec::with_capacity(node.op.args.len().saturating_sub(1));
    for binding_name in &node.op.args[1..] {
        let binding = match state.expect_value(binding_name)?.clone() {
            Value::Binding(binding) => binding,
            other => {
                return Err(format!(
                    "shader.bind_set expects binding values, got {}",
                    other
                ))
            }
        };
        bindings.push(binding);
    }

    Ok(Value::BindingSet(ShaderBindingSet { pipeline, bindings }))
}

fn execute_begin_pass(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let target = match state.expect_value(&node.op.args[0])?.clone() {
        Value::Target(target) => target,
        other => {
            return Err(format!(
                "shader.begin_pass expects target value, got {}",
                other
            ))
        }
    };
    let pipeline = match state.expect_value(&node.op.args[1])?.clone() {
        Value::Pipeline(pipeline) => pipeline,
        other => {
            return Err(format!(
                "shader.begin_pass expects pipeline value, got {}",
                other
            ))
        }
    };
    let viewport = match state.expect_value(&node.op.args[2])?.clone() {
        Value::Viewport(viewport) => viewport,
        other => {
            return Err(format!(
                "shader.begin_pass expects viewport value, got {}",
                other
            ))
        }
    };
    Ok(Value::RenderPass(RenderPass {
        target,
        pipeline,
        viewport,
    }))
}

fn execute_observe(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let value = state.expect_value(&node.op.args[0])?.clone();
    let flow = parse_shader_flow_state(&node.op.args[1])?;
    Ok(Value::ShaderResult(ShaderResultHandle {
        state: flow,
        value: Box::new(value),
    }))
}

fn execute_sample(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let texture = match state.expect_value(&node.op.args[0])?.clone() {
        Value::Texture(texture) => texture,
        other => {
            return Err(format!(
                "shader.{} expects texture value, got {}",
                node.op.instruction, other
            ))
        }
    };
    let sampler = match state.expect_value(&node.op.args[1])?.clone() {
        Value::Sampler(sampler) => sampler,
        other => {
            return Err(format!(
                "shader.{} expects sampler value, got {}",
                node.op.instruction, other
            ))
        }
    };
    let x = state.expect_int(&node.op.args[2])?;
    let y = state.expect_int(&node.op.args[3])?;
    let sampled = match node.op.instruction.as_str() {
        "sample_nearest" => sample_texture_nearest(&texture, &sampler, x, y),
        _ => sample_texture_by_filter(&texture, &sampler, x, y),
    };
    Ok(Value::Int(sampled))
}

fn execute_sample_uv(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let op_name = format!("shader.{}", node.op.instruction);
    let texture = expect_texture_value(state, &node.op.args[0], &op_name)?;
    let sampler = expect_sampler_value(state, &node.op.args[1], &op_name)?;
    let (u, v) = expect_uv_value(state, &node.op.args[2], &op_name)?;
    let sampled = match node.op.instruction.as_str() {
        "sample_uv_nearest" => {
            let (x, y) = normalized_uv_to_texel(&texture, u, v);
            sample_texture_nearest(&texture, &sampler, x, y)
        }
        "sample_uv_linear" => sample_texture_linear(&texture, &sampler, u, v),
        _ => sample_texture_uv_by_filter(&texture, &sampler, u, v),
    };
    Ok(Value::Int(sampled))
}

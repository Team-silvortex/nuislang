use yir_core::{
    BlendState, DepthState, ExecutionState, FrameSurface, IndexBuffer, InstructionSemantics, Node,
    RasterState, RegisteredMod, RenderPass, RenderPipeline, RenderStateSet, Resource, SamplerState,
    ShaderBinding, ShaderBindingSet, StructValue, SurfaceTarget, Texture2D, Value, VertexBuffer,
    VertexLayout, Viewport,
};

pub struct ShaderMod;

impl RegisteredMod for ShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        require_shader_resource(node, resource)?;

        match node.op.instruction.as_str() {
            "const" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const <name> <resource> <value>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid integer literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_bool" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const_bool <name> <resource> <value>`",
                        node.name
                    ));
                }
                match node.op.args[0].as_str() {
                    "true" | "false" => Ok(InstructionSemantics::pure(Vec::new())),
                    other => Err(format!(
                        "node `{}` has invalid bool literal `{other}`",
                        node.name
                    )),
                }
            }
            "const_i32" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const_i32 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<i32>().map_err(|_| {
                    format!(
                        "node `{}` has invalid i32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_i64" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const_i64 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid i64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_f32" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const_f32 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<f32>().map_err(|_| {
                    format!(
                        "node `{}` has invalid f32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_f64" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.const_f64 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<f64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid f64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "add" | "sub" | "mul" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <lhs> <rhs>`",
                        node.name, node.op.instruction
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "add_i32" | "mul_i32" | "add_f32" | "mul_f32" | "add_f64" | "mul_f64" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <lhs> <rhs>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "target" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `shader.target <name> <resource> <format> <width> <height>`",
                        node.name
                    ));
                }

                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid width `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid height `{}`",
                        node.name, node.op.args[2]
                    )
                })?;

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "viewport" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.viewport <name> <resource> <width> <height>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid width `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid height `{}`",
                        node.name, node.op.args[1]
                    )
                })?;

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "pipeline" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.pipeline <name> <resource> <shading_model> <topology>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "vertex_layout" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.vertex_layout <name> <resource> <stride> <csv-attributes>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vertex stride `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "vertex_buffer" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.vertex_buffer <name> <resource> <vertex_count> <csv-elements>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vertex count `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                parse_csv_ints(node, &node.op.args[1], "vertex element")?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "index_buffer" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.index_buffer <name> <resource> <csv-indices>`",
                        node.name
                    ));
                }
                parse_csv_indices(node, &node.op.args[0])?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "blend_state" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.blend_state <name> <resource> <enabled> <mode>`",
                        node.name
                    ));
                }
                parse_bool_flag(node, 0, "blend enabled")?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "depth_state" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `shader.depth_state <name> <resource> <test_enabled> <write_enabled> <compare>`",
                        node.name
                    ));
                }
                parse_bool_flag(node, 0, "depth test")?;
                parse_bool_flag(node, 1, "depth write")?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "raster_state" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.raster_state <name> <resource> <cull_mode> <front_face>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "render_state" => {
                if node.op.args.len() != 4 {
                    return Err(format!(
                        "node `{}` expects `shader.render_state <name> <resource> <pipeline> <blend> <depth> <raster>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "uv" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.uv <name> <resource> <u_1024> <v_1024>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid u coord `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid v coord `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "texture2d" => {
                if node.op.args.len() != 4 {
                    return Err(format!(
                        "node `{}` expects `shader.texture2d <name> <resource> <format> <width> <height> <csv-texels>`",
                        node.name
                    ));
                }
                validate_texture_literal(node)?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "sampler" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.sampler <name> <resource> <filter> <address_mode>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "pack_ball_state" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.pack_ball_state <name> <resource> <color> <speed>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "begin_pass" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `shader.begin_pass <name> <resource> <target> <pipeline> <viewport>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "uniform"
            | "storage"
            | "attachment"
            | "texture_binding"
            | "sampler_binding"
            | "vertex_layout_binding"
            | "vertex_binding"
            | "index_binding" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <slot> <value>`",
                        node.name, node.op.instruction
                    ));
                }

                node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid binding slot `{}`",
                        node.name, node.op.args[0]
                    )
                })?;

                Ok(InstructionSemantics::pure(vec![node.op.args[1].clone()]))
            }
            "bind_set" => {
                if node.op.args.len() < 2 {
                    return Err(format!(
                        "node `{}` expects `shader.bind_set <name> <resource> <pipeline> <binding> [binding...]`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "clear" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.clear <name> <resource> <target> <fill>`",
                        node.name
                    ));
                }

                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid clear fill `{}`",
                        node.name, node.op.args[1]
                    )
                })?;

                Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
            }
            "overlay" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.overlay <name> <resource> <base> <top>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "sample" | "sample_nearest" => {
                if node.op.args.len() != 4 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <texture> <sampler> <x> <y>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "sample_uv" | "sample_uv_nearest" | "sample_uv_linear" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <texture> <sampler> <uv>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "dispatch" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.dispatch <name> <resource> <input>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "draw_instanced" => {
                if !(node.op.args.len() == 4 || node.op.args.len() == 5) {
                    return Err(format!(
                        "node `{}` expects `shader.draw_instanced <name> <resource> <pass> <packet> <vertex_count> <instance_count> [bind_set]`",
                        node.name
                    ));
                }

                node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vertex_count `{}`",
                        node.name, node.op.args[2]
                    )
                })?;
                node.op.args[3].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid instance_count `{}`",
                        node.name, node.op.args[3]
                    )
                })?;

                let mut deps = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                if let Some(bind_set) = node.op.args.get(4) {
                    deps.push(bind_set.clone());
                }
                Ok(InstructionSemantics::effect(deps))
            }
            "draw_ball" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.draw_ball <name> <resource> <packet>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "draw_sphere" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.draw_sphere <name> <resource> <packet>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "print" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.print <name> <resource> <input>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            other => Err(format!("unknown shader instruction `{other}`")),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        match node.op.instruction.as_str() {
            "const" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid integer literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_bool" => Ok(Value::Bool(match node.op.args[0].as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(format!(
                        "node `{}` has invalid bool literal `{other}`",
                        node.name
                    ))
                }
            })),
            "const_i32" => Ok(Value::I32(node.op.args[0].parse::<i32>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid i32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_i64" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid i64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_f32" => Ok(Value::F32(node.op.args[0].parse::<f32>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid f32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_f64" => Ok(Value::F64(node.op.args[0].parse::<f64>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid f64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
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
            "target" => {
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
            "viewport" => {
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
            "pipeline" => Ok(Value::Pipeline(RenderPipeline {
                shading_model: node.op.args[0].clone(),
                topology: node.op.args[1].clone(),
            })),
            "vertex_layout" => Ok(Value::VertexLayout(VertexLayout {
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
            })),
            "vertex_buffer" => Ok(Value::VertexBuffer(VertexBuffer {
                vertex_count: node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vertex count `{}`",
                        node.name, node.op.args[0]
                    )
                })?,
                elements: parse_csv_ints(node, &node.op.args[1], "vertex element")?,
            })),
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
            "render_state" => {
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
            "uv" => Ok(Value::Tuple(vec![
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
            ])),
            "texture2d" => Ok(Value::Texture(parse_texture_literal(node)?)),
            "sampler" => Ok(Value::Sampler(SamplerState {
                filter: node.op.args[0].clone(),
                address_mode: node.op.args[1].clone(),
            })),
            "uniform" | "storage" | "attachment" => {
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
            "texture_binding"
            | "sampler_binding"
            | "vertex_layout_binding"
            | "vertex_binding"
            | "index_binding" => {
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
                match (&*expected, &value) {
                    ("texture", Value::Texture(_)) | ("sampler", Value::Sampler(_)) => {}
                    ("vertex_layout", Value::VertexLayout(_)) => {}
                    ("vertex_buffer", Value::VertexBuffer(_))
                    | ("index_buffer", Value::IndexBuffer(_)) => {}
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
            "bind_set" => {
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
            "pack_ball_state" => {
                let color = state.expect_value(&node.op.args[0])?.clone();
                let speed = state.expect_value(&node.op.args[1])?.clone();
                Ok(Value::Tuple(vec![color, speed]))
            }
            "begin_pass" => {
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
            "clear" => {
                let target = match state.expect_value(&node.op.args[0])?.clone() {
                    Value::Target(target) => target,
                    other => {
                        return Err(format!("shader.clear expects target value, got {}", other))
                    }
                };
                let fill = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid clear fill `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                let frame = clear_target_surface(&target, fill);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.clear @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Frame(frame))
            }
            "overlay" => {
                let base = match state.expect_value(&node.op.args[0])?.clone() {
                    Value::Frame(frame) => frame,
                    other => {
                        return Err(format!("shader.overlay expects base frame, got {}", other))
                    }
                };
                let top = match state.expect_value(&node.op.args[1])?.clone() {
                    Value::Frame(frame) => frame,
                    other => {
                        return Err(format!("shader.overlay expects top frame, got {}", other))
                    }
                };
                let frame = overlay_surfaces(&base, &top)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.overlay @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Frame(frame))
            }
            "sample" | "sample_nearest" => {
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
            "sample_uv" | "sample_uv_nearest" | "sample_uv_linear" => {
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
            "dispatch" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.dispatch @{} [{}]: {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(value)
            }
            "draw_instanced" => {
                let pass = match state.expect_value(&node.op.args[0])?.clone() {
                    Value::RenderPass(pass) => pass,
                    other => {
                        return Err(format!(
                            "shader.draw_instanced expects render pass, got {}",
                            other
                        ))
                    }
                };
                let packet = state.expect_value(&node.op.args[1])?.clone();
                let vertex_count = node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vertex_count `{}`",
                        node.name, node.op.args[2]
                    )
                })?;
                let instance_count = node.op.args[3].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid instance_count `{}`",
                        node.name, node.op.args[3]
                    )
                })?;
                let bindings = match node.op.args.get(4) {
                    Some(name) => match state.expect_value(name)?.clone() {
                        Value::BindingSet(bindings) => Some(bindings),
                        other => {
                            return Err(format!(
                                "shader.draw_instanced expects bind_set value, got {}",
                                other
                            ))
                        }
                    },
                    None => None,
                };
                let frame = draw_render_pass_surface(
                    &pass,
                    &packet,
                    vertex_count,
                    instance_count,
                    bindings.as_ref(),
                )?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.draw_instanced @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Frame(frame))
            }
            "draw_ball" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let frame = draw_ball_surface(&value)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.draw_ball @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Frame(frame))
            }
            "draw_sphere" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let frame = draw_sphere_surface(&value)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.draw_sphere @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Frame(frame))
            }
            "print" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect shader.print @{} [{}]: {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::Unit)
            }
            other => Err(format!("unknown shader instruction `{other}`")),
        }
    }
}

fn require_shader_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("shader") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses shader mod on non-shader resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn draw_ball_surface(value: &Value) -> Result<FrameSurface, String> {
    let packet = parse_ball_packet(value, "shader.draw_ball")?;

    let width = 16usize;
    let height = 9usize;
    let speed = packet.speed;
    let center_x = (((speed).round() as i64).rem_euclid(width as i64)) as usize;
    let center_y = ((((speed / 2.0).round()) as i64).rem_euclid(height as i64)) as usize;
    let glyph = match packet.color_key.rem_euclid(3) {
        0 => 'o',
        1 => 'O',
        _ => '@',
    };

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        for x in 0..width {
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);
            if dx <= 1 && dy <= 1 {
                row.push(glyph);
            } else {
                row.push('.');
            }
        }
        rows.push(row);
    }

    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}

fn parse_texture_shape(node: &Node) -> Result<(usize, usize), String> {
    let width = node.op.args[1].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid width `{}`",
            node.name, node.op.args[1]
        )
    })?;
    let height = node.op.args[2].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid height `{}`",
            node.name, node.op.args[2]
        )
    })?;
    if width == 0 || height == 0 {
        return Err(format!(
            "node `{}` texture shape must be non-zero",
            node.name
        ));
    }
    Ok((width, height))
}

fn parse_bool_flag(node: &Node, index: usize, label: &str) -> Result<bool, String> {
    let raw = node
        .op
        .args
        .get(index)
        .ok_or_else(|| format!("node `{}` missing {}", node.name, label))?;
    match raw.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!(
            "node `{}` has invalid {} `{}`; expected 0 or 1",
            node.name, label, raw
        )),
    }
}

fn validate_texture_literal(node: &Node) -> Result<(), String> {
    let (width, height) = parse_texture_shape(node)?;
    let texels = parse_csv_ints(node, &node.op.args[3], "texture literal texel")?;
    if texels.len() != width * height {
        return Err(format!(
            "node `{}` expected {} texture texels, got {}",
            node.name,
            width * height,
            texels.len()
        ));
    }
    Ok(())
}

fn parse_texture_literal(node: &Node) -> Result<Texture2D, String> {
    let (width, height) = parse_texture_shape(node)?;
    let texels = parse_csv_ints(node, &node.op.args[3], "texture literal texel")?;
    if texels.len() != width * height {
        return Err(format!(
            "node `{}` expected {} texture texels, got {}",
            node.name,
            width * height,
            texels.len()
        ));
    }
    Ok(Texture2D {
        format: node.op.args[0].clone(),
        width,
        height,
        texels,
    })
}

fn parse_csv_ints(node: &Node, raw: &str, label: &str) -> Result<Vec<i64>, String> {
    raw.split(',')
        .map(|part| {
            let value = part.trim();
            value
                .parse::<i64>()
                .map_err(|_| format!("node `{}` has invalid {} `{value}`", node.name, label))
        })
        .collect()
}

fn parse_csv_indices(node: &Node, raw: &str) -> Result<Vec<usize>, String> {
    raw.split(',')
        .map(|part| {
            let value = part.trim();
            value
                .parse::<usize>()
                .map_err(|_| format!("node `{}` has invalid index literal `{value}`", node.name))
        })
        .collect()
}

fn sample_texture_nearest(texture: &Texture2D, sampler: &SamplerState, x: i64, y: i64) -> i64 {
    let address = sampler.address_mode.as_str();
    let ix = apply_address_mode(x, texture.width, address);
    let iy = apply_address_mode(y, texture.height, address);
    texture.texels[iy * texture.width + ix]
}

fn sample_texture_by_filter(texture: &Texture2D, sampler: &SamplerState, x: i64, y: i64) -> i64 {
    match sampler.filter.as_str() {
        "nearest" => sample_texture_nearest(texture, sampler, x, y),
        "linear" => {
            let u = texel_coord_to_normalized_1024(texture.width, x);
            let v = texel_coord_to_normalized_1024(texture.height, y);
            sample_texture_linear(texture, sampler, u, v)
        }
        _ => sample_texture_nearest(texture, sampler, x, y),
    }
}

fn sample_texture_uv_by_filter(
    texture: &Texture2D,
    sampler: &SamplerState,
    u_1024: i64,
    v_1024: i64,
) -> i64 {
    match sampler.filter.as_str() {
        "nearest" => {
            let (x, y) = normalized_uv_to_texel(texture, u_1024, v_1024);
            sample_texture_nearest(texture, sampler, x, y)
        }
        "linear" => sample_texture_linear(texture, sampler, u_1024, v_1024),
        _ => {
            let (x, y) = normalized_uv_to_texel(texture, u_1024, v_1024);
            sample_texture_nearest(texture, sampler, x, y)
        }
    }
}

fn sample_texture_linear(
    texture: &Texture2D,
    sampler: &SamplerState,
    u_1024: i64,
    v_1024: i64,
) -> i64 {
    let (base_x, frac_x) = normalized_uv_to_linear_coord(texture.width, u_1024);
    let (base_y, frac_y) = normalized_uv_to_linear_coord(texture.height, v_1024);
    let address = sampler.address_mode.as_str();

    let x0 = apply_address_mode(base_x, texture.width, address);
    let x1 = apply_address_mode(base_x + 1, texture.width, address);
    let y0 = apply_address_mode(base_y, texture.height, address);
    let y1 = apply_address_mode(base_y + 1, texture.height, address);

    let t00 = texture.texels[y0 * texture.width + x0];
    let t10 = texture.texels[y0 * texture.width + x1];
    let t01 = texture.texels[y1 * texture.width + x0];
    let t11 = texture.texels[y1 * texture.width + x1];

    let top = lerp_fixed(t00, t10, frac_x);
    let bottom = lerp_fixed(t01, t11, frac_x);
    lerp_fixed(top, bottom, frac_y)
}

fn texel_coord_to_normalized_1024(extent: usize, coord: i64) -> i64 {
    if extent <= 1 {
        return 0;
    }
    let max_index = extent.saturating_sub(1) as i64;
    let clamped = coord.clamp(0, max_index);
    ((clamped * 1024) + (max_index / 2)) / max_index.max(1)
}

fn apply_address_mode(coord: i64, extent: usize, address_mode: &str) -> usize {
    if extent == 0 {
        return 0;
    }
    match address_mode {
        "repeat" | "wrap" => coord.rem_euclid(extent as i64) as usize,
        _ => coord.clamp(0, extent.saturating_sub(1) as i64) as usize,
    }
}

fn expect_texture_value(state: &ExecutionState, name: &str, op: &str) -> Result<Texture2D, String> {
    match state.expect_value(name)?.clone() {
        Value::Texture(texture) => Ok(texture),
        other => Err(format!("{op} expects texture value, got {}", other)),
    }
}

fn expect_sampler_value(
    state: &ExecutionState,
    name: &str,
    op: &str,
) -> Result<SamplerState, String> {
    match state.expect_value(name)?.clone() {
        Value::Sampler(sampler) => Ok(sampler),
        other => Err(format!("{op} expects sampler value, got {}", other)),
    }
}

fn expect_uv_value(state: &ExecutionState, name: &str, op: &str) -> Result<(i64, i64), String> {
    match state.expect_value(name)?.clone() {
        Value::Tuple(values) if values.len() == 2 => match (&values[0], &values[1]) {
            (Value::Int(u), Value::Int(v)) => Ok((*u, *v)),
            _ => Err(format!("{op} expects uv tuple `(int, int)`")),
        },
        other => Err(format!("{op} expects uv tuple, got {}", other)),
    }
}

fn normalized_uv_to_texel(texture: &Texture2D, u_1024: i64, v_1024: i64) -> (i64, i64) {
    (
        normalized_component_to_texel(texture.width, u_1024),
        normalized_component_to_texel(texture.height, v_1024),
    )
}

fn normalized_component_to_texel(extent: usize, value_1024: i64) -> i64 {
    if extent <= 1 {
        return 0;
    }
    ((value_1024 * (extent as i64 - 1)) + 512) / 1024
}

fn normalized_uv_to_linear_coord(extent: usize, value_1024: i64) -> (i64, i64) {
    if extent <= 1 {
        return (0, 0);
    }
    let scaled = value_1024 * (extent as i64 - 1);
    let base = scaled.div_euclid(1024);
    let frac = scaled.rem_euclid(1024);
    (base, frac)
}

fn lerp_fixed(a: i64, b: i64, t_1024: i64) -> i64 {
    ((a * (1024 - t_1024)) + (b * t_1024) + 512) / 1024
}

fn clear_target_surface(target: &SurfaceTarget, fill: i64) -> FrameSurface {
    let width = target.width.max(1);
    let height = target.height.max(1);
    let glyph = match fill.rem_euclid(5) {
        0 => '.',
        1 => ':',
        2 => '-',
        3 => '=',
        _ => ' ',
    };
    let row = std::iter::repeat_n(glyph, width).collect::<String>();
    FrameSurface {
        width,
        height,
        rows: vec![row; height],
    }
}

fn overlay_surfaces(base: &FrameSurface, top: &FrameSurface) -> Result<FrameSurface, String> {
    if base.width != top.width || base.height != top.height {
        return Err(format!(
            "shader.overlay expects matching frame dimensions, got {}x{} and {}x{}",
            base.width, base.height, top.width, top.height
        ));
    }

    let rows = base
        .rows
        .iter()
        .zip(&top.rows)
        .map(|(base_row, top_row)| {
            base_row
                .chars()
                .zip(top_row.chars())
                .map(|(base_char, top_char)| if top_char != '.' { top_char } else { base_char })
                .collect::<String>()
        })
        .collect();

    Ok(FrameSurface {
        width: base.width,
        height: base.height,
        rows,
    })
}

fn draw_sphere_surface(value: &Value) -> Result<FrameSurface, String> {
    draw_sphere_surface_with_size(value, 48, 32)
}

fn draw_render_pass_surface(
    pass: &RenderPass,
    packet: &Value,
    vertex_count: i64,
    instance_count: i64,
    bindings: Option<&ShaderBindingSet>,
) -> Result<FrameSurface, String> {
    if vertex_count <= 0 || instance_count <= 0 {
        return Err("shader.draw_instanced expects positive vertex/instance counts".to_owned());
    }

    let geometry = bindings.map(resolve_geometry_inputs).transpose()?;

    let width = pass.viewport.width.min(pass.target.width).max(1);
    let height = pass.viewport.height.min(pass.target.height).max(1);
    if let Some(geometry) = &geometry {
        let expected_elements = geometry
            .vertex_layout
            .stride
            .saturating_mul(geometry.vertex_buffer.vertex_count);
        if geometry.vertex_buffer.elements.len() < expected_elements {
            return Err(format!(
                "shader.draw_instanced expects at least {} vertex elements from layout stride {}, got {}",
                expected_elements,
                geometry.vertex_layout.stride,
                geometry.vertex_buffer.elements.len()
            ));
        }
        if vertex_count as usize > geometry.vertex_buffer.vertex_count {
            return Err(format!(
                "shader.draw_instanced requests {} vertices but bound vertex buffer only has {}",
                vertex_count, geometry.vertex_buffer.vertex_count
            ));
        }
        if let Some(index_buffer) = &geometry.index_buffer {
            if vertex_count as usize > index_buffer.indices.len() {
                return Err(format!(
                    "shader.draw_instanced requests {} indices but bound index buffer only has {}",
                    vertex_count,
                    index_buffer.indices.len()
                ));
            }
        }
    }

    let mut frame = match pass.pipeline.shading_model.as_str() {
        "ball" | "sphere" | "lit_sphere" => draw_sphere_surface_with_size(packet, width, height),
        _ => draw_ball_surface_with_size(packet, width, height),
    }?;
    if let Some(geometry) = geometry.as_ref() {
        render_geometry_overlay(
            &mut frame,
            geometry,
            vertex_count as usize,
            pass.pipeline.topology.as_str(),
        );
    }
    Ok(frame)
}

fn draw_ball_surface_with_size(
    value: &Value,
    width: usize,
    height: usize,
) -> Result<FrameSurface, String> {
    let packet = parse_ball_packet(value, "shader.draw_ball")?;

    let width = width.max(8);
    let height = height.max(8);
    let radius = (0.72f32 * packet.radius_scale).clamp(0.18, 0.95);
    let offset_x = (packet.speed * 0.03).sin() * 0.22;
    let offset_y = (packet.speed * 0.02).cos() * 0.16;
    let palette = sphere_palette(packet.color_key);

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        let ny = ((y as f32 / (height - 1) as f32) * 2.0 - 1.0) - offset_y;
        for x in 0..width {
            let nx = ((x as f32 / (width - 1) as f32) * 2.0 - 1.0) - offset_x;
            let r2 = nx * nx + ny * ny;
            if r2 > radius * radius {
                row.push('.');
                continue;
            }

            let nz = (radius * radius - r2).sqrt();
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
            let lx = -0.45f32;
            let ly = -0.35f32;
            let lz = 0.82f32;
            let ll = (lx * lx + ly * ly + lz * lz).sqrt();
            let light =
                ((nx / len) * (lx / ll) + (ny / len) * (ly / ll) + (nz / len) * (lz / ll)).max(0.0);
            let rim = (1.0 - (nz / radius)).powf(1.6) * 0.35;
            let shade = (light * 0.85 + rim).clamp(0.0, 1.0);
            let index =
                ((shade * (palette.len() - 1) as f32).round() as usize).min(palette.len() - 1);
            row.push(palette[index]);
        }
        rows.push(row);
    }

    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}

fn draw_sphere_surface_with_size(
    value: &Value,
    width: usize,
    height: usize,
) -> Result<FrameSurface, String> {
    let width = width.max(8);
    let height = height.max(8);
    let packet = parse_ball_packet(value, "shader.draw_sphere")?;

    let radius = (0.72f32 * packet.radius_scale).clamp(0.18, 0.95);
    let offset_x = (packet.speed * 0.03).sin() * 0.22;
    let offset_y = (packet.speed * 0.02).cos() * 0.16;
    let palette = sphere_palette(packet.color_key);

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        let ny = ((y as f32 / (height - 1) as f32) * 2.0 - 1.0) - offset_y;
        for x in 0..width {
            let nx = ((x as f32 / (width - 1) as f32) * 2.0 - 1.0) - offset_x;
            let r2 = nx * nx + ny * ny;
            if r2 > radius * radius {
                row.push('.');
                continue;
            }

            let nz = (radius * radius - r2).sqrt();
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
            let lx = -0.45f32;
            let ly = -0.35f32;
            let lz = 0.82f32;
            let ll = (lx * lx + ly * ly + lz * lz).sqrt();
            let light =
                ((nx / len) * (lx / ll) + (ny / len) * (ly / ll) + (nz / len) * (lz / ll)).max(0.0);
            let rim = (1.0 - (nz / radius)).powf(1.6) * 0.35;
            let shade = (light * 0.85 + rim).clamp(0.0, 1.0);
            let index =
                ((shade * (palette.len() - 1) as f32).round() as usize).min(palette.len() - 1);
            row.push(palette[index]);
        }
        rows.push(row);
    }

    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}

fn sphere_palette(color: i64) -> &'static [char] {
    match color.rem_euclid(3) {
        0 => &[':', '-', '=', '+', '*', 'o'],
        1 => &[':', '-', '=', '+', '*', 'O'],
        _ => &[':', '-', '=', '+', '*', '@'],
    }
}

#[derive(Debug, Clone, Copy)]
struct BallPacket {
    color_key: i64,
    speed: f32,
    radius_scale: f32,
}

fn parse_ball_packet(value: &Value, op: &str) -> Result<BallPacket, String> {
    match value {
        Value::Tuple(items) if items.len() >= 2 => {
            let color_key = scalar_to_color_key(&items[0], op)?;
            let speed = scalar_to_f32(&items[1], op)?;
            let radius_scale = match items.get(2) {
                Some(value) => scalar_to_f32(value, op)?,
                None => 1.0,
            };
            Ok(BallPacket {
                color_key,
                speed,
                radius_scale,
            })
        }
        Value::Struct(packet) => parse_ball_packet_struct(packet, op),
        _ => Err(format!(
            "{op} expects a packet tuple `(color, speed[, radius_scale])` or struct with `color` and `speed`"
        )),
    }
}

fn parse_ball_packet_struct(packet: &StructValue, op: &str) -> Result<BallPacket, String> {
    let color = packet
        .fields
        .iter()
        .find(|(name, _)| name == "color")
        .map(|(_, value)| value)
        .ok_or_else(|| format!("{op} struct packet is missing `color` field"))?;
    let speed = packet
        .fields
        .iter()
        .find(|(name, _)| name == "speed")
        .map(|(_, value)| value)
        .ok_or_else(|| format!("{op} struct packet is missing `speed` field"))?;
    let radius_scale = packet
        .fields
        .iter()
        .find(|(name, _)| name == "radius_scale")
        .map(|(_, value)| scalar_to_f32(value, op))
        .transpose()?
        .unwrap_or(1.0);

    Ok(BallPacket {
        color_key: scalar_to_color_key(color, op)?,
        speed: scalar_to_f32(speed, op)?,
        radius_scale,
    })
}

fn scalar_to_color_key(value: &Value, op: &str) -> Result<i64, String> {
    match value {
        Value::Bool(value) => Ok(if *value { 1 } else { 0 }),
        Value::I32(value) => Ok(*value as i64),
        Value::Int(value) => Ok(*value),
        Value::F32(value) => Ok(value.round() as i64),
        Value::F64(value) => Ok(value.round() as i64),
        other => Err(format!("{op} expects scalar `color` value, got {}", other)),
    }
}

fn scalar_to_f32(value: &Value, op: &str) -> Result<f32, String> {
    match value {
        Value::Bool(value) => Ok(if *value { 1.0 } else { 0.0 }),
        Value::I32(value) => Ok(*value as f32),
        Value::Int(value) => Ok(*value as f32),
        Value::F32(value) => Ok(*value),
        Value::F64(value) => Ok(*value as f32),
        other => Err(format!("{op} expects scalar numeric value, got {}", other)),
    }
}

struct GeometryInputs {
    vertex_layout: VertexLayout,
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VertexAttributeKind {
    Pos2,
    Color2,
    Uv2,
    Unknown,
}

fn resolve_geometry_inputs(bindings: &ShaderBindingSet) -> Result<GeometryInputs, String> {
    let vertex_layout = bindings
        .bindings
        .iter()
        .find(|binding| binding.kind == "vertex_layout_binding")
        .ok_or_else(|| "shader.draw_instanced bind_set is missing vertex_layout_binding".to_owned())
        .and_then(|binding| match binding.value.as_ref() {
            Value::VertexLayout(layout) => Ok(layout.clone()),
            other => Err(format!(
                "shader.draw_instanced expected vertex_layout binding, got {}",
                other
            )),
        })?;
    let vertex_buffer = bindings
        .bindings
        .iter()
        .find(|binding| binding.kind == "vertex_binding")
        .ok_or_else(|| "shader.draw_instanced bind_set is missing vertex_binding".to_owned())
        .and_then(|binding| match binding.value.as_ref() {
            Value::VertexBuffer(buffer) => Ok(buffer.clone()),
            other => Err(format!(
                "shader.draw_instanced expected vertex_buffer binding, got {}",
                other
            )),
        })?;

    let index_buffer = bindings
        .bindings
        .iter()
        .find(|binding| binding.kind == "index_binding")
        .map(|binding| match binding.value.as_ref() {
            Value::IndexBuffer(buffer) => Ok(buffer.clone()),
            other => Err(format!(
                "shader.draw_instanced expected index_buffer binding, got {}",
                other
            )),
        })
        .transpose()?;

    Ok(GeometryInputs {
        vertex_layout,
        vertex_buffer,
        index_buffer,
    })
}

fn render_geometry_overlay(
    frame: &mut FrameSurface,
    geometry: &GeometryInputs,
    vertex_count: usize,
    topology: &str,
) {
    if frame.rows.is_empty() || frame.width == 0 {
        return;
    }

    let attributes = geometry
        .vertex_layout
        .attributes
        .iter()
        .map(|attr| parse_vertex_attribute_kind(attr))
        .collect::<Vec<_>>();
    let referenced_vertices = referenced_vertex_indices(geometry, vertex_count);
    let mut rows = frame
        .rows
        .iter()
        .map(|row| row.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut samples = Vec::new();
    for vertex_index in referenced_vertices {
        if let Some(sample) = interpret_vertex(geometry, &attributes, vertex_index) {
            let x = sample_to_frame_coord(sample.x, frame.width);
            let y = sample_to_frame_coord(-sample.y, frame.height);
            stamp_vertex_marker(&mut rows, x, y, sample.glyph);
            samples.push((x, y, sample.glyph));
        }
    }
    draw_topology_edges(&mut rows, &samples, topology);

    for (x, y, glyph) in samples {
        stamp_vertex_marker(&mut rows, x, y, glyph);
    }

    for (row, chars) in frame.rows.iter_mut().zip(rows) {
        *row = chars.into_iter().collect();
    }
}

fn draw_topology_edges(rows: &mut [Vec<char>], samples: &[(usize, usize, char)], topology: &str) {
    match topology {
        "triangle_strip" => {
            for window in samples.windows(3) {
                let [(ax, ay, _), (bx, by, _), (cx, cy, _)] = [window[0], window[1], window[2]];
                stamp_triangle_fill(rows, ax, ay, bx, by, cx, cy, ',');
            }
            for window in samples.windows(2) {
                let [(ax, ay, _), (bx, by, _)] = [window[0], window[1]];
                stamp_line(rows, ax, ay, bx, by, '+');
            }
            for window in samples.windows(3) {
                let [(ax, ay, _), (_, _, _), (cx, cy, _)] = [window[0], window[1], window[2]];
                stamp_line(rows, ax, ay, cx, cy, '+');
            }
        }
        "triangle" => {
            for chunk in samples.chunks(3) {
                if chunk.len() == 3 {
                    let (ax, ay, _) = chunk[0];
                    let (bx, by, _) = chunk[1];
                    let (cx, cy, _) = chunk[2];
                    stamp_triangle_fill(rows, ax, ay, bx, by, cx, cy, ',');
                    stamp_line(rows, ax, ay, bx, by, '+');
                    stamp_line(rows, bx, by, cx, cy, '+');
                    stamp_line(rows, cx, cy, ax, ay, '+');
                }
            }
        }
        _ => {
            for window in samples.windows(2) {
                let [(ax, ay, _), (bx, by, _)] = [window[0], window[1]];
                stamp_line(rows, ax, ay, bx, by, '+');
            }
        }
    }
}

fn referenced_vertex_indices(geometry: &GeometryInputs, vertex_count: usize) -> Vec<usize> {
    if let Some(index_buffer) = &geometry.index_buffer {
        index_buffer
            .indices
            .iter()
            .copied()
            .take(vertex_count)
            .collect()
    } else {
        (0..vertex_count.min(geometry.vertex_buffer.vertex_count)).collect()
    }
}

struct VertexSample {
    x: f32,
    y: f32,
    glyph: char,
}

fn interpret_vertex(
    geometry: &GeometryInputs,
    attributes: &[VertexAttributeKind],
    vertex_index: usize,
) -> Option<VertexSample> {
    if vertex_index >= geometry.vertex_buffer.vertex_count {
        return None;
    }
    let stride = geometry.vertex_layout.stride;
    let base = vertex_index.checked_mul(stride)?;
    if geometry.vertex_buffer.elements.len() < base + stride {
        return None;
    }
    let slice = &geometry.vertex_buffer.elements[base..base + stride];

    let mut cursor = 0usize;
    let mut pos = None;
    let mut color = None;
    let mut uv = None;
    for attr in attributes {
        match attr {
            VertexAttributeKind::Pos2 => {
                if cursor + 2 <= slice.len() {
                    pos = Some((slice[cursor] as f32, slice[cursor + 1] as f32));
                }
                cursor += 2;
            }
            VertexAttributeKind::Color2 => {
                if cursor + 2 <= slice.len() {
                    color = Some((slice[cursor] as f32, slice[cursor + 1] as f32));
                }
                cursor += 2;
            }
            VertexAttributeKind::Uv2 => {
                if cursor + 2 <= slice.len() {
                    uv = Some((slice[cursor] as f32, slice[cursor + 1] as f32));
                }
                cursor += 2;
            }
            VertexAttributeKind::Unknown => {
                cursor += 1;
            }
        }
    }

    let (x, y) = pos?;
    let glyph = if let Some((u, v)) = uv {
        if u + v >= 1.5 {
            'u'
        } else {
            'v'
        }
    } else if let Some((r, g)) = color {
        match ((r + g) * 0.5).round() as i64 {
            value if value <= 0 => '#',
            1 => '%',
            _ => '@',
        }
    } else {
        '#'
    };

    Some(VertexSample { x, y, glyph })
}

fn parse_vertex_attribute_kind(raw: &str) -> VertexAttributeKind {
    match raw.trim() {
        "pos2f" => VertexAttributeKind::Pos2,
        "color2f" => VertexAttributeKind::Color2,
        "uv2f" => VertexAttributeKind::Uv2,
        _ => VertexAttributeKind::Unknown,
    }
}

fn sample_to_frame_coord(value: f32, extent: usize) -> usize {
    if extent <= 1 {
        return 0;
    }
    let normalized = ((value.clamp(-1.0, 1.0) + 1.0) * 0.5) * (extent as f32 - 1.0);
    normalized.round() as usize
}

fn stamp_vertex_marker(rows: &mut [Vec<char>], x: usize, y: usize, glyph: char) {
    if rows.is_empty() {
        return;
    }
    let height = rows.len();
    let width = rows[0].len();
    let positions = [
        (x, y),
        (x.saturating_sub(1), y),
        ((x + 1).min(width.saturating_sub(1)), y),
        (x, y.saturating_sub(1)),
        (x, (y + 1).min(height.saturating_sub(1))),
    ];
    for (px, py) in positions {
        if let Some(row) = rows.get_mut(py) {
            if let Some(cell) = row.get_mut(px) {
                *cell = glyph;
            }
        }
    }
}

fn stamp_line(rows: &mut [Vec<char>], ax: usize, ay: usize, bx: usize, by: usize, glyph: char) {
    if rows.is_empty() {
        return;
    }

    let mut x0 = ax as isize;
    let mut y0 = ay as isize;
    let x1 = bx as isize;
    let y1 = by as isize;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if let Some(row) = rows.get_mut(y0.max(0) as usize) {
            if let Some(cell) = row.get_mut(x0.max(0) as usize) {
                if *cell == '.' {
                    *cell = glyph;
                }
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn stamp_triangle_fill(
    rows: &mut [Vec<char>],
    ax: usize,
    ay: usize,
    bx: usize,
    by: usize,
    cx: usize,
    cy: usize,
    glyph: char,
) {
    if rows.is_empty() || rows[0].is_empty() {
        return;
    }

    let min_x = ax.min(bx).min(cx);
    let max_x = ax.max(bx).max(cx).min(rows[0].len().saturating_sub(1));
    let min_y = ay.min(by).min(cy);
    let max_y = ay.max(by).max(cy).min(rows.len().saturating_sub(1));

    let axf = ax as f32;
    let ayf = ay as f32;
    let bxf = bx as f32;
    let byf = by as f32;
    let cxf = cx as f32;
    let cyf = cy as f32;

    let area = edge_function(axf, ayf, bxf, byf, cxf, cyf);
    if area.abs() < f32::EPSILON {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pxf = x as f32 + 0.5;
            let pyf = y as f32 + 0.5;
            let w0 = edge_function(bxf, byf, cxf, cyf, pxf, pyf);
            let w1 = edge_function(cxf, cyf, axf, ayf, pxf, pyf);
            let w2 = edge_function(axf, ayf, bxf, byf, pxf, pyf);
            let all_positive = w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0;
            let all_negative = w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0;
            if all_positive || all_negative {
                if let Some(row) = rows.get_mut(y) {
                    if let Some(cell) = row.get_mut(x) {
                        if *cell == '.' {
                            *cell = glyph;
                        }
                    }
                }
            }
        }
    }
}

fn edge_function(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

use yir_core::{
    BlendState, DepthState, ExecutionState, FrameSurface, IndexBuffer, InstructionSemantics, Node,
    RasterState, RegisteredMod, RenderPass, RenderPipeline, RenderStateSet, Resource, SamplerState,
    ShaderBinding, ShaderBindingSet, ShaderFlowState, ShaderResultHandle, StructValue,
    SurfaceTarget, Texture2D, Value, VertexBuffer, VertexLayout, Viewport,
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
            "inline_wgsl" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.inline_wgsl <name> <resource> <entry> <source>`",
                        node.name
                    ));
                }
                let entry = node.op.args[0].trim();
                let source = node.op.args[1].trim();
                if entry.is_empty() {
                    return Err(format!("node `{}` has empty inline_wgsl entry", node.name));
                }
                if source.is_empty() {
                    return Err(format!("node `{}` has empty inline_wgsl source", node.name));
                }
                if !source.contains("@vertex") || !source.contains("@fragment") {
                    return Err(format!(
                        "node `{}` inline_wgsl source must contain both @vertex and @fragment",
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
            "observe" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `shader.observe <name> <resource> <input> <state>`",
                        node.name
                    ));
                }
                parse_shader_flow_state(&node.op.args[1]).map_err(|error| {
                    format!(
                        "node `{}` has invalid shader observe state: {error}",
                        node.name
                    )
                })?;
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            "is_pass_ready" | "is_frame_ready" | "value" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `shader.{} <name> <resource> <result>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
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

                let mut deps = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                if node.op.args[2].parse::<i64>().is_err() {
                    deps.push(node.op.args[2].clone());
                }
                if node.op.args[3].parse::<i64>().is_err() {
                    deps.push(node.op.args[3].clone());
                }
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
            "inline_wgsl" => Ok(Value::Struct(StructValue {
                type_name: "ShaderInlineWgsl".to_owned(),
                fields: vec![
                    ("entry".to_owned(), Value::Symbol(node.op.args[0].clone())),
                    ("source".to_owned(), Value::Symbol(node.op.args[1].clone())),
                ],
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
            "observe" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let flow = parse_shader_flow_state(&node.op.args[1])?;
                Ok(Value::ShaderResult(ShaderResultHandle {
                    state: flow,
                    value: Box::new(value),
                }))
            }
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
                let packet = unwrap_data_window(state.expect_value(&node.op.args[1])?.clone());
                let vertex_count = resolve_draw_count(state, node, 2, "vertex_count")?;
                let instance_count = resolve_draw_count(state, node, 3, "instance_count")?;
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

fn draw_control_panel_surface(
    value: &Value,
    width: usize,
    height: usize,
) -> Result<FrameSurface, String> {
    let packet = parse_ball_packet(value, "shader.draw_instanced")?;
    let width = width.max(32);
    let height = height.max(24);
    let color_value = normalize_control_value(packet.color_key, packet.color_min, packet.color_max);
    let speed_value = normalize_control_value(
        packet.speed.round() as i64,
        packet.speed_min,
        packet.speed_max,
    );
    let radius_value = normalize_control_value(
        (packet.radius_scale * 96.0).round() as i64,
        packet.radius_min,
        packet.radius_max,
    );
    let progress_value = normalize_control_value(packet.progress_value, 0, packet.progress_max);
    let meter_value = normalize_control_value(packet.meter_value, 0, packet.meter_max);
    let accent = control_panel_accent(packet.accent, packet.contrast);
    let button_on = packet.button_state != 0;
    let toggle_disabled = packet.toggle_disabled != 0;
    let viewport_shift_x = packet.viewport_x.rem_euclid(3) as usize;
    let viewport_shift_y = packet.viewport_y.rem_euclid(2) as usize;
    let viewport_width = packet.viewport_width.max(24) as usize;
    let viewport_height = packet.viewport_height.max(12) as usize;
    let layer_hidden = packet.layer_visibility == 0;
    let blend_fill = match packet.layer_blend.rem_euclid(3) {
        0 => '.',
        1 => ':',
        _ => ';',
    };

    let mut rows = vec![vec![' '; width]; height];
    fill_panel_background(
        &mut rows,
        packet.surface,
        packet.contrast + packet.surface_density + packet.surface_sheen,
    );
    let panel_left = 2usize;
    let panel_top = 1usize;
    let panel_right = width.saturating_sub(3);
    let panel_bottom = height.saturating_sub(2);
    let viewport_left = (panel_left + 2 + viewport_shift_x).min(panel_right.saturating_sub(8));
    let viewport_top = (panel_top + 5 + viewport_shift_y).min(panel_bottom.saturating_sub(8));
    let viewport_right = (viewport_left + viewport_width)
        .min(panel_right.saturating_sub(30))
        .max(viewport_left + 8);
    let viewport_bottom = (viewport_top + viewport_height)
        .min(panel_bottom.saturating_sub(1))
        .max(viewport_top + 6);

    draw_box(
        &mut rows,
        panel_left,
        panel_top,
        panel_right,
        panel_bottom,
        '/',
        '\\',
        '\\',
        '/',
        '-',
        '|',
    );
    fill_rect(
        &mut rows,
        panel_left + 1,
        panel_top + 1,
        panel_right.saturating_sub(1),
        panel_bottom.saturating_sub(1),
        '.',
    );
    draw_card(
        &mut rows,
        viewport_left,
        viewport_top,
        viewport_right,
        viewport_bottom,
        accent,
        if packet.panel_mode.rem_euclid(2) == 0 {
            blend_fill
        } else {
            if packet.surface_grid.rem_euclid(2) == 0 {
                ':'
            } else {
                ';'
            }
        },
    );
    draw_card(
        &mut rows,
        panel_right.saturating_sub(29),
        panel_top + 3,
        panel_right.saturating_sub(2),
        panel_top + 13,
        accent,
        ':',
    );
    draw_card(
        &mut rows,
        panel_right.saturating_sub(29),
        panel_top + 14,
        panel_right.saturating_sub(2),
        panel_top + 20,
        accent,
        '.',
    );
    draw_card(
        &mut rows,
        panel_left + 2,
        panel_bottom.saturating_sub(9),
        panel_right.saturating_sub(30),
        panel_bottom.saturating_sub(1),
        accent,
        if packet.layer_clip.rem_euclid(2) == 0 {
            ':'
        } else {
            '.'
        },
    );
    let status_bar_left = panel_left + 3;
    let status_bar_right = panel_right.saturating_sub(4);
    fill_rect(
        &mut rows,
        status_bar_left,
        panel_top + 1,
        status_bar_right,
        panel_top + 1,
        '=',
    );
    put_text(
        &mut rows,
        panel_left + 3,
        panel_top + 2,
        if packet.header_title_mode.rem_euclid(2) == 0 {
            if packet.panel_mode.rem_euclid(2) == 0 {
                "ns-nova control panel"
            } else {
                "ns-nova studio workspace"
            }
        } else {
            if packet.panel_mode.rem_euclid(2) == 0 {
                "ns-nova reactive controls"
            } else {
                "ns-nova reactive cockpit"
            }
        },
    );
    put_text(
        &mut rows,
        panel_left + 3,
        panel_top + 3,
        "range / button / meter / text / select",
    );
    put_text(
        &mut rows,
        panel_left + 34,
        panel_top + 3,
        "list / table / tree / inspector / outline",
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 2,
        &format!(
            "vp {}x{} @{},{}",
            viewport_width, viewport_height, packet.viewport_x, packet.viewport_y
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 4,
        &format!(
            "layer o{} b{} {}",
            packet.layer_order,
            packet.layer_blend,
            if layer_hidden { "hidden" } else { "live" }
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 5,
        &format!(
            "surf d{} e{} g{}",
            packet.surface_density, packet.surface_elevation, packet.surface_grid
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 6,
        &format!(
            "scene r{} l{} a{}",
            packet.scene_root_count, packet.scene_light_count, packet.scene_animation_phase
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 7,
        &format!(
            "cam k{} f{} z{}",
            packet.camera_kind, packet.camera_focus, packet.camera_zoom
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 8,
        &format!(
            "mat s{} r{} e{}",
            packet.material_shader_kind, packet.material_roughness, packet.material_emissive
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 9,
        &format!(
            "lit k{} i{} r{}",
            packet.light_kind, packet.light_intensity, packet.light_range
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 10,
        &format!(
            "mesh p{} v{} i{}",
            packet.mesh_primitive, packet.mesh_vertex_count, packet.mesh_index_count
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 11,
        &format!("skin {:>3}", packet.mesh_skinning),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 12,
        &format!(
            "xform t{} r{} s{}",
            packet.transform_translate, packet.transform_rotate, packet.transform_scale
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 13,
        &format!("pivot {:>3}", packet.transform_pivot),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 14,
        &format!(
            "node {}<-{} d{}",
            packet.node_id, packet.node_parent_id, packet.node_depth
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 15,
        &format!("flags {:>3}", packet.node_flags),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 21,
        &format!(
            "link n{} m{}",
            packet.scene_link_node_slot, packet.scene_link_mesh_slot
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 22,
        &format!(
            "mat{} lit{}",
            packet.scene_link_material_slot, packet.scene_link_light_slot
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 16,
        &format!(
            "pass s{} c{} x{}",
            packet.pass_stage, packet.pass_clear_mode, packet.pass_sample_count
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 17,
        &format!("dbg {:>3}", packet.pass_debug_view),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 18,
        &format!(
            "frm {:>3} pm{} v{}",
            packet.frame_index, packet.frame_present_mode, packet.frame_sync_interval
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 19,
        &format!("exp {:>3}", packet.frame_exposure),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 20,
        &format!(
            "tgt k{} {:>2}x{:>2}",
            packet.target_kind, packet.target_width, packet.target_height
        ),
    );
    draw_scene_preview(
        &mut rows,
        viewport_left,
        viewport_top,
        viewport_right,
        viewport_bottom,
        &packet,
        accent,
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 21,
        &format!("msaa {:>2}", packet.target_multisample),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 22,
        &format!(
            "fg p{} t{} ps{}",
            packet.frame_graph_passes, packet.frame_graph_targets, packet.frame_graph_present_stage
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 23,
        &format!("ovr {:>3}", packet.frame_graph_debug_overlay),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 24,
        &format!(
            "att {} f{} l{} s{}",
            packet.attachment_slot,
            packet.attachment_format_kind,
            packet.attachment_load_op,
            packet.attachment_store_op
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 25,
        &format!(
            "pch s{} f{} r{}",
            packet.pass_chain_stages, packet.pass_chain_fanout, packet.pass_chain_resolve_stage
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 26,
        &format!("bar {:>3}", packet.pass_chain_barrier_mode),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 27,
        &format!(
            "sync {} {}>{}",
            packet.barrier_scope, packet.barrier_source_stage, packet.barrier_target_stage
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 28,
        &format!("flush {:>2}", packet.barrier_flush_mode),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 29,
        &format!(
            "rs b{} t{} s{}",
            packet.resource_buffers, packet.resource_textures, packet.resource_samplers
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 30,
        &format!("res {:>3}", packet.resource_residency),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 31,
        &format!(
            "sch l{} q{}",
            packet.schedule_lanes, packet.schedule_queue_depth
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 32,
        &format!(
            "ab {:>3} tm{}",
            packet.schedule_async_budget, packet.schedule_tick_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 33,
        &format!(
            "sub b{} f{}",
            packet.submission_batches, packet.submission_fences
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 34,
        &format!(
            "sig {} ph{}",
            packet.submission_signal_mode, packet.submission_present_hint
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 35,
        &format!("q k{} p{}", packet.queue_kind, packet.queue_priority),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 36,
        &format!("qb {:>3} ow{}", packet.queue_budget, packet.queue_ownership),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 37,
        &format!(
            "sem w{} s{}",
            packet.semaphore_wait_count, packet.semaphore_signal_count
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 38,
        &format!(
            "tm {} sc{}",
            packet.semaphore_timeline_mode, packet.semaphore_scope
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 39,
        &format!("tl v{} st{}", packet.timeline_value, packet.timeline_step),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 40,
        &format!("ep {} dm{}", packet.timeline_epoch, packet.timeline_domain),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 41,
        &format!("fn s{} e{}", packet.fence_signaled, packet.fence_epoch),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 42,
        &format!("fs {} rc{}", packet.fence_scope, packet.fence_recycle_mode),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 43,
        &format!("sg k{} ph{}", packet.signal_kind, packet.signal_phase),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 44,
        &format!("sf {} ak{}", packet.signal_fanout, packet.signal_ack_mode),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 45,
        &format!("ev k{} rt{}", packet.event_kind, packet.event_route),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 46,
        &format!(
            "ep {} pm{}",
            packet.event_priority, packet.event_payload_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 47,
        &format!(
            "dp q{} l{}",
            packet.dispatch_queue_kind, packet.dispatch_lane
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 48,
        &format!(
            "db {} cm{}",
            packet.dispatch_batch, packet.dispatch_completion_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 49,
        &format!(
            "fb st{} lt{}",
            packet.feedback_status, packet.feedback_latency
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 50,
        &format!(
            "fr {} ch{}",
            packet.feedback_retries, packet.feedback_channel
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 51,
        &format!("in k{} tg{}", packet.intent_kind, packet.intent_target),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 52,
        &format!("iu {} pl{}", packet.intent_urgency, packet.intent_policy),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 53,
        &format!(
            "rk {} rs{}",
            packet.reaction_kind, packet.reaction_result_slot
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 54,
        &format!(
            "rb {} em{}",
            packet.reaction_stability, packet.reaction_echo_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 55,
        &format!("ok {} fs{}", packet.outcome_kind, packet.outcome_final_slot),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 56,
        &format!(
            "oc {} sm{}",
            packet.outcome_confidence, packet.outcome_settle_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 57,
        &format!(
            "rs {} cs{}",
            packet.resolution_kind, packet.resolution_commit_slot
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 58,
        &format!(
            "rc {} pm{}",
            packet.resolution_convergence, packet.resolution_policy_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 59,
        &format!("cm {} as{}", packet.commit_kind, packet.commit_applied_slot),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 60,
        &format!(
            "cd {} md{}",
            packet.commit_durability, packet.commit_commit_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 61,
        &format!(
            "sn {} ss{}",
            packet.snapshot_kind, packet.snapshot_source_slot
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 62,
        &format!(
            "sr {} rm{}",
            packet.snapshot_retention, packet.snapshot_replay_mode
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 63,
        &format!(
            "ck {} as{}",
            packet.checkpoint_kind, packet.checkpoint_anchor_slot
        ),
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(26),
        panel_top + 64,
        &format!(
            "cr {} rm{}",
            packet.checkpoint_rollback_depth, packet.checkpoint_resume_mode
        ),
    );
    if layer_hidden {
        put_text(
            &mut rows,
            viewport_left + 3,
            viewport_top + 2,
            "layer hidden: overlay retained for debug",
        );
    }
    put_text(
        &mut rows,
        panel_right.saturating_sub(18),
        panel_top + 2,
        if toggle_disabled {
            "mode: locked"
        } else if packet.toggle_state != 0 {
            "mode: live"
        } else {
            "mode: idle"
        },
    );
    put_text(
        &mut rows,
        panel_left + 3,
        panel_top + 1,
        if packet.select_committed != 0 {
            "nova scene live graph"
        } else {
            "nova scene staging graph"
        },
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(28),
        panel_top + 1,
        &format!(
            "theme s{} m{} c{}",
            packet.surface.rem_euclid(10),
            packet.panel_mode.rem_euclid(10),
            packet.contrast.rem_euclid(10)
        ),
    );

    let slider_left = panel_left + 16;
    let slider_right = panel_right.saturating_sub(12);
    let slider_width = slider_right.saturating_sub(slider_left + 1).max(8);
    let slider_specs = [
        (
            "COLOR",
            color_value,
            panel_top + 6,
            packet.color_disabled != 0,
            packet.color_min,
            packet.color_max,
            packet.color_step,
        ),
        (
            "SPEED",
            speed_value,
            panel_top + 10,
            packet.speed_disabled != 0,
            packet.speed_min,
            packet.speed_max,
            packet.speed_step,
        ),
        (
            "RADIUS",
            radius_value,
            panel_top + 14,
            packet.radius_disabled != 0,
            packet.radius_min,
            packet.radius_max,
            packet.radius_step,
        ),
    ];
    for (label, value, y, disabled, min_value, max_value, step_value) in slider_specs {
        put_text(&mut rows, panel_left + 3, y, label);
        draw_slider(
            &mut rows,
            slider_left,
            y,
            slider_width,
            value.min(127),
            if disabled { ':' } else { accent },
        );
        put_text(
            &mut rows,
            slider_right + 2,
            y,
            &format!("{:>3}", value.min(127)),
        );
        let meta = format!("{min_value}..{max_value} /{step_value}");
        put_text(&mut rows, slider_left, y.saturating_sub(1), &meta);
        if disabled {
            put_text(&mut rows, slider_right + 7, y, "off");
        }
    }

    put_text(
        &mut rows,
        panel_left + 4,
        panel_top + 12,
        &format!(
            "camera orbit {:>3}  albedo {:>3}",
            packet.camera_orbit, packet.material_albedo
        ),
    );
    put_text(
        &mut rows,
        panel_left + 4,
        panel_top + 13,
        &format!(
            "scene roots {:>2}  active {:>2}",
            packet.scene_root_count, packet.scene_active_camera
        ),
    );
    put_text(
        &mut rows,
        panel_left + 4,
        panel_top + 14,
        &format!("light reactive {:>3}", packet.light_reactive),
    );

    let progress_y = panel_top + 17;
    put_text(
        &mut rows,
        panel_left + 3,
        progress_y.saturating_sub(1),
        "frame",
    );
    put_text(&mut rows, panel_left + 3, progress_y, "PROGRESS");
    draw_slider(
        &mut rows,
        slider_left,
        progress_y,
        slider_width,
        progress_value.min(127),
        accent,
    );
    put_text(
        &mut rows,
        slider_right + 2,
        progress_y,
        &format!("{:>3}", progress_value.min(127)),
    );

    let meter_y = panel_top + 19;
    put_text(
        &mut rows,
        panel_left + 3,
        meter_y.saturating_sub(1),
        "energy",
    );
    put_text(&mut rows, panel_left + 3, meter_y, "METER");
    draw_slider(
        &mut rows,
        slider_left,
        meter_y,
        slider_width,
        meter_value.min(127),
        accent,
    );
    put_text(
        &mut rows,
        slider_right + 2,
        meter_y,
        &format!("{:>3}", meter_value.min(127)),
    );

    let button_left = panel_right.saturating_sub(15);
    let button_right = panel_right.saturating_sub(4);
    let button_top = panel_top + 3;
    let button_bottom = button_top + 3;
    draw_box(
        &mut rows,
        button_left,
        button_top,
        button_right,
        button_bottom,
        '[',
        ']',
        ']',
        '[',
        '=',
        '|',
    );
    fill_rect(
        &mut rows,
        button_left + 1,
        button_top + 1,
        button_right.saturating_sub(1),
        button_bottom.saturating_sub(1),
        if toggle_disabled {
            '.'
        } else if button_on {
            accent
        } else {
            ':'
        },
    );
    put_text(
        &mut rows,
        button_left + 2,
        button_top + 1,
        match packet.button_intent.rem_euclid(3) {
            _ if toggle_disabled => "LOCK ",
            0 if button_on => "APPLY",
            0 => "READY",
            1 if button_on => "LIVE ",
            1 => "ARM  ",
            _ if button_on => "SYNC ",
            _ => "HOLD ",
        },
    );
    put_text(
        &mut rows,
        button_left + 2,
        button_bottom,
        if button_on { "pulse" } else { "standby" },
    );

    let knob_center_x = panel_left + 8;
    let knob_center_y = panel_bottom.saturating_sub(5);
    draw_knob(
        &mut rows,
        knob_center_x,
        knob_center_y,
        4,
        radius_value.min(127),
        accent,
    );
    put_text(
        &mut rows,
        panel_left + 3,
        panel_bottom.saturating_sub(1),
        "gain",
    );

    let text_left = panel_left + 4;
    let text_right = panel_left + 24;
    let text_top = panel_bottom.saturating_sub(5);
    let text_bottom = text_top + 2;
    draw_box(
        &mut rows,
        text_left,
        text_top,
        text_right,
        text_bottom,
        '[',
        ']',
        ']',
        '[',
        '-',
        '|',
    );
    let text_value = format!("nova-{:03}", packet.text_echo.abs() % 1000);
    put_text(&mut rows, text_left + 2, text_top + 1, &text_value);
    if packet.text_placeholder.rem_euclid(2) != 0 {
        put_text(&mut rows, text_left + 2, text_top, "query");
    }
    if packet.text_read_only != 0 {
        put_text(&mut rows, text_right.saturating_sub(6), text_top, "ro");
    }
    if packet.text_dirty != 0 {
        put_text(&mut rows, text_right.saturating_sub(12), text_top, "dirty");
    }
    let caret_x =
        text_left + 2 + (packet.text_caret.rem_euclid(text_value.len() as i64 + 1) as usize);
    if caret_x < text_right {
        rows[text_bottom][caret_x] = accent;
    }

    let select_left = panel_right.saturating_sub(28);
    let select_y = panel_bottom.saturating_sub(3);
    let option_count = packet.select_options.clamp(2, 4) as usize;
    let labels = match option_count {
        2 => ["AUTO", "MAN ", "", ""],
        3 => ["AUTO", "MAN ", "GPU ", ""],
        _ => ["AUTO", "MAN ", "GPU ", "CPU "],
    };
    put_text(&mut rows, select_left, select_y, labels[0]);
    put_text(&mut rows, select_left + 7, select_y, labels[1]);
    if option_count >= 3 {
        put_text(&mut rows, select_left + 13, select_y, labels[2]);
    }
    if option_count >= 4 {
        put_text(&mut rows, select_left + 19, select_y, labels[3]);
    }
    if packet.select_multiple != 0 {
        put_text(&mut rows, select_left, select_y.saturating_sub(1), "multi");
    }
    put_text(
        &mut rows,
        select_left + 22,
        select_y,
        if packet.select_committed != 0 {
            "ok"
        } else {
            "pending"
        },
    );
    let selected_x = match packet.select_index.rem_euclid(option_count as i64) {
        0 => select_left.saturating_sub(2),
        1 => select_left + 5,
        2 => select_left + 11,
        _ => select_left + 17,
    };
    put_text(&mut rows, selected_x, select_y, ">");

    let checkbox_y = panel_top + 6;
    let checkbox_left = button_left;
    put_text(&mut rows, checkbox_left, checkbox_y, "CHECK");
    put_text(
        &mut rows,
        checkbox_left,
        checkbox_y + 1,
        if packet.checkbox_disabled != 0 {
            "[~] disabled"
        } else if packet.checkbox_checked != 0 {
            "[x] enabled "
        } else {
            "[ ] enabled "
        },
    );

    let radio_y = panel_top + 10;
    let radio_left = button_left;
    put_text(&mut rows, radio_left, radio_y, "RADIO");
    let radio_count = packet.radio_options.clamp(2, 4) as usize;
    for idx in 0..radio_count {
        let label = match idx {
            0 => "fast",
            1 => "safe",
            2 => "gpu ",
            _ => "cpu ",
        };
        let mark = if packet.radio_selected.rem_euclid(radio_count as i64) as usize == idx {
            "(*)"
        } else {
            "( )"
        };
        put_text(
            &mut rows,
            radio_left,
            radio_y + 1 + idx,
            &format!("{mark} {label}"),
        );
    }
    if packet.radio_disabled != 0 {
        put_text(&mut rows, radio_left + 8, radio_y, "off");
    }

    let tabs_y = panel_top + 4;
    let tabs_count = packet.tabs_count.clamp(2, 4) as usize;
    for idx in 0..tabs_count {
        let label = match idx {
            0 => "scene",
            1 => "logic",
            2 => "perf ",
            _ => "gpu  ",
        };
        let active = packet.tabs_active.rem_euclid(tabs_count as i64) as usize == idx;
        let compact = packet.tabs_compact != 0;
        let text = if active {
            if compact {
                "[*]"
            } else {
                "[tab]"
            }
        } else if compact {
            "[ ]"
        } else {
            "[---]"
        };
        put_text(
            &mut rows,
            panel_left + 3 + idx * 10,
            tabs_y,
            &format!("{text} {}", &label[..label.len().min(5)]),
        );
    }

    let textarea_left = panel_left + 27;
    let textarea_right = panel_right.saturating_sub(30);
    let textarea_top = panel_bottom.saturating_sub(8);
    let textarea_bottom = textarea_top + 4;
    draw_box(
        &mut rows,
        textarea_left,
        textarea_top,
        textarea_right,
        textarea_bottom,
        '[',
        ']',
        ']',
        '[',
        '-',
        '|',
    );
    put_text(&mut rows, textarea_left + 2, textarea_top, "notes");
    let visible_lines = packet.textarea_lines.clamp(2, 3) as usize;
    for line in 0..visible_lines {
        let scroll = packet.textarea_scroll.rem_euclid(9) as usize;
        let text = format!(
            "line {} :: {}",
            line + 1 + scroll,
            packet.textarea_placeholder
        );
        put_text(&mut rows, textarea_left + 2, textarea_top + 1 + line, &text);
    }
    if packet.textarea_read_only != 0 {
        put_text(
            &mut rows,
            textarea_right.saturating_sub(6),
            textarea_top,
            "ro",
        );
    }
    if packet.textarea_dirty != 0 {
        put_text(
            &mut rows,
            textarea_right.saturating_sub(13),
            textarea_top,
            "dirty",
        );
    }

    let list_left = panel_left + 3;
    let list_top = panel_bottom.saturating_sub(8);
    put_text(&mut rows, list_left, list_top, "list");
    let list_items = packet.list_items.clamp(3, 5) as usize;
    for idx in 0..list_items {
        let marker = if packet.list_selected.rem_euclid(list_items as i64) as usize == idx {
            ">"
        } else {
            " "
        };
        let row = if packet.list_dense != 0 {
            format!("{marker} item-{}", idx + 1)
        } else {
            format!(
                "{marker} row {}  accent {}",
                idx + 1,
                packet.accent.rem_euclid(9)
            )
        };
        put_text(&mut rows, list_left, list_top + 1 + idx, &row);
    }

    let table_left = panel_left + 50;
    let table_top = panel_bottom.saturating_sub(8);
    put_text(&mut rows, table_left, table_top, "table");
    let rows_count = packet.table_rows.clamp(2, 4) as usize;
    let cols_count = packet.table_cols.clamp(2, 4) as usize;
    let mut header = String::from("+");
    for _ in 0..cols_count {
        header.push_str("---+");
    }
    put_text(&mut rows, table_left, table_top + 1, &header);
    for row_idx in 0..rows_count {
        let mut body = String::from("|");
        for col_idx in 0..cols_count {
            let glyph = if packet.table_zebra != 0 && row_idx % 2 == 1 {
                ':'
            } else {
                '.'
            };
            let active =
                packet.table_selected_row.rem_euclid(rows_count as i64) as usize == row_idx;
            let cell = if active && col_idx == 0 {
                format!(">{glyph}{glyph}")
            } else {
                format!("{glyph}{glyph}{glyph}")
            };
            body.push_str(&cell);
            body.push('|');
        }
        put_text(&mut rows, table_left, table_top + 2 + row_idx, &body);
    }

    let tree_left = panel_right.saturating_sub(26);
    let tree_top = panel_top + 15;
    put_text(&mut rows, tree_left, tree_top, "tree");
    let node_count = packet.tree_nodes.clamp(3, 6) as usize;
    for idx in 0..node_count {
        let selected = packet.tree_selected.rem_euclid(node_count as i64) as usize == idx;
        let expanded = packet.tree_expanded != 0;
        let prefix = match idx {
            0 => {
                if expanded {
                    "v root"
                } else {
                    "> root"
                }
            }
            1 | 2 => "  |- child",
            _ => "  `- leaf ",
        };
        let line = if selected {
            format!("> {prefix}{}", idx + 1)
        } else {
            format!("  {prefix}{}", idx + 1)
        };
        put_text(&mut rows, tree_left, tree_top + 1 + idx, &line);
    }

    let inspector_left = panel_right.saturating_sub(26);
    let inspector_top = panel_top + 4;
    put_text(&mut rows, inspector_left, inspector_top, "inspector");
    put_text(
        &mut rows,
        inspector_left,
        inspector_top + 1,
        if packet.inspector_pinned != 0 {
            "[pin] locked"
        } else {
            "[pin] float "
        },
    );
    let inspector_fields = packet.inspector_fields.clamp(2, 4) as usize;
    for idx in 0..inspector_fields {
        let selected = packet
            .inspector_selected
            .rem_euclid(inspector_fields as i64) as usize
            == idx;
        let line = if selected {
            format!(
                "> field_{} = {}",
                idx + 1,
                packet.accent.rem_euclid(9) + idx as i64
            )
        } else {
            format!(
                "  field_{} = {}",
                idx + 1,
                packet.accent.rem_euclid(9) + idx as i64
            )
        };
        put_text(&mut rows, inspector_left, inspector_top + 2 + idx, &line);
    }

    let outline_left = panel_left + 3;
    let outline_top = panel_top + 17;
    put_text(&mut rows, outline_left, outline_top, "outline");
    let outline_items = packet.outline_items.clamp(3, 6) as usize;
    for idx in 0..outline_items {
        let selected = packet.outline_selected.rem_euclid(outline_items as i64) as usize == idx;
        let collapsed = packet.outline_collapsed != 0;
        let line = if idx == 0 {
            if selected {
                if collapsed {
                    "> > section".to_owned()
                } else {
                    "> v section".to_owned()
                }
            } else if collapsed {
                "  > section".to_owned()
            } else {
                "  v section".to_owned()
            }
        } else if collapsed {
            if selected {
                format!("> hidden {}", idx + 1)
            } else {
                format!("  hidden {}", idx + 1)
            }
        } else if selected {
            format!("> item {}", idx + 1)
        } else {
            format!("  item {}", idx + 1)
        };
        put_text(&mut rows, outline_left, outline_top + 1 + idx, &line);
    }

    let focus_target = packet.focus_index.rem_euclid(6) as usize;
    let focus_marker = match focus_target {
        0 => (slider_left.saturating_sub(3), panel_top + 6),
        1 => (slider_left.saturating_sub(3), panel_top + 10),
        2 => (slider_left.saturating_sub(3), panel_top + 14),
        3 => (button_left.saturating_sub(2), button_top + 1),
        4 => (text_left.saturating_sub(2), text_top + 1),
        _ => (select_left.saturating_sub(2), select_y),
    };
    if focus_marker.1 < rows.len() && focus_marker.0 < rows[focus_marker.1].len() {
        rows[focus_marker.1][focus_marker.0] = '>';
    }

    let rows = rows
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>();
    Ok(FrameSurface {
        width,
        height,
        rows,
    })
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
        "control_panel" | "nova_controls" | "ui_controls" => {
            draw_control_panel_surface(packet, width, height)
        }
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

fn control_panel_accent(color: i64, contrast: i64) -> char {
    match (color + contrast).rem_euclid(5) {
        0 => '#',
        1 => '*',
        2 => '@',
        3 => '+',
        _ => '%',
    }
}

fn draw_slider(
    rows: &mut [Vec<char>],
    left: usize,
    y: usize,
    width: usize,
    value: usize,
    accent: char,
) {
    if y >= rows.len() || left >= rows[y].len() || width < 4 {
        return;
    }
    let right = left.saturating_add(width.saturating_sub(1));
    if right >= rows[y].len() {
        return;
    }
    rows[y][left] = '[';
    rows[y][right] = ']';
    for x in (left + 1)..right {
        rows[y][x] = '-';
    }
    let inner = right.saturating_sub(left + 1).max(1);
    let fill = (value.min(127) * inner) / 127;
    for x in 0..fill.min(inner) {
        rows[y][left + 1 + x] = '=';
    }
    let knob_x = left + 1 + fill.min(inner.saturating_sub(1));
    rows[y][knob_x] = accent;
}

fn draw_knob(
    rows: &mut [Vec<char>],
    cx: usize,
    cy: usize,
    radius: usize,
    value: usize,
    accent: char,
) {
    if rows.is_empty() || radius == 0 {
        return;
    }
    let angle =
        (value.min(127) as f32 / 127.0) * std::f32::consts::PI * 1.5 + std::f32::consts::PI * 0.75;
    let needle_x = cx as f32 + angle.cos() * radius as f32 * 0.7;
    let needle_y = cy as f32 + angle.sin() * radius as f32 * 0.7;
    for y in cy.saturating_sub(radius)..=(cy + radius).min(rows.len().saturating_sub(1)) {
        for x in cx.saturating_sub(radius)..=(cx + radius).min(rows[y].len().saturating_sub(1)) {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;
            let dist2 = dx * dx + dy * dy;
            let r2 = (radius as isize) * (radius as isize);
            if dist2 <= r2 && dist2 >= r2.saturating_sub(radius as isize * 2) {
                rows[y][x] = 'o';
            }
        }
    }
    if cy < rows.len() && cx < rows[cy].len() {
        rows[cy][cx] = accent;
    }
    let nx = needle_x
        .round()
        .clamp(0.0, (rows[0].len().saturating_sub(1)) as f32) as usize;
    let ny = needle_y
        .round()
        .clamp(0.0, (rows.len().saturating_sub(1)) as f32) as usize;
    stamp_line(rows, cx, cy, nx, ny, accent);
    if ny < rows.len() && nx < rows[ny].len() {
        rows[ny][nx] = accent;
    }
}

fn fill_panel_background(rows: &mut [Vec<char>], surface: i64, contrast: i64) {
    let palette = match (surface + contrast).rem_euclid(5) {
        0 => [' ', '.', '.', ':'],
        1 => [' ', '.', ':', '*'],
        2 => [' ', '.', '`', '+'],
        3 => [' ', '.', '.', '='],
        _ => [' ', '·', '.', ':'],
    };
    for (y, row) in rows.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let band = ((x / 7) + (y / 3)) % palette.len();
            *cell = palette[band];
        }
    }
}

fn draw_card(
    rows: &mut [Vec<char>],
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    accent: char,
    fill: char,
) {
    if right <= left + 1 || bottom <= top + 1 {
        return;
    }
    draw_box(
        rows, left, top, right, bottom, '.', '.', '\'', '\'', '-', '|',
    );
    fill_rect(
        rows,
        left + 1,
        top + 1,
        right.saturating_sub(1),
        bottom.saturating_sub(1),
        fill,
    );
    if top < rows.len() && left + 2 < rows[top].len() {
        rows[top][left + 2] = accent;
    }
    if top < rows.len() && right >= 2 && right - 2 < rows[top].len() {
        rows[top][right - 2] = accent;
    }
}

fn draw_box(
    rows: &mut [Vec<char>],
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    tl: char,
    tr: char,
    br: char,
    bl: char,
    horizontal: char,
    vertical: char,
) {
    if rows.is_empty() || top >= rows.len() || bottom >= rows.len() || left >= right {
        return;
    }
    for x in left..=right.min(rows[top].len().saturating_sub(1)) {
        rows[top][x] = horizontal;
        rows[bottom][x] = horizontal;
    }
    for y in top..=bottom {
        if left < rows[y].len() {
            rows[y][left] = vertical;
        }
        if right < rows[y].len() {
            rows[y][right] = vertical;
        }
    }
    if left < rows[top].len() {
        rows[top][left] = tl;
    }
    if right < rows[top].len() {
        rows[top][right] = tr;
    }
    if right < rows[bottom].len() {
        rows[bottom][right] = br;
    }
    if left < rows[bottom].len() {
        rows[bottom][left] = bl;
    }
}

fn fill_rect(
    rows: &mut [Vec<char>],
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    fill: char,
) {
    if rows.is_empty() {
        return;
    }
    for y in top..=bottom.min(rows.len().saturating_sub(1)) {
        for x in left..=right.min(rows[y].len().saturating_sub(1)) {
            rows[y][x] = fill;
        }
    }
}

fn put_text(rows: &mut [Vec<char>], left: usize, y: usize, text: &str) {
    if y >= rows.len() {
        return;
    }
    for (offset, ch) in text.chars().enumerate() {
        let x = left + offset;
        if x >= rows[y].len() {
            break;
        }
        rows[y][x] = ch;
    }
}

fn draw_scene_preview(
    rows: &mut [Vec<char>],
    viewport_left: usize,
    viewport_top: usize,
    viewport_right: usize,
    viewport_bottom: usize,
    packet: &BallPacket,
    accent: char,
) {
    if viewport_right <= viewport_left + 6 || viewport_bottom <= viewport_top + 5 {
        return;
    }

    let preview_left = viewport_left + 2;
    let preview_top = viewport_top + 2;
    let preview_right = viewport_right.saturating_sub(2);
    let preview_bottom = viewport_bottom.saturating_sub(2);
    let width = preview_right.saturating_sub(preview_left).max(6);
    let ground_y = preview_bottom.saturating_sub(1);
    let object_y = ground_y
        .saturating_sub(2 + packet.node_depth.rem_euclid(2) as usize)
        .saturating_sub(packet.transform_pivot.rem_euclid(2) as usize);
    let scene_phase = packet.transform_translate
        + packet.camera_orbit
        + packet.scene_link_node_slot
        + packet.frame_index;
    let object_x = preview_left + scene_phase.rem_euclid(width as i64) as usize;
    let light_x = preview_left + packet.light_range.rem_euclid(width as i64) as usize;
    let light_y = preview_top + packet.scene_link_light_slot.rem_euclid(3) as usize;
    let radius =
        ((packet.transform_scale.abs() + packet.mesh_vertex_count.abs()) / 24).clamp(1, 4) as usize;
    let glyph = match (packet.mesh_primitive + packet.material_shader_kind).rem_euclid(4) {
        0 => '#',
        1 => '@',
        2 => '%',
        _ => '&',
    };
    let shadow = if packet.layer_visibility == 0 {
        ':'
    } else {
        '_'
    };

    for x in preview_left..=preview_right {
        if x < rows[ground_y].len() {
            rows[ground_y][x] = if x % 2 == 0 { '_' } else { '.' };
        }
    }

    let shadow_left = object_x.saturating_sub(radius + 1).max(preview_left);
    let shadow_right = (object_x + radius + 1).min(preview_right);
    for x in shadow_left..=shadow_right {
        if x < rows[ground_y].len() {
            rows[ground_y][x] = shadow;
        }
    }

    if packet.light_intensity > 0 {
        if let Some(row) = rows.get_mut(light_y) {
            let light_slot = light_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(light_slot) {
                *cell = '*';
            }
        }
        stamp_line(rows, light_x, light_y, object_x, object_y, '.');
    }

    match packet.mesh_primitive.rem_euclid(3) {
        0 => {
            let top_y = object_y.saturating_sub(radius);
            let left_x = object_x.saturating_sub(radius).max(preview_left);
            let right_x = (object_x + radius).min(preview_right);
            stamp_line(rows, object_x, top_y, left_x, object_y + radius, glyph);
            stamp_line(rows, object_x, top_y, right_x, object_y + radius, glyph);
            stamp_line(
                rows,
                left_x,
                object_y + radius,
                right_x,
                object_y + radius,
                glyph,
            );
        }
        1 => {
            let left_x = object_x.saturating_sub(radius).max(preview_left);
            let right_x = (object_x + radius).min(preview_right);
            let top_y = object_y.saturating_sub(radius).max(preview_top);
            let bottom_y = (object_y + radius).min(ground_y.saturating_sub(1));
            draw_box(
                rows, left_x, top_y, right_x, bottom_y, glyph, glyph, glyph, glyph, glyph, glyph,
            );
            if left_x + 1 < right_x && top_y + 1 < bottom_y {
                fill_rect(
                    rows,
                    left_x + 1,
                    top_y + 1,
                    right_x - 1,
                    bottom_y - 1,
                    glyph,
                );
            }
        }
        _ => {
            let top_y = object_y.saturating_sub(radius);
            let bottom_y = (object_y + radius).min(ground_y.saturating_sub(1));
            let left_x = object_x.saturating_sub(radius).max(preview_left);
            let right_x = (object_x + radius).min(preview_right);
            stamp_line(rows, object_x, top_y, left_x, object_y, glyph);
            stamp_line(rows, object_x, top_y, right_x, object_y, glyph);
            stamp_line(rows, left_x, object_y, object_x, bottom_y, glyph);
            stamp_line(rows, right_x, object_y, object_x, bottom_y, glyph);
        }
    }

    if let Some(row) = rows.get_mut(object_y.min(rows.len().saturating_sub(1))) {
        let object_slot = object_x.min(row.len().saturating_sub(1));
        if let Some(cell) = row.get_mut(object_slot) {
            *cell = accent;
        }
    }

    let link_label = format!(
        "n{} t{} m{}",
        packet.scene_link_node_slot, packet.scene_link_transform_slot, packet.scene_link_mesh_slot
    );
    put_text(rows, preview_left, preview_bottom, &link_label);
    let material_label = format!(
        "mat{} lit{} ly{} i{}",
        packet.scene_link_material_slot,
        packet.scene_link_light_slot,
        packet.scene_link_layer_slot,
        packet.instance_node_slot
    );
    put_text(
        rows,
        preview_left,
        preview_bottom.saturating_sub(1),
        &material_label,
    );
    let instance_label = format!(
        "c{} s{} p{} l{}",
        packet.instance_count,
        packet.instance_stride,
        packet.instance_phase.rem_euclid(10),
        packet.instance_light_slot
    );
    put_text(rows, preview_left, preview_top, &instance_label);
    let graph_label = format!(
        "g{} l{} i{} a{}",
        packet.scene_graph_node_count,
        packet.scene_graph_link_count,
        packet.scene_graph_instance_count,
        packet.scene_graph_active_layer
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(1),
        &graph_label,
    );
    let scene_node_label = format!(
        "sn{} c{} s{} i{} v{}",
        packet.scene_node_slot,
        packet.scene_node_first_child_slot,
        packet.scene_node_sibling_slot,
        packet.scene_node_instance_slot,
        packet.scene_node_visibility
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(2),
        &scene_node_label,
    );
    let group_label = format!(
        "ig{} g{} v{} p{}",
        packet.instance_group_root_slot,
        packet.instance_group_count,
        packet.instance_group_visible_count,
        packet.instance_group_phase_bias.rem_euclid(10)
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(3),
        &group_label,
    );
    let cluster_label = format!(
        "cl{} n{} g{} l{}",
        packet.scene_cluster_root_slot,
        packet.scene_cluster_node_budget,
        packet.scene_cluster_instance_group_slot,
        packet.scene_cluster_layer_slot
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(4),
        &cluster_label,
    );
    let visibility_label = format!(
        "vs{} v{} o{} d{}",
        packet.visibility_cluster_slot,
        packet.visibility_visible_nodes,
        packet.visibility_occlusion_mode,
        packet.visibility_distance_band
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(5),
        &visibility_label,
    );
    let cull_label = format!(
        "cu{} k{} m{} l{}",
        packet.cull_cluster_slot, packet.cull_kept_nodes, packet.cull_mode, packet.cull_lod_band
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(6),
        &cull_label,
    );
    let lod_label = format!(
        "ld{} n{} a{} s{}",
        packet.lod_cluster_slot,
        packet.lod_level_count,
        packet.lod_active_level,
        packet.lod_switch_distance
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(7),
        &lod_label,
    );
    let streaming_label = format!(
        "st{} r{} p{} e{}",
        packet.streaming_cluster_slot,
        packet.streaming_resident_levels,
        packet.streaming_prefetch_mode,
        packet.streaming_evict_budget
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(8),
        &streaming_label,
    );

    let instance_count = packet.instance_count.clamp(1, 4) as usize;
    let instance_stride = packet.instance_stride.abs().clamp(2, 6) as usize;
    let mut last_x = object_x;
    for idx in 1..instance_count {
        let shifted_x = (object_x + idx * instance_stride)
            .min(preview_right.saturating_sub(1))
            .max(preview_left + 1);
        let shifted_y = object_y
            .saturating_add((packet.instance_phase + idx as i64).rem_euclid(2) as usize)
            .min(ground_y.saturating_sub(1));
        let ghost = match (packet.instance_material_slot + idx as i64).rem_euclid(3) {
            0 => ':',
            1 => ';',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(shifted_y) {
            let shifted_slot = shifted_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(shifted_slot) {
                *cell = ghost;
            }
        }
        stamp_line(rows, last_x, object_y, shifted_x, shifted_y, '.');
        last_x = shifted_x;
    }

    let root_y = preview_top
        .saturating_add(packet.scene_graph_root_slot.rem_euclid(3) as usize)
        .min(ground_y.saturating_sub(2));
    let graph_span = packet.scene_graph_node_count.clamp(2, 6) as usize;
    for idx in 0..graph_span {
        let branch_x = preview_left
            .saturating_add(2 + idx * 2)
            .min(preview_right.saturating_sub(1));
        if let Some(row) = rows.get_mut(root_y) {
            let slot = branch_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = if idx == 0 { '@' } else { '|' };
            }
        }
        let depth_y = (root_y + 1 + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        if let Some(row) = rows.get_mut(depth_y) {
            let slot = branch_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = '.';
            }
        }
    }

    let node_y = root_y
        .saturating_add(1 + packet.scene_node_slot.rem_euclid(2) as usize)
        .min(ground_y.saturating_sub(1));
    let child_x = preview_left
        .saturating_add(3 + packet.scene_node_first_child_slot.rem_euclid(8) as usize)
        .min(preview_right.saturating_sub(1));
    let sibling_x = preview_left
        .saturating_add(5 + packet.scene_node_sibling_slot.rem_euclid(8) as usize)
        .min(preview_right.saturating_sub(1));
    let node_glyph = if packet.scene_node_visibility != 0 {
        '#'
    } else {
        'x'
    };
    if let Some(row) = rows.get_mut(node_y) {
        let slot = child_x.min(row.len().saturating_sub(1));
        if let Some(cell) = row.get_mut(slot) {
            *cell = node_glyph;
        }
    }
    stamp_line(rows, child_x, node_y, sibling_x, node_y, '=');

    let group_visible = packet.instance_group_visible_count.clamp(1, 4) as usize;
    let group_stride = (packet.instance_group_phase_bias.abs().clamp(2, 6)) as usize;
    for idx in 0..group_visible {
        let gx = preview_left
            .saturating_add(10 + idx * group_stride)
            .min(preview_right.saturating_sub(1));
        let gy = root_y
            .saturating_add(2 + idx.rem_euclid(2))
            .min(ground_y.saturating_sub(1));
        let glyph = match (packet.instance_group_material_slot + idx as i64).rem_euclid(3) {
            0 => '*',
            1 => '+',
            _ => '%',
        };
        if let Some(row) = rows.get_mut(gy) {
            let slot = gx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, child_x, node_y, gx, gy, ':');
    }

    let cluster_span = packet.scene_cluster_node_budget.clamp(2, 5) as usize;
    let cluster_root_x = preview_left
        .saturating_add(18 + packet.scene_cluster_root_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let cluster_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..cluster_span {
        let cx = (cluster_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let cy = (cluster_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.scene_cluster_material_slot + idx as i64).rem_euclid(3) {
            0 => 'o',
            1 => '0',
            _ => '8',
        };
        if let Some(row) = rows.get_mut(cy) {
            let slot = cx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, cluster_root_x, cluster_root_y, cx, cy, '~');
    }

    let visibility_span = packet.visibility_visible_nodes.clamp(1, 5) as usize;
    let vis_root_x = preview_left
        .saturating_add(24 + packet.visibility_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let vis_root_y = root_y.saturating_add(2).min(ground_y.saturating_sub(1));
    for idx in 0..visibility_span {
        let vx = (vis_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let vy = (vis_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.visibility_mask + idx as i64).rem_euclid(4) {
            0 => 'v',
            1 => 'V',
            2 => '^',
            _ => '/',
        };
        if let Some(row) = rows.get_mut(vy) {
            let slot = vx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.visibility_occlusion_mode != 0 {
            '!'
        } else {
            '.'
        };
        stamp_line(rows, cluster_root_x, cluster_root_y, vx, vy, connector);
    }

    let cull_span = packet.cull_kept_nodes.clamp(1, 4) as usize;
    let cull_root_x = preview_left
        .saturating_add(30 + packet.cull_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let cull_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..cull_span {
        let cx = (cull_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let cy = (cull_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.cull_mask + idx as i64).rem_euclid(4) {
            0 => 'c',
            1 => 'C',
            2 => '<',
            _ => '>',
        };
        if let Some(row) = rows.get_mut(cy) {
            let slot = cx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.cull_mode != 0 { '-' } else { '_' };
        stamp_line(rows, vis_root_x, vis_root_y, cx, cy, connector);
    }

    let lod_span = packet.lod_level_count.clamp(1, 4) as usize;
    let lod_root_x = preview_left
        .saturating_add(36 + packet.lod_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let lod_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..lod_span {
        let lx = (lod_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ly = (lod_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = if idx as i64 == packet.lod_active_level.rem_euclid(lod_span as i64) {
            match packet.lod_bias.rem_euclid(3) {
                0 => 'L',
                1 => 'M',
                _ => 'H',
            }
        } else {
            '.'
        };
        if let Some(row) = rows.get_mut(ly) {
            let slot = lx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, cull_root_x, cull_root_y, lx, ly, '=');
    }

    let streaming_span = packet.streaming_resident_levels.clamp(1, 4) as usize;
    let streaming_root_x = preview_left
        .saturating_add(42 + packet.streaming_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let streaming_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..streaming_span {
        let sx = (streaming_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let sy = (streaming_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.streaming_channel + idx as i64).rem_euclid(4) {
            0 => 's',
            1 => '$',
            2 => '~',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(sy) {
            let slot = sx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.streaming_prefetch_mode != 0 {
            ':'
        } else {
            '.'
        };
        stamp_line(rows, lod_root_x, lod_root_y, sx, sy, connector);
    }
}

#[derive(Debug, Clone, Copy)]
struct BallPacket {
    color_key: i64,
    speed: f32,
    radius_scale: f32,
    color_min: i64,
    color_max: i64,
    color_step: i64,
    color_disabled: i64,
    speed_min: i64,
    speed_max: i64,
    speed_step: i64,
    speed_disabled: i64,
    radius_min: i64,
    radius_max: i64,
    radius_step: i64,
    radius_disabled: i64,
    accent: i64,
    surface: i64,
    panel_mode: i64,
    contrast: i64,
    surface_density: i64,
    surface_elevation: i64,
    surface_grid: i64,
    surface_sheen: i64,
    viewport_x: i64,
    viewport_y: i64,
    viewport_width: i64,
    viewport_height: i64,
    layer_order: i64,
    layer_blend: i64,
    layer_visibility: i64,
    layer_clip: i64,
    scene_root_count: i64,
    scene_active_camera: i64,
    scene_light_count: i64,
    scene_animation_phase: i64,
    camera_kind: i64,
    camera_focus: i64,
    camera_zoom: i64,
    camera_orbit: i64,
    material_shader_kind: i64,
    material_albedo: i64,
    material_roughness: i64,
    material_emissive: i64,
    light_kind: i64,
    light_intensity: i64,
    light_range: i64,
    light_reactive: i64,
    mesh_primitive: i64,
    mesh_vertex_count: i64,
    mesh_index_count: i64,
    mesh_skinning: i64,
    transform_translate: i64,
    transform_rotate: i64,
    transform_scale: i64,
    transform_pivot: i64,
    node_id: i64,
    node_parent_id: i64,
    node_flags: i64,
    node_depth: i64,
    scene_link_node_slot: i64,
    scene_link_transform_slot: i64,
    scene_link_mesh_slot: i64,
    scene_link_material_slot: i64,
    scene_link_light_slot: i64,
    scene_link_layer_slot: i64,
    instance_node_slot: i64,
    instance_count: i64,
    instance_stride: i64,
    instance_phase: i64,
    instance_material_slot: i64,
    instance_light_slot: i64,
    scene_graph_root_slot: i64,
    scene_graph_node_count: i64,
    scene_graph_link_count: i64,
    scene_graph_instance_count: i64,
    scene_graph_active_layer: i64,
    scene_node_slot: i64,
    scene_node_first_child_slot: i64,
    scene_node_sibling_slot: i64,
    scene_node_instance_slot: i64,
    scene_node_visibility: i64,
    instance_group_root_slot: i64,
    instance_group_count: i64,
    instance_group_visible_count: i64,
    instance_group_phase_bias: i64,
    instance_group_material_slot: i64,
    scene_cluster_root_slot: i64,
    scene_cluster_node_budget: i64,
    scene_cluster_instance_group_slot: i64,
    scene_cluster_material_slot: i64,
    scene_cluster_layer_slot: i64,
    visibility_cluster_slot: i64,
    visibility_visible_nodes: i64,
    visibility_occlusion_mode: i64,
    visibility_distance_band: i64,
    visibility_mask: i64,
    cull_cluster_slot: i64,
    cull_kept_nodes: i64,
    cull_mode: i64,
    cull_lod_band: i64,
    cull_mask: i64,
    lod_cluster_slot: i64,
    lod_level_count: i64,
    lod_active_level: i64,
    lod_switch_distance: i64,
    lod_bias: i64,
    streaming_cluster_slot: i64,
    streaming_resident_levels: i64,
    streaming_prefetch_mode: i64,
    streaming_evict_budget: i64,
    streaming_channel: i64,
    pass_stage: i64,
    pass_clear_mode: i64,
    pass_sample_count: i64,
    pass_debug_view: i64,
    frame_index: i64,
    frame_present_mode: i64,
    frame_sync_interval: i64,
    frame_exposure: i64,
    target_kind: i64,
    target_width: i64,
    target_height: i64,
    target_multisample: i64,
    frame_graph_passes: i64,
    frame_graph_targets: i64,
    frame_graph_present_stage: i64,
    frame_graph_debug_overlay: i64,
    attachment_slot: i64,
    attachment_format_kind: i64,
    attachment_load_op: i64,
    attachment_store_op: i64,
    pass_chain_stages: i64,
    pass_chain_fanout: i64,
    pass_chain_resolve_stage: i64,
    pass_chain_barrier_mode: i64,
    barrier_scope: i64,
    barrier_source_stage: i64,
    barrier_target_stage: i64,
    barrier_flush_mode: i64,
    resource_buffers: i64,
    resource_textures: i64,
    resource_samplers: i64,
    resource_residency: i64,
    schedule_lanes: i64,
    schedule_queue_depth: i64,
    schedule_async_budget: i64,
    schedule_tick_mode: i64,
    submission_batches: i64,
    submission_fences: i64,
    submission_signal_mode: i64,
    submission_present_hint: i64,
    queue_kind: i64,
    queue_priority: i64,
    queue_budget: i64,
    queue_ownership: i64,
    semaphore_wait_count: i64,
    semaphore_signal_count: i64,
    semaphore_timeline_mode: i64,
    semaphore_scope: i64,
    timeline_value: i64,
    timeline_step: i64,
    timeline_epoch: i64,
    timeline_domain: i64,
    fence_signaled: i64,
    fence_epoch: i64,
    fence_scope: i64,
    fence_recycle_mode: i64,
    signal_kind: i64,
    signal_phase: i64,
    signal_fanout: i64,
    signal_ack_mode: i64,
    event_kind: i64,
    event_route: i64,
    event_priority: i64,
    event_payload_mode: i64,
    dispatch_queue_kind: i64,
    dispatch_lane: i64,
    dispatch_batch: i64,
    dispatch_completion_mode: i64,
    feedback_status: i64,
    feedback_latency: i64,
    feedback_retries: i64,
    feedback_channel: i64,
    intent_kind: i64,
    intent_target: i64,
    intent_urgency: i64,
    intent_policy: i64,
    reaction_kind: i64,
    reaction_result_slot: i64,
    reaction_stability: i64,
    reaction_echo_mode: i64,
    outcome_kind: i64,
    outcome_final_slot: i64,
    outcome_confidence: i64,
    outcome_settle_mode: i64,
    resolution_kind: i64,
    resolution_commit_slot: i64,
    resolution_convergence: i64,
    resolution_policy_mode: i64,
    commit_kind: i64,
    commit_applied_slot: i64,
    commit_durability: i64,
    commit_commit_mode: i64,
    snapshot_kind: i64,
    snapshot_source_slot: i64,
    snapshot_retention: i64,
    snapshot_replay_mode: i64,
    checkpoint_kind: i64,
    checkpoint_anchor_slot: i64,
    checkpoint_rollback_depth: i64,
    checkpoint_resume_mode: i64,
    toggle_state: i64,
    focus_index: i64,
    progress_value: i64,
    progress_max: i64,
    meter_value: i64,
    meter_max: i64,
    button_state: i64,
    button_intent: i64,
    header_title_mode: i64,
    toggle_disabled: i64,
    text_caret: i64,
    text_echo: i64,
    text_placeholder: i64,
    text_read_only: i64,
    text_dirty: i64,
    select_index: i64,
    select_options: i64,
    select_multiple: i64,
    select_committed: i64,
    checkbox_checked: i64,
    checkbox_disabled: i64,
    radio_selected: i64,
    radio_options: i64,
    radio_disabled: i64,
    textarea_lines: i64,
    textarea_scroll: i64,
    textarea_placeholder: i64,
    textarea_read_only: i64,
    textarea_dirty: i64,
    tabs_active: i64,
    tabs_count: i64,
    tabs_compact: i64,
    list_selected: i64,
    list_items: i64,
    list_dense: i64,
    table_rows: i64,
    table_cols: i64,
    table_selected_row: i64,
    table_zebra: i64,
    tree_selected: i64,
    tree_nodes: i64,
    tree_expanded: i64,
    inspector_selected: i64,
    inspector_fields: i64,
    inspector_pinned: i64,
    outline_selected: i64,
    outline_items: i64,
    outline_collapsed: i64,
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
                color_min: 0,
                color_max: 127,
                color_step: 4,
                color_disabled: 0,
                speed_min: 0,
                speed_max: 63,
                speed_step: 2,
                speed_disabled: 0,
                radius_min: 0,
                radius_max: 127,
                radius_step: 3,
                radius_disabled: 0,
                accent: color_key,
                surface: (radius_scale * 8.0).round() as i64,
                panel_mode: if speed.round() as i64 % 2 == 0 { 0 } else { 1 },
                contrast: speed.round() as i64,
                surface_density: speed.round() as i64,
                surface_elevation: (radius_scale * 16.0).round() as i64,
                surface_grid: color_key.rem_euclid(3),
                surface_sheen: color_key,
                viewport_x: color_key.rem_euclid(4),
                viewport_y: speed.round() as i64 % 3,
                viewport_width: 48,
                viewport_height: 18,
                layer_order: 1,
                layer_blend: speed.round() as i64 % 3,
                layer_visibility: 1,
                layer_clip: (radius_scale * 8.0).round() as i64,
                scene_root_count: 7,
                scene_active_camera: color_key.rem_euclid(3),
                scene_light_count: 3,
                scene_animation_phase: speed.round() as i64 % 4,
                camera_kind: speed.round() as i64 % 3,
                camera_focus: color_key.rem_euclid(6),
                camera_zoom: speed.round() as i64,
                camera_orbit: (radius_scale * 12.0).round() as i64,
                material_shader_kind: speed.round() as i64 % 3,
                material_albedo: color_key,
                material_roughness: speed.round() as i64,
                material_emissive: (radius_scale * 24.0).round() as i64,
                light_kind: speed.round() as i64 % 3,
                light_intensity: speed.round() as i64,
                light_range: (radius_scale * 18.0).round() as i64,
                light_reactive: color_key,
                mesh_primitive: speed.round() as i64 % 3,
                mesh_vertex_count: (speed.round() as i64).max(3),
                mesh_index_count: (radius_scale * 18.0).round() as i64,
                mesh_skinning: color_key,
                transform_translate: speed.round() as i64,
                transform_rotate: speed.round() as i64 % 4,
                transform_scale: (radius_scale * 16.0).round() as i64,
                transform_pivot: color_key.rem_euclid(6),
                node_id: color_key.rem_euclid(8),
                node_parent_id: speed.round() as i64 % 4,
                node_flags: color_key,
                node_depth: 2,
                scene_link_node_slot: color_key.rem_euclid(8),
                scene_link_transform_slot: speed.round() as i64,
                scene_link_mesh_slot: (radius_scale * 18.0).round() as i64,
                scene_link_material_slot: color_key,
                scene_link_light_slot: speed.round() as i64 % 3,
                scene_link_layer_slot: 1,
                instance_node_slot: color_key.rem_euclid(8),
                instance_count: 3,
                instance_stride: color_key.rem_euclid(5) + 2,
                instance_phase: speed.round() as i64,
                instance_material_slot: color_key,
                instance_light_slot: speed.round() as i64 % 3,
                scene_graph_root_slot: color_key.rem_euclid(8),
                scene_graph_node_count: 6,
                scene_graph_link_count: 3,
                scene_graph_instance_count: 3,
                scene_graph_active_layer: 1,
                scene_node_slot: color_key.rem_euclid(8),
                scene_node_first_child_slot: speed.round() as i64,
                scene_node_sibling_slot: (radius_scale * 18.0).round() as i64,
                scene_node_instance_slot: 3,
                scene_node_visibility: 1,
                instance_group_root_slot: 3,
                instance_group_count: 4,
                instance_group_visible_count: 3,
                instance_group_phase_bias: speed.round() as i64,
                instance_group_material_slot: color_key,
                scene_cluster_root_slot: color_key.rem_euclid(8),
                scene_cluster_node_budget: 6,
                scene_cluster_instance_group_slot: 3,
                scene_cluster_material_slot: color_key,
                scene_cluster_layer_slot: 1,
                visibility_cluster_slot: 3,
                visibility_visible_nodes: 5,
                visibility_occlusion_mode: 1,
                visibility_distance_band: speed.round() as i64 % 4,
                visibility_mask: 7,
                cull_cluster_slot: 3,
                cull_kept_nodes: 4,
                cull_mode: speed.round() as i64 % 2,
                cull_lod_band: speed.round() as i64 % 4,
                cull_mask: 7,
                lod_cluster_slot: 3,
                lod_level_count: 4,
                lod_active_level: speed.round() as i64 % 3,
                lod_switch_distance: (radius_scale * 24.0).round() as i64,
                lod_bias: color_key,
                streaming_cluster_slot: 3,
                streaming_resident_levels: 2,
                streaming_prefetch_mode: speed.round() as i64 % 2,
                streaming_evict_budget: (radius_scale * 16.0).round() as i64,
                streaming_channel: color_key,
                pass_stage: speed.round() as i64 % 3,
                pass_clear_mode: color_key,
                pass_sample_count: 4,
                pass_debug_view: color_key.rem_euclid(6),
                frame_index: speed.round() as i64,
                frame_present_mode: color_key.rem_euclid(3),
                frame_sync_interval: 1,
                frame_exposure: (radius_scale * 24.0).round() as i64,
                target_kind: color_key.rem_euclid(3),
                target_width: 48,
                target_height: 18,
                target_multisample: color_key,
                frame_graph_passes: 2,
                frame_graph_targets: 1,
                frame_graph_present_stage: speed.round() as i64 % 3,
                frame_graph_debug_overlay: color_key.rem_euclid(6),
                attachment_slot: 0,
                attachment_format_kind: color_key,
                attachment_load_op: speed.round() as i64 % 3,
                attachment_store_op: 1,
                pass_chain_stages: 2,
                pass_chain_fanout: 1,
                pass_chain_resolve_stage: speed.round() as i64 % 3,
                pass_chain_barrier_mode: color_key,
                barrier_scope: 1,
                barrier_source_stage: speed.round() as i64 % 3,
                barrier_target_stage: 2,
                barrier_flush_mode: color_key,
                resource_buffers: 2,
                resource_textures: 1,
                resource_samplers: 1,
                resource_residency: color_key,
                schedule_lanes: 2,
                schedule_queue_depth: 4,
                schedule_async_budget: (radius_scale * 24.0).round() as i64,
                schedule_tick_mode: speed.round() as i64 % 3,
                submission_batches: 2,
                submission_fences: 1,
                submission_signal_mode: speed.round() as i64 % 3,
                submission_present_hint: color_key,
                queue_kind: speed.round() as i64 % 3,
                queue_priority: 2,
                queue_budget: (radius_scale * 24.0).round() as i64,
                queue_ownership: color_key,
                semaphore_wait_count: 1,
                semaphore_signal_count: 2,
                semaphore_timeline_mode: speed.round() as i64 % 3,
                semaphore_scope: color_key,
                timeline_value: (radius_scale * 24.0).round() as i64,
                timeline_step: 1,
                timeline_epoch: 0,
                timeline_domain: color_key,
                fence_signaled: if speed.round() as i64 % 2 == 0 { 0 } else { 1 },
                fence_epoch: 0,
                fence_scope: color_key,
                fence_recycle_mode: 1,
                signal_kind: speed.round() as i64 % 3,
                signal_phase: 2,
                signal_fanout: 3,
                signal_ack_mode: color_key,
                event_kind: speed.round() as i64 % 3,
                event_route: 2,
                event_priority: 3,
                event_payload_mode: color_key,
                dispatch_queue_kind: speed.round() as i64 % 3,
                dispatch_lane: 2,
                dispatch_batch: 3,
                dispatch_completion_mode: color_key,
                feedback_status: if speed.round() as i64 % 2 == 0 { 0 } else { 1 },
                feedback_latency: speed.round() as i64,
                feedback_retries: radius_scale.round() as i64 % 4,
                feedback_channel: color_key,
                intent_kind: speed.round() as i64 % 3,
                intent_target: color_key.rem_euclid(6),
                intent_urgency: speed.round() as i64,
                intent_policy: color_key,
                reaction_kind: speed.round() as i64 % 3,
                reaction_result_slot: color_key.rem_euclid(6),
                reaction_stability: radius_scale.round() as i64 % 4,
                reaction_echo_mode: color_key,
                outcome_kind: speed.round() as i64 % 3,
                outcome_final_slot: color_key.rem_euclid(6),
                outcome_confidence: speed.round() as i64,
                outcome_settle_mode: color_key,
                resolution_kind: speed.round() as i64 % 3,
                resolution_commit_slot: color_key.rem_euclid(6),
                resolution_convergence: radius_scale.round() as i64 % 4,
                resolution_policy_mode: color_key,
                commit_kind: speed.round() as i64 % 3,
                commit_applied_slot: color_key.rem_euclid(6),
                commit_durability: speed.round() as i64,
                commit_commit_mode: color_key,
                snapshot_kind: speed.round() as i64 % 3,
                snapshot_source_slot: color_key.rem_euclid(6),
                snapshot_retention: radius_scale.round() as i64 % 4,
                snapshot_replay_mode: color_key,
                checkpoint_kind: speed.round() as i64 % 3,
                checkpoint_anchor_slot: color_key.rem_euclid(6),
                checkpoint_rollback_depth: speed.round() as i64,
                checkpoint_resume_mode: color_key,
                toggle_state: if speed.round() as i64 % 2 == 0 { 0 } else { 1 },
                focus_index: color_key.rem_euclid(3),
                progress_value: speed.round() as i64,
                progress_max: 63,
                meter_value: (radius_scale * 96.0).round() as i64,
                meter_max: 127,
                button_state: if speed.round() as i64 % 2 == 0 { 0 } else { 1 },
                button_intent: color_key.rem_euclid(3),
                header_title_mode: color_key.rem_euclid(2),
                toggle_disabled: radius_scale.round() as i64 % 2,
                text_caret: color_key.rem_euclid(6),
                text_echo: color_key,
                text_placeholder: radius_scale.round() as i64,
                text_read_only: 0,
                text_dirty: 0,
                select_index: color_key.rem_euclid(3),
                select_options: 3,
                select_multiple: 0,
                select_committed: 1,
                checkbox_checked: color_key.rem_euclid(2),
                checkbox_disabled: 0,
                radio_selected: color_key.rem_euclid(4),
                radio_options: 4,
                radio_disabled: 0,
                textarea_lines: 3,
                textarea_scroll: speed.round() as i64,
                textarea_placeholder: radius_scale.round() as i64,
                textarea_read_only: 0,
                textarea_dirty: 0,
                tabs_active: color_key.rem_euclid(4),
                tabs_count: 4,
                tabs_compact: 0,
                list_selected: color_key.rem_euclid(5),
                list_items: 5,
                list_dense: 0,
                table_rows: 4,
                table_cols: 3,
                table_selected_row: color_key.rem_euclid(4),
                table_zebra: 1,
                tree_selected: color_key.rem_euclid(6),
                tree_nodes: 6,
                tree_expanded: speed.round() as i64 % 2,
                inspector_selected: color_key.rem_euclid(4),
                inspector_fields: 4,
                inspector_pinned: speed.round() as i64 % 2,
                outline_selected: color_key.rem_euclid(6),
                outline_items: 6,
                outline_collapsed: speed.round() as i64 % 2,
            })
        }
        Value::Struct(packet) => parse_ball_packet_struct(packet, op),
        _ => Err(format!(
            "{op} expects a packet tuple `(color, speed[, radius_scale])` or struct with `color` and `speed`"
        )),
    }
}

fn parse_ball_packet_struct(packet: &StructValue, op: &str) -> Result<BallPacket, String> {
    let color = find_slider_packet_value(packet, "color")
        .or_else(|| find_flat_packet_field(packet, &["color", "slider_color"]))
        .ok_or_else(|| format!("{op} struct packet is missing `color` field"))?;
    let speed = find_slider_packet_value(packet, "speed")
        .or_else(|| find_flat_packet_field(packet, &["speed", "slider_speed"]))
        .ok_or_else(|| format!("{op} struct packet is missing `speed` field"))?;
    let radius_scale = find_slider_packet_value(packet, "radius")
        .or_else(|| find_flat_packet_field(packet, &["radius_scale", "slider_radius"]))
        .map(|value| scalar_to_f32(value, op))
        .transpose()?
        .unwrap_or(1.0);
    let accent = find_packet_field(
        packet,
        &["accent", "header_accent"],
        &["theme", "header"],
        &["accent"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or_else(|| scalar_to_color_key(color, op).unwrap_or(0));
    let surface = find_packet_field(packet, &["theme_surface"], &["theme"], &["surface"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 8.0).round() as i64);
    let panel_mode = find_packet_field(packet, &["panel_mode"], &["theme"], &["panel_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(2));
    let contrast = find_packet_field(packet, &["contrast"], &["theme"], &["contrast"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let surface_density = find_packet_field(packet, &["density"], &["surface"], &["density"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let surface_elevation = find_packet_field(packet, &["elevation"], &["surface"], &["elevation"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 16.0).round() as i64);
    let surface_grid = find_packet_field(packet, &["grid"], &["surface"], &["grid"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(3));
    let surface_sheen = find_packet_field(packet, &["sheen"], &["surface"], &["sheen"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let viewport_x = find_packet_field(packet, &["origin_x"], &["viewport"], &["origin_x"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(4));
    let viewport_y = find_packet_field(packet, &["origin_y"], &["viewport"], &["origin_y"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let viewport_width = find_packet_field(packet, &["width"], &["viewport"], &["width"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(48);
    let viewport_height = find_packet_field(packet, &["height"], &["viewport"], &["height"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(18);
    let layer_order = find_packet_field(packet, &["order"], &["layer"], &["order"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let layer_blend = find_packet_field(packet, &["blend"], &["layer"], &["blend"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let layer_visibility = find_packet_field(packet, &["visibility"], &["layer"], &["visibility"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let layer_clip = find_packet_field(packet, &["clip"], &["layer"], &["clip"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 8.0).round() as i64);
    let scene_root_count = find_packet_field(packet, &["root_count"], &["scene"], &["root_count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(7);
    let scene_active_camera =
        find_packet_field(packet, &["active_camera"], &["scene"], &["active_camera"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent.rem_euclid(6));
    let scene_light_count =
        find_packet_field(packet, &["light_count"], &["scene"], &["light_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(3);
    let scene_animation_phase = find_packet_field(
        packet,
        &["animation_phase"],
        &["scene"],
        &["animation_phase"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(4));
    let camera_kind = find_packet_field(packet, &["kind"], &["camera"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let camera_focus = find_packet_field(packet, &["camera_focus"], &["camera"], &["focus"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(6));
    let camera_zoom = find_packet_field(packet, &["zoom"], &["camera"], &["zoom"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let camera_orbit = find_packet_field(packet, &["orbit"], &["camera"], &["orbit"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 12.0).round() as i64);
    let material_shader_kind =
        find_packet_field(packet, &["shader_kind"], &["material"], &["shader_kind"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let material_albedo = find_packet_field(packet, &["albedo"], &["material"], &["albedo"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let material_roughness =
        find_packet_field(packet, &["roughness"], &["material"], &["roughness"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let material_emissive = find_packet_field(packet, &["emissive"], &["material"], &["emissive"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let light_kind = find_packet_field(packet, &["kind"], &["light"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let light_intensity = find_packet_field(packet, &["intensity"], &["light"], &["intensity"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let light_range = find_packet_field(packet, &["range"], &["light"], &["range"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 18.0).round() as i64);
    let light_reactive = find_packet_field(packet, &["reactive"], &["light"], &["reactive"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let mesh_primitive = find_packet_field(packet, &["primitive"], &["mesh"], &["primitive"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let mesh_vertex_count =
        find_packet_field(packet, &["vertex_count"], &["mesh"], &["vertex_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(3));
    let mesh_index_count = find_packet_field(packet, &["index_count"], &["mesh"], &["index_count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 18.0).round() as i64);
    let mesh_skinning = find_packet_field(packet, &["skinning"], &["mesh"], &["skinning"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let transform_translate =
        find_packet_field(packet, &["translate"], &["transform"], &["translate"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let transform_rotate = find_packet_field(packet, &["rotate"], &["transform"], &["rotate"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(4));
    let transform_scale = find_packet_field(packet, &["scale"], &["transform"], &["scale"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 16.0).round() as i64);
    let transform_pivot = find_packet_field(packet, &["pivot"], &["transform"], &["pivot"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(6));
    let node_id = find_packet_field(packet, &["node_id"], &["node"], &["node_id"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(8));
    let node_parent_id = find_packet_field(packet, &["parent_id"], &["node"], &["parent_id"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(4));
    let node_flags = find_packet_field(packet, &["flags"], &["node"], &["flags"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let node_depth = find_packet_field(packet, &["depth"], &["node"], &["depth"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let scene_link_node_slot =
        find_packet_field(packet, &["node_slot"], &["scene_link"], &["node_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(node_id);
    let scene_link_transform_slot = find_packet_field(
        packet,
        &["transform_slot"],
        &["scene_link"],
        &["transform_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(transform_translate);
    let scene_link_mesh_slot =
        find_packet_field(packet, &["mesh_slot"], &["scene_link"], &["mesh_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(mesh_vertex_count);
    let scene_link_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["scene_link"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(material_albedo);
    let scene_link_light_slot =
        find_packet_field(packet, &["light_slot"], &["scene_link"], &["light_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(light_kind);
    let scene_link_layer_slot =
        find_packet_field(packet, &["layer_slot"], &["scene_link"], &["layer_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(layer_order);
    let instance_node_slot =
        find_packet_field(packet, &["node_slot"], &["instance"], &["node_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_link_node_slot);
    let instance_count = find_packet_field(packet, &["count"], &["instance"], &["count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let instance_stride = find_packet_field(packet, &["stride"], &["instance"], &["stride"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let instance_phase = find_packet_field(packet, &["phase"], &["instance"], &["phase"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let instance_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["instance"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_material_slot);
    let instance_light_slot =
        find_packet_field(packet, &["light_slot"], &["instance"], &["light_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_link_light_slot);
    let scene_graph_root_slot =
        find_packet_field(packet, &["root_slot"], &["scene_graph"], &["root_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_link_node_slot);
    let scene_graph_node_count =
        find_packet_field(packet, &["node_count"], &["scene_graph"], &["node_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(6);
    let scene_graph_link_count =
        find_packet_field(packet, &["link_count"], &["scene_graph"], &["link_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(3);
    let scene_graph_instance_count = find_packet_field(
        packet,
        &["instance_count"],
        &["scene_graph"],
        &["instance_count"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_count);
    let scene_graph_active_layer = find_packet_field(
        packet,
        &["active_layer"],
        &["scene_graph"],
        &["active_layer"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_layer_slot);
    let scene_node_slot =
        find_packet_field(packet, &["node_slot"], &["scene_node"], &["node_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_graph_root_slot);
    let scene_node_first_child_slot = find_packet_field(
        packet,
        &["first_child_slot"],
        &["scene_node"],
        &["first_child_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_transform_slot);
    let scene_node_sibling_slot = find_packet_field(
        packet,
        &["sibling_slot"],
        &["scene_node"],
        &["sibling_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_mesh_slot);
    let scene_node_instance_slot = find_packet_field(
        packet,
        &["instance_slot"],
        &["scene_node"],
        &["instance_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(3);
    let scene_node_visibility =
        find_packet_field(packet, &["visibility"], &["scene_node"], &["visibility"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let instance_group_root_slot = find_packet_field(
        packet,
        &["root_instance_slot"],
        &["instance_group"],
        &["root_instance_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_node_instance_slot);
    let instance_group_count = find_packet_field(
        packet,
        &["group_count"],
        &["instance_group"],
        &["group_count"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(4);
    let instance_group_visible_count = find_packet_field(
        packet,
        &["visible_count"],
        &["instance_group"],
        &["visible_count"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_count);
    let instance_group_phase_bias = find_packet_field(
        packet,
        &["phase_bias"],
        &["instance_group"],
        &["phase_bias"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_phase);
    let instance_group_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["instance_group"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_material_slot);
    let scene_cluster_root_slot = find_packet_field(
        packet,
        &["root_node_slot"],
        &["scene_cluster"],
        &["root_node_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_node_slot);
    let scene_cluster_node_budget = find_packet_field(
        packet,
        &["node_budget"],
        &["scene_cluster"],
        &["node_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_graph_node_count);
    let scene_cluster_instance_group_slot = find_packet_field(
        packet,
        &["instance_group_slot"],
        &["scene_cluster"],
        &["instance_group_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_root_slot);
    let scene_cluster_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["scene_cluster"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_material_slot);
    let scene_cluster_layer_slot =
        find_packet_field(packet, &["layer_slot"], &["scene_cluster"], &["layer_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_graph_active_layer);
    let visibility_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_visibility"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_cluster_instance_group_slot);
    let visibility_visible_nodes = find_packet_field(
        packet,
        &["visible_nodes"],
        &["scene_visibility"],
        &["visible_nodes"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_visible_count);
    let visibility_occlusion_mode = find_packet_field(
        packet,
        &["occlusion_mode"],
        &["scene_visibility"],
        &["occlusion_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_node_visibility);
    let visibility_distance_band = find_packet_field(
        packet,
        &["distance_band"],
        &["scene_visibility"],
        &["distance_band"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_phase_bias);
    let visibility_mask = find_packet_field(packet, &["mask"], &["scene_visibility"], &["mask"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(7);
    let cull_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_cull"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(visibility_cluster_slot);
    let cull_kept_nodes =
        find_packet_field(packet, &["kept_nodes"], &["scene_cull"], &["kept_nodes"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(visibility_visible_nodes);
    let cull_mode = find_packet_field(packet, &["cull_mode"], &["scene_cull"], &["cull_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(visibility_occlusion_mode);
    let cull_lod_band = find_packet_field(packet, &["lod_band"], &["scene_cull"], &["lod_band"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(visibility_distance_band);
    let cull_mask = find_packet_field(packet, &["mask"], &["scene_cull"], &["mask"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(visibility_mask);
    let lod_cluster_slot =
        find_packet_field(packet, &["cluster_slot"], &["scene_lod"], &["cluster_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(cull_cluster_slot);
    let lod_level_count =
        find_packet_field(packet, &["level_count"], &["scene_lod"], &["level_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let lod_active_level =
        find_packet_field(packet, &["active_level"], &["scene_lod"], &["active_level"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(cull_mode);
    let lod_switch_distance = find_packet_field(
        packet,
        &["switch_distance"],
        &["scene_lod"],
        &["switch_distance"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(cull_lod_band);
    let lod_bias = find_packet_field(packet, &["bias"], &["scene_lod"], &["bias"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(cull_mask);
    let streaming_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_streaming"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(lod_cluster_slot);
    let streaming_resident_levels = find_packet_field(
        packet,
        &["resident_levels"],
        &["scene_streaming"],
        &["resident_levels"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(2);
    let streaming_prefetch_mode = find_packet_field(
        packet,
        &["prefetch_mode"],
        &["scene_streaming"],
        &["prefetch_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(lod_active_level);
    let streaming_evict_budget = find_packet_field(
        packet,
        &["evict_budget"],
        &["scene_streaming"],
        &["evict_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(lod_switch_distance);
    let streaming_channel =
        find_packet_field(packet, &["channel"], &["scene_streaming"], &["channel"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(lod_bias);
    let pass_stage = find_packet_field(packet, &["stage"], &["pass"], &["stage"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let pass_clear_mode = find_packet_field(packet, &["clear_mode"], &["pass"], &["clear_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let pass_sample_count =
        find_packet_field(packet, &["sample_count"], &["pass"], &["sample_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let pass_debug_view = find_packet_field(packet, &["debug_view"], &["pass"], &["debug_view"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(6));
    let frame_index = find_packet_field(packet, &["frame_index"], &["frame"], &["frame_index"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let frame_present_mode =
        find_packet_field(packet, &["present_mode"], &["frame"], &["present_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent.rem_euclid(3));
    let frame_sync_interval =
        find_packet_field(packet, &["sync_interval"], &["frame"], &["sync_interval"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let frame_exposure = find_packet_field(packet, &["exposure"], &["frame"], &["exposure"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let target_kind = find_packet_field(packet, &["kind"], &["target"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(3));
    let target_width = find_packet_field(packet, &["width"], &["target"], &["width"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(48);
    let target_height = find_packet_field(packet, &["height"], &["target"], &["height"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(18);
    let target_multisample =
        find_packet_field(packet, &["multisample"], &["target"], &["multisample"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let frame_graph_passes = find_packet_field(packet, &["passes"], &["frame_graph"], &["passes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let frame_graph_targets =
        find_packet_field(packet, &["targets"], &["frame_graph"], &["targets"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let frame_graph_present_stage = find_packet_field(
        packet,
        &["present_stage"],
        &["frame_graph"],
        &["present_stage"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(3));
    let frame_graph_debug_overlay = find_packet_field(
        packet,
        &["debug_overlay"],
        &["frame_graph"],
        &["debug_overlay"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent.rem_euclid(6));
    let attachment_slot = find_packet_field(packet, &["slot"], &["attachment"], &["slot"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let attachment_format_kind =
        find_packet_field(packet, &["format_kind"], &["attachment"], &["format_kind"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let attachment_load_op = find_packet_field(packet, &["load_op"], &["attachment"], &["load_op"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let attachment_store_op =
        find_packet_field(packet, &["store_op"], &["attachment"], &["store_op"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let pass_chain_stages = find_packet_field(packet, &["stages"], &["pass_chain"], &["stages"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let pass_chain_fanout = find_packet_field(packet, &["fanout"], &["pass_chain"], &["fanout"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let pass_chain_resolve_stage = find_packet_field(
        packet,
        &["resolve_stage"],
        &["pass_chain"],
        &["resolve_stage"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(3));
    let pass_chain_barrier_mode = find_packet_field(
        packet,
        &["barrier_mode"],
        &["pass_chain"],
        &["barrier_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent);
    let barrier_scope = find_packet_field(packet, &["scope"], &["barrier"], &["scope"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let barrier_source_stage =
        find_packet_field(packet, &["source_stage"], &["barrier"], &["source_stage"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let barrier_target_stage =
        find_packet_field(packet, &["target_stage"], &["barrier"], &["target_stage"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(2);
    let barrier_flush_mode =
        find_packet_field(packet, &["flush_mode"], &["barrier"], &["flush_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let resource_buffers = find_packet_field(packet, &["buffers"], &["resource_set"], &["buffers"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let resource_textures =
        find_packet_field(packet, &["textures"], &["resource_set"], &["textures"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let resource_samplers =
        find_packet_field(packet, &["samplers"], &["resource_set"], &["samplers"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let resource_residency =
        find_packet_field(packet, &["residency"], &["resource_set"], &["residency"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let schedule_lanes = find_packet_field(packet, &["lanes"], &["schedule"], &["lanes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let schedule_queue_depth =
        find_packet_field(packet, &["queue_depth"], &["schedule"], &["queue_depth"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let schedule_async_budget =
        find_packet_field(packet, &["async_budget"], &["schedule"], &["async_budget"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or((radius_scale * 24.0).round() as i64);
    let schedule_tick_mode =
        find_packet_field(packet, &["tick_mode"], &["schedule"], &["tick_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let submission_batches = find_packet_field(packet, &["batches"], &["submission"], &["batches"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let submission_fences = find_packet_field(packet, &["fences"], &["submission"], &["fences"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let submission_signal_mode =
        find_packet_field(packet, &["signal_mode"], &["submission"], &["signal_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let submission_present_hint = find_packet_field(
        packet,
        &["present_hint"],
        &["submission"],
        &["present_hint"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent);
    let queue_kind = find_packet_field(packet, &["kind"], &["queue"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let queue_priority = find_packet_field(packet, &["priority"], &["queue"], &["priority"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let queue_budget = find_packet_field(packet, &["budget"], &["queue"], &["budget"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let queue_ownership = find_packet_field(packet, &["ownership"], &["queue"], &["ownership"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let semaphore_wait_count =
        find_packet_field(packet, &["wait_count"], &["semaphore"], &["wait_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let semaphore_signal_count =
        find_packet_field(packet, &["signal_count"], &["semaphore"], &["signal_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(2);
    let semaphore_timeline_mode = find_packet_field(
        packet,
        &["timeline_mode"],
        &["semaphore"],
        &["timeline_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(3));
    let semaphore_scope = find_packet_field(packet, &["scope"], &["semaphore"], &["scope"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let timeline_value = find_packet_field(packet, &["value"], &["timeline"], &["value"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let timeline_step = find_packet_field(packet, &["step"], &["timeline"], &["step"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let timeline_epoch = find_packet_field(packet, &["epoch"], &["timeline"], &["epoch"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let timeline_domain = find_packet_field(packet, &["domain"], &["timeline"], &["domain"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let fence_signaled = find_packet_field(packet, &["signaled"], &["fence"], &["signaled"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let fence_epoch = find_packet_field(packet, &["epoch"], &["fence"], &["epoch"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let fence_scope = find_packet_field(packet, &["scope"], &["fence"], &["scope"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let fence_recycle_mode =
        find_packet_field(packet, &["recycle_mode"], &["fence"], &["recycle_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let signal_kind = find_packet_field(packet, &["kind"], &["signal"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let signal_phase = find_packet_field(packet, &["phase"], &["signal"], &["phase"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let signal_fanout = find_packet_field(packet, &["fanout"], &["signal"], &["fanout"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let signal_ack_mode = find_packet_field(packet, &["ack_mode"], &["signal"], &["ack_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let event_kind = find_packet_field(packet, &["kind"], &["event"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let event_route = find_packet_field(packet, &["route"], &["event"], &["route"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let event_priority = find_packet_field(packet, &["priority"], &["event"], &["priority"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let event_payload_mode =
        find_packet_field(packet, &["payload_mode"], &["event"], &["payload_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let dispatch_queue_kind =
        find_packet_field(packet, &["queue_kind"], &["dispatch"], &["queue_kind"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let dispatch_lane = find_packet_field(packet, &["lane"], &["dispatch"], &["lane"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let dispatch_batch = find_packet_field(packet, &["batch"], &["dispatch"], &["batch"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let dispatch_completion_mode = find_packet_field(
        packet,
        &["completion_mode"],
        &["dispatch"],
        &["completion_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent);
    let feedback_status = find_packet_field(packet, &["status"], &["feedback"], &["status"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0).rem_euclid(2));
    let feedback_latency = find_packet_field(packet, &["latency"], &["feedback"], &["latency"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let feedback_retries = find_packet_field(packet, &["retries"], &["feedback"], &["retries"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(radius_scale.round() as i64 % 4);
    let feedback_channel = find_packet_field(packet, &["channel"], &["feedback"], &["channel"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let intent_kind = find_packet_field(packet, &["kind"], &["intent"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let intent_target = find_packet_field(packet, &["target_slot"], &["intent"], &["target_slot"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast);
    let intent_urgency = find_packet_field(packet, &["urgency"], &["intent"], &["urgency"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let intent_policy = find_packet_field(packet, &["policy"], &["intent"], &["policy"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let reaction_kind = find_packet_field(packet, &["kind"], &["reaction"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let reaction_result_slot =
        find_packet_field(packet, &["result_slot"], &["reaction"], &["result_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let reaction_stability =
        find_packet_field(packet, &["stability"], &["reaction"], &["stability"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(radius_scale.round() as i64 % 4);
    let reaction_echo_mode =
        find_packet_field(packet, &["echo_mode"], &["reaction"], &["echo_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let outcome_kind = find_packet_field(packet, &["kind"], &["outcome"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let outcome_final_slot =
        find_packet_field(packet, &["final_slot"], &["outcome"], &["final_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let outcome_confidence =
        find_packet_field(packet, &["confidence"], &["outcome"], &["confidence"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let outcome_settle_mode =
        find_packet_field(packet, &["settle_mode"], &["outcome"], &["settle_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let resolution_kind = find_packet_field(packet, &["kind"], &["resolution"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let resolution_commit_slot =
        find_packet_field(packet, &["commit_slot"], &["resolution"], &["commit_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let resolution_convergence =
        find_packet_field(packet, &["convergence"], &["resolution"], &["convergence"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(radius_scale.round() as i64 % 4);
    let resolution_policy_mode =
        find_packet_field(packet, &["policy_mode"], &["resolution"], &["policy_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let commit_kind = find_packet_field(packet, &["kind"], &["commit"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let commit_applied_slot =
        find_packet_field(packet, &["applied_slot"], &["commit"], &["applied_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let commit_durability =
        find_packet_field(packet, &["durability"], &["commit"], &["durability"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let commit_commit_mode =
        find_packet_field(packet, &["commit_mode"], &["commit"], &["commit_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let snapshot_kind = find_packet_field(packet, &["kind"], &["snapshot"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let snapshot_source_slot =
        find_packet_field(packet, &["source_slot"], &["snapshot"], &["source_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let snapshot_retention =
        find_packet_field(packet, &["retention"], &["snapshot"], &["retention"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(radius_scale.round() as i64 % 4);
    let snapshot_replay_mode =
        find_packet_field(packet, &["replay_mode"], &["snapshot"], &["replay_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let checkpoint_kind = find_packet_field(packet, &["kind"], &["checkpoint"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let checkpoint_anchor_slot =
        find_packet_field(packet, &["anchor_slot"], &["checkpoint"], &["anchor_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let checkpoint_rollback_depth = find_packet_field(
        packet,
        &["rollback_depth"],
        &["checkpoint"],
        &["rollback_depth"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let checkpoint_resume_mode =
        find_packet_field(packet, &["resume_mode"], &["checkpoint"], &["resume_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let color_min = find_slider_packet_field(packet, "color", "min")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let color_max = find_slider_packet_field(packet, "color", "max")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(127);
    let color_step = find_slider_packet_field(packet, "color", "step")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let color_disabled = find_slider_packet_field(packet, "color", "disabled")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let speed_min = find_slider_packet_field(packet, "speed", "min")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let speed_max = find_slider_packet_field(packet, "speed", "max")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(63);
    let speed_step = find_slider_packet_field(packet, "speed", "step")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let speed_disabled = find_slider_packet_field(packet, "speed", "disabled")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let radius_min = find_slider_packet_field(packet, "radius", "min")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let radius_max = find_slider_packet_field(packet, "radius", "max")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(127);
    let radius_step = find_slider_packet_field(packet, "radius", "step")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let radius_disabled = find_slider_packet_field(packet, "radius", "disabled")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let toggle_state = find_packet_field(
        packet,
        &["toggle_state", "toggle_live"],
        &["toggle"],
        &["live"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(1);
    let toggle_disabled =
        find_packet_field(packet, &["toggle_disabled"], &["toggle"], &["disabled"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let focus_index = find_packet_field(
        packet,
        &["focus_index", "focus_slot"],
        &["focus"],
        &["slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(0);
    let progress_value = find_packet_field(packet, &["progress_value"], &["progress"], &["value"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let progress_max = find_packet_field(packet, &["progress_max"], &["progress"], &["max"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(63);
    let meter_value = find_packet_field(packet, &["meter_value"], &["meter"], &["value"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| (radius_scale * 96.0).round() as i64);
    let meter_max = find_packet_field(packet, &["meter_max"], &["meter"], &["max"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(127);
    let button_state = find_packet_field(packet, &["button_state"], &["button"], &["active"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(toggle_state);
    let button_intent = find_packet_field(packet, &["button_intent"], &["button"], &["intent"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let header_title_mode =
        find_packet_field(packet, &["header_title_mode"], &["header"], &["title_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(focus_index.rem_euclid(2));
    let text_caret = find_packet_field(packet, &["text_caret"], &["text_input"], &["caret"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let text_echo = find_packet_field(packet, &["text_echo"], &["text_input"], &["echo"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let text_placeholder = find_packet_field(
        packet,
        &["text_placeholder"],
        &["text_input"],
        &["placeholder"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(radius_scale.round() as i64);
    let text_read_only =
        find_packet_field(packet, &["text_read_only"], &["text_input"], &["read_only"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let text_dirty = find_packet_field(packet, &["text_dirty"], &["text_input"], &["dirty"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let select_index = find_packet_field(packet, &["select_index"], &["select"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let select_options = find_packet_field(packet, &["select_options"], &["select"], &["options"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let select_multiple =
        find_packet_field(packet, &["select_multiple"], &["select"], &["multiple"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let select_committed =
        find_packet_field(packet, &["select_committed"], &["select"], &["committed"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let checkbox_checked =
        find_packet_field(packet, &["checkbox_checked"], &["checkbox"], &["checked"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(toggle_state);
    let checkbox_disabled =
        find_packet_field(packet, &["checkbox_disabled"], &["checkbox"], &["disabled"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let radio_selected = find_packet_field(packet, &["radio_selected"], &["radio"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let radio_options = find_packet_field(packet, &["radio_options"], &["radio"], &["options"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let radio_disabled = find_packet_field(packet, &["radio_disabled"], &["radio"], &["disabled"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let textarea_lines = find_packet_field(packet, &["textarea_lines"], &["textarea"], &["lines"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let textarea_scroll =
        find_packet_field(packet, &["textarea_scroll"], &["textarea"], &["scroll"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(text_caret);
    let textarea_placeholder = find_packet_field(
        packet,
        &["textarea_placeholder"],
        &["textarea"],
        &["placeholder"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(text_placeholder);
    let textarea_read_only = find_packet_field(
        packet,
        &["textarea_read_only"],
        &["textarea"],
        &["read_only"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(text_read_only);
    let textarea_dirty = find_packet_field(packet, &["textarea_dirty"], &["textarea"], &["dirty"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(text_dirty);
    let tabs_active = find_packet_field(packet, &["tabs_active"], &["tabs"], &["active"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let tabs_count = find_packet_field(packet, &["tabs_count"], &["tabs"], &["count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let tabs_compact = find_packet_field(packet, &["tabs_compact"], &["tabs"], &["compact"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let list_selected = find_packet_field(packet, &["list_selected"], &["list"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let list_items = find_packet_field(packet, &["list_items"], &["list"], &["items"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(5);
    let list_dense = find_packet_field(packet, &["list_dense"], &["list"], &["dense"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let table_rows = find_packet_field(packet, &["table_rows"], &["table"], &["rows"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let table_cols = find_packet_field(packet, &["table_cols"], &["table"], &["cols"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let table_selected_row = find_packet_field(
        packet,
        &["table_selected_row"],
        &["table"],
        &["selected_row"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(focus_index);
    let table_zebra = find_packet_field(packet, &["table_zebra"], &["table"], &["zebra"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let tree_selected = find_packet_field(packet, &["tree_selected"], &["tree"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let tree_nodes = find_packet_field(packet, &["tree_nodes"], &["tree"], &["nodes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(6);
    let tree_expanded = find_packet_field(packet, &["tree_expanded"], &["tree"], &["expanded"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(toggle_state);
    let inspector_selected = find_packet_field(
        packet,
        &["inspector_selected"],
        &["inspector"],
        &["selected"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(focus_index);
    let inspector_fields =
        find_packet_field(packet, &["inspector_fields"], &["inspector"], &["fields"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let inspector_pinned =
        find_packet_field(packet, &["inspector_pinned"], &["inspector"], &["pinned"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(toggle_state);
    let outline_selected =
        find_packet_field(packet, &["outline_selected"], &["outline"], &["selected"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(focus_index);
    let outline_items = find_packet_field(packet, &["outline_items"], &["outline"], &["items"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(6);
    let outline_collapsed =
        find_packet_field(packet, &["outline_collapsed"], &["outline"], &["collapsed"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(toggle_state);

    Ok(BallPacket {
        color_key: scalar_to_color_key(color, op)?,
        speed: scalar_to_f32(speed, op)?,
        radius_scale,
        color_min,
        color_max,
        color_step,
        color_disabled,
        speed_min,
        speed_max,
        speed_step,
        speed_disabled,
        radius_min,
        radius_max,
        radius_step,
        radius_disabled,
        accent,
        surface,
        panel_mode,
        contrast,
        surface_density,
        surface_elevation,
        surface_grid,
        surface_sheen,
        viewport_x,
        viewport_y,
        viewport_width,
        viewport_height,
        layer_order,
        layer_blend,
        layer_visibility,
        layer_clip,
        scene_root_count,
        scene_active_camera,
        scene_light_count,
        scene_animation_phase,
        camera_kind,
        camera_focus,
        camera_zoom,
        camera_orbit,
        material_shader_kind,
        material_albedo,
        material_roughness,
        material_emissive,
        light_kind,
        light_intensity,
        light_range,
        light_reactive,
        mesh_primitive,
        mesh_vertex_count,
        mesh_index_count,
        mesh_skinning,
        transform_translate,
        transform_rotate,
        transform_scale,
        transform_pivot,
        node_id,
        node_parent_id,
        node_flags,
        node_depth,
        scene_link_node_slot,
        scene_link_transform_slot,
        scene_link_mesh_slot,
        scene_link_material_slot,
        scene_link_light_slot,
        scene_link_layer_slot,
        instance_node_slot,
        instance_count,
        instance_stride,
        instance_phase,
        instance_material_slot,
        instance_light_slot,
        scene_graph_root_slot,
        scene_graph_node_count,
        scene_graph_link_count,
        scene_graph_instance_count,
        scene_graph_active_layer,
        scene_node_slot,
        scene_node_first_child_slot,
        scene_node_sibling_slot,
        scene_node_instance_slot,
        scene_node_visibility,
        instance_group_root_slot,
        instance_group_count,
        instance_group_visible_count,
        instance_group_phase_bias,
        instance_group_material_slot,
        scene_cluster_root_slot,
        scene_cluster_node_budget,
        scene_cluster_instance_group_slot,
        scene_cluster_material_slot,
        scene_cluster_layer_slot,
        visibility_cluster_slot,
        visibility_visible_nodes,
        visibility_occlusion_mode,
        visibility_distance_band,
        visibility_mask,
        cull_cluster_slot,
        cull_kept_nodes,
        cull_mode,
        cull_lod_band,
        cull_mask,
        lod_cluster_slot,
        lod_level_count,
        lod_active_level,
        lod_switch_distance,
        lod_bias,
        streaming_cluster_slot,
        streaming_resident_levels,
        streaming_prefetch_mode,
        streaming_evict_budget,
        streaming_channel,
        pass_stage,
        pass_clear_mode,
        pass_sample_count,
        pass_debug_view,
        frame_index,
        frame_present_mode,
        frame_sync_interval,
        frame_exposure,
        target_kind,
        target_width,
        target_height,
        target_multisample,
        frame_graph_passes,
        frame_graph_targets,
        frame_graph_present_stage,
        frame_graph_debug_overlay,
        attachment_slot,
        attachment_format_kind,
        attachment_load_op,
        attachment_store_op,
        pass_chain_stages,
        pass_chain_fanout,
        pass_chain_resolve_stage,
        pass_chain_barrier_mode,
        barrier_scope,
        barrier_source_stage,
        barrier_target_stage,
        barrier_flush_mode,
        resource_buffers,
        resource_textures,
        resource_samplers,
        resource_residency,
        schedule_lanes,
        schedule_queue_depth,
        schedule_async_budget,
        schedule_tick_mode,
        submission_batches,
        submission_fences,
        submission_signal_mode,
        submission_present_hint,
        queue_kind,
        queue_priority,
        queue_budget,
        queue_ownership,
        semaphore_wait_count,
        semaphore_signal_count,
        semaphore_timeline_mode,
        semaphore_scope,
        timeline_value,
        timeline_step,
        timeline_epoch,
        timeline_domain,
        fence_signaled,
        fence_epoch,
        fence_scope,
        fence_recycle_mode,
        signal_kind,
        signal_phase,
        signal_fanout,
        signal_ack_mode,
        event_kind,
        event_route,
        event_priority,
        event_payload_mode,
        dispatch_queue_kind,
        dispatch_lane,
        dispatch_batch,
        dispatch_completion_mode,
        feedback_status,
        feedback_latency,
        feedback_retries,
        feedback_channel,
        intent_kind,
        intent_target,
        intent_urgency,
        intent_policy,
        reaction_kind,
        reaction_result_slot,
        reaction_stability,
        reaction_echo_mode,
        outcome_kind,
        outcome_final_slot,
        outcome_confidence,
        outcome_settle_mode,
        resolution_kind,
        resolution_commit_slot,
        resolution_convergence,
        resolution_policy_mode,
        commit_kind,
        commit_applied_slot,
        commit_durability,
        commit_commit_mode,
        snapshot_kind,
        snapshot_source_slot,
        snapshot_retention,
        snapshot_replay_mode,
        checkpoint_kind,
        checkpoint_anchor_slot,
        checkpoint_rollback_depth,
        checkpoint_resume_mode,
        toggle_state,
        focus_index,
        progress_value,
        progress_max,
        meter_value,
        meter_max,
        button_state,
        button_intent,
        header_title_mode,
        toggle_disabled,
        text_caret,
        text_echo,
        text_placeholder,
        text_read_only,
        text_dirty,
        select_index,
        select_options,
        select_multiple,
        select_committed,
        checkbox_checked,
        checkbox_disabled,
        radio_selected,
        radio_options,
        radio_disabled,
        textarea_lines,
        textarea_scroll,
        textarea_placeholder,
        textarea_read_only,
        textarea_dirty,
        tabs_active,
        tabs_count,
        tabs_compact,
        list_selected,
        list_items,
        list_dense,
        table_rows,
        table_cols,
        table_selected_row,
        table_zebra,
        tree_selected,
        tree_nodes,
        tree_expanded,
        inspector_selected,
        inspector_fields,
        inspector_pinned,
        outline_selected,
        outline_items,
        outline_collapsed,
    })
}

fn find_packet_field<'a>(
    packet: &'a StructValue,
    flat_names: &[&str],
    nested_struct_names: &[&str],
    nested_field_names: &[&str],
) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| flat_names.iter().any(|candidate| name == candidate))
        .map(|(_, value)| value)
        .or_else(|| {
            packet
                .fields
                .iter()
                .find(|(name, _)| {
                    nested_struct_names
                        .iter()
                        .any(|candidate| name == candidate)
                })
                .and_then(|(_, value)| match value {
                    Value::Struct(inner) => inner
                        .fields
                        .iter()
                        .find(|(name, _)| {
                            nested_field_names.iter().any(|candidate| name == candidate)
                        })
                        .map(|(_, value)| value),
                    _ => None,
                })
        })
}

fn find_flat_packet_field<'a>(packet: &'a StructValue, flat_names: &[&str]) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| flat_names.iter().any(|candidate| name == candidate))
        .map(|(_, value)| value)
}

fn find_slider_packet_value<'a>(packet: &'a StructValue, slider_name: &str) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| name == "sliders")
        .and_then(|(_, value)| match value {
            Value::Struct(group) => group
                .fields
                .iter()
                .find(|(name, _)| name == slider_name)
                .and_then(|(_, value)| match value {
                    Value::Struct(slider) => slider
                        .fields
                        .iter()
                        .find(|(name, _)| name == "value")
                        .map(|(_, value)| value),
                    _ => None,
                }),
            _ => None,
        })
}

fn find_slider_packet_field<'a>(
    packet: &'a StructValue,
    slider_name: &str,
    field_name: &str,
) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| name == "sliders")
        .and_then(|(_, value)| match value {
            Value::Struct(group) => group
                .fields
                .iter()
                .find(|(name, _)| name == slider_name)
                .and_then(|(_, value)| match value {
                    Value::Struct(slider) => slider
                        .fields
                        .iter()
                        .find(|(name, _)| name == field_name)
                        .map(|(_, value)| value),
                    _ => None,
                }),
            _ => None,
        })
}

fn normalize_control_value(value: i64, min: i64, max: i64) -> usize {
    let max = max.max(min + 1);
    let clamped = value.clamp(min, max);
    (((clamped - min) * 127) / (max - min)) as usize
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

fn parse_shader_flow_state(raw: &str) -> Result<ShaderFlowState, String> {
    match raw {
        "pass_ready" => Ok(ShaderFlowState::PassReady),
        "frame_ready" => Ok(ShaderFlowState::FrameReady),
        other => Err(format!("unknown shader flow state `{other}`")),
    }
}

fn unwrap_data_window(value: Value) -> Value {
    match value {
        Value::DataWindow(window) => (*window.base).clone(),
        other => other,
    }
}

fn resolve_draw_count(
    state: &ExecutionState,
    node: &Node,
    index: usize,
    label: &str,
) -> Result<i64, String> {
    let raw = &node.op.args[index];
    if let Ok(value) = raw.parse::<i64>() {
        return Ok(value);
    }
    match state.expect_value(raw)? {
        Value::Int(value) => Ok(*value),
        Value::I32(value) => Ok(*value as i64),
        Value::Bool(value) => Ok(if *value { 1 } else { 0 }),
        other => Err(format!(
            "node `{}` expects integer-like {} value, got {}",
            node.name, label, other
        )),
    }
}

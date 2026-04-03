use std::collections::BTreeMap;

use nuis_semantics::model::{NirBinaryOp, NirExpr, NirFunction, NirModule, NirStmt};
use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

use crate::registry::NustarPackageManifest;

pub fn lower_nir_to_yir(
    module: &NirModule,
    nustar_manifest: &NustarPackageManifest,
) -> Result<YirModule, String> {
    dispatch_nustar_lowering(module, nustar_manifest)
}

fn dispatch_nustar_lowering(
    module: &NirModule,
    nustar_manifest: &NustarPackageManifest,
) -> Result<YirModule, String> {
    if nustar_manifest.domain_family != module.domain {
        return Err(format!(
            "nustar package `{}` cannot lower mod domain `{}`",
            nustar_manifest.package_id, module.domain
        ));
    }
    match nustar_manifest.yir_lowering_entry.as_str() {
        "cpu.yir.lowering.v1" => lower_nir_to_yir_builtin_cpu(module),
        other => Err(format!(
            "nuisc scheduler has no bootstrap compatibility shim for lowering entry `{other}`; this must be provided by the loaded nustar implementation"
        )),
    }
}

fn lower_nir_to_yir_builtin_cpu(module: &NirModule) -> Result<YirModule, String> {
    if module.domain != "cpu" {
        return Err(format!(
            "minimal nuisc lowering currently only supports `mod cpu`, found `{}`",
            module.domain
        ));
    }

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .ok_or_else(|| "minimal nuisc lowering expects `fn main()`".to_owned())?;

    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();

    let mut yir = YirModule::new("0.1");
    yir.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.arm64"),
    });

    let mut state = LoweringState {
        yir: &mut yir,
        function_map,
        value_counter: 0,
        print_counter: 0,
        call_stack: Vec::new(),
    };

    let mut bindings = BTreeMap::<String, String>::new();
    lower_function_body(main, &mut state, &mut bindings, true)?;

    Ok(yir)
}

struct LoweringState<'a> {
    yir: &'a mut YirModule,
    function_map: BTreeMap<&'a str, &'a NirFunction>,
    value_counter: usize,
    print_counter: usize,
    call_stack: Vec<String>,
}

fn lower_function_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    allow_implicit_return: bool,
) -> Result<Option<String>, String> {
    for stmt in &function.body {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Print(value) => {
                let lowered = lower_expr(value, state, bindings)?;
                let print_name = format!("print_{}", state.print_counter);
                state.print_counter += 1;
                state.yir.nodes.push(Node {
                    name: print_name.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "print".to_owned(),
                        args: vec![lowered.clone()],
                    },
                });
                push_dep_edges(state, &lowered, &print_name);
                state.yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: lowered,
                    to: print_name,
                });
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if let Some(returned) =
                    lower_if_stmt(condition, then_body, else_body, state, bindings)?
                {
                    return Ok(Some(returned));
                }
            }
            NirStmt::Expr(expr) => {
                let _ = lower_expr(expr, state, bindings)?;
            }
            NirStmt::Return(value) => {
                return match value {
                    Some(value) => Ok(Some(lower_expr(value, state, bindings)?)),
                    None => Ok(None),
                };
            }
        }
    }

    if allow_implicit_return {
        Ok(None)
    } else {
        Err(format!(
            "function `{}` ended without `return` in expression-call lowering",
            function.name
        ))
    }
}

fn lower_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    match expr {
        NirExpr::Bool(value) => {
            let name = next_name(state, "bool");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "const_bool".to_owned(),
                    args: vec![value.to_string()],
                },
            });
            Ok(name)
        }
        NirExpr::Text(text) => {
            let name = next_name(state, "text");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "text".to_owned(),
                    args: vec![text.clone()],
                },
            });
            Ok(name)
        }
        NirExpr::Int(value) => {
            let name = next_name(state, "int");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "const_i64".to_owned(),
                    args: vec![value.to_string()],
                },
            });
            Ok(name)
        }
        NirExpr::Var(name) => bindings
            .get(name)
            .cloned()
            .ok_or_else(|| format!("minimal nuisc lowering found unbound variable `{name}`")),
        NirExpr::Null => {
            let name = next_name(state, "null");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "null".to_owned(),
                    args: vec![],
                },
            });
            Ok(name)
        }
        NirExpr::Borrow(value) => lower_unary_cpu_expr("borrow", value, state, bindings),
        NirExpr::Move(value) => {
            let ptr = lower_expr(value, state, bindings)?;
            let name = next_name(state, "move");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "move_ptr".to_owned(),
                    args: vec![ptr.clone()],
                },
            });
            push_dep_edges(state, &ptr, &name);
            push_lifetime_edge(state, &ptr, &name);
            Ok(name)
        }
        NirExpr::AllocNode { value, next } => {
            let value_name = lower_expr(value, state, bindings)?;
            let next_ptr_name = lower_expr(next, state, bindings)?;
            let name = next_name(state, "alloc_node");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "alloc_node".to_owned(),
                    args: vec![value_name.clone(), next_ptr_name.clone()],
                },
            });
            push_dep_edges(state, &value_name, &name);
            push_dep_edges(state, &next_ptr_name, &name);
            Ok(name)
        }
        NirExpr::AllocBuffer { len, fill } => {
            let len_name = lower_expr(len, state, bindings)?;
            let fill_name = lower_expr(fill, state, bindings)?;
            let name = next_name(state, "alloc_buffer");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "alloc_buffer".to_owned(),
                    args: vec![len_name.clone(), fill_name.clone()],
                },
            });
            push_dep_edges(state, &len_name, &name);
            push_dep_edges(state, &fill_name, &name);
            Ok(name)
        }
        NirExpr::Instantiate { domain, unit } => {
            let name = next_name(state, "instantiate_unit");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "instantiate_unit".to_owned(),
                    args: vec![domain.clone(), unit.clone()],
                },
            });
            Ok(name)
        }
        NirExpr::DataBindCore(core_index) => {
            ensure_fabric_resource(state.yir);
            let name = next_name(state, "data_bind_core");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "bind_core".to_owned(),
                    args: vec![core_index.to_string()],
                },
            });
            Ok(name)
        }
        NirExpr::DataMarker(tag) => {
            ensure_fabric_resource(state.yir);
            let name = next_name(state, "data_marker");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "marker".to_owned(),
                    args: vec![tag.clone()],
                },
            });
            Ok(name)
        }
        NirExpr::DataOutputPipe(value) => {
            ensure_fabric_resource(state.yir);
            let value_name = lower_expr(value, state, bindings)?;
            let name = next_name(state, "data_output_pipe");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "output_pipe".to_owned(),
                    args: vec![value_name.clone()],
                },
            });
            push_dep_edges(state, &value_name, &name);
            Ok(name)
        }
        NirExpr::DataInputPipe(pipe) => {
            ensure_fabric_resource(state.yir);
            let pipe_name = lower_expr(pipe, state, bindings)?;
            let name = next_name(state, "data_input_pipe");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "input_pipe".to_owned(),
                    args: vec![pipe_name.clone()],
                },
            });
            push_dep_edges(state, &pipe_name, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: pipe_name,
                to: name.clone(),
            });
            Ok(name)
        }
        NirExpr::DataCopyWindow { input, offset, len } => {
            ensure_fabric_resource(state.yir);
            let input_name = lower_expr(input, state, bindings)?;
            let offset_name = lower_expr(offset, state, bindings)?;
            let len_name = lower_expr(len, state, bindings)?;
            let name = next_name(state, "data_copy_window");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "copy_window".to_owned(),
                    args: vec![input_name.clone(), offset_name.clone(), len_name.clone()],
                },
            });
            push_dep_edges(state, &input_name, &name);
            push_dep_edges(state, &offset_name, &name);
            push_dep_edges(state, &len_name, &name);
            Ok(name)
        }
        NirExpr::DataImmutableWindow { input, offset, len } => {
            ensure_fabric_resource(state.yir);
            let input_name = lower_expr(input, state, bindings)?;
            let offset_name = lower_expr(offset, state, bindings)?;
            let len_name = lower_expr(len, state, bindings)?;
            let name = next_name(state, "data_immutable_window");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "immutable_window".to_owned(),
                    args: vec![input_name.clone(), offset_name.clone(), len_name.clone()],
                },
            });
            push_dep_edges(state, &input_name, &name);
            push_dep_edges(state, &offset_name, &name);
            push_dep_edges(state, &len_name, &name);
            Ok(name)
        }
        NirExpr::DataHandleTable(entries) => {
            ensure_fabric_resource(state.yir);
            let name = next_name(state, "data_handle_table");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "handle_table".to_owned(),
                    args: entries
                        .iter()
                        .map(|(slot, resource)| format!("{slot}={resource}"))
                        .collect(),
                },
            });
            Ok(name)
        }
        NirExpr::CpuBindCore(core_index) => {
            let name = next_name(state, "cpu_bind_core");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "bind_core".to_owned(),
                    args: vec![core_index.to_string()],
                },
            });
            Ok(name)
        }
        NirExpr::CpuWindow { width, height, title } => {
            let name = next_name(state, "cpu_window");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "window".to_owned(),
                    args: vec![width.to_string(), height.to_string(), title.clone()],
                },
            });
            Ok(name)
        }
        NirExpr::CpuInputI64 {
            channel,
            default,
            min,
            max,
            step,
        } => {
            let name = next_name(state, "cpu_input_i64");
            let mut args = vec![channel.clone(), default.to_string()];
            if let (Some(min), Some(max), Some(step)) = (min, max, step) {
                args.push(min.to_string());
                args.push(max.to_string());
                args.push(step.to_string());
            }
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "input_i64".to_owned(),
                    args,
                },
            });
            Ok(name)
        }
        NirExpr::CpuTickI64 { start, step } => {
            let name = next_name(state, "cpu_tick_i64");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "tick_i64".to_owned(),
                    args: vec![start.to_string(), step.to_string()],
                },
            });
            Ok(name)
        }
        NirExpr::CpuPresentFrame(frame) => {
            let frame_name = lower_expr(frame, state, bindings)?;
            let name = next_name(state, "cpu_present_frame");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "present_frame".to_owned(),
                    args: vec![frame_name.clone()],
                },
            });
            push_xfer_edge(state, &frame_name, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: frame_name,
                to: name.clone(),
            });
            Ok(name)
        }
        NirExpr::CpuExternCall {
            abi,
            interface: _,
            callee,
            args,
        } => {
            let lowered_args = args
                .iter()
                .map(|arg| lower_expr(arg, state, bindings))
                .collect::<Result<Vec<_>, _>>()?;
            let name = next_name(state, "cpu_extern_call");
            let mut op_args = vec![abi.clone(), callee.clone()];
            op_args.extend(lowered_args.clone());
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "extern_call_i64".to_owned(),
                    args: op_args,
                },
            });
            for arg in lowered_args {
                push_dep_edges(state, &arg, &name);
            }
            Ok(name)
        }
        NirExpr::ShaderProfileTargetRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "target")
        }
        NirExpr::ShaderProfileViewportRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "viewport")
        }
        NirExpr::ShaderProfilePipelineRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "pipeline")
        }
        NirExpr::ShaderProfileVertexCountRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "vertex_count")
        }
        NirExpr::ShaderProfileInstanceCountRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "instance_count")
        }
        NirExpr::ShaderProfilePacketColorSlotRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "packet_color_slot")
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "packet_speed_slot")
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "packet_radius_slot")
        }
        NirExpr::DataProfileBindCoreRef { unit } => {
            lower_project_profile_ref(state, "data", unit, "bind_core")
        }
        NirExpr::DataProfileWindowOffsetRef { unit } => {
            lower_project_profile_ref(state, "data", unit, "window_offset")
        }
        NirExpr::DataProfileUplinkLenRef { unit } => {
            lower_project_profile_ref(state, "data", unit, "uplink_len")
        }
        NirExpr::DataProfileDownlinkLenRef { unit } => {
            lower_project_profile_ref(state, "data", unit, "downlink_len")
        }
        NirExpr::DataProfileHandleTableRef { unit } => {
            lower_project_profile_ref(state, "data", unit, "handle_table")
        }
        NirExpr::DataProfileMarkerRef { unit, tag } => {
            lower_project_profile_ref(state, "data", unit, &format!("marker:{tag}"))
        }
        NirExpr::ShaderTarget {
            format,
            width,
            height,
        } => {
            ensure_shader_resource(state.yir);
            let name = next_name(state, "shader_target");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "shader0".to_owned(),
                op: Operation {
                    module: "shader".to_owned(),
                    instruction: "target".to_owned(),
                    args: vec![format.clone(), width.to_string(), height.to_string()],
                },
            });
            Ok(name)
        }
        NirExpr::ShaderViewport { width, height } => {
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
            Ok(name)
        }
        NirExpr::ShaderPipeline { name: pipe_name, topology } => {
            ensure_shader_resource(state.yir);
            let name = next_name(state, "shader_pipeline");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "shader0".to_owned(),
                op: Operation {
                    module: "shader".to_owned(),
                    instruction: "pipeline".to_owned(),
                    args: vec![pipe_name.clone(), topology.clone()],
                },
            });
            Ok(name)
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
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
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
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
        NirExpr::LoadValue(value) => lower_unary_cpu_expr("load_value", value, state, bindings),
        NirExpr::LoadNext(value) => lower_unary_cpu_expr("load_next", value, state, bindings),
        NirExpr::BufferLen(value) => lower_unary_cpu_expr("buffer_len", value, state, bindings),
        NirExpr::IsNull(value) => lower_unary_cpu_expr("is_null", value, state, bindings),
        NirExpr::LoadAt { buffer, index } => {
            let buffer_name = lower_expr(buffer, state, bindings)?;
            let index_name = lower_expr(index, state, bindings)?;
            let name = next_name(state, "load_at");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "load_at".to_owned(),
                    args: vec![buffer_name.clone(), index_name.clone()],
                },
            });
            push_dep_edges(state, &buffer_name, &name);
            push_dep_edges(state, &index_name, &name);
            Ok(name)
        }
        NirExpr::StoreValue { target, value } => {
            let target_name = lower_expr(target, state, bindings)?;
            let value_name = lower_expr(value, state, bindings)?;
            let name = next_name(state, "store_value");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "store_value".to_owned(),
                    args: vec![target_name.clone(), value_name.clone()],
                },
            });
            push_dep_edges(state, &target_name, &name);
            push_dep_edges(state, &value_name, &name);
            push_lifetime_edge(state, &target_name, &name);
            Ok(name)
        }
        NirExpr::StoreNext { target, next } => {
            let target_name = lower_expr(target, state, bindings)?;
            let next_name_value = lower_expr(next, state, bindings)?;
            let name = next_name(state, "store_next");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "store_next".to_owned(),
                    args: vec![target_name.clone(), next_name_value.clone()],
                },
            });
            push_dep_edges(state, &target_name, &name);
            push_dep_edges(state, &next_name_value, &name);
            push_lifetime_edge(state, &target_name, &name);
            Ok(name)
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            let buffer_name = lower_expr(buffer, state, bindings)?;
            let index_name = lower_expr(index, state, bindings)?;
            let value_name = lower_expr(value, state, bindings)?;
            let name = next_name(state, "store_at");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "store_at".to_owned(),
                    args: vec![buffer_name.clone(), index_name.clone(), value_name.clone()],
                },
            });
            push_dep_edges(state, &buffer_name, &name);
            push_dep_edges(state, &index_name, &name);
            push_dep_edges(state, &value_name, &name);
            push_lifetime_edge(state, &buffer_name, &name);
            Ok(name)
        }
        NirExpr::Free(value) => {
            let ptr = lower_expr(value, state, bindings)?;
            let name = next_name(state, "free");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "free".to_owned(),
                    args: vec![ptr.clone()],
                },
            });
            push_dep_edges(state, &ptr, &name);
            push_lifetime_edge(state, &ptr, &name);
            Ok(name)
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let instruction = match op {
                NirBinaryOp::Add => "add",
                NirBinaryOp::Sub => "sub",
                NirBinaryOp::Mul => "mul",
                NirBinaryOp::Div => "div",
            };
            let name = next_name(state, instruction);
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: instruction.to_owned(),
                    args: vec![lhs_name.clone(), rhs_name.clone()],
                },
            });
            push_dep_edges(state, &lhs_name, &name);
            push_dep_edges(state, &rhs_name, &name);
            Ok(name)
        }
        NirExpr::Call { callee, args } => lower_call_expr(callee, args, state, bindings),
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let mut call_args = Vec::with_capacity(args.len() + 1);
            call_args.push((**receiver).clone());
            call_args.extend(args.iter().cloned());
            lower_call_expr(method, &call_args, state, bindings)
        }
        NirExpr::StructLiteral { type_name, fields } => {
            let mut args_out = vec![type_name.clone()];
            let name = next_name(state, "struct");
            let mut lowered_fields = Vec::new();
            for (field_name, field_expr) in fields {
                let lowered = lower_expr(field_expr, state, bindings)?;
                lowered_fields.push(lowered.clone());
                args_out.push(format!("{field_name}={lowered}"));
            }
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "struct".to_owned(),
                    args: args_out,
                },
            });
            for lowered in lowered_fields {
                push_dep_edges(state, &lowered, &name);
            }
            Ok(name)
        }
        NirExpr::FieldAccess { base, field } => {
            let base_name = lower_expr(base, state, bindings)?;
            let name = next_name(state, "field");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "field".to_owned(),
                    args: vec![base_name.clone(), field.clone()],
                },
            });
            push_dep_edges(state, &base_name, &name);
            Ok(name)
        }
    }
}

fn lower_if_stmt(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let condition_name = lower_expr(condition, state, bindings)?;
    let lowered = lower_if_pair(condition_name, then_body, else_body, state, bindings)?;
    match lowered {
        LoweredIfOutcome::Bind { name, value } => {
            bindings.insert(name, value);
            Ok(None)
        }
        LoweredIfOutcome::Printed => Ok(None),
        LoweredIfOutcome::Returned(value) => Ok(Some(value)),
    }
}

enum LoweredIfOutcome {
    Bind { name: String, value: String },
    Printed,
    Returned(String),
}

fn lower_if_pair(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<LoweredIfOutcome, String> {
    if then_body.len() != 1 || else_body.len() != 1 {
        return Err(
            "minimal nuisc lowering currently only supports `if` where both branches contain exactly one statement"
                .to_owned(),
        );
    }

    match (&then_body[0], &else_body[0]) {
        (NirStmt::Print(lhs), NirStmt::Print(rhs)) => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
            let print_name = format!("print_{}", state.print_counter);
            state.print_counter += 1;
            state.yir.nodes.push(Node {
                name: print_name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "print".to_owned(),
                    args: vec![selected.clone()],
                },
            });
            push_dep_edges(state, &selected, &print_name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: selected,
                to: print_name,
            });
            Ok(LoweredIfOutcome::Printed)
        }
        (
            NirStmt::Let {
                name: lhs_name,
                value: lhs_value,
                ..
            },
            NirStmt::Let {
                name: rhs_name,
                value: rhs_value,
                ..
            },
        )
        | (
            NirStmt::Const {
                name: lhs_name,
                value: lhs_value,
                ..
            },
            NirStmt::Const {
                name: rhs_name,
                value: rhs_value,
                ..
            },
        ) if lhs_name == rhs_name => {
            let lhs_value = lower_expr(lhs_value, state, bindings)?;
            let rhs_value = lower_expr(rhs_value, state, bindings)?;
            let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
            Ok(LoweredIfOutcome::Bind {
                name: lhs_name.clone(),
                value: selected,
            })
        }
        (NirStmt::Return(Some(lhs)), NirStmt::Return(Some(rhs))) => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
            Ok(LoweredIfOutcome::Returned(selected))
        }
        _ => Err(
            "minimal nuisc lowering currently only supports `if` branches as matching `print`, matching `let/const`, or `return <expr>`"
                .to_owned(),
        ),
    }
}

fn lower_select(
    condition_name: String,
    then_name: String,
    else_name: String,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    let name = next_name(state, "select");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "select".to_owned(),
            args: vec![condition_name.clone(), then_name.clone(), else_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &then_name, &name);
    push_dep_edges(state, &else_name, &name);
    Ok(name)
}

fn lower_call_expr(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if callee == "print" {
        return Err("`print(...)` is only valid as a statement".to_owned());
    }

    if state.call_stack.iter().any(|active| active == callee) {
        return Err(format!(
            "recursive function call `{callee}` is not yet supported by minimal nuisc lowering"
        ));
    }

    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;

    if function.params.len() != args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let mut local_bindings = BTreeMap::new();
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let lowered = lower_expr(arg, state, bindings)?;
        local_bindings.insert(param.name.clone(), lowered);
    }

    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();

    returned.ok_or_else(|| format!("function `{callee}` did not return a value"))
}

fn next_name(state: &mut LoweringState<'_>, prefix: &str) -> String {
    let name = format!("{prefix}_{}", state.value_counter);
    state.value_counter += 1;
    name
}

fn lower_project_profile_ref(
    state: &mut LoweringState<'_>,
    domain: &str,
    unit: &str,
    slot: &str,
) -> Result<String, String> {
    let name = next_name(state, "project_profile_ref");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "project_profile_ref".to_owned(),
            args: vec![domain.to_owned(), unit.to_owned(), slot.to_owned()],
        },
    });
    Ok(name)
}

fn ensure_fabric_resource(yir: &mut YirModule) {
    if yir.resources.iter().any(|resource| resource.name == "fabric0") {
        return;
    }
    yir.resources.push(Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    });
}

fn ensure_shader_resource(yir: &mut YirModule) {
    if yir.resources.iter().any(|resource| resource.name == "shader0") {
        return;
    }
    yir.resources.push(Resource {
        name: "shader0".to_owned(),
        kind: ResourceKind::parse("shader.render"),
    });
}

fn push_dep_edges(state: &mut LoweringState<'_>, from: &str, to: &str) {
    let from_resource = state
        .yir
        .nodes
        .iter()
        .find(|node| node.name == from)
        .map(|node| node.resource.as_str());
    let to_resource = state
        .yir
        .nodes
        .iter()
        .find(|node| node.name == to)
        .map(|node| node.resource.as_str());
    if let (Some(from_resource), Some(to_resource)) = (from_resource, to_resource) {
        if from_resource != to_resource {
            push_xfer_edge(state, from, to);
            return;
        }
    }
    state.yir.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

fn push_xfer_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    state.yir.edges.push(Edge {
        kind: EdgeKind::CrossDomainExchange,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

fn push_lifetime_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    state.yir.edges.push(Edge {
        kind: EdgeKind::Lifetime,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

fn lower_unary_cpu_expr(
    instruction: &str,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let lowered = lower_expr(value, state, bindings)?;
    let name = next_name(state, instruction);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![lowered.clone()],
        },
    });
    push_dep_edges(state, &lowered, &name);
    Ok(name)
}

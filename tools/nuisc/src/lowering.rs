use std::{collections::BTreeMap, path::Path};

use nuis_semantics::model::{NirBinaryOp, NirExpr, NirFunction, NirModule, NirStmt};
use yir_core::{
    Edge, EdgeKind, Node, Operation, Resource, ResourceKind, SemanticOp, TaskLifecycleState,
    YirResultRole, YirResultState, YirModule,
};

use crate::registry::NustarPackageManifest;

pub fn lower_nir_to_yir(
    module: &NirModule,
    nustar_manifest: &NustarPackageManifest,
) -> Result<YirModule, String> {
    dispatch_nustar_lowering(module, nustar_manifest)
}

trait BootstrapLoweringProvider {
    fn lowering_entry(&self) -> &'static str;
    fn lower(&self, module: &NirModule) -> Result<YirModule, String>;
}

#[derive(Clone, Copy)]
enum ResultLoweringDomain {
    Data,
    Shader,
    Kernel,
}

impl ResultLoweringDomain {
    fn module_name(self) -> &'static str {
        match self {
            Self::Data => "data",
            Self::Shader => "shader",
            Self::Kernel => "kernel",
        }
    }

    fn resource_name(self) -> &'static str {
        match self {
            Self::Data => "fabric0",
            Self::Shader => "shader0",
            Self::Kernel => "kernel0",
        }
    }

    fn ensure_resource(self, yir: &mut YirModule) {
        match self {
            Self::Data => ensure_fabric_resource(yir),
            Self::Shader => ensure_shader_resource(yir),
            Self::Kernel => ensure_kernel_resource(yir),
        }
    }
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
    let provider = bootstrap_lowering_provider(nustar_manifest.yir_lowering_entry.as_str())
        .ok_or_else(|| {
            format!(
                "nuisc scheduler has no bootstrap compatibility shim for lowering entry `{}`; this must be provided by the loaded nustar implementation",
                nustar_manifest.yir_lowering_entry
            )
        })?;
    provider.lower(module)
}

fn bootstrap_lowering_provider(entry: &str) -> Option<&'static dyn BootstrapLoweringProvider> {
    static CPU_PROVIDER: CpuBootstrapLoweringProvider = CpuBootstrapLoweringProvider;
    [(&CPU_PROVIDER as &dyn BootstrapLoweringProvider)]
        .into_iter()
        .find(|provider| provider.lowering_entry() == entry)
}

struct CpuBootstrapLoweringProvider;

impl BootstrapLoweringProvider for CpuBootstrapLoweringProvider {
    fn lowering_entry(&self) -> &'static str {
        "cpu.yir.lowering.v1"
    }

    fn lower(&self, module: &NirModule) -> Result<YirModule, String> {
        lower_nir_to_yir_builtin_cpu(module)
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
        await_counter: 0,
        call_stack: Vec::new(),
    };

    let mut bindings = BTreeMap::<String, String>::new();
    lower_function_body(main, &mut state, &mut bindings, true)?;
    assign_default_lanes(&mut yir);

    Ok(yir)
}

pub fn assign_default_lanes(module: &mut YirModule) {
    let lane_policy = load_declared_lane_policy(module);
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource.kind.family()))
        .collect::<BTreeMap<_, _>>();

    module.node_lanes.clear();
    for node in &module.nodes {
        let family = resource_families
            .get(node.resource.as_str())
            .copied()
            .unwrap_or("unknown");
        let lane = default_lane_for_node(&lane_policy, family, node);
        module.node_lanes.insert(node.name.clone(), lane.to_owned());
    }
}

fn load_declared_lane_policy(module: &YirModule) -> BTreeMap<String, String> {
    let mut policy = BTreeMap::<String, String>::new();
    for family in module
        .resources
        .iter()
        .map(|resource| resource.kind.family().to_owned())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let Ok(manifest) =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &family)
        else {
            continue;
        };
        for entry in manifest.default_lanes {
            let Some((pattern, lane)) = entry.split_once('=') else {
                continue;
            };
            if !pattern.is_empty() && !lane.is_empty() {
                policy.insert(pattern.trim().to_owned(), lane.trim().to_owned());
            }
        }
    }
    policy
}

fn default_lane_for_node<'a>(
    lane_policy: &'a BTreeMap<String, String>,
    family: &str,
    node: &'a Node,
) -> &'a str {
    if node.name.starts_with("project_link_") {
        return "contract";
    }
    if node.name.starts_with("project_profile_") {
        if family == "cpu" && node.op.instruction == "text" {
            return "contract";
        }
        if family == "cpu" {
            return "profile";
        }
        if family == "data" {
            return match node.op.semantic_op() {
                SemanticOp::DataImmutableWindow => "profile_uplink",
                SemanticOp::DataCopyWindow | SemanticOp::DataInputPipe => "profile_downlink",
                SemanticOp::DataHandleTable | SemanticOp::DataBindCore | SemanticOp::DataMarker => {
                    "profile_control"
                }
                SemanticOp::DataMove => "profile_fabric",
                _ => "profile_data",
            };
        }
        if family == "shader" {
            return "profile_setup";
        }
        if family == "kernel" || family == "npu" {
            return "profile_compute";
        }
    }
    if let Some(lane) = lane_policy.get(&node.op.full_name()) {
        return lane.as_str();
    }
    match family {
        "cpu" => match node.op.semantic_op() {
            SemanticOp::CpuAllocNode
            | SemanticOp::CpuAllocBuffer
            | SemanticOp::CpuBorrow
            | SemanticOp::CpuBorrowEnd
            | SemanticOp::CpuMovePtr
            | SemanticOp::CpuLoadValue
            | SemanticOp::CpuLoadNext
            | SemanticOp::CpuBufferLen
            | SemanticOp::CpuLoadAt
            | SemanticOp::CpuStoreValue
            | SemanticOp::CpuStoreNext
            | SemanticOp::CpuStoreAt
            | SemanticOp::CpuFree => "mem",
            _ => match node.op.instruction.as_str() {
                "window" | "input_i64" | "tick_i64" | "extern_call_i64" | "present_frame"
                | "print" | "bind_core" | "instantiate_unit" => "main",
                _ => "main",
            },
        },
        "data" => match node.op.semantic_op() {
            SemanticOp::DataImmutableWindow | SemanticOp::DataOutputPipe => "uplink",
            SemanticOp::DataCopyWindow | SemanticOp::DataInputPipe => "downlink",
            SemanticOp::DataHandleTable | SemanticOp::DataBindCore | SemanticOp::DataMarker => {
                "control"
            }
            SemanticOp::DataMove => "fabric",
            _ => "fabric",
        },
        "shader" => match node.op.semantic_op() {
            SemanticOp::ShaderBeginPass | SemanticOp::ShaderDrawInstanced => "render",
            SemanticOp::ShaderPipeline | SemanticOp::ShaderInlineWgsl => "setup",
            _ => "setup",
        },
        "kernel" | "npu" => "compute",
        _ => "main",
    }
}

struct LoweringState<'a> {
    yir: &'a mut YirModule,
    function_map: BTreeMap<&'a str, &'a NirFunction>,
    value_counter: usize,
    print_counter: usize,
    await_counter: usize,
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
            NirStmt::Await(value) => {
                let awaited = match value {
                    NirExpr::Call { callee, args } => {
                        lower_async_call_boundary(callee, args, state, bindings)?
                    }
                    _ => lower_expr(value, state, bindings)?,
                };
                let await_name = push_await_node(state, &awaited);
                state.yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: awaited,
                    to: await_name,
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
        NirExpr::Await(value) => {
            let awaited = match value.as_ref() {
                NirExpr::Call { callee, args } => {
                    lower_async_call_boundary(callee, args, state, bindings)?
                }
                _ => lower_expr(value, state, bindings)?,
            };
            let await_name = push_await_node(state, &awaited);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: awaited,
                to: await_name.clone(),
            });
            Ok(await_name)
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
        NirExpr::BorrowEnd(value) => lower_unary_cpu_expr("borrow_end", value, state, bindings),
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
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => {
            let expanded = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new((**base).clone()),
                        rhs: Box::new((**delta).clone()),
                    }),
                    rhs: Box::new(NirExpr::ShaderProfilePacketColorSlotRef { unit: unit.clone() }),
                }),
                rhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::ShaderProfileMaterialModeRef { unit: unit.clone() }),
                    rhs: Box::new(NirExpr::ShaderProfilePassKindRef { unit: unit.clone() }),
                }),
            };
            lower_expr(&expanded, state, bindings)
        }
        NirExpr::ShaderProfileSpeedSeed {
            unit,
            delta,
            scale,
            base,
        } => {
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
                                lhs: Box::new((**delta).clone()),
                                rhs: Box::new((**scale).clone()),
                            }),
                            rhs: Box::new((**base).clone()),
                        }),
                        rhs: Box::new(NirExpr::ShaderProfileInstanceCountRef {
                            unit: unit.clone(),
                        }),
                    }),
                    rhs: Box::new(NirExpr::ShaderProfilePacketSpeedSlotRef { unit: unit.clone() }),
                }),
                rhs: Box::new(NirExpr::ShaderProfilePacketTagRef { unit: unit.clone() }),
            };
            lower_expr(&expanded, state, bindings)
        }
        NirExpr::ShaderProfileRadiusSeed { unit, base, delta } => {
            let expanded = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Binary {
                            op: NirBinaryOp::Add,
                            lhs: Box::new((**base).clone()),
                            rhs: Box::new((**delta).clone()),
                        }),
                        rhs: Box::new(NirExpr::ShaderProfileVertexCountRef { unit: unit.clone() }),
                    }),
                    rhs: Box::new(NirExpr::ShaderProfilePacketRadiusSlotRef { unit: unit.clone() }),
                }),
                rhs: Box::new(NirExpr::ShaderProfilePacketFieldCountRef { unit: unit.clone() }),
            };
            lower_expr(&expanded, state, bindings)
        }
        NirExpr::ShaderProfilePacket {
            unit,
            color,
            speed,
            radius,
        } => {
            let color_name = lower_expr(color, state, bindings)?;
            let speed_name = lower_expr(speed, state, bindings)?;
            let radius_name = lower_expr(radius, state, bindings)?;
            let name = next_name(state, "shader_profile_packet");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "struct".to_owned(),
                    args: vec![
                        format!("{unit}Packet"),
                        format!("color={color_name}"),
                        format!("speed={speed_name}"),
                        format!("radius_scale={radius_name}"),
                    ],
                },
            });
            push_dep_edges(state, &color_name, &name);
            push_dep_edges(state, &speed_name, &name);
            push_dep_edges(state, &radius_name, &name);
            Ok(name)
        }
        NirExpr::DataProfileSendUplink { unit, input } => lower_data_profile_send(
            state,
            bindings,
            unit,
            input,
            "data_immutable_window",
            "immutable_window",
            "uplink_len",
        ),
        NirExpr::DataProfileSendDownlink { unit, input } => lower_data_profile_send(
            state,
            bindings,
            unit,
            input,
            "data_copy_window",
            "copy_window",
            "downlink_len",
        ),
        NirExpr::ShaderProfileRender { unit, packet } => {
            let expanded = NirExpr::ShaderDrawInstanced {
                pass: Box::new(NirExpr::ShaderBeginPass {
                    target: Box::new(NirExpr::ShaderProfileTargetRef { unit: unit.clone() }),
                    pipeline: Box::new(NirExpr::ShaderProfilePipelineRef { unit: unit.clone() }),
                    viewport: Box::new(NirExpr::ShaderProfileViewportRef { unit: unit.clone() }),
                }),
                packet: Box::new((**packet).clone()),
                vertex_count: Box::new(NirExpr::ShaderProfileVertexCountRef { unit: unit.clone() }),
                instance_count: Box::new(NirExpr::ShaderProfileInstanceCountRef {
                    unit: unit.clone(),
                }),
            };
            lower_expr(&expanded, state, bindings)
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
        NirExpr::DataResult { value, state: flow } => {
            lower_result_observe_node(
                state,
                bindings,
                ResultLoweringDomain::Data,
                value,
                "data_result",
                flow.render(),
            )
        }
        NirExpr::DataReady(result) => {
            lower_result_unary_value_effect(
                state,
                bindings,
                ResultLoweringDomain::Data,
                result,
                "data_ready",
                "is_ready",
            )
        }
        NirExpr::DataMoved(result) => {
            lower_result_unary_value_effect(
                state,
                bindings,
                ResultLoweringDomain::Data,
                result,
                "data_moved",
                "is_moved",
            )
        }
        NirExpr::DataWindowed(result) => {
            lower_result_unary_value_effect(
                state,
                bindings,
                ResultLoweringDomain::Data,
                result,
                "data_windowed",
                "is_windowed",
            )
        }
        NirExpr::DataValue(result) => {
            lower_result_unary_value_effect(
                state,
                bindings,
                ResultLoweringDomain::Data,
                result,
                "data_value",
                "value",
            )
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
        NirExpr::CpuWindow {
            width,
            height,
            title,
        } => {
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
        NirExpr::CpuSpawn { callee, args } => {
            let returned = lower_async_call_boundary(callee, args, state, bindings)?;
            let name = next_name(state, "cpu_spawn_task");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "spawn_task".to_owned(),
                    args: vec![callee.clone(), returned.clone()],
                },
            });
            push_dep_edges(state, &returned, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: returned,
                to: name.clone(),
            });
            Ok(name)
        }
        NirExpr::CpuJoin(task) => {
            let task_name = lower_expr(task, state, bindings)?;
            let name = next_name(state, "cpu_join");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "join".to_owned(),
                    args: vec![task_name.clone()],
                },
            });
            push_dep_edges(state, &task_name, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: task_name,
                to: name.clone(),
            });
            Ok(name)
        }
        NirExpr::CpuCancel(task) => {
            let task_name = lower_expr(task, state, bindings)?;
            let name = next_name(state, "cpu_cancel");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "cancel".to_owned(),
                    args: vec![task_name.clone()],
                },
            });
            push_dep_edges(state, &task_name, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: task_name,
                to: name.clone(),
            });
            Ok(name)
        }
        NirExpr::CpuJoinResult(task) => {
            lower_task_result_entry_node(state, bindings, task)
        }
        NirExpr::CpuTaskCompleted(result) => lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::Completed)),
        ),
        NirExpr::CpuTaskTimedOut(result) => lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::TimedOut)),
        ),
        NirExpr::CpuTaskCancelled(result) => lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::Cancelled)),
        ),
        NirExpr::CpuTaskValue(result) => lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::PayloadExtractor,
            None,
        ),
        NirExpr::CpuTimeout { task, limit } => {
            let task_name = lower_expr(task, state, bindings)?;
            let limit_name = lower_expr(limit, state, bindings)?;
            let name = next_name(state, "cpu_timeout");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "timeout".to_owned(),
                    args: vec![task_name.clone(), limit_name.clone()],
                },
            });
            push_dep_edges(state, &task_name, &name);
            push_dep_edges(state, &limit_name, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: task_name,
                to: name.clone(),
            });
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: limit_name,
                to: name.clone(),
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
        NirExpr::ShaderProfilePacketTagRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "packet_tag")
        }
        NirExpr::ShaderProfileMaterialModeRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "material_mode")
        }
        NirExpr::ShaderProfilePassKindRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "pass_kind")
        }
        NirExpr::ShaderProfilePacketFieldCountRef { unit } => {
            lower_project_profile_ref(state, "shader", unit, "packet_field_count")
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
        NirExpr::KernelProfileBindCoreRef { unit } => {
            lower_project_profile_ref(state, "kernel", unit, "bind_core")
        }
        NirExpr::KernelProfileQueueDepthRef { unit } => {
            lower_project_profile_ref(state, "kernel", unit, "queue_depth")
        }
        NirExpr::KernelProfileBatchLanesRef { unit } => {
            lower_project_profile_ref(state, "kernel", unit, "batch_lanes")
        }
        NirExpr::KernelResult { value, state: flow } => {
            lower_result_observe_node(
                state,
                bindings,
                ResultLoweringDomain::Kernel,
                value,
                "kernel_result",
                flow.render(),
            )
        }
        NirExpr::KernelConfigReady(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            result,
            "kernel_config_ready",
            "is_config_ready",
        ),
        NirExpr::KernelValue(result) => {
            lower_result_unary_value_effect(
                state,
                bindings,
                ResultLoweringDomain::Kernel,
                result,
                "kernel_value",
                "value",
            )
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
        NirExpr::ShaderPipeline {
            name: pipe_name,
            topology,
        } => {
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
        NirExpr::ShaderInlineWgsl { entry, source } => {
            ensure_shader_resource(state.yir);
            let name = next_name(state, "shader_inline_wgsl");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "shader0".to_owned(),
                op: Operation {
                    module: "shader".to_owned(),
                    instruction: "inline_wgsl".to_owned(),
                    args: vec![entry.clone(), source.clone()],
                },
            });
            Ok(name)
        }
        NirExpr::ShaderResult { value, state: flow } => {
            lower_result_observe_node(
                state,
                bindings,
                ResultLoweringDomain::Shader,
                value,
                "shader_result",
                flow.render(),
            )
        }
        NirExpr::ShaderPassReady(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_pass_ready",
            "is_pass_ready",
        ),
        NirExpr::ShaderFrameReady(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_frame_ready",
            "is_frame_ready",
        ),
        NirExpr::ShaderValue(result) => {
            lower_result_unary_value_effect(
                state,
                bindings,
                ResultLoweringDomain::Shader,
                result,
                "shader_value",
                "value",
            )
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

fn lower_async_call_boundary(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;
    if !function.is_async {
        return lower_call_expr(callee, args, state, bindings);
    }
    if state.call_stack.iter().any(|active| active == callee) {
        return Err(format!(
            "recursive async function call `{callee}` is not yet supported by minimal nuisc lowering"
        ));
    }
    if function.params.len() != args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let mut local_bindings = BTreeMap::new();
    let mut lowered_args = Vec::new();
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let lowered = lower_expr(arg, state, bindings)?;
        lowered_args.push(lowered.clone());
        local_bindings.insert(param.name.clone(), lowered);
    }

    let call_name = next_name(state, "async_call");
    let mut op_args = vec![callee.to_owned()];
    op_args.extend(lowered_args.clone());
    state.yir.nodes.push(Node {
        name: call_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "async_call".to_owned(),
            args: op_args,
        },
    });
    for arg in &lowered_args {
        push_dep_edges(state, arg, &call_name);
    }

    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();
    let returned = returned.ok_or_else(|| format!("function `{callee}` did not return a value"))?;
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: call_name,
        to: returned.clone(),
    });
    Ok(returned)
}

fn push_await_node(state: &mut LoweringState<'_>, awaited: &str) -> String {
    let await_name = format!("await_{}", state.await_counter);
    state.await_counter += 1;
    state.yir.nodes.push(Node {
        name: await_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "await".to_owned(),
            args: vec![awaited.to_owned()],
        },
    });
    push_dep_edges(state, awaited, &await_name);
    await_name
}

fn lower_cpu_unary_value_effect(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    input: &NirExpr,
    prefix: &str,
    instruction: &str,
) -> Result<String, String> {
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: input_name,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_result_observe_node(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    domain: ResultLoweringDomain,
    input: &NirExpr,
    prefix: &str,
    observed_state: &str,
) -> Result<String, String> {
    domain.ensure_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: domain.resource_name().to_owned(),
        op: Operation {
            module: domain.module_name().to_owned(),
            instruction: "observe".to_owned(),
            args: vec![input_name.clone(), observed_state.to_owned()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

fn lower_task_result_entry_node(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    input: &NirExpr,
) -> Result<String, String> {
    let task_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, "cpu_join_result");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "join_result".to_owned(),
            args: vec![task_name.clone()],
        },
    });
    push_dep_edges(state, &task_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: task_name,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_task_result_observer_node(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    input: &NirExpr,
    role: YirResultRole,
    observed_state: Option<YirResultState>,
) -> Result<String, String> {
    let (prefix, instruction) = match (role, observed_state) {
        (YirResultRole::StateProbe, Some(YirResultState::Task(TaskLifecycleState::Completed))) => {
            ("cpu_task_completed", "task_completed")
        }
        (YirResultRole::StateProbe, Some(YirResultState::Task(TaskLifecycleState::TimedOut))) => {
            ("cpu_task_timed_out", "task_timed_out")
        }
        (YirResultRole::StateProbe, Some(YirResultState::Task(TaskLifecycleState::Cancelled))) => {
            ("cpu_task_cancelled", "task_cancelled")
        }
        (YirResultRole::PayloadExtractor, None) => ("cpu_task_value", "task_value"),
        (YirResultRole::Entry, _) => {
            return Err("task result entry must lower through lower_task_result_entry_node".to_owned())
        }
        (YirResultRole::StateProbe, Some(other)) => {
            return Err(format!("unsupported non-task result probe state `{other:?}` for task observer"))
        }
        (YirResultRole::StateProbe, None) => {
            return Err("task state probe requires an explicit task lifecycle state".to_owned())
        }
        (YirResultRole::PayloadExtractor, Some(_)) => {
            return Err("task payload extractor must not carry an explicit result state".to_owned())
        }
    };
    lower_cpu_unary_value_effect(state, bindings, input, prefix, instruction)
}

fn lower_result_unary_value_effect(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    domain: ResultLoweringDomain,
    input: &NirExpr,
    prefix: &str,
    instruction: &str,
) -> Result<String, String> {
    domain.ensure_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: domain.resource_name().to_owned(),
        op: Operation {
            module: domain.module_name().to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
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

fn lower_data_profile_send(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    unit: &str,
    input: &NirExpr,
    window_prefix: &str,
    window_instruction: &str,
    len_slot: &str,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);

    let input_name = lower_expr(input, state, bindings)?;
    let offset_name = lower_project_profile_ref(state, "data", unit, "window_offset")?;
    let len_name = lower_project_profile_ref(state, "data", unit, len_slot)?;
    let handle_table_name = lower_project_profile_ref(state, "data", unit, "handle_table")?;

    let window_name = next_name(state, window_prefix);
    state.yir.nodes.push(Node {
        name: window_name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: window_instruction.to_owned(),
            args: vec![input_name.clone(), offset_name.clone(), len_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &window_name);
    push_dep_edges(state, &offset_name, &window_name);
    push_dep_edges(state, &len_name, &window_name);
    push_dep_edges(state, &handle_table_name, &window_name);

    let output_name = next_name(state, "data_output_pipe");
    state.yir.nodes.push(Node {
        name: output_name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "output_pipe".to_owned(),
            args: vec![window_name.clone()],
        },
    });
    push_dep_edges(state, &window_name, &output_name);
    push_dep_edges(state, &handle_table_name, &output_name);

    let input_pipe_name = next_name(state, "data_input_pipe");
    state.yir.nodes.push(Node {
        name: input_pipe_name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "input_pipe".to_owned(),
            args: vec![output_name.clone()],
        },
    });
    push_dep_edges(state, &output_name, &input_pipe_name);
    push_dep_edges(state, &handle_table_name, &input_pipe_name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: output_name,
        to: input_pipe_name.clone(),
    });

    Ok(input_pipe_name)
}

fn ensure_fabric_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "fabric0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    });
}

fn ensure_shader_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "shader0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "shader0".to_owned(),
        kind: ResourceKind::parse("shader.render"),
    });
}

fn ensure_kernel_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "kernel0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "kernel0".to_owned(),
        kind: ResourceKind::parse("kernel.compute"),
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

#[cfg(test)]
mod tests {
    use super::lower_nir_to_yir_builtin_cpu;
    use crate::frontend::parse_nuis_module;
    use yir_core::EdgeKind;

    #[test]
    fn lowers_await_stmt_into_cpu_await_node() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main() {
                await data_profile_bind_core("FabricPlane");
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        let await_node = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "await")
            .unwrap();
        let awaited = await_node.op.args.first().unwrap();
        assert!(yir.edges.iter().any(|edge| edge.from == *awaited
            && edge.to == await_node.name
            && matches!(edge.kind, EdgeKind::Effect)));
    }

    #[test]
    fn lowers_async_call_with_explicit_schedule_boundary() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() {
                await ping();
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        let async_call = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
            .unwrap();
        let await_node = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "await")
            .unwrap();
        assert!(yir.edges.iter().any(|edge| {
            edge.from == async_call.name
                && edge.to == await_node.op.args[0]
                && matches!(edge.kind, EdgeKind::Effect)
        }));
    }

    #[test]
    fn lowers_await_expression_into_value_producing_boundary() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() -> i64 {
                let value: i64 = await ping();
                return value;
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        let async_call = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
            .unwrap();
        let await_node = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "await")
            .unwrap();
        assert!(yir.edges.iter().any(|edge| {
            edge.from == async_call.name
                && edge.to == await_node.op.args[0]
                && matches!(edge.kind, EdgeKind::Effect)
        }));
        assert_eq!(await_node.op.args.len(), 1);
    }

    #[test]
    fn lowers_explicit_task_primitives_into_cpu_effect_nodes() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = spawn(ping());
                cancel(task);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "join"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
    }

    #[test]
    fn lowers_explicit_timeout_primitive_into_cpu_effect_node() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
    }

    #[test]
    fn lowers_data_result_primitives_into_data_nodes() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
                let moved: bool = data_moved(result);
                return data_value(result);
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "data" && node.op.instruction == "observe"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "data" && node.op.instruction == "is_moved"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "data" && node.op.instruction == "value"));
    }

    #[test]
    fn lowers_shader_result_primitives_into_shader_nodes() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let result: ShaderResult<Pass> = shader_result(shader_begin_pass(
                  shader_target("rgba8", 8, 8),
                  shader_pipeline("flat", "triangle"),
                  shader_viewport(8, 8)
                ));
                let ready: bool = shader_pass_ready(result);
                return 1;
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "shader" && node.op.instruction == "observe"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "shader" && node.op.instruction == "is_pass_ready"));
    }

    #[test]
    fn lowers_kernel_result_primitives_into_kernel_nodes() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let result: KernelResult<i64> = kernel_result(kernel_profile_queue_depth("KernelUnit"));
                let ready: bool = kernel_config_ready(result);
                return kernel_value(result);
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "kernel" && node.op.instruction == "is_config_ready"));
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.module == "kernel" && node.op.instruction == "value"));
    }

    #[test]
    fn lowers_join_result_and_task_state_primitives_into_cpu_nodes() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                let result: TaskResult<i64> = join_result(task);
                let done: bool = task_completed(result);
                let timed_out: bool = task_timed_out(result);
                let value: i64 = task_value(result);
                return value;
              }
            }
            "#,
        )
        .unwrap();
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

        let join_result = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
            .unwrap();
        let completed = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
            .unwrap();
        let timed_out = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "task_timed_out")
            .unwrap();
        let value = yir
            .nodes
            .iter()
            .find(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
            .unwrap();

        assert_eq!(completed.op.args, vec![join_result.name.clone()]);
        assert_eq!(timed_out.op.args, vec![join_result.name.clone()]);
        assert_eq!(value.op.args, vec![join_result.name.clone()]);
    }
}

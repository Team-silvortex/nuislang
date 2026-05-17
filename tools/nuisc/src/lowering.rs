use std::{collections::BTreeMap, path::Path};

use nuis_semantics::model::{NirBinaryOp, NirExpr, NirFunction, NirModule, NirStmt};
use yir_core::{
    Edge, EdgeKind, Node, Operation, Resource, ResourceKind, SemanticOp, TaskLifecycleState,
    YirModule, YirResultRole, YirResultState,
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
            packet_type_name,
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
        } => {
            let color_name = lower_expr(color, state, bindings)?;
            let speed_name = lower_expr(speed, state, bindings)?;
            let radius_name = lower_expr(radius, state, bindings)?;
            let accent_name = accent
                .as_ref()
                .map(|expr| lower_expr(expr, state, bindings))
                .transpose()?;
            let toggle_name = toggle_state
                .as_ref()
                .map(|expr| lower_expr(expr, state, bindings))
                .transpose()?;
            let focus_name = focus_index
                .as_ref()
                .map(|expr| lower_expr(expr, state, bindings))
                .transpose()?;
            let name = next_name(state, "shader_profile_packet");
            let packet_type = packet_type_name
                .clone()
                .unwrap_or_else(|| format!("{unit}Packet"));
            let is_nova_panel = packet_type == "NovaPanelPacket";
            let mut panel_group_nodes = Vec::new();
            let args = if is_nova_panel {
                let accent_name = accent_name
                    .as_ref()
                    .expect("nova panel packet must carry header accent");
                let toggle_name = toggle_name
                    .as_ref()
                    .expect("nova panel packet must carry toggle state");
                let focus_name = focus_name
                    .as_ref()
                    .expect("nova panel packet must carry focus slot");

                let header_struct = next_name(state, "nova_panel_header");
                state.yir.nodes.push(Node {
                    name: header_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaHeaderPacket".to_owned(),
                            format!("accent={accent_name}"),
                            format!("title_mode={focus_name}"),
                        ],
                    },
                });
                push_dep_edges(state, accent_name, &header_struct);
                push_dep_edges(state, focus_name, &header_struct);
                panel_group_nodes.push(header_struct.clone());

                let color_slider = next_name(state, "nova_slider_color");
                state.yir.nodes.push(Node {
                    name: color_slider.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSliderPacket".to_owned(),
                            format!("value={color_name}"),
                            "min=0".to_owned(),
                            "max=127".to_owned(),
                            "step=4".to_owned(),
                            "disabled=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, &color_name, &color_slider);

                let speed_slider = next_name(state, "nova_slider_speed");
                state.yir.nodes.push(Node {
                    name: speed_slider.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSliderPacket".to_owned(),
                            format!("value={speed_name}"),
                            "min=0".to_owned(),
                            "max=63".to_owned(),
                            "step=2".to_owned(),
                            "disabled=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &speed_slider);

                let radius_slider = next_name(state, "nova_slider_radius");
                state.yir.nodes.push(Node {
                    name: radius_slider.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSliderPacket".to_owned(),
                            format!("value={radius_name}"),
                            "min=0".to_owned(),
                            "max=127".to_owned(),
                            "step=3".to_owned(),
                            "disabled=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, &radius_name, &radius_slider);

                let slider_struct = next_name(state, "nova_panel_sliders");
                state.yir.nodes.push(Node {
                    name: slider_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSliderGroupPacket".to_owned(),
                            format!("color={color_slider}"),
                            format!("speed={speed_slider}"),
                            format!("radius={radius_slider}"),
                        ],
                    },
                });
                push_dep_edges(state, &color_slider, &slider_struct);
                push_dep_edges(state, &speed_slider, &slider_struct);
                push_dep_edges(state, &radius_slider, &slider_struct);
                panel_group_nodes.push(slider_struct.clone());

                let toggle_struct = next_name(state, "nova_panel_toggle");
                state.yir.nodes.push(Node {
                    name: toggle_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTogglePacket".to_owned(),
                            format!("live={toggle_name}"),
                            "disabled=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &toggle_struct);
                panel_group_nodes.push(toggle_struct.clone());

                let progress_struct = next_name(state, "nova_panel_progress");
                state.yir.nodes.push(Node {
                    name: progress_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaProgressPacket".to_owned(),
                            format!("value={speed_name}"),
                            "max=63".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &progress_struct);
                panel_group_nodes.push(progress_struct.clone());

                let meter_struct = next_name(state, "nova_panel_meter");
                state.yir.nodes.push(Node {
                    name: meter_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaMeterPacket".to_owned(),
                            format!("value={radius_name}"),
                            "max=127".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, &radius_name, &meter_struct);
                panel_group_nodes.push(meter_struct.clone());

                let button_struct = next_name(state, "nova_panel_button");
                state.yir.nodes.push(Node {
                    name: button_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaButtonPacket".to_owned(),
                            format!("active={toggle_name}"),
                            format!("accent={accent_name}"),
                            format!("intent={focus_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &button_struct);
                push_dep_edges(state, accent_name, &button_struct);
                push_dep_edges(state, focus_name, &button_struct);
                panel_group_nodes.push(button_struct.clone());

                let text_input_struct = next_name(state, "nova_panel_text_input");
                state.yir.nodes.push(Node {
                    name: text_input_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTextInputPacket".to_owned(),
                            format!("echo={color_name}"),
                            format!("caret={focus_name}"),
                            format!("placeholder={accent_name}"),
                            "read_only=0".to_owned(),
                            "dirty=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, &color_name, &text_input_struct);
                push_dep_edges(state, focus_name, &text_input_struct);
                push_dep_edges(state, accent_name, &text_input_struct);
                panel_group_nodes.push(text_input_struct.clone());

                let select_struct = next_name(state, "nova_panel_select");
                state.yir.nodes.push(Node {
                    name: select_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSelectPacket".to_owned(),
                            format!("selected={focus_name}"),
                            format!("accent={accent_name}"),
                            "options=3".to_owned(),
                            "multiple=0".to_owned(),
                            "committed=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &select_struct);
                push_dep_edges(state, accent_name, &select_struct);
                panel_group_nodes.push(select_struct.clone());

                let checkbox_struct = next_name(state, "nova_panel_checkbox");
                state.yir.nodes.push(Node {
                    name: checkbox_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaCheckboxPacket".to_owned(),
                            format!("checked={toggle_name}"),
                            format!("accent={accent_name}"),
                            "disabled=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &checkbox_struct);
                push_dep_edges(state, accent_name, &checkbox_struct);
                panel_group_nodes.push(checkbox_struct.clone());

                let radio_struct = next_name(state, "nova_panel_radio");
                state.yir.nodes.push(Node {
                    name: radio_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaRadioPacket".to_owned(),
                            format!("selected={focus_name}"),
                            "options=4".to_owned(),
                            format!("accent={accent_name}"),
                            "disabled=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &radio_struct);
                push_dep_edges(state, accent_name, &radio_struct);
                panel_group_nodes.push(radio_struct.clone());

                let textarea_struct = next_name(state, "nova_panel_textarea");
                state.yir.nodes.push(Node {
                    name: textarea_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTextAreaPacket".to_owned(),
                            "lines=3".to_owned(),
                            format!("scroll={focus_name}"),
                            format!("placeholder={accent_name}"),
                            "read_only=0".to_owned(),
                            "dirty=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &textarea_struct);
                push_dep_edges(state, accent_name, &textarea_struct);
                panel_group_nodes.push(textarea_struct.clone());

                let tabs_struct = next_name(state, "nova_panel_tabs");
                state.yir.nodes.push(Node {
                    name: tabs_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTabsPacket".to_owned(),
                            format!("active={focus_name}"),
                            "count=4".to_owned(),
                            format!("accent={accent_name}"),
                            "compact=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &tabs_struct);
                push_dep_edges(state, accent_name, &tabs_struct);
                panel_group_nodes.push(tabs_struct.clone());

                let list_struct = next_name(state, "nova_panel_list");
                state.yir.nodes.push(Node {
                    name: list_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaListPacket".to_owned(),
                            format!("selected={focus_name}"),
                            "items=5".to_owned(),
                            format!("accent={accent_name}"),
                            "dense=0".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &list_struct);
                push_dep_edges(state, accent_name, &list_struct);
                panel_group_nodes.push(list_struct.clone());

                let table_struct = next_name(state, "nova_panel_table");
                state.yir.nodes.push(Node {
                    name: table_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTablePacket".to_owned(),
                            "rows=4".to_owned(),
                            "cols=3".to_owned(),
                            format!("selected_row={focus_name}"),
                            "zebra=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &table_struct);
                panel_group_nodes.push(table_struct.clone());

                let tree_struct = next_name(state, "nova_panel_tree");
                state.yir.nodes.push(Node {
                    name: tree_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTreePacket".to_owned(),
                            format!("selected={focus_name}"),
                            "nodes=6".to_owned(),
                            format!("expanded={toggle_name}"),
                            format!("accent={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &tree_struct);
                push_dep_edges(state, toggle_name, &tree_struct);
                push_dep_edges(state, accent_name, &tree_struct);
                panel_group_nodes.push(tree_struct.clone());

                let inspector_struct = next_name(state, "nova_panel_inspector");
                state.yir.nodes.push(Node {
                    name: inspector_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaInspectorPacket".to_owned(),
                            format!("selected={focus_name}"),
                            "fields=4".to_owned(),
                            format!("pinned={toggle_name}"),
                            format!("accent={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &inspector_struct);
                push_dep_edges(state, toggle_name, &inspector_struct);
                push_dep_edges(state, accent_name, &inspector_struct);
                panel_group_nodes.push(inspector_struct.clone());

                let outline_struct = next_name(state, "nova_panel_outline");
                state.yir.nodes.push(Node {
                    name: outline_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaOutlinePacket".to_owned(),
                            format!("selected={focus_name}"),
                            "items=6".to_owned(),
                            format!("collapsed={toggle_name}"),
                            format!("accent={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &outline_struct);
                push_dep_edges(state, toggle_name, &outline_struct);
                push_dep_edges(state, accent_name, &outline_struct);
                panel_group_nodes.push(outline_struct.clone());

                let theme_struct = next_name(state, "nova_panel_theme");
                state.yir.nodes.push(Node {
                    name: theme_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaThemePacket".to_owned(),
                            format!("accent={accent_name}"),
                            format!("surface={radius_name}"),
                            format!("panel_mode={toggle_name}"),
                            format!("contrast={speed_name}"),
                        ],
                    },
                });
                push_dep_edges(state, accent_name, &theme_struct);
                push_dep_edges(state, &radius_name, &theme_struct);
                push_dep_edges(state, toggle_name, &theme_struct);
                push_dep_edges(state, &speed_name, &theme_struct);
                panel_group_nodes.push(theme_struct.clone());

                let surface_struct = next_name(state, "nova_panel_surface");
                state.yir.nodes.push(Node {
                    name: surface_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSurfacePacket".to_owned(),
                            format!("density={speed_name}"),
                            format!("elevation={radius_name}"),
                            format!("grid={toggle_name}"),
                            format!("sheen={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &surface_struct);
                push_dep_edges(state, &radius_name, &surface_struct);
                push_dep_edges(state, toggle_name, &surface_struct);
                push_dep_edges(state, accent_name, &surface_struct);
                panel_group_nodes.push(surface_struct.clone());

                let viewport_struct = next_name(state, "nova_panel_viewport");
                state.yir.nodes.push(Node {
                    name: viewport_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaViewportPacket".to_owned(),
                            format!("origin_x={focus_name}"),
                            format!("origin_y={toggle_name}"),
                            "width=48".to_owned(),
                            "height=18".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &viewport_struct);
                push_dep_edges(state, toggle_name, &viewport_struct);
                panel_group_nodes.push(viewport_struct.clone());

                let layer_struct = next_name(state, "nova_panel_layer");
                state.yir.nodes.push(Node {
                    name: layer_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaLayerPacket".to_owned(),
                            "order=1".to_owned(),
                            format!("blend={toggle_name}"),
                            "visibility=1".to_owned(),
                            format!("clip={radius_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &layer_struct);
                push_dep_edges(state, &radius_name, &layer_struct);
                panel_group_nodes.push(layer_struct.clone());

                let scene_struct = next_name(state, "nova_panel_scene");
                state.yir.nodes.push(Node {
                    name: scene_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaScenePacket".to_owned(),
                            "root_count=7".to_owned(),
                            format!("active_camera={focus_name}"),
                            "light_count=3".to_owned(),
                            format!("animation_phase={toggle_name}"),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &scene_struct);
                push_dep_edges(state, toggle_name, &scene_struct);
                panel_group_nodes.push(scene_struct.clone());

                let camera_struct = next_name(state, "nova_panel_camera");
                state.yir.nodes.push(Node {
                    name: camera_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaCameraPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("focus={focus_name}"),
                            format!("zoom={speed_name}"),
                            format!("orbit={radius_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &camera_struct);
                push_dep_edges(state, focus_name, &camera_struct);
                push_dep_edges(state, &speed_name, &camera_struct);
                push_dep_edges(state, &radius_name, &camera_struct);
                panel_group_nodes.push(camera_struct.clone());

                let material_struct = next_name(state, "nova_panel_material");
                state.yir.nodes.push(Node {
                    name: material_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaMaterialPacket".to_owned(),
                            format!("shader_kind={toggle_name}"),
                            format!("albedo={accent_name}"),
                            format!("roughness={speed_name}"),
                            format!("emissive={radius_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &material_struct);
                push_dep_edges(state, accent_name, &material_struct);
                push_dep_edges(state, &speed_name, &material_struct);
                push_dep_edges(state, &radius_name, &material_struct);
                panel_group_nodes.push(material_struct.clone());

                let light_struct = next_name(state, "nova_panel_light");
                state.yir.nodes.push(Node {
                    name: light_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaLightPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("intensity={speed_name}"),
                            format!("range={radius_name}"),
                            format!("reactive={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &light_struct);
                push_dep_edges(state, &speed_name, &light_struct);
                push_dep_edges(state, &radius_name, &light_struct);
                push_dep_edges(state, accent_name, &light_struct);
                panel_group_nodes.push(light_struct.clone());

                let mesh_struct = next_name(state, "nova_panel_mesh");
                state.yir.nodes.push(Node {
                    name: mesh_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaMeshPacket".to_owned(),
                            format!("primitive={toggle_name}"),
                            format!("vertex_count={speed_name}"),
                            format!("index_count={radius_name}"),
                            format!("skinning={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &mesh_struct);
                push_dep_edges(state, &speed_name, &mesh_struct);
                push_dep_edges(state, &radius_name, &mesh_struct);
                push_dep_edges(state, accent_name, &mesh_struct);
                panel_group_nodes.push(mesh_struct.clone());

                let transform_struct = next_name(state, "nova_panel_transform");
                state.yir.nodes.push(Node {
                    name: transform_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTransformPacket".to_owned(),
                            format!("translate={speed_name}"),
                            format!("rotate={toggle_name}"),
                            format!("scale={radius_name}"),
                            format!("pivot={focus_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &transform_struct);
                push_dep_edges(state, toggle_name, &transform_struct);
                push_dep_edges(state, &radius_name, &transform_struct);
                push_dep_edges(state, focus_name, &transform_struct);
                panel_group_nodes.push(transform_struct.clone());

                let node_struct = next_name(state, "nova_panel_node");
                state.yir.nodes.push(Node {
                    name: node_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaNodePacket".to_owned(),
                            format!("node_id={focus_name}"),
                            format!("parent_id={toggle_name}"),
                            format!("flags={accent_name}"),
                            "depth=2".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &node_struct);
                push_dep_edges(state, toggle_name, &node_struct);
                push_dep_edges(state, accent_name, &node_struct);
                panel_group_nodes.push(node_struct.clone());

                let scene_link_struct = next_name(state, "nova_panel_scene_link");
                state.yir.nodes.push(Node {
                    name: scene_link_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSceneLinkPacket".to_owned(),
                            format!("node_slot={focus_name}"),
                            format!("transform_slot={speed_name}"),
                            format!("mesh_slot={radius_name}"),
                            format!("material_slot={accent_name}"),
                            format!("light_slot={toggle_name}"),
                            "layer_slot=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &scene_link_struct);
                push_dep_edges(state, &speed_name, &scene_link_struct);
                push_dep_edges(state, &radius_name, &scene_link_struct);
                push_dep_edges(state, accent_name, &scene_link_struct);
                push_dep_edges(state, toggle_name, &scene_link_struct);
                panel_group_nodes.push(scene_link_struct.clone());

                let instance_struct = next_name(state, "nova_panel_instance");
                state.yir.nodes.push(Node {
                    name: instance_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaInstancePacket".to_owned(),
                            format!("node_slot={focus_name}"),
                            "count=3".to_owned(),
                            format!("stride={radius_name}"),
                            format!("phase={speed_name}"),
                            format!("material_slot={accent_name}"),
                            format!("light_slot={toggle_name}"),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &instance_struct);
                push_dep_edges(state, &radius_name, &instance_struct);
                push_dep_edges(state, &speed_name, &instance_struct);
                push_dep_edges(state, accent_name, &instance_struct);
                push_dep_edges(state, toggle_name, &instance_struct);
                panel_group_nodes.push(instance_struct.clone());

                let scene_graph_struct = next_name(state, "nova_panel_scene_graph");
                state.yir.nodes.push(Node {
                    name: scene_graph_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSceneGraphPacket".to_owned(),
                            format!("root_slot={focus_name}"),
                            "node_count=6".to_owned(),
                            "link_count=3".to_owned(),
                            "instance_count=3".to_owned(),
                            "active_layer=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &scene_graph_struct);
                panel_group_nodes.push(scene_graph_struct.clone());

                let scene_node_struct = next_name(state, "nova_panel_scene_node");
                state.yir.nodes.push(Node {
                    name: scene_node_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSceneNodePacket".to_owned(),
                            format!("node_slot={focus_name}"),
                            format!("first_child_slot={speed_name}"),
                            format!("sibling_slot={radius_name}"),
                            "instance_slot=3".to_owned(),
                            format!("visibility={toggle_name}"),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &scene_node_struct);
                push_dep_edges(state, &speed_name, &scene_node_struct);
                push_dep_edges(state, &radius_name, &scene_node_struct);
                push_dep_edges(state, toggle_name, &scene_node_struct);
                panel_group_nodes.push(scene_node_struct.clone());

                let instance_group_struct = next_name(state, "nova_panel_instance_group");
                state.yir.nodes.push(Node {
                    name: instance_group_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaInstanceGroupPacket".to_owned(),
                            "root_instance_slot=3".to_owned(),
                            "group_count=4".to_owned(),
                            "visible_count=3".to_owned(),
                            format!("phase_bias={speed_name}"),
                            format!("material_slot={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &instance_group_struct);
                push_dep_edges(state, accent_name, &instance_group_struct);
                panel_group_nodes.push(instance_group_struct.clone());

                let scene_cluster_struct = next_name(state, "nova_panel_scene_cluster");
                state.yir.nodes.push(Node {
                    name: scene_cluster_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSceneClusterPacket".to_owned(),
                            format!("root_node_slot={focus_name}"),
                            "node_budget=6".to_owned(),
                            "instance_group_slot=3".to_owned(),
                            format!("material_slot={accent_name}"),
                            "layer_slot=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, focus_name, &scene_cluster_struct);
                push_dep_edges(state, accent_name, &scene_cluster_struct);
                panel_group_nodes.push(scene_cluster_struct.clone());

                let visibility_struct = next_name(state, "nova_panel_visibility");
                state.yir.nodes.push(Node {
                    name: visibility_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaVisibilityPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "visible_nodes=5".to_owned(),
                            format!("occlusion_mode={toggle_name}"),
                            format!("distance_band={speed_name}"),
                            "mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &visibility_struct);
                push_dep_edges(state, &speed_name, &visibility_struct);
                panel_group_nodes.push(visibility_struct.clone());

                let cull_struct = next_name(state, "nova_panel_cull");
                state.yir.nodes.push(Node {
                    name: cull_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaCullPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "kept_nodes=4".to_owned(),
                            format!("cull_mode={toggle_name}"),
                            format!("lod_band={speed_name}"),
                            "mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &cull_struct);
                push_dep_edges(state, &speed_name, &cull_struct);
                panel_group_nodes.push(cull_struct.clone());

                let lod_struct = next_name(state, "nova_panel_lod");
                state.yir.nodes.push(Node {
                    name: lod_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaLodPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "level_count=4".to_owned(),
                            format!("active_level={toggle_name}"),
                            format!("switch_distance={speed_name}"),
                            format!("bias={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &lod_struct);
                push_dep_edges(state, &speed_name, &lod_struct);
                push_dep_edges(state, accent_name, &lod_struct);
                panel_group_nodes.push(lod_struct.clone());

                let streaming_struct = next_name(state, "nova_panel_streaming");
                state.yir.nodes.push(Node {
                    name: streaming_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaStreamingPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "resident_levels=2".to_owned(),
                            format!("prefetch_mode={toggle_name}"),
                            format!("evict_budget={speed_name}"),
                            format!("channel={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &streaming_struct);
                push_dep_edges(state, &speed_name, &streaming_struct);
                push_dep_edges(state, accent_name, &streaming_struct);
                panel_group_nodes.push(streaming_struct.clone());

                let residency_struct = next_name(state, "nova_panel_residency");
                state.yir.nodes.push(Node {
                    name: residency_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaResidencyPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "committed_levels=2".to_owned(),
                            format!("residency_mode={toggle_name}"),
                            format!("spill_budget={speed_name}"),
                            "residency_mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &residency_struct);
                push_dep_edges(state, &speed_name, &residency_struct);
                panel_group_nodes.push(residency_struct.clone());

                let eviction_struct = next_name(state, "nova_panel_eviction");
                state.yir.nodes.push(Node {
                    name: eviction_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaEvictionPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "evicted_levels=1".to_owned(),
                            format!("eviction_mode={toggle_name}"),
                            format!("reclaim_budget={speed_name}"),
                            "eviction_mask=6".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &eviction_struct);
                push_dep_edges(state, &speed_name, &eviction_struct);
                panel_group_nodes.push(eviction_struct.clone());

                let prefetch_struct = next_name(state, "nova_panel_prefetch");
                state.yir.nodes.push(Node {
                    name: prefetch_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaPrefetchPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "requested_levels=2".to_owned(),
                            format!("prefetch_window={toggle_name}"),
                            format!("warm_budget={speed_name}"),
                            "prefetch_mask=5".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &prefetch_struct);
                push_dep_edges(state, &speed_name, &prefetch_struct);
                panel_group_nodes.push(prefetch_struct.clone());

                let budget_struct = next_name(state, "nova_panel_budget");
                state.yir.nodes.push(Node {
                    name: budget_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaBudgetPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "total_budget=12".to_owned(),
                            format!("used_budget={speed_name}"),
                            "headroom=5".to_owned(),
                            format!("budget_policy={toggle_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &budget_struct);
                push_dep_edges(state, toggle_name, &budget_struct);
                panel_group_nodes.push(budget_struct.clone());

                let pressure_struct = next_name(state, "nova_panel_pressure");
                state.yir.nodes.push(Node {
                    name: pressure_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaPressurePacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "pressure_level=2".to_owned(),
                            "saturation=7".to_owned(),
                            format!("throttled={toggle_name}"),
                            "pressure_mask=6".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &pressure_struct);
                panel_group_nodes.push(pressure_struct.clone());

                let thermal_struct = next_name(state, "nova_panel_thermal");
                state.yir.nodes.push(Node {
                    name: thermal_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaThermalPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "thermal_level=2".to_owned(),
                            format!("cooling_mode={toggle_name}"),
                            format!("throttled={toggle_name}"),
                            "thermal_mask=6".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &thermal_struct);
                panel_group_nodes.push(thermal_struct.clone());

                let power_struct = next_name(state, "nova_panel_power");
                state.yir.nodes.push(Node {
                    name: power_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaPowerPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "power_level=2".to_owned(),
                            format!("source_mode={toggle_name}"),
                            format!("capped={toggle_name}"),
                            "power_mask=6".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &power_struct);
                panel_group_nodes.push(power_struct.clone());

                let latency_struct = next_name(state, "nova_panel_latency");
                state.yir.nodes.push(Node {
                    name: latency_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaLatencyPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "frame_latency=4".to_owned(),
                            "input_latency=2".to_owned(),
                            format!("jitter={toggle_name}"),
                            "latency_mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &latency_struct);
                panel_group_nodes.push(latency_struct.clone());

                let frame_pacing_struct = next_name(state, "nova_panel_frame_pacing");
                state.yir.nodes.push(Node {
                    name: frame_pacing_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaFramePacingPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "cadence=4".to_owned(),
                            "variance=1".to_owned(),
                            format!("vsync_mode={toggle_name}"),
                            "pacing_mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &frame_pacing_struct);
                panel_group_nodes.push(frame_pacing_struct.clone());

                let frame_variance_struct = next_name(state, "nova_panel_frame_variance");
                state.yir.nodes.push(Node {
                    name: frame_variance_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaFrameVariancePacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "frame_variance=2".to_owned(),
                            format!("input_variance={toggle_name}"),
                            "burst_mode=4".to_owned(),
                            "variance_mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &frame_variance_struct);
                panel_group_nodes.push(frame_variance_struct.clone());

                let jank_struct = next_name(state, "nova_panel_jank");
                state.yir.nodes.push(Node {
                    name: jank_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaJankPacket".to_owned(),
                            "cluster_slot=3".to_owned(),
                            "spikes=2".to_owned(),
                            format!("severity={toggle_name}"),
                            "recovery=4".to_owned(),
                            "jank_mask=7".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &jank_struct);
                panel_group_nodes.push(jank_struct.clone());

                let pass_struct = next_name(state, "nova_panel_pass");
                state.yir.nodes.push(Node {
                    name: pass_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaPassPacket".to_owned(),
                            format!("stage={toggle_name}"),
                            format!("clear_mode={accent_name}"),
                            "sample_count=4".to_owned(),
                            format!("debug_view={focus_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &pass_struct);
                push_dep_edges(state, accent_name, &pass_struct);
                push_dep_edges(state, focus_name, &pass_struct);
                panel_group_nodes.push(pass_struct.clone());

                let frame_struct = next_name(state, "nova_panel_frame");
                state.yir.nodes.push(Node {
                    name: frame_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaFramePacket".to_owned(),
                            format!("frame_index={speed_name}"),
                            format!("present_mode={toggle_name}"),
                            "sync_interval=1".to_owned(),
                            format!("exposure={radius_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &speed_name, &frame_struct);
                push_dep_edges(state, toggle_name, &frame_struct);
                push_dep_edges(state, &radius_name, &frame_struct);
                panel_group_nodes.push(frame_struct.clone());

                let target_struct = next_name(state, "nova_panel_target");
                state.yir.nodes.push(Node {
                    name: target_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTargetPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            "width=48".to_owned(),
                            "height=18".to_owned(),
                            format!("multisample={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &target_struct);
                push_dep_edges(state, accent_name, &target_struct);
                panel_group_nodes.push(target_struct.clone());

                let frame_graph_struct = next_name(state, "nova_panel_frame_graph");
                state.yir.nodes.push(Node {
                    name: frame_graph_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaFrameGraphPacket".to_owned(),
                            "passes=2".to_owned(),
                            "targets=1".to_owned(),
                            format!("present_stage={toggle_name}"),
                            format!("debug_overlay={focus_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &frame_graph_struct);
                push_dep_edges(state, focus_name, &frame_graph_struct);
                panel_group_nodes.push(frame_graph_struct.clone());

                let attachment_struct = next_name(state, "nova_panel_attachment");
                state.yir.nodes.push(Node {
                    name: attachment_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaAttachmentPacket".to_owned(),
                            "slot=0".to_owned(),
                            format!("format_kind={accent_name}"),
                            format!("load_op={toggle_name}"),
                            "store_op=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, accent_name, &attachment_struct);
                push_dep_edges(state, toggle_name, &attachment_struct);
                panel_group_nodes.push(attachment_struct.clone());

                let pass_chain_struct = next_name(state, "nova_panel_pass_chain");
                state.yir.nodes.push(Node {
                    name: pass_chain_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaPassChainPacket".to_owned(),
                            "stages=2".to_owned(),
                            "fanout=1".to_owned(),
                            format!("resolve_stage={toggle_name}"),
                            format!("barrier_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &pass_chain_struct);
                push_dep_edges(state, accent_name, &pass_chain_struct);
                panel_group_nodes.push(pass_chain_struct.clone());

                let barrier_struct = next_name(state, "nova_panel_barrier");
                state.yir.nodes.push(Node {
                    name: barrier_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaBarrierPacket".to_owned(),
                            "scope=1".to_owned(),
                            format!("source_stage={toggle_name}"),
                            "target_stage=2".to_owned(),
                            format!("flush_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &barrier_struct);
                push_dep_edges(state, accent_name, &barrier_struct);
                panel_group_nodes.push(barrier_struct.clone());

                let resource_set_struct = next_name(state, "nova_panel_resource_set");
                state.yir.nodes.push(Node {
                    name: resource_set_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaResourceSetPacket".to_owned(),
                            "buffers=2".to_owned(),
                            "textures=1".to_owned(),
                            "samplers=1".to_owned(),
                            format!("residency={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, accent_name, &resource_set_struct);
                panel_group_nodes.push(resource_set_struct.clone());

                let schedule_struct = next_name(state, "nova_panel_schedule");
                state.yir.nodes.push(Node {
                    name: schedule_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSchedulePacket".to_owned(),
                            "lanes=2".to_owned(),
                            "queue_depth=4".to_owned(),
                            format!("async_budget={radius_name}"),
                            format!("tick_mode={toggle_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &radius_name, &schedule_struct);
                push_dep_edges(state, toggle_name, &schedule_struct);
                panel_group_nodes.push(schedule_struct.clone());

                let submission_struct = next_name(state, "nova_panel_submission");
                state.yir.nodes.push(Node {
                    name: submission_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSubmissionPacket".to_owned(),
                            "batches=2".to_owned(),
                            "fences=1".to_owned(),
                            format!("signal_mode={toggle_name}"),
                            format!("present_hint={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &submission_struct);
                push_dep_edges(state, accent_name, &submission_struct);
                panel_group_nodes.push(submission_struct.clone());

                let queue_struct = next_name(state, "nova_panel_queue");
                state.yir.nodes.push(Node {
                    name: queue_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaQueuePacket".to_owned(),
                            format!("kind={toggle_name}"),
                            "priority=2".to_owned(),
                            format!("budget={radius_name}"),
                            format!("ownership={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &queue_struct);
                push_dep_edges(state, &radius_name, &queue_struct);
                push_dep_edges(state, accent_name, &queue_struct);
                panel_group_nodes.push(queue_struct.clone());

                let semaphore_struct = next_name(state, "nova_panel_semaphore");
                state.yir.nodes.push(Node {
                    name: semaphore_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSemaphorePacket".to_owned(),
                            "wait_count=1".to_owned(),
                            "signal_count=2".to_owned(),
                            format!("timeline_mode={toggle_name}"),
                            format!("scope={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &semaphore_struct);
                push_dep_edges(state, accent_name, &semaphore_struct);
                panel_group_nodes.push(semaphore_struct.clone());

                let timeline_struct = next_name(state, "nova_panel_timeline");
                state.yir.nodes.push(Node {
                    name: timeline_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaTimelinePacket".to_owned(),
                            format!("value={radius_name}"),
                            "step=1".to_owned(),
                            "epoch=0".to_owned(),
                            format!("domain={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, &radius_name, &timeline_struct);
                push_dep_edges(state, accent_name, &timeline_struct);
                panel_group_nodes.push(timeline_struct.clone());

                let fence_struct = next_name(state, "nova_panel_fence");
                state.yir.nodes.push(Node {
                    name: fence_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaFencePacket".to_owned(),
                            format!("signaled={toggle_name}"),
                            "epoch=0".to_owned(),
                            format!("scope={accent_name}"),
                            "recycle_mode=1".to_owned(),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &fence_struct);
                push_dep_edges(state, accent_name, &fence_struct);
                panel_group_nodes.push(fence_struct.clone());

                let signal_struct = next_name(state, "nova_panel_signal");
                state.yir.nodes.push(Node {
                    name: signal_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSignalPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            "phase=2".to_owned(),
                            "fanout=3".to_owned(),
                            format!("ack_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &signal_struct);
                push_dep_edges(state, accent_name, &signal_struct);
                panel_group_nodes.push(signal_struct.clone());

                let event_struct = next_name(state, "nova_panel_event");
                state.yir.nodes.push(Node {
                    name: event_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaEventPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            "route=2".to_owned(),
                            "priority=3".to_owned(),
                            format!("payload_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &event_struct);
                push_dep_edges(state, accent_name, &event_struct);
                panel_group_nodes.push(event_struct.clone());

                let dispatch_struct = next_name(state, "nova_panel_dispatch");
                state.yir.nodes.push(Node {
                    name: dispatch_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaDispatchPacket".to_owned(),
                            format!("queue_kind={toggle_name}"),
                            "lane=2".to_owned(),
                            "batch=3".to_owned(),
                            format!("completion_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &dispatch_struct);
                push_dep_edges(state, accent_name, &dispatch_struct);
                panel_group_nodes.push(dispatch_struct.clone());

                let feedback_struct = next_name(state, "nova_panel_feedback");
                state.yir.nodes.push(Node {
                    name: feedback_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaFeedbackPacket".to_owned(),
                            format!("status={toggle_name}"),
                            format!("latency={speed_name}"),
                            format!("retries={radius_name}"),
                            format!("channel={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &feedback_struct);
                push_dep_edges(state, &speed_name, &feedback_struct);
                push_dep_edges(state, &radius_name, &feedback_struct);
                push_dep_edges(state, accent_name, &feedback_struct);
                panel_group_nodes.push(feedback_struct.clone());

                let intent_struct = next_name(state, "nova_panel_intent");
                state.yir.nodes.push(Node {
                    name: intent_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaIntentPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("target_slot={focus_name}"),
                            format!("urgency={speed_name}"),
                            format!("policy={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &intent_struct);
                push_dep_edges(state, focus_name, &intent_struct);
                push_dep_edges(state, &speed_name, &intent_struct);
                push_dep_edges(state, accent_name, &intent_struct);
                panel_group_nodes.push(intent_struct.clone());

                let reaction_struct = next_name(state, "nova_panel_reaction");
                state.yir.nodes.push(Node {
                    name: reaction_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaReactionPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("result_slot={focus_name}"),
                            format!("stability={radius_name}"),
                            format!("echo_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &reaction_struct);
                push_dep_edges(state, focus_name, &reaction_struct);
                push_dep_edges(state, &radius_name, &reaction_struct);
                push_dep_edges(state, accent_name, &reaction_struct);
                panel_group_nodes.push(reaction_struct.clone());

                let outcome_struct = next_name(state, "nova_panel_outcome");
                state.yir.nodes.push(Node {
                    name: outcome_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaOutcomePacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("final_slot={focus_name}"),
                            format!("confidence={speed_name}"),
                            format!("settle_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &outcome_struct);
                push_dep_edges(state, focus_name, &outcome_struct);
                push_dep_edges(state, &speed_name, &outcome_struct);
                push_dep_edges(state, accent_name, &outcome_struct);
                panel_group_nodes.push(outcome_struct.clone());

                let resolution_struct = next_name(state, "nova_panel_resolution");
                state.yir.nodes.push(Node {
                    name: resolution_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaResolutionPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("commit_slot={focus_name}"),
                            format!("convergence={radius_name}"),
                            format!("policy_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &resolution_struct);
                push_dep_edges(state, focus_name, &resolution_struct);
                push_dep_edges(state, &radius_name, &resolution_struct);
                push_dep_edges(state, accent_name, &resolution_struct);
                panel_group_nodes.push(resolution_struct.clone());

                let commit_struct = next_name(state, "nova_panel_commit");
                state.yir.nodes.push(Node {
                    name: commit_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaCommitPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("applied_slot={focus_name}"),
                            format!("durability={speed_name}"),
                            format!("commit_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &commit_struct);
                push_dep_edges(state, focus_name, &commit_struct);
                push_dep_edges(state, &speed_name, &commit_struct);
                push_dep_edges(state, accent_name, &commit_struct);
                panel_group_nodes.push(commit_struct.clone());

                let snapshot_struct = next_name(state, "nova_panel_snapshot");
                state.yir.nodes.push(Node {
                    name: snapshot_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaSnapshotPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("source_slot={focus_name}"),
                            format!("retention={radius_name}"),
                            format!("replay_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &snapshot_struct);
                push_dep_edges(state, focus_name, &snapshot_struct);
                push_dep_edges(state, &radius_name, &snapshot_struct);
                push_dep_edges(state, accent_name, &snapshot_struct);
                panel_group_nodes.push(snapshot_struct.clone());

                let checkpoint_struct = next_name(state, "nova_panel_checkpoint");
                state.yir.nodes.push(Node {
                    name: checkpoint_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec![
                            "NovaCheckpointPacket".to_owned(),
                            format!("kind={toggle_name}"),
                            format!("anchor_slot={focus_name}"),
                            format!("rollback_depth={speed_name}"),
                            format!("resume_mode={accent_name}"),
                        ],
                    },
                });
                push_dep_edges(state, toggle_name, &checkpoint_struct);
                push_dep_edges(state, focus_name, &checkpoint_struct);
                push_dep_edges(state, &speed_name, &checkpoint_struct);
                push_dep_edges(state, accent_name, &checkpoint_struct);
                panel_group_nodes.push(checkpoint_struct.clone());

                let focus_struct = next_name(state, "nova_panel_focus");
                state.yir.nodes.push(Node {
                    name: focus_struct.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "struct".to_owned(),
                        args: vec!["NovaFocusPacket".to_owned(), format!("slot={focus_name}")],
                    },
                });
                push_dep_edges(state, focus_name, &focus_struct);
                panel_group_nodes.push(focus_struct.clone());

                vec![
                    packet_type,
                    format!("header={header_struct}"),
                    format!("sliders={slider_struct}"),
                    format!("toggle={toggle_struct}"),
                    format!("progress={progress_struct}"),
                    format!("meter={meter_struct}"),
                    format!("button={button_struct}"),
                    format!("text_input={text_input_struct}"),
                    format!("select={select_struct}"),
                    format!("checkbox={checkbox_struct}"),
                    format!("radio={radio_struct}"),
                    format!("textarea={textarea_struct}"),
                    format!("tabs={tabs_struct}"),
                    format!("list={list_struct}"),
                    format!("table={table_struct}"),
                    format!("tree={tree_struct}"),
                    format!("inspector={inspector_struct}"),
                    format!("outline={outline_struct}"),
                    format!("theme={theme_struct}"),
                    format!("surface={surface_struct}"),
                    format!("viewport={viewport_struct}"),
                    format!("layer={layer_struct}"),
                    format!("scene={scene_struct}"),
                    format!("camera={camera_struct}"),
                    format!("material={material_struct}"),
                    format!("light={light_struct}"),
                    format!("mesh={mesh_struct}"),
                    format!("transform={transform_struct}"),
                    format!("node={node_struct}"),
                    format!("scene_link={scene_link_struct}"),
                    format!("instance={instance_struct}"),
                    format!("scene_graph={scene_graph_struct}"),
                    format!("scene_node={scene_node_struct}"),
                    format!("instance_group={instance_group_struct}"),
                    format!("scene_cluster={scene_cluster_struct}"),
                    format!("scene_visibility={visibility_struct}"),
                    format!("scene_cull={cull_struct}"),
                    format!("scene_lod={lod_struct}"),
                    format!("scene_streaming={streaming_struct}"),
                    format!("scene_residency={residency_struct}"),
                    format!("scene_eviction={eviction_struct}"),
                    format!("scene_prefetch={prefetch_struct}"),
                    format!("scene_budget={budget_struct}"),
                    format!("scene_pressure={pressure_struct}"),
                    format!("scene_thermal={thermal_struct}"),
                    format!("scene_power={power_struct}"),
                    format!("scene_latency={latency_struct}"),
                    format!("scene_frame_pacing={frame_pacing_struct}"),
                    format!("scene_frame_variance={frame_variance_struct}"),
                    format!("scene_jank={jank_struct}"),
                    format!("pass={pass_struct}"),
                    format!("frame={frame_struct}"),
                    format!("target={target_struct}"),
                    format!("frame_graph={frame_graph_struct}"),
                    format!("attachment={attachment_struct}"),
                    format!("pass_chain={pass_chain_struct}"),
                    format!("barrier={barrier_struct}"),
                    format!("resource_set={resource_set_struct}"),
                    format!("schedule={schedule_struct}"),
                    format!("submission={submission_struct}"),
                    format!("queue={queue_struct}"),
                    format!("semaphore={semaphore_struct}"),
                    format!("timeline={timeline_struct}"),
                    format!("fence={fence_struct}"),
                    format!("signal={signal_struct}"),
                    format!("event={event_struct}"),
                    format!("dispatch={dispatch_struct}"),
                    format!("feedback={feedback_struct}"),
                    format!("intent={intent_struct}"),
                    format!("reaction={reaction_struct}"),
                    format!("outcome={outcome_struct}"),
                    format!("resolution={resolution_struct}"),
                    format!("commit={commit_struct}"),
                    format!("snapshot={snapshot_struct}"),
                    format!("checkpoint={checkpoint_struct}"),
                    format!("focus={focus_struct}"),
                ]
            } else {
                vec![
                    packet_type,
                    format!("color={color_name}"),
                    format!("speed={speed_name}"),
                    format!("radius_scale={radius_name}"),
                ]
            };
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "struct".to_owned(),
                    args,
                },
            });
            push_dep_edges(state, &color_name, &name);
            push_dep_edges(state, &speed_name, &name);
            push_dep_edges(state, &radius_name, &name);
            for group_node in panel_group_nodes {
                push_dep_edges(state, &group_node, &name);
            }
            if !is_nova_panel {
                if let Some(accent_name) = &accent_name {
                    push_dep_edges(state, accent_name, &name);
                }
                if let Some(toggle_name) = &toggle_name {
                    push_dep_edges(state, toggle_name, &name);
                }
                if let Some(focus_name) = &focus_name {
                    push_dep_edges(state, focus_name, &name);
                }
            }
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
            "data_immutable_window",
            "immutable_window",
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
        NirExpr::DataResult { value, state: flow } => lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Data,
            value,
            "data_result",
            flow.render(),
        ),
        NirExpr::DataReady(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_ready",
            "is_ready",
        ),
        NirExpr::DataMoved(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_moved",
            "is_moved",
        ),
        NirExpr::DataWindowed(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_windowed",
            "is_windowed",
        ),
        NirExpr::DataValue(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_value",
            "value",
        ),
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
        NirExpr::DataReadWindow { window, index } => {
            ensure_fabric_resource(state.yir);
            let window_name = lower_expr(window, state, bindings)?;
            let index_name = lower_expr(index, state, bindings)?;
            let name = next_name(state, "data_read_window");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "read_window".to_owned(),
                    args: vec![window_name.clone(), index_name.clone()],
                },
            });
            push_dep_edges(state, &window_name, &name);
            push_dep_edges(state, &index_name, &name);
            Ok(name)
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            ensure_fabric_resource(state.yir);
            let window_name = lower_expr(window, state, bindings)?;
            let index_name = lower_expr(index, state, bindings)?;
            let value_name = lower_expr(value, state, bindings)?;
            let name = next_name(state, "data_write_window");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "write_window".to_owned(),
                    args: vec![window_name.clone(), index_name.clone(), value_name.clone()],
                },
            });
            push_dep_edges(state, &window_name, &name);
            push_dep_edges(state, &index_name, &name);
            push_dep_edges(state, &value_name, &name);
            Ok(name)
        }
        NirExpr::DataFreezeWindow(input) => {
            ensure_fabric_resource(state.yir);
            let input_name = lower_expr(input, state, bindings)?;
            let name = next_name(state, "data_freeze_window");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "fabric0".to_owned(),
                op: Operation {
                    module: "data".to_owned(),
                    instruction: "freeze_window".to_owned(),
                    args: vec![input_name.clone()],
                },
            });
            push_dep_edges(state, &input_name, &name);
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
        NirExpr::CpuJoinResult(task) => lower_task_result_entry_node(state, bindings, task),
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
        NirExpr::KernelResult { value, state: flow } => lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            value,
            "kernel_result",
            flow.render(),
        ),
        NirExpr::KernelConfigReady(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            result,
            "kernel_config_ready",
            "is_config_ready",
        ),
        NirExpr::KernelValue(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            result,
            "kernel_value",
            "value",
        ),
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
        NirExpr::ShaderResult { value, state: flow } => lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            value,
            "shader_result",
            flow.render(),
        ),
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
        NirExpr::ShaderValue(result) => lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_value",
            "value",
        ),
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
                NirBinaryOp::Eq => "eq",
                NirBinaryOp::Lt => "lt",
                NirBinaryOp::Gt => "gt",
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
        LoweredIfOutcome::Continued => Ok(None),
        LoweredIfOutcome::Bind { name, value } => {
            bindings.insert(name, value);
            Ok(None)
        }
        LoweredIfOutcome::Printed => Ok(None),
        LoweredIfOutcome::Returned(value) => Ok(Some(value)),
    }
}

enum LoweredIfOutcome {
    Continued,
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
        if then_body.len() == 1 && else_body.is_empty() {
            if let NirStmt::Return(Some(value)) = &then_body[0] {
                let lowered = lower_expr(value, state, bindings)?;
                lower_guard_return(condition_name, lowered, state);
                return Ok(LoweredIfOutcome::Continued);
            }
        }
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

fn lower_guard_return(
    condition_name: String,
    return_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "guard_return");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "guard_return".to_owned(),
            args: vec![condition_name.clone(), return_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &return_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: condition_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: return_name,
        to: name,
    });
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
            return Err(
                "task result entry must lower through lower_task_result_entry_node".to_owned(),
            )
        }
        (YirResultRole::StateProbe, Some(other)) => {
            return Err(format!(
                "unsupported non-task result probe state `{other:?}` for task observer"
            ))
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
    push_unique_edge(state, EdgeKind::Dep, from, to);
}

fn push_xfer_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    push_unique_edge(state, EdgeKind::CrossDomainExchange, from, to);
}

fn push_lifetime_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    push_unique_edge(state, EdgeKind::Lifetime, from, to);
}

fn push_unique_edge(state: &mut LoweringState<'_>, kind: EdgeKind, from: &str, to: &str) {
    let exists = state
        .yir
        .edges
        .iter()
        .any(|edge| edge.kind == kind && edge.from == from && edge.to == to);
    if exists {
        return;
    }
    state.yir.edges.push(Edge {
        kind,
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

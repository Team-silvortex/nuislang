use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, YirModule};

use super::shader_ir::{build_shader_ir_stage_contracts, decode_inline_shader_source};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShaderInlineWgslModule {
    resource: String,
    entry: String,
    source: String,
}

pub fn analyze_shader_lowering(module: &YirModule) -> ShaderLoweringContract {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let incoming = module
        .edges
        .iter()
        .filter(|edge| matches!(edge.kind, EdgeKind::Dep | EdgeKind::Effect))
        .fold(BTreeMap::<&str, Vec<&str>>::new(), |mut acc, edge| {
            acc.entry(edge.to.as_str())
                .or_default()
                .push(edge.from.as_str());
            acc
        });

    let mut stages = Vec::new();
    let fabric_handle_tables: Vec<FabricHandleTableContract> = module
        .nodes
        .iter()
        .filter(|node| {
            matches!(node.op.module.as_str(), "data" | "fabric")
                && node.op.instruction == "handle_table"
        })
        .map(|node| FabricHandleTableContract {
            node: node.name.clone(),
            entries: node
                .op
                .args
                .iter()
                .filter_map(|entry| entry.split_once('='))
                .map(|(slot, resource)| FabricHandleEntry {
                    slot: slot.trim().to_owned(),
                    resource: resource.trim().to_owned(),
                })
                .collect(),
        })
        .collect();
    let fabric_core_bindings: Vec<FabricCoreBindingContract> = module
        .nodes
        .iter()
        .filter(|node| {
            matches!(node.op.module.as_str(), "data" | "fabric")
                && node.op.instruction == "bind_core"
                && node.op.args.len() == 1
        })
        .filter_map(|node| {
            node.op.args[0]
                .parse::<usize>()
                .ok()
                .map(|core_index| FabricCoreBindingContract {
                    node: node.name.clone(),
                    resource: node.resource.clone(),
                    core_index,
                })
        })
        .collect();
    let inline_wgsl_modules: Vec<ShaderInlineWgslModule> = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "shader"
                && node.op.instruction == "inline_wgsl"
                && node.op.args.len() == 2
        })
        .map(|node| ShaderInlineWgslModule {
            resource: node.resource.clone(),
            entry: node.op.args[0].clone(),
            source: decode_inline_shader_source(&node.op.args[1]),
        })
        .collect();

    for node in &module.nodes {
        if node.op.module != "shader" {
            continue;
        }

        match node.op.instruction.as_str() {
            "draw_instanced" => stages.push(analyze_draw_instanced(
                node,
                &nodes,
                &incoming,
                &fabric_handle_tables,
                &inline_wgsl_modules,
            )),
            "draw_ball" | "draw_sphere" => stages.push(ShaderStageContract {
                node: node.name.clone(),
                op: node.op.full_name(),
                resource: node.resource.clone(),
                lowering: ShaderLoweringMode::PrerenderOnly,
                reason: "legacy reference raster op is only available through prerender fallback"
                    .to_owned(),
                pipeline: None,
                target_format: None,
                topology: None,
                wgsl_entry: None,
                wgsl_source: None,
                fabric_handle_table: None,
                bindings: Vec::new(),
                blend_mode: None,
                blend_enabled: None,
                depth_compare: None,
                depth_test_enabled: None,
                depth_write_enabled: None,
                cull_mode: None,
                front_face: None,
                shader_ir_stages: Vec::new(),
            }),
            "dispatch" => stages.push(ShaderStageContract {
                node: node.name.clone(),
                op: node.op.full_name(),
                resource: node.resource.clone(),
                lowering: ShaderLoweringMode::PrerenderOnly,
                reason: "generic shader.dispatch lacks a backend ABI contract".to_owned(),
                pipeline: None,
                target_format: None,
                topology: None,
                wgsl_entry: None,
                wgsl_source: None,
                fabric_handle_table: None,
                bindings: Vec::new(),
                blend_mode: None,
                blend_enabled: None,
                depth_compare: None,
                depth_test_enabled: None,
                depth_write_enabled: None,
                cull_mode: None,
                front_face: None,
                shader_ir_stages: Vec::new(),
            }),
            _ => {}
        }
    }

    ShaderLoweringContract {
        stages,
        fabric_handle_tables,
        fabric_core_bindings,
    }
}

fn analyze_draw_instanced(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
    incoming: &BTreeMap<&str, Vec<&str>>,
    fabric_handle_tables: &[FabricHandleTableContract],
    inline_wgsl_modules: &[ShaderInlineWgslModule],
) -> ShaderStageContract {
    let Some(pass_name) = node.op.args.first() else {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            resource: node.resource.clone(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: "draw_instanced is missing its render pass input".to_owned(),
            pipeline: None,
            target_format: None,
            topology: None,
            wgsl_entry: None,
            wgsl_source: None,
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
            shader_ir_stages: Vec::new(),
        };
    };

    let Some(pass_node) = nodes.get(pass_name.as_str()).copied() else {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            resource: node.resource.clone(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: format!("render pass `{pass_name}` is not present in the graph"),
            pipeline: None,
            target_format: None,
            topology: None,
            wgsl_entry: None,
            wgsl_source: None,
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
            shader_ir_stages: Vec::new(),
        };
    };

    if pass_node.op.instruction != "begin_pass" || pass_node.op.args.len() != 3 {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            resource: node.resource.clone(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: format!("render pass `{pass_name}` does not resolve to shader.begin_pass"),
            pipeline: None,
            target_format: None,
            topology: None,
            wgsl_entry: None,
            wgsl_source: None,
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
            shader_ir_stages: Vec::new(),
        };
    }

    let target_node = nodes.get(pass_node.op.args[0].as_str()).copied();
    let pipeline_node = nodes.get(pass_node.op.args[1].as_str()).copied();

    let target_format = target_node.and_then(parse_target_format);
    let (pipeline_name, topology) = pipeline_node
        .and_then(parse_pipeline_signature)
        .unwrap_or((None, None));
    let (wgsl_entry, wgsl_source) = find_matching_inline_wgsl(
        &node.resource,
        pipeline_name.as_deref(),
        inline_wgsl_modules,
    )
    .map(|module| (Some(module.entry.clone()), Some(module.source.clone())))
    .unwrap_or((None, None));

    let (lowering, reason) = classify_backend_eligibility(
        target_format.as_deref(),
        pipeline_name.as_deref(),
        topology.as_deref(),
    );
    let bindings = incoming
        .get(node.name.as_str())
        .into_iter()
        .flat_map(|names| names.iter().copied())
        .filter_map(|name| nodes.get(name).copied())
        .filter(|candidate| {
            candidate.op.module == "shader" && candidate.op.instruction == "bind_set"
        })
        .flat_map(|bind_set| extract_bindings(bind_set, nodes))
        .collect();
    let render_state = incoming
        .get(node.name.as_str())
        .into_iter()
        .flat_map(|names| names.iter().copied())
        .filter_map(|name| nodes.get(name).copied())
        .find(|candidate| {
            candidate.op.module == "shader" && candidate.op.instruction == "render_state"
        });
    let render_state = render_state
        .and_then(|state| extract_render_state(state, nodes))
        .unwrap_or_default();
    let fabric_handle_table = fabric_handle_tables
        .iter()
        .find(|table| {
            table
                .entries
                .iter()
                .any(|entry| entry.resource == node.resource)
        })
        .map(|table| table.node.clone());
    let shader_ir_stages = wgsl_source
        .as_deref()
        .map(build_shader_ir_stage_contracts)
        .unwrap_or_default();

    ShaderStageContract {
        node: node.name.clone(),
        op: node.op.full_name(),
        resource: node.resource.clone(),
        lowering,
        reason: reason.to_owned(),
        pipeline: pipeline_name,
        target_format,
        topology,
        wgsl_entry,
        wgsl_source,
        fabric_handle_table,
        bindings,
        blend_mode: render_state.blend_mode,
        blend_enabled: render_state.blend_enabled,
        depth_compare: render_state.depth_compare,
        depth_test_enabled: render_state.depth_test_enabled,
        depth_write_enabled: render_state.depth_write_enabled,
        cull_mode: render_state.cull_mode,
        front_face: render_state.front_face,
        shader_ir_stages,
    }
}

fn extract_bindings(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Vec<ShaderResourceBinding> {
    node.op.args[1..]
        .iter()
        .filter_map(|binding_name| {
            let binding = nodes.get(binding_name.as_str()).copied()?;
            if binding.op.module != "shader" {
                return None;
            }
            let kind = binding.op.instruction.as_str();
            if !matches!(
                kind,
                "uniform"
                    | "storage"
                    | "attachment"
                    | "texture_binding"
                    | "sampler_binding"
                    | "vertex_layout_binding"
                    | "vertex_binding"
                    | "index_binding"
            ) || binding.op.args.len() != 2
            {
                return None;
            }
            let slot = binding.op.args[0].parse::<usize>().ok()?;
            let source = binding.op.args[1].clone();
            let binding_metadata = extract_binding_metadata(kind, source.as_str(), nodes);
            Some(ShaderResourceBinding {
                slot,
                kind: kind.to_owned(),
                source,
                texture_format: binding_metadata.texture_format,
                texture_width: binding_metadata.texture_width,
                texture_height: binding_metadata.texture_height,
                sampler_filter: binding_metadata.sampler_filter,
                sampler_address_mode: binding_metadata.sampler_address_mode,
            })
        })
        .collect()
}

fn find_matching_inline_wgsl<'a>(
    resource: &str,
    pipeline_name: Option<&str>,
    inline_wgsl_modules: &'a [ShaderInlineWgslModule],
) -> Option<&'a ShaderInlineWgslModule> {
    inline_wgsl_modules
        .iter()
        .find(|module| {
            module.resource == resource
                && pipeline_name
                    .map(|pipeline| pipeline == module.entry)
                    .unwrap_or(true)
        })
        .or_else(|| {
            inline_wgsl_modules
                .iter()
                .find(|module| module.resource == resource)
        })
}

#[derive(Default)]
struct BindingMetadata {
    texture_format: Option<String>,
    texture_width: Option<usize>,
    texture_height: Option<usize>,
    sampler_filter: Option<String>,
    sampler_address_mode: Option<String>,
}

#[derive(Default)]
struct RenderStateMetadata {
    blend_mode: Option<String>,
    blend_enabled: Option<bool>,
    depth_compare: Option<String>,
    depth_test_enabled: Option<bool>,
    depth_write_enabled: Option<bool>,
    cull_mode: Option<String>,
    front_face: Option<String>,
}

fn extract_binding_metadata(
    kind: &str,
    source: &str,
    nodes: &BTreeMap<&str, &Node>,
) -> BindingMetadata {
    let Some(source_node) = nodes.get(source).copied() else {
        return BindingMetadata::default();
    };

    match kind {
        "texture_binding"
            if source_node.op.module == "shader"
                && source_node.op.instruction == "texture2d"
                && source_node.op.args.len() == 4 =>
        {
            let width = source_node.op.args[1].parse::<usize>().ok();
            let height = source_node.op.args[2].parse::<usize>().ok();
            BindingMetadata {
                texture_format: Some(source_node.op.args[0].clone()),
                texture_width: width,
                texture_height: height,
                ..BindingMetadata::default()
            }
        }
        "sampler_binding"
            if source_node.op.module == "shader"
                && source_node.op.instruction == "sampler"
                && source_node.op.args.len() == 2 =>
        {
            BindingMetadata {
                sampler_filter: Some(source_node.op.args[0].clone()),
                sampler_address_mode: Some(source_node.op.args[1].clone()),
                ..BindingMetadata::default()
            }
        }
        _ => BindingMetadata::default(),
    }
}

fn extract_render_state(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Option<RenderStateMetadata> {
    if node.op.args.len() != 4 {
        return None;
    }
    let blend = nodes.get(node.op.args[1].as_str()).copied()?;
    let depth = nodes.get(node.op.args[2].as_str()).copied()?;
    let raster = nodes.get(node.op.args[3].as_str()).copied()?;

    let (blend_enabled, blend_mode) = if blend.op.module == "shader"
        && blend.op.instruction == "blend_state"
        && blend.op.args.len() == 2
    {
        Some((
            parse_bool_literal(&blend.op.args[0])?,
            Some(blend.op.args[1].clone()),
        ))
    } else {
        None
    }?;
    let (depth_test_enabled, depth_write_enabled, depth_compare) = if depth.op.module == "shader"
        && depth.op.instruction == "depth_state"
        && depth.op.args.len() == 3
    {
        Some((
            parse_bool_literal(&depth.op.args[0])?,
            parse_bool_literal(&depth.op.args[1])?,
            Some(depth.op.args[2].clone()),
        ))
    } else {
        None
    }?;
    let (cull_mode, front_face) = if raster.op.module == "shader"
        && raster.op.instruction == "raster_state"
        && raster.op.args.len() == 2
    {
        Some((
            Some(raster.op.args[0].clone()),
            Some(raster.op.args[1].clone()),
        ))
    } else {
        None
    }?;

    Some(RenderStateMetadata {
        blend_mode,
        blend_enabled: Some(blend_enabled),
        depth_compare,
        depth_test_enabled: Some(depth_test_enabled),
        depth_write_enabled: Some(depth_write_enabled),
        cull_mode,
        front_face,
    })
}

fn parse_bool_literal(raw: &str) -> Option<bool> {
    match raw {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

fn parse_target_format(node: &Node) -> Option<String> {
    if node.op.module == "shader" && node.op.instruction == "target" && node.op.args.len() == 3 {
        Some(node.op.args[0].clone())
    } else {
        None
    }
}

fn parse_pipeline_signature(node: &Node) -> Option<(Option<String>, Option<String>)> {
    if node.op.module == "shader" && node.op.instruction == "pipeline" && node.op.args.len() == 2 {
        Some((Some(node.op.args[0].clone()), Some(node.op.args[1].clone())))
    } else {
        None
    }
}

fn classify_backend_eligibility(
    target_format: Option<&str>,
    shading_model: Option<&str>,
    topology: Option<&str>,
) -> (ShaderLoweringMode, &'static str) {
    let Some(target_format) = target_format else {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "missing shader.target format contract",
        );
    };
    let Some(shading_model) = shading_model else {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "missing shader.pipeline shading model contract",
        );
    };
    let Some(topology) = topology else {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "missing shader.pipeline topology contract",
        );
    };

    let supported_format = matches!(target_format, "rgba8_unorm" | "bgra8_unorm");
    if !supported_format {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "target format is outside the current backend portability subset",
        );
    }

    let supported_topology = matches!(topology, "triangle" | "triangle_strip");
    if !supported_topology {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "pipeline topology is outside the current backend portability subset",
        );
    }

    let supported_shading_model = matches!(
        shading_model,
        "flat_color" | "ball" | "sphere" | "lit_sphere"
    );
    if !supported_shading_model {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "shading model is outside the current backend portability subset",
        );
    }

    (
        ShaderLoweringMode::BackendEligible,
        "stage fits the current Metal/Vulkan common lowering subset",
    )
}

use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, YirModule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLoweringContract {
    pub stages: Vec<ShaderStageContract>,
    pub fabric_handle_tables: Vec<FabricHandleTableContract>,
    pub fabric_core_bindings: Vec<FabricCoreBindingContract>,
}

impl ShaderLoweringContract {
    pub fn has_shader_work(&self) -> bool {
        !self.stages.is_empty()
    }

    pub fn has_backend_eligible_work(&self) -> bool {
        self.stages
            .iter()
            .any(|stage| stage.lowering == ShaderLoweringMode::BackendEligible)
    }

    pub fn requires_prerender_fallback(&self) -> bool {
        self.stages
            .iter()
            .any(|stage| stage.lowering == ShaderLoweringMode::PrerenderOnly)
    }

    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();
        for table in &self.fabric_handle_tables {
            lines.push(format!("fabric_handle_table={}", table.node));
            for entry in &table.entries {
                lines.push(format!("  handle {} -> {}", entry.slot, entry.resource));
            }
        }
        for binding in &self.fabric_core_bindings {
            lines.push(format!(
                "fabric_core_binding={} resource={} core={}",
                binding.node, binding.resource, binding.core_index
            ));
        }
        for stage in &self.stages {
            lines.push(format!(
                "stage={} op={} resource={} lowering={} reason={}",
                stage.node,
                stage.op,
                stage.resource,
                stage.lowering.as_str(),
                stage.reason
            ));
            if let Some(table) = &stage.fabric_handle_table {
                lines.push(format!("  fabric_handle_table={}", table));
            }

            if let Some(pipeline) = &stage.pipeline {
                lines.push(format!("  pipeline={}", pipeline));
            }
            if let Some(target_format) = &stage.target_format {
                lines.push(format!("  target_format={}", target_format));
            }
            if let Some(topology) = &stage.topology {
                lines.push(format!("  topology={}", topology));
            }
            for binding in &stage.bindings {
                lines.push(format!(
                    "  binding slot={} kind={} source={}",
                    binding.slot, binding.kind, binding.source
                ));
                if let Some(filter) = &binding.sampler_filter {
                    lines.push(format!(
                        "    sampler filter={} address_mode={}",
                        filter,
                        binding.sampler_address_mode.as_deref().unwrap_or("clamp")
                    ));
                }
                if let Some(format) = &binding.texture_format {
                    lines.push(format!(
                        "    texture format={} size={}x{}",
                        format,
                        binding.texture_width.unwrap_or(0),
                        binding.texture_height.unwrap_or(0)
                    ));
                }
            }
            if let Some(blend_mode) = &stage.blend_mode {
                lines.push(format!(
                    "  blend enabled={} mode={}",
                    stage.blend_enabled.unwrap_or(false),
                    blend_mode
                ));
            }
            if let Some(compare) = &stage.depth_compare {
                lines.push(format!(
                    "  depth test={} write={} compare={}",
                    stage.depth_test_enabled.unwrap_or(false),
                    stage.depth_write_enabled.unwrap_or(false),
                    compare
                ));
            }
            if let Some(cull_mode) = &stage.cull_mode {
                lines.push(format!(
                    "  raster cull={} front={}",
                    cull_mode,
                    stage.front_face.as_deref().unwrap_or("ccw")
                ));
            }
        }
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" }
    }

    pub fn render_package_manifest(&self) -> String {
        let mut out = String::new();
        out.push_str("manifest_version = 1\n");
        out.push_str("package_kind = \"shader_package\"\n");
        out.push_str(&format!("stage_count = {}\n", self.stages.len()));
        out.push_str(&format!(
            "fabric_handle_table_count = {}\n",
            self.fabric_handle_tables.len()
        ));
        out.push_str(&format!(
            "fabric_core_binding_count = {}\n",
            self.fabric_core_bindings.len()
        ));
        out.push_str(&format!(
            "backend_eligible = {}\n",
            self.has_backend_eligible_work()
        ));
        out.push_str(&format!(
            "requires_prerender_fallback = {}\n",
            self.requires_prerender_fallback()
        ));

        for table in &self.fabric_handle_tables {
            out.push_str("\n[[fabric_handle_table]]\n");
            out.push_str(&format!("id = \"{}\"\n", table.node));
            for entry in &table.entries {
                out.push_str("\n[[fabric_handle_table.entry]]\n");
                out.push_str(&format!("slot = \"{}\"\n", escape_toml(&entry.slot)));
                out.push_str(&format!(
                    "resource = \"{}\"\n",
                    escape_toml(&entry.resource)
                ));
            }
        }

        for binding in &self.fabric_core_bindings {
            out.push_str("\n[[fabric_core_binding]]\n");
            out.push_str(&format!("id = \"{}\"\n", binding.node));
            out.push_str(&format!(
                "resource = \"{}\"\n",
                escape_toml(&binding.resource)
            ));
            out.push_str(&format!("core_index = {}\n", binding.core_index));
        }

        for stage in &self.stages {
            out.push_str("\n[[stage]]\n");
            out.push_str(&format!("id = \"{}\"\n", stage.node));
            out.push_str(&format!("op = \"{}\"\n", stage.op));
            out.push_str(&format!("resource = \"{}\"\n", escape_toml(&stage.resource)));
            out.push_str(&format!("lowering = \"{}\"\n", stage.lowering.as_str()));
            out.push_str(&format!("reason = \"{}\"\n", escape_toml(&stage.reason)));
            if let Some(table) = &stage.fabric_handle_table {
                out.push_str(&format!(
                    "fabric_handle_table = \"{}\"\n",
                    escape_toml(table)
                ));
            }
            if let Some(pipeline) = &stage.pipeline {
                out.push_str(&format!("pipeline = \"{}\"\n", escape_toml(pipeline)));
            }
            if let Some(target_format) = &stage.target_format {
                out.push_str(&format!(
                    "target_format = \"{}\"\n",
                    escape_toml(target_format)
                ));
            }
            if let Some(topology) = &stage.topology {
                out.push_str(&format!("topology = \"{}\"\n", escape_toml(topology)));
            }
            if let Some(blend_mode) = &stage.blend_mode {
                out.push_str(&format!(
                    "blend_enabled = {}\nblend_mode = \"{}\"\n",
                    stage.blend_enabled.unwrap_or(false),
                    escape_toml(blend_mode)
                ));
            }
            if let Some(compare) = &stage.depth_compare {
                out.push_str(&format!(
                    "depth_test_enabled = {}\ndepth_write_enabled = {}\ndepth_compare = \"{}\"\n",
                    stage.depth_test_enabled.unwrap_or(false),
                    stage.depth_write_enabled.unwrap_or(false),
                    escape_toml(compare)
                ));
            }
            if let Some(cull_mode) = &stage.cull_mode {
                out.push_str(&format!(
                    "cull_mode = \"{}\"\nfront_face = \"{}\"\n",
                    escape_toml(cull_mode),
                    escape_toml(stage.front_face.as_deref().unwrap_or("ccw"))
                ));
            }
            for binding in &stage.bindings {
                out.push_str("\n[[stage.binding]]\n");
                out.push_str(&format!("slot = {}\n", binding.slot));
                out.push_str(&format!("kind = \"{}\"\n", escape_toml(&binding.kind)));
                out.push_str(&format!("source = \"{}\"\n", escape_toml(&binding.source)));
                if let Some(format) = &binding.texture_format {
                    out.push_str(&format!(
                        "texture_format = \"{}\"\ntexture_width = {}\ntexture_height = {}\n",
                        escape_toml(format),
                        binding.texture_width.unwrap_or(0),
                        binding.texture_height.unwrap_or(0)
                    ));
                }
                if let Some(filter) = &binding.sampler_filter {
                    out.push_str(&format!(
                        "sampler_filter = \"{}\"\nsampler_address_mode = \"{}\"\n",
                        escape_toml(filter),
                        escape_toml(binding.sampler_address_mode.as_deref().unwrap_or("clamp"))
                    ));
                }
            }

            for variant in stage.backend_variants() {
                out.push_str("\n[[stage.variant]]\n");
                out.push_str(&format!("backend = \"{}\"\n", variant.backend));
                out.push_str(&format!("kind = \"{}\"\n", variant.kind));
                out.push_str(&format!("status = \"{}\"\n", variant.status));
                out.push_str(&format!(
                    "entry = \"{}\"\n",
                    escape_toml(&variant.entry)
                ));
                out.push_str(&format!(
                    "artifact = \"{}\"\n",
                    escape_toml(&variant.artifact)
                ));
                out.push_str(&format!(
                    "notes = \"{}\"\n",
                    escape_toml(&variant.notes)
                ));
            }

            if stage.lowering == ShaderLoweringMode::PrerenderOnly {
                out.push_str("\n[[stage.variant]]\n");
                out.push_str("backend = \"reference\"\n");
                out.push_str("kind = \"prerender\"\n");
                out.push_str("status = \"active\"\n");
                out.push_str(&format!("entry = \"{}\"\n", stage.node));
                out.push_str("artifact = \"assets/<stage>.ppm\"\n");
                out.push_str("notes = \"reference fallback artifact\"\n");
            }
        }

        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderStageContract {
    pub node: String,
    pub op: String,
    pub resource: String,
    pub lowering: ShaderLoweringMode,
    pub reason: String,
    pub pipeline: Option<String>,
    pub target_format: Option<String>,
    pub topology: Option<String>,
    pub fabric_handle_table: Option<String>,
    pub bindings: Vec<ShaderResourceBinding>,
    pub blend_mode: Option<String>,
    pub blend_enabled: Option<bool>,
    pub depth_compare: Option<String>,
    pub depth_test_enabled: Option<bool>,
    pub depth_write_enabled: Option<bool>,
    pub cull_mode: Option<String>,
    pub front_face: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricHandleTableContract {
    pub node: String,
    pub entries: Vec<FabricHandleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricHandleEntry {
    pub slot: String,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricCoreBindingContract {
    pub node: String,
    pub resource: String,
    pub core_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderResourceBinding {
    pub slot: usize,
    pub kind: String,
    pub source: String,
    pub texture_format: Option<String>,
    pub texture_width: Option<usize>,
    pub texture_height: Option<usize>,
    pub sampler_filter: Option<String>,
    pub sampler_address_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderBackendVariant {
    pub backend: &'static str,
    pub kind: &'static str,
    pub status: &'static str,
    pub entry: String,
    pub artifact: String,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLoweringMode {
    BackendEligible,
    PrerenderOnly,
}

impl ShaderLoweringMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackendEligible => "backend_eligible",
            Self::PrerenderOnly => "prerender_only",
        }
    }
}

impl ShaderStageContract {
    pub fn backend_variants(&self) -> Vec<ShaderBackendVariant> {
        if self.lowering != ShaderLoweringMode::BackendEligible {
            return Vec::new();
        }

        let stage_id = self.node.clone();
        vec![
            ShaderBackendVariant {
                backend: "metal",
                kind: "library",
                status: "planned",
                entry: stage_id.clone(),
                artifact: format!("metal/{stage_id}.metallib"),
                notes: "Apple GPU backend artifact".to_owned(),
            },
            ShaderBackendVariant {
                backend: "vulkan",
                kind: "spirv",
                status: "planned",
                entry: stage_id.clone(),
                artifact: format!("vulkan/{stage_id}.spv"),
                notes: "Portable Vulkan/SPIR-V artifact".to_owned(),
            },
            ShaderBackendVariant {
                backend: "directx",
                kind: "dxil",
                status: "planned",
                entry: stage_id.clone(),
                artifact: format!("directx/{stage_id}.dxil"),
                notes: "Windows DirectX backend artifact".to_owned(),
            },
            ShaderBackendVariant {
                backend: "opengl",
                kind: "glsl",
                status: "planned",
                entry: stage_id,
                artifact: format!("opengl/{}.glsl", self.node),
                notes: "Legacy OpenGL fallback artifact".to_owned(),
            },
        ]
    }
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
            acc.entry(edge.to.as_str()).or_default().push(edge.from.as_str());
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
                fabric_handle_table: None,
                bindings: Vec::new(),
                blend_mode: None,
                blend_enabled: None,
                depth_compare: None,
                depth_test_enabled: None,
                depth_write_enabled: None,
                cull_mode: None,
                front_face: None,
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
                fabric_handle_table: None,
                bindings: Vec::new(),
                blend_mode: None,
                blend_enabled: None,
                depth_compare: None,
                depth_test_enabled: None,
                depth_write_enabled: None,
                cull_mode: None,
                front_face: None,
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
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
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
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
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
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
        };
    }

    let target_node = nodes.get(pass_node.op.args[0].as_str()).copied();
    let pipeline_node = nodes.get(pass_node.op.args[1].as_str()).copied();

    let target_format = target_node.and_then(parse_target_format);
    let (pipeline_name, topology) = pipeline_node
        .and_then(parse_pipeline_signature)
        .unwrap_or((None, None));

    let (lowering, reason) =
        classify_backend_eligibility(target_format.as_deref(), pipeline_name.as_deref(), topology.as_deref());
    let bindings = incoming
        .get(node.name.as_str())
        .into_iter()
        .flat_map(|names| names.iter().copied())
        .filter_map(|name| nodes.get(name).copied())
        .filter(|candidate| candidate.op.module == "shader" && candidate.op.instruction == "bind_set")
        .flat_map(|bind_set| extract_bindings(bind_set, nodes))
        .collect();
    let render_state = incoming
        .get(node.name.as_str())
        .into_iter()
        .flat_map(|names| names.iter().copied())
        .filter_map(|name| nodes.get(name).copied())
        .find(|candidate| candidate.op.module == "shader" && candidate.op.instruction == "render_state");
    let (
        blend_mode,
        blend_enabled,
        depth_compare,
        depth_test_enabled,
        depth_write_enabled,
        cull_mode,
        front_face,
    ) = render_state
        .and_then(|state| extract_render_state(state, nodes))
        .unwrap_or((None, None, None, None, None, None, None));
    let fabric_handle_table = fabric_handle_tables
        .iter()
        .find(|table| table.entries.iter().any(|entry| entry.resource == node.resource))
        .map(|table| table.node.clone());

    ShaderStageContract {
        node: node.name.clone(),
        op: node.op.full_name(),
        resource: node.resource.clone(),
        lowering,
        reason: reason.to_owned(),
        pipeline: pipeline_name,
        target_format,
        topology,
        fabric_handle_table,
        bindings,
        blend_mode,
        blend_enabled,
        depth_compare,
        depth_test_enabled,
        depth_write_enabled,
        cull_mode,
        front_face,
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
            let (texture_format, texture_width, texture_height, sampler_filter, sampler_address_mode) =
                extract_binding_metadata(kind, source.as_str(), nodes);
            Some(ShaderResourceBinding {
                slot,
                kind: kind.to_owned(),
                source,
                texture_format,
                texture_width,
                texture_height,
                sampler_filter,
                sampler_address_mode,
            })
        })
        .collect()
}

fn extract_binding_metadata(
    kind: &str,
    source: &str,
    nodes: &BTreeMap<&str, &Node>,
) -> (
    Option<String>,
    Option<usize>,
    Option<usize>,
    Option<String>,
    Option<String>,
) {
    let Some(source_node) = nodes.get(source).copied() else {
        return (None, None, None, None, None);
    };

    match kind {
        "texture_binding" if source_node.op.module == "shader"
            && source_node.op.instruction == "texture2d"
            && source_node.op.args.len() == 4 =>
        {
            let width = source_node.op.args[1].parse::<usize>().ok();
            let height = source_node.op.args[2].parse::<usize>().ok();
            (
                Some(source_node.op.args[0].clone()),
                width,
                height,
                None,
                None,
            )
        }
        "sampler_binding" if source_node.op.module == "shader"
            && source_node.op.instruction == "sampler"
            && source_node.op.args.len() == 2 =>
        {
            (
                None,
                None,
                None,
                Some(source_node.op.args[0].clone()),
                Some(source_node.op.args[1].clone()),
            )
        }
        _ => (None, None, None, None, None),
    }
}

fn extract_render_state(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
) -> Option<(
    Option<String>,
    Option<bool>,
    Option<String>,
    Option<bool>,
    Option<bool>,
    Option<String>,
    Option<String>,
)> {
    if node.op.args.len() != 4 {
        return None;
    }
    let blend = nodes.get(node.op.args[1].as_str()).copied()?;
    let depth = nodes.get(node.op.args[2].as_str()).copied()?;
    let raster = nodes.get(node.op.args[3].as_str()).copied()?;

    let (blend_enabled, blend_mode) =
        if blend.op.module == "shader" && blend.op.instruction == "blend_state" && blend.op.args.len() == 2 {
            Some((parse_bool_literal(&blend.op.args[0])?, Some(blend.op.args[1].clone())))
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
    let (cull_mode, front_face) =
        if raster.op.module == "shader" && raster.op.instruction == "raster_state" && raster.op.args.len() == 2 {
            Some((Some(raster.op.args[0].clone()), Some(raster.op.args[1].clone())))
        } else {
            None
        }?;

    Some((
        blend_mode,
        Some(blend_enabled),
        depth_compare,
        Some(depth_test_enabled),
        Some(depth_write_enabled),
        cull_mode,
        front_face,
    ))
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

    let supported_shading_model =
        matches!(shading_model, "flat_color" | "ball" | "sphere" | "lit_sphere");
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

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

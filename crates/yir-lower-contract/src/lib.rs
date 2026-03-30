use std::collections::BTreeMap;

use yir_core::{Node, YirModule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLoweringContract {
    pub stages: Vec<ShaderStageContract>,
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
        for stage in &self.stages {
            lines.push(format!(
                "stage={} op={} lowering={} reason={}",
                stage.node,
                stage.op,
                stage.lowering.as_str(),
                stage.reason
            ));

            if let Some(pipeline) = &stage.pipeline {
                lines.push(format!("  pipeline={}", pipeline));
            }
            if let Some(target_format) = &stage.target_format {
                lines.push(format!("  target_format={}", target_format));
            }
            if let Some(topology) = &stage.topology {
                lines.push(format!("  topology={}", topology));
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
            "backend_eligible = {}\n",
            self.has_backend_eligible_work()
        ));
        out.push_str(&format!(
            "requires_prerender_fallback = {}\n",
            self.requires_prerender_fallback()
        ));

        for stage in &self.stages {
            out.push_str("\n[[stage]]\n");
            out.push_str(&format!("id = \"{}\"\n", stage.node));
            out.push_str(&format!("op = \"{}\"\n", stage.op));
            out.push_str(&format!("lowering = \"{}\"\n", stage.lowering.as_str()));
            out.push_str(&format!("reason = \"{}\"\n", escape_toml(&stage.reason)));
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
    pub lowering: ShaderLoweringMode,
    pub reason: String,
    pub pipeline: Option<String>,
    pub target_format: Option<String>,
    pub topology: Option<String>,
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

    let mut stages = Vec::new();

    for node in &module.nodes {
        if node.op.module != "shader" {
            continue;
        }

        match node.op.instruction.as_str() {
            "draw_instanced" => stages.push(analyze_draw_instanced(node, &nodes)),
            "draw_ball" | "draw_sphere" => stages.push(ShaderStageContract {
                node: node.name.clone(),
                op: node.op.full_name(),
                lowering: ShaderLoweringMode::PrerenderOnly,
                reason: "legacy reference raster op is only available through prerender fallback"
                    .to_owned(),
                pipeline: None,
                target_format: None,
                topology: None,
            }),
            "dispatch" => stages.push(ShaderStageContract {
                node: node.name.clone(),
                op: node.op.full_name(),
                lowering: ShaderLoweringMode::PrerenderOnly,
                reason: "generic shader.dispatch lacks a backend ABI contract".to_owned(),
                pipeline: None,
                target_format: None,
                topology: None,
            }),
            _ => {}
        }
    }

    ShaderLoweringContract { stages }
}

fn analyze_draw_instanced(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
) -> ShaderStageContract {
    let Some(pass_name) = node.op.args.first() else {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: "draw_instanced is missing its render pass input".to_owned(),
            pipeline: None,
            target_format: None,
            topology: None,
        };
    };

    let Some(pass_node) = nodes.get(pass_name.as_str()).copied() else {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: format!("render pass `{pass_name}` is not present in the graph"),
            pipeline: None,
            target_format: None,
            topology: None,
        };
    };

    if pass_node.op.instruction != "begin_pass" || pass_node.op.args.len() != 3 {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: format!("render pass `{pass_name}` does not resolve to shader.begin_pass"),
            pipeline: None,
            target_format: None,
            topology: None,
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

    ShaderStageContract {
        node: node.name.clone(),
        op: node.op.full_name(),
        lowering,
        reason: reason.to_owned(),
        pipeline: pipeline_name,
        target_format,
        topology,
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

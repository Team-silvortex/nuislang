use std::collections::{BTreeMap, BTreeSet};

use crate::{
    aot_encoding::fnv1a64_hex,
    aot_toml::{escape_toml_string, render_string_array},
    shader_msl_render_emitter::lower_canonical_inline_wgsl_render_for_profile,
};

pub const SHADER_RENDER_CODEGEN_TABLE_CONTRACT: &str = "nuis-shader-render-codegen-table-v1";
pub const SHADER_RENDER_CODE_ASSET_CONTRACT: &str = "nuis-shader-render-code-asset-v1";
pub const SHADER_RENDER_PASS_PROJECTION_CONTRACT: &str = "nuis-shader-render-pass-projection-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderRenderCodeAsset {
    pub contract: &'static str,
    pub asset_id: String,
    pub file_name: String,
    pub format: &'static str,
    pub target: String,
    pub entries: Vec<String>,
    pub source: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderRenderPassProjection {
    pub contract: &'static str,
    pub pass_node: String,
    pub module_node: String,
    pub asset_id: String,
    pub target_format: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderRenderCodegenTable {
    pub contract: &'static str,
    pub source_yir_version: String,
    pub source_fnv1a64: String,
    pub lowering_target: String,
    pub assets: Vec<ShaderRenderCodeAsset>,
    pub passes: Vec<ShaderRenderPassProjection>,
}

pub fn table_from_compiled_project_yir(
    source: &str,
    lowering_target: &str,
) -> Result<Option<ShaderRenderCodegenTable>, String> {
    if !matches!(
        lowering_target,
        "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu"
    ) {
        return Err(format!(
            "Shader render codegen table has no registered producer for `{lowering_target}`"
        ));
    }
    let module = yir_syntax::parse_module(source)
        .map_err(|error| format!("failed to parse compiled project YIR: {error}"))?;
    yir_verify::verify_module(&module)
        .map_err(|error| format!("compiled project YIR failed verification: {error}"))?;
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let render_passes = module
        .nodes
        .iter()
        .filter(|node| node.op.full_name() == "shader.begin_pass" && node.op.args.len() == 4)
        .collect::<Vec<_>>();
    if render_passes.is_empty() {
        return Ok(None);
    }

    let mut assets = BTreeMap::<String, ShaderRenderCodeAsset>::new();
    let mut passes = Vec::with_capacity(render_passes.len());
    for pass in render_passes {
        let inline =
            required_shader_node(&nodes, &pass.op.args[3], "inline_wgsl", pass.name.as_str())?;
        let target = required_shader_node(&nodes, &pass.op.args[0], "target", pass.name.as_str())?;
        let viewport =
            required_shader_node(&nodes, &pass.op.args[2], "viewport", pass.name.as_str())?;
        let target_format = target.op.args[0].clone();
        if target_format != "rgba8_unorm" {
            return Err(format!(
                "Shader render pass `{}` uses unsupported target format `{target_format}`",
                pass.name
            ));
        }
        let target_width = positive_dimension(&target.op.args[1], "target width", &pass.name)?;
        let target_height = positive_dimension(&target.op.args[2], "target height", &pass.name)?;
        let viewport_width =
            positive_dimension(&viewport.op.args[0], "viewport width", &pass.name)?;
        let viewport_height =
            positive_dimension(&viewport.op.args[1], "viewport height", &pass.name)?;
        let lowered = lower_canonical_inline_wgsl_render_for_profile(
            &inline.op.args[1],
            &inline.op.args[0],
            lowering_target,
        )?;
        let content_hash = fnv1a64_hex(lowered.source.as_bytes());
        let asset_id = format!("shader.metal.project.{}", &content_hash[2..]);
        let asset = ShaderRenderCodeAsset {
            contract: SHADER_RENDER_CODE_ASSET_CONTRACT,
            asset_id: asset_id.clone(),
            file_name: format!("nuis.shader.project.{}.metal", &content_hash[2..]),
            format: "metal-source",
            target: lowering_target.to_owned(),
            entries: vec![lowered.vertex_entry, lowered.fragment_entry],
            source: lowered.source,
            content_hash,
        };
        if let Some(existing) = assets.get(&asset_id) {
            if existing != &asset {
                return Err("Shader render code asset hash collision".to_owned());
            }
        } else {
            assets.insert(asset_id.clone(), asset);
        }
        passes.push(ShaderRenderPassProjection {
            contract: SHADER_RENDER_PASS_PROJECTION_CONTRACT,
            pass_node: pass.name.clone(),
            module_node: inline.name.clone(),
            asset_id,
            target_format,
            width: target_width.min(viewport_width),
            height: target_height.min(viewport_height),
        });
    }

    let table = ShaderRenderCodegenTable {
        contract: SHADER_RENDER_CODEGEN_TABLE_CONTRACT,
        source_yir_version: module.version,
        source_fnv1a64: fnv1a64_hex(source.as_bytes()),
        lowering_target: lowering_target.to_owned(),
        assets: assets.into_values().collect(),
        passes,
    };
    validate_codegen_table(&table)?;
    Ok(Some(table))
}

fn required_shader_node<'a>(
    nodes: &BTreeMap<&str, &'a yir_core::Node>,
    name: &str,
    instruction: &str,
    pass: &str,
) -> Result<&'a yir_core::Node, String> {
    nodes
        .get(name)
        .copied()
        .filter(|node| node.op.module == "shader" && node.op.instruction == instruction)
        .ok_or_else(|| {
            format!("Shader render pass `{pass}` has no bound shader.{instruction} node `{name}`")
        })
}

fn positive_dimension(value: &str, role: &str, pass: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("Shader render pass `{pass}` has invalid {role} `{value}`"))
}

pub fn validate_codegen_table(table: &ShaderRenderCodegenTable) -> Result<(), String> {
    if table.contract != SHADER_RENDER_CODEGEN_TABLE_CONTRACT
        || table.source_yir_version.is_empty()
        || !valid_fnv1a64(&table.source_fnv1a64)
        || !matches!(
            table.lowering_target.as_str(),
            "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu"
        )
        || table.assets.is_empty()
        || table.passes.is_empty()
    {
        return Err("Shader render codegen table header is invalid".to_owned());
    }
    let mut asset_ids = BTreeSet::new();
    let mut file_names = BTreeSet::new();
    for asset in &table.assets {
        if asset.contract != SHADER_RENDER_CODE_ASSET_CONTRACT
            || !asset_ids.insert(asset.asset_id.as_str())
            || !file_names.insert(asset.file_name.as_str())
            || !asset.asset_id.starts_with("shader.metal.project.")
            || !asset.file_name.starts_with("nuis.shader.project.")
            || asset.format != "metal-source"
            || asset.target != table.lowering_target
            || asset.entries.len() != 2
            || asset.entries.iter().any(|entry| !valid_symbol(entry))
            || asset.source.is_empty()
            || fnv1a64_hex(asset.source.as_bytes()) != asset.content_hash
        {
            return Err(format!(
                "Shader render code asset `{}` is invalid",
                asset.asset_id
            ));
        }
    }
    let mut pass_nodes = BTreeSet::new();
    for pass in &table.passes {
        if pass.contract != SHADER_RENDER_PASS_PROJECTION_CONTRACT
            || !pass_nodes.insert(pass.pass_node.as_str())
            || pass.module_node.is_empty()
            || !asset_ids.contains(pass.asset_id.as_str())
            || pass.target_format != "rgba8_unorm"
            || pass.width == 0
            || pass.height == 0
        {
            return Err(format!(
                "Shader render pass projection `{}` is invalid",
                pass.pass_node
            ));
        }
    }
    Ok(())
}

pub fn render_codegen_table(table: &ShaderRenderCodegenTable) -> Result<String, String> {
    validate_codegen_table(table)?;
    let mut out = format!(
        "schema = \"{SHADER_RENDER_CODEGEN_TABLE_CONTRACT}\"\nsource_yir_version = \"{}\"\nsource_fnv1a64 = \"{}\"\nlowering_target = \"{}\"\nasset_count = {}\npass_count = {}\n",
        escape_toml_string(&table.source_yir_version),
        table.source_fnv1a64,
        escape_toml_string(&table.lowering_target),
        table.assets.len(),
        table.passes.len(),
    );
    for asset in &table.assets {
        out.push_str("\n[[asset]]\n");
        out.push_str(&format!(
            "contract = \"{}\"\nasset_id = \"{}\"\nfile_name = \"{}\"\nformat = \"{}\"\ntarget = \"{}\"\nentries = {}\nbyte_length = {}\ncontent_hash = \"{}\"\n",
            asset.contract,
            escape_toml_string(&asset.asset_id),
            escape_toml_string(&asset.file_name),
            asset.format,
            escape_toml_string(&asset.target),
            render_string_array(&asset.entries),
            asset.source.len(),
            asset.content_hash,
        ));
    }
    for pass in &table.passes {
        out.push_str("\n[[pass]]\n");
        out.push_str(&format!(
            "contract = \"{}\"\npass_node = \"{}\"\nmodule_node = \"{}\"\nasset_id = \"{}\"\ntarget_format = \"{}\"\nwidth = {}\nheight = {}\n",
            pass.contract,
            escape_toml_string(&pass.pass_node),
            escape_toml_string(&pass.module_node),
            escape_toml_string(&pass.asset_id),
            escape_toml_string(&pass.target_format),
            pass.width,
            pass.height,
        ));
    }
    Ok(out)
}

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn valid_symbol(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[cfg(test)]
#[path = "shader_render_codegen_table_tests.rs"]
mod tests;

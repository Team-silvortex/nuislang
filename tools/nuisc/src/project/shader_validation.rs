use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{NirExpr, NirStmt};
use yir_core::{SemanticOp, YirModule};

use super::support_contracts::{
    require_declared_support_surface, shader_profile_slot_targets, shader_support_surface_contract,
    support_profile_slots_for_domain, support_surface_for_domain,
};
use super::{
    collect_profile_int_bindings, require_declared_profile_slot,
    resolve_project_profile_target_name, LoadedProject,
};

pub(super) fn validate_shader_profile_for_link(
    project: &LoadedProject,
    module: &YirModule,
    endpoint: &str,
) -> Result<(), String> {
    let (domain, unit) = super::split_domain_unit(endpoint)?;
    if domain != "shader" {
        return Ok(());
    }
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "shader")?;
    let declared_slots = support_profile_slots_for_domain("shader")?;
    for required_surface in shader_support_surface_contract() {
        require_declared_support_surface(&declared_support, "shader", &unit, required_surface)?;
    }
    let packet_contract = infer_shader_packet_contract(project, &unit)?;
    for (slot, node_name) in shader_profile_slot_targets(
        &unit,
        packet_contract.as_ref().map(shader_packet_slot_names),
    ) {
        require_declared_profile_slot(&declared_slots, "shader", &unit, slot)?;
        let exists = module.nodes.iter().any(|node| node.name == node_name);
        if !exists {
            return Err(format!(
                "project shader unit `shader.{}` requires support profile slot `{}` in YIR",
                unit, slot
            ));
        }
    }

    validate_shader_profile_flow(module, &unit)?;
    validate_shader_packet_contract(project, &unit)?;

    Ok(())
}

pub(super) fn infer_shader_packet_contract(
    project: &LoadedProject,
    unit: &str,
) -> Result<Option<ShaderPacketContract>, String> {
    let mut discovered = Vec::new();
    for project_module in &project.modules {
        let nir = super::lower_project_module_to_nir(project, project_module)?;
        collect_shader_packet_contracts_from_stmts(&nir.functions, unit, &mut discovered);
    }
    if discovered.is_empty() {
        return Ok(None);
    }
    let first = discovered[0].clone();
    if discovered.iter().any(|contract| contract != &first) {
        return Err(format!(
            "project shader unit `shader.{}` has inconsistent CPU-side packet contracts",
            unit
        ));
    }
    Ok(Some(first))
}

pub(super) fn shader_packet_slot_names(contract: &ShaderPacketContract) -> &'static [&'static str] {
    if contract.type_name == "NovaPanelPacket" {
        &[
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ]
    } else {
        &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShaderPacketContract {
    pub(super) type_name: String,
    pub(super) field_count: usize,
}

pub(super) fn validate_shader_packet_contract(
    project: &LoadedProject,
    unit: &str,
) -> Result<(), String> {
    let profile_module = project
        .modules
        .iter()
        .find(|module| module.ast.domain == "shader" && module.ast.unit == unit)
        .ok_or_else(|| format!("project is missing support module `shader.{unit}`"))?;
    let profile_fn = profile_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
        .ok_or_else(|| {
            format!(
                "project shader unit `shader.{}` requires a `profile()` function",
                unit
            )
        })?;
    let int_bindings = collect_profile_int_bindings(&profile_fn.body);
    let Some(contract) = infer_shader_packet_contract(project, unit)? else {
        return Ok(());
    };
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "shader")?;
    for required_surface in shader_packet_support_surface_contract(&contract) {
        require_declared_support_surface(&declared_support, "shader", unit, required_surface)?;
    }

    let packet_field_count = int_bindings
        .get("packet_field_count")
        .copied()
        .ok_or_else(|| {
            format!(
                "project shader unit `shader.{}` requires `packet_field_count` profile const",
                unit
            )
        })?;
    if packet_field_count != contract.field_count as i64 {
        return Err(format!(
            "project shader unit `shader.{}` requires `packet_field_count = {}` to match inferred packet `{}`",
            unit, contract.field_count, contract.type_name
        ));
    }

    let slot_names = shader_packet_slot_names(&contract);
    let mut seen = BTreeSet::new();
    for &slot in slot_names {
        let value = int_bindings.get(slot).copied().ok_or_else(|| {
            format!(
                "project shader unit `shader.{}` requires `{}` profile const",
                unit, slot
            )
        })?;
        if value < 0 || value >= contract.field_count as i64 {
            return Err(format!(
                "project shader unit `shader.{}` requires `{}` to be within packet field range 0..{}",
                unit, slot, contract.field_count
            ));
        }
        if !seen.insert(value) {
            return Err(format!(
                "project shader unit `shader.{}` requires packet slot indices to be unique",
                unit
            ));
        }
    }

    Ok(())
}

pub(super) fn shader_packet_support_surface_contract(
    contract: &ShaderPacketContract,
) -> &'static [&'static str] {
    if contract.type_name == "NovaPanelPacket" {
        &["shader.profile.packet.nova.v1"]
    } else {
        &[]
    }
}

fn collect_shader_packet_contracts_from_stmts(
    functions: &[nuis_semantics::model::NirFunction],
    unit: &str,
    discovered: &mut Vec<ShaderPacketContract>,
) {
    for function in functions {
        collect_shader_packet_contracts_in_body(&function.body, unit, discovered);
    }
}

fn collect_shader_packet_contracts_in_body(
    body: &[NirStmt],
    unit: &str,
    discovered: &mut Vec<ShaderPacketContract>,
) {
    for stmt in body {
        match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value:
                    NirExpr::ShaderProfilePacket {
                        unit: shader_unit,
                        packet_type_name,
                        accent,
                        toggle_state,
                        focus_index,
                        ..
                    },
                ..
            }
            | NirStmt::Const {
                ty,
                value:
                    NirExpr::ShaderProfilePacket {
                        unit: shader_unit,
                        packet_type_name,
                        accent,
                        toggle_state,
                        focus_index,
                        ..
                    },
                ..
            } if shader_unit == unit || packet_type_name.as_deref() == Some("NovaPanelPacket") => {
                let extended = accent.is_some() || toggle_state.is_some() || focus_index.is_some();
                discovered.push(ShaderPacketContract {
                    type_name: packet_type_name.clone().unwrap_or_else(|| ty.render()),
                    field_count: if extended { 6 } else { 3 },
                });
            }
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }
            | NirStmt::Const {
                ty,
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } if type_name == "NovaPanelPacket" || ty.render() == "NovaPanelPacket" => {
                discovered.push(ShaderPacketContract {
                    type_name: "NovaPanelPacket".to_owned(),
                    field_count: 6,
                });
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_shader_packet_contracts_in_body(then_body, unit, discovered);
                collect_shader_packet_contracts_in_body(else_body, unit, discovered);
            }
            _ => {}
        }
    }
}

pub(super) fn validate_shader_profile_flow(module: &YirModule, unit: &str) -> Result<(), String> {
    let target = resolve_project_profile_target_name("shader", unit, "target");
    let viewport = resolve_project_profile_target_name("shader", unit, "viewport");
    let pipeline = resolve_project_profile_target_name("shader", unit, "pipeline");
    let vertex_count = resolve_project_profile_target_name("shader", unit, "vertex_count");
    let instance_count = resolve_project_profile_target_name("shader", unit, "instance_count");
    let pass_kind = resolve_project_profile_target_name("shader", unit, "pass_kind");
    let packet_field_count =
        resolve_project_profile_target_name("shader", unit, "packet_field_count");

    let begin_passes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_shader_semantic_op(SemanticOp::ShaderBeginPass))
        .map(|node| node.name.as_str())
        .collect::<Vec<_>>();
    let begin_pass_wired = begin_passes.iter().any(|pass| {
        has_edge_to(module, &target, pass)
            && has_edge_to(module, &viewport, pass)
            && has_edge_to(module, &pipeline, pass)
            && has_edge_to(module, &pass_kind, pass)
    });
    if !begin_pass_wired {
        return Err(format!(
            "project shader unit `shader.{}` requires target/viewport/pipeline/pass_kind profile nodes to feed a shader.begin_pass node",
            unit
        ));
    }

    let draws = module
        .nodes
        .iter()
        .filter(|node| {
            node.op
                .is_shader_semantic_op(SemanticOp::ShaderDrawInstanced)
        })
        .map(|node| node.name.as_str())
        .collect::<Vec<_>>();
    let draw_wired = draws.iter().any(|draw| {
        has_edge_to(module, &vertex_count, draw)
            && has_edge_to(module, &instance_count, draw)
            && has_edge_to(module, &packet_field_count, draw)
    });
    if !draw_wired {
        return Err(format!(
            "project shader unit `shader.{}` requires vertex_count/instance_count/packet_field_count profile nodes to feed a shader.draw_instanced node",
            unit
        ));
    }

    let pipeline_models = module
        .nodes
        .iter()
        .filter(|node| node.op.is_shader_semantic_op(SemanticOp::ShaderPipeline))
        .filter_map(|node| node.op.args.first().cloned())
        .collect::<BTreeSet<_>>();
    let inline_entries = module
        .nodes
        .iter()
        .filter(|node| node.op.is_shader_semantic_op(SemanticOp::ShaderInlineWgsl))
        .map(|node| (node.name.as_str(), node.op.args.clone()))
        .collect::<Vec<_>>();
    if inline_entries.is_empty() {
        return Err(format!(
            "project shader unit `shader.{}` requires at least one shader_inline_wgsl(\"entry\", wgsl {{ ... }}) profile node",
            unit
        ));
    }
    let mut matched_pipeline_entry = false;
    for (node_name, args) in inline_entries {
        let Some(entry) = args.first() else {
            return Err(format!(
                "project shader unit `shader.{}` has malformed inline_wgsl node `{}` (missing entry)",
                unit, node_name
            ));
        };
        let Some(source) = args.get(1) else {
            return Err(format!(
                "project shader unit `shader.{}` has malformed inline_wgsl node `{}` (missing source)",
                unit, node_name
            ));
        };
        if !pipeline_models.is_empty() && pipeline_models.contains(entry) {
            matched_pipeline_entry = true;
        }
        if source.trim().is_empty() {
            return Err(format!(
                "project shader unit `shader.{}` has empty inline WGSL source in node `{}`",
                unit, node_name
            ));
        }
        if !source.contains("@vertex") || !source.contains("@fragment") {
            return Err(format!(
                "project shader unit `shader.{}` inline WGSL node `{}` must contain both @vertex and @fragment stages",
                unit, node_name
            ));
        }
    }
    if !pipeline_models.is_empty() && !matched_pipeline_entry {
        return Err(format!(
            "project shader unit `shader.{}` requires shader_inline_wgsl entry to match shader_pipeline shading model ({})",
            unit,
            pipeline_models.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    Ok(())
}

pub(super) fn has_edge_to(module: &YirModule, from: &str, to: &str) -> bool {
    module
        .edges
        .iter()
        .any(|edge| edge.from == from && edge.to == to)
}

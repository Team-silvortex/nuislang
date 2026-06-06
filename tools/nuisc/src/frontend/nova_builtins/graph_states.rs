use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_graph_state_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "nova_transform_state" => build_four_field_state(
            args,
            "nova_transform_state(...) expects 1 arg",
            "NovaTransformPacket",
            "NovaTransformState",
            ["translate", "rotate", "scale", "pivot"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_node_state" => build_four_field_state(
            args,
            "nova_node_state(...) expects 1 arg",
            "NovaNodePacket",
            "NovaNodeState",
            ["node_id", "parent_id", "flags", "depth"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_graph_state" => build_five_field_state(
            args,
            "nova_scene_graph_state(...) expects 1 arg",
            "NovaSceneGraphPacket",
            "NovaSceneGraphState",
            [
                "root_slot",
                "node_count",
                "link_count",
                "instance_count",
                "active_layer",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_node_state" => build_five_field_state(
            args,
            "nova_scene_node_state(...) expects 1 arg",
            "NovaSceneNodePacket",
            "NovaSceneNodeState",
            [
                "node_slot",
                "first_child_slot",
                "sibling_slot",
                "instance_slot",
                "visibility",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_instance_group_state" => build_five_field_state(
            args,
            "nova_instance_group_state(...) expects 1 arg",
            "NovaInstanceGroupPacket",
            "NovaInstanceGroupState",
            [
                "root_instance_slot",
                "group_count",
                "visible_count",
                "phase_bias",
                "material_slot",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_cluster_state" => build_five_field_state(
            args,
            "nova_scene_cluster_state(...) expects 1 arg",
            "NovaSceneClusterPacket",
            "NovaSceneClusterState",
            [
                "root_node_slot",
                "node_budget",
                "instance_group_slot",
                "material_slot",
                "layer_slot",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_link_state" => build_six_field_state(
            args,
            "nova_scene_link_state(...) expects 1 arg",
            "NovaSceneLinkPacket",
            "NovaSceneLinkState",
            [
                "node_slot",
                "transform_slot",
                "mesh_slot",
                "material_slot",
                "light_slot",
                "layer_slot",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_instance_state" => build_six_field_state(
            args,
            "nova_instance_state(...) expects 1 arg",
            "NovaInstancePacket",
            "NovaInstanceState",
            [
                "node_slot",
                "count",
                "stride",
                "phase",
                "material_slot",
                "light_slot",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn build_four_field_state(
    args: &[AstExpr],
    arg_error: &str,
    packet_type: &str,
    state_type: &str,
    fields: [&str; 4],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [packet] = args else {
        return Err(arg_error.to_owned());
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(packet_type)),
    )?;
    Ok(NirExpr::StructLiteral {
        type_name: state_type.to_owned(),
        type_args: Vec::new(),
        fields: vec![
            (fields[0].to_owned(), field(packet.clone(), fields[0])),
            (fields[1].to_owned(), field(packet.clone(), fields[1])),
            (fields[2].to_owned(), field(packet.clone(), fields[2])),
            (fields[3].to_owned(), field(packet, fields[3])),
        ],
    })
}

fn build_five_field_state(
    args: &[AstExpr],
    arg_error: &str,
    packet_type: &str,
    state_type: &str,
    fields: [&str; 5],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [packet] = args else {
        return Err(arg_error.to_owned());
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(packet_type)),
    )?;
    Ok(NirExpr::StructLiteral {
        type_name: state_type.to_owned(),
        type_args: Vec::new(),
        fields: vec![
            (fields[0].to_owned(), field(packet.clone(), fields[0])),
            (fields[1].to_owned(), field(packet.clone(), fields[1])),
            (fields[2].to_owned(), field(packet.clone(), fields[2])),
            (fields[3].to_owned(), field(packet.clone(), fields[3])),
            (fields[4].to_owned(), field(packet, fields[4])),
        ],
    })
}

fn build_six_field_state(
    args: &[AstExpr],
    arg_error: &str,
    packet_type: &str,
    state_type: &str,
    fields: [&str; 6],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [packet] = args else {
        return Err(arg_error.to_owned());
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(packet_type)),
    )?;
    Ok(NirExpr::StructLiteral {
        type_name: state_type.to_owned(),
        type_args: Vec::new(),
        fields: vec![
            (fields[0].to_owned(), field(packet.clone(), fields[0])),
            (fields[1].to_owned(), field(packet.clone(), fields[1])),
            (fields[2].to_owned(), field(packet.clone(), fields[2])),
            (fields[3].to_owned(), field(packet.clone(), fields[3])),
            (fields[4].to_owned(), field(packet.clone(), fields[4])),
            (fields[5].to_owned(), field(packet, fields[5])),
        ],
    })
}

fn field(base: NirExpr, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: field.to_owned(),
    }
}

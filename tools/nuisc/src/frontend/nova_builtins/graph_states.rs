use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_graph_state_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let NovaBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        ..
    } = input;
    let expr = match callee {
        "nova_transform_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_transform_state(...) expects 1 arg",
            packet_type: "NovaTransformPacket",
            state_type: "NovaTransformState",
            fields: &["translate", "rotate", "scale", "pivot"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        })?,
        "nova_node_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_node_state(...) expects 1 arg",
            packet_type: "NovaNodePacket",
            state_type: "NovaNodeState",
            fields: &["node_id", "parent_id", "flags", "depth"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        })?,
        "nova_scene_graph_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_scene_graph_state(...) expects 1 arg",
            packet_type: "NovaSceneGraphPacket",
            state_type: "NovaSceneGraphState",
            fields: &[
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
        })?,
        "nova_scene_node_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_scene_node_state(...) expects 1 arg",
            packet_type: "NovaSceneNodePacket",
            state_type: "NovaSceneNodeState",
            fields: &[
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
        })?,
        "nova_instance_group_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_instance_group_state(...) expects 1 arg",
            packet_type: "NovaInstanceGroupPacket",
            state_type: "NovaInstanceGroupState",
            fields: &[
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
        })?,
        "nova_scene_cluster_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_scene_cluster_state(...) expects 1 arg",
            packet_type: "NovaSceneClusterPacket",
            state_type: "NovaSceneClusterState",
            fields: &[
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
        })?,
        "nova_scene_link_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_scene_link_state(...) expects 1 arg",
            packet_type: "NovaSceneLinkPacket",
            state_type: "NovaSceneLinkState",
            fields: &[
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
        })?,
        "nova_instance_state" => build_state(StateBuildInput {
            args,
            arg_error: "nova_instance_state(...) expects 1 arg",
            packet_type: "NovaInstancePacket",
            state_type: "NovaInstanceState",
            fields: &[
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
        })?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

struct StateBuildInput<'a> {
    args: &'a [AstExpr],
    arg_error: &'a str,
    packet_type: &'a str,
    state_type: &'a str,
    fields: &'a [&'a str],
    current_domain: &'a str,
    bindings: &'a BTreeMap<String, NirTypeRef>,
    signatures: &'a BTreeMap<String, FunctionSignature>,
    struct_table: &'a BTreeMap<String, NirStructDef>,
}

fn build_state(input: StateBuildInput<'_>) -> Result<NirExpr, String> {
    let StateBuildInput {
        args,
        arg_error,
        packet_type,
        state_type,
        fields,
        current_domain,
        bindings,
        signatures,
        struct_table,
    } = input;
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
        fields: fields
            .iter()
            .map(|name| ((*name).to_owned(), field(packet.clone(), name)))
            .collect(),
    })
}

fn field(base: NirExpr, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: field.to_owned(),
    }
}

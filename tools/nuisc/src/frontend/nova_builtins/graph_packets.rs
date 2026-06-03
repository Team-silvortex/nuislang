use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{FunctionSignature, ModuleConstValue};
use super::packet_helpers::{build_struct_literal, lower_i64_arg_list};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_graph_packet_builtin_call(
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
        "nova_transform_packet" => build_packet(
            args,
            4,
            "nova_transform_packet(...) expects 4 args",
            "NovaTransformPacket",
            &["translate", "rotate", "scale", "pivot"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_node_packet" => build_packet(
            args,
            4,
            "nova_node_packet(...) expects 4 args",
            "NovaNodePacket",
            &["node_id", "parent_id", "flags", "depth"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_link_packet" => build_packet(
            args,
            6,
            "nova_scene_link_packet(...) expects 6 args",
            "NovaSceneLinkPacket",
            &[
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
        "nova_instance_packet" => build_packet(
            args,
            6,
            "nova_instance_packet(...) expects 6 args",
            "NovaInstancePacket",
            &[
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
        "nova_scene_graph_packet" => build_packet(
            args,
            5,
            "nova_scene_graph_packet(...) expects 5 args",
            "NovaSceneGraphPacket",
            &[
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
        "nova_scene_node_packet" => build_packet(
            args,
            5,
            "nova_scene_node_packet(...) expects 5 args",
            "NovaSceneNodePacket",
            &[
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
        "nova_instance_group_packet" => build_packet(
            args,
            5,
            "nova_instance_group_packet(...) expects 5 args",
            "NovaInstanceGroupPacket",
            &[
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
        "nova_scene_cluster_packet" => build_packet(
            args,
            5,
            "nova_scene_cluster_packet(...) expects 5 args",
            "NovaSceneClusterPacket",
            &[
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
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn build_packet(
    args: &[AstExpr],
    expected_len: usize,
    arg_error: &str,
    type_name: &str,
    fields: &[&str],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let values = lower_i64_arg_list(
        args,
        expected_len,
        arg_error,
        current_domain,
        bindings,
        signatures,
        struct_table,
    )?;
    Ok(build_struct_literal(type_name, fields, values))
}

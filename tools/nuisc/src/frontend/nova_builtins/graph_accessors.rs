use nuis_semantics::model::NirExpr;

use super::super::{lower_expr, named_type};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_graph_accessor_builtin_call(
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
    let Some((expected_type, field_name)) = graph_state_accessor_target(callee) else {
        return Ok(None);
    };
    let [state] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let state = lower_expr(
        state,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(state),
        field: field_name.to_owned(),
    }))
}

fn graph_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_transform_state_translate" => ("NovaTransformState", "translate"),
        "nova_transform_state_rotate" => ("NovaTransformState", "rotate"),
        "nova_transform_state_scale" => ("NovaTransformState", "scale"),
        "nova_transform_state_pivot" => ("NovaTransformState", "pivot"),
        "nova_node_state_node_id" => ("NovaNodeState", "node_id"),
        "nova_node_state_parent_id" => ("NovaNodeState", "parent_id"),
        "nova_node_state_flags" => ("NovaNodeState", "flags"),
        "nova_node_state_depth" => ("NovaNodeState", "depth"),
        "nova_scene_link_state_node" => ("NovaSceneLinkState", "node_slot"),
        "nova_scene_link_state_transform" => ("NovaSceneLinkState", "transform_slot"),
        "nova_scene_link_state_mesh" => ("NovaSceneLinkState", "mesh_slot"),
        "nova_scene_link_state_material" => ("NovaSceneLinkState", "material_slot"),
        "nova_scene_link_state_light" => ("NovaSceneLinkState", "light_slot"),
        "nova_scene_link_state_layer" => ("NovaSceneLinkState", "layer_slot"),
        "nova_instance_state_node" => ("NovaInstanceState", "node_slot"),
        "nova_instance_state_count" => ("NovaInstanceState", "count"),
        "nova_instance_state_stride" => ("NovaInstanceState", "stride"),
        "nova_instance_state_phase" => ("NovaInstanceState", "phase"),
        "nova_instance_state_material" => ("NovaInstanceState", "material_slot"),
        "nova_instance_state_light" => ("NovaInstanceState", "light_slot"),
        "nova_scene_graph_state_root" => ("NovaSceneGraphState", "root_slot"),
        "nova_scene_graph_state_nodes" => ("NovaSceneGraphState", "node_count"),
        "nova_scene_graph_state_links" => ("NovaSceneGraphState", "link_count"),
        "nova_scene_graph_state_instances" => ("NovaSceneGraphState", "instance_count"),
        "nova_scene_graph_state_layer" => ("NovaSceneGraphState", "active_layer"),
        "nova_scene_node_state_node" => ("NovaSceneNodeState", "node_slot"),
        "nova_scene_node_state_first_child" => ("NovaSceneNodeState", "first_child_slot"),
        "nova_scene_node_state_sibling" => ("NovaSceneNodeState", "sibling_slot"),
        "nova_scene_node_state_instance" => ("NovaSceneNodeState", "instance_slot"),
        "nova_scene_node_state_visibility" => ("NovaSceneNodeState", "visibility"),
        "nova_instance_group_state_root" => ("NovaInstanceGroupState", "root_instance_slot"),
        "nova_instance_group_state_groups" => ("NovaInstanceGroupState", "group_count"),
        "nova_instance_group_state_visible" => ("NovaInstanceGroupState", "visible_count"),
        "nova_instance_group_state_phase_bias" => ("NovaInstanceGroupState", "phase_bias"),
        "nova_instance_group_state_material" => ("NovaInstanceGroupState", "material_slot"),
        "nova_scene_cluster_state_root" => ("NovaSceneClusterState", "root_node_slot"),
        "nova_scene_cluster_state_budget" => ("NovaSceneClusterState", "node_budget"),
        "nova_scene_cluster_state_instance_group" => {
            ("NovaSceneClusterState", "instance_group_slot")
        }
        "nova_scene_cluster_state_material" => ("NovaSceneClusterState", "material_slot"),
        "nova_scene_cluster_state_layer" => ("NovaSceneClusterState", "layer_slot"),
        _ => return None,
    })
}

use super::*;

#[test]
fn lowers_nova_transform_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
            let state: NovaTransformState = nova_transform_state(transform);
            let scale: i64 = nova_transform_state_scale(state);
            return scale;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaTransformState" && type_name == "NovaTransformState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_node_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
            let state: NovaNodeState = nova_node_state(node);
            let depth: i64 = nova_node_state_depth(state);
            return depth;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaNodeState" && type_name == "NovaNodeState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_scene_link_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let link: NovaSceneLinkPacket = nova_scene_link_packet(1, 2, 3, 4, 5, 6);
            let state: NovaSceneLinkState = nova_scene_link_state(link);
            let mesh_slot: i64 = nova_scene_link_state_mesh(state);
            return mesh_slot;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaSceneLinkState" && type_name == "NovaSceneLinkState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_instance_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let instance: NovaInstancePacket = nova_instance_packet(1, 2, 3, 4, 5, 6);
            let state: NovaInstanceState = nova_instance_state(instance);
            let count: i64 = nova_instance_state_count(state);
            return count;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaInstanceState" && type_name == "NovaInstanceState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_scene_graph_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let graph: NovaSceneGraphPacket = nova_scene_graph_packet(1, 6, 3, 2, 1);
            let state: NovaSceneGraphState = nova_scene_graph_state(graph);
            let roots: i64 = nova_scene_graph_state_root(state);
            return roots;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaSceneGraphState" && type_name == "NovaSceneGraphState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_scene_node_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let node: NovaSceneNodePacket = nova_scene_node_packet(1, 2, 3, 4, 1);
            let state: NovaSceneNodeState = nova_scene_node_state(node);
            let child: i64 = nova_scene_node_state_first_child(state);
            return child;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaSceneNodeState" && type_name == "NovaSceneNodeState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_instance_group_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let group: NovaInstanceGroupPacket = nova_instance_group_packet(1, 4, 3, 2, 8);
            let state: NovaInstanceGroupState = nova_instance_group_state(group);
            let visible: i64 = nova_instance_group_state_visible(state);
            return visible;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaInstanceGroupState" && type_name == "NovaInstanceGroupState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_scene_cluster_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(1, 6, 3, 8, 1);
            let state: NovaSceneClusterState = nova_scene_cluster_state(cluster);
            let budget: i64 = nova_scene_cluster_state_budget(state);
            return budget;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaSceneClusterState" && type_name == "NovaSceneClusterState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_visibility_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
            let state: NovaVisibilityState = nova_visibility_state(visibility);
            let visible: i64 = nova_visibility_state_visible(state);
            return visible;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaVisibilityState" && type_name == "NovaVisibilityState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_cull_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
            let state: NovaCullState = nova_cull_state(cull);
            let kept: i64 = nova_cull_state_kept(state);
            return kept;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaCullState" && type_name == "NovaCullState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_lod_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
            let state: NovaLodState = nova_lod_state(lod);
            let active: i64 = nova_lod_state_active(state);
            return active;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaLodState" && type_name == "NovaLodState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_streaming_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
            let state: NovaStreamingState = nova_streaming_state(streaming);
            let resident: i64 = nova_streaming_state_resident(state);
            return resident;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaStreamingState" && type_name == "NovaStreamingState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_residency_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
            let state: NovaResidencyState = nova_residency_state(residency);
            let committed: i64 = nova_residency_state_committed(state);
            return committed;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaResidencyState" && type_name == "NovaResidencyState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_eviction_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
            let state: NovaEvictionState = nova_eviction_state(eviction);
            let evicted: i64 = nova_eviction_state_evicted(state);
            return evicted;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaEvictionState" && type_name == "NovaEvictionState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_prefetch_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
            let state: NovaPrefetchState = nova_prefetch_state(prefetch);
            let requested: i64 = nova_prefetch_state_requested(state);
            return requested;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaPrefetchState" && type_name == "NovaPrefetchState",
        _ => false,
    }));
}

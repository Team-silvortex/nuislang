use super::*;

pub(super) fn ensure_fabric_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "fabric0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "fabric0".to_owned(),
        kind: ResourceKind::parse("data.fabric"),
    });
}

pub(super) fn ensure_shader_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "shader0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "shader0".to_owned(),
        kind: ResourceKind::parse("shader.render"),
    });
}

pub(super) fn ensure_kernel_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "kernel0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "kernel0".to_owned(),
        kind: ResourceKind::parse("kernel.compute"),
    });
}

pub(super) fn ensure_network_resource(yir: &mut YirModule) {
    if yir
        .resources
        .iter()
        .any(|resource| resource.name == "network0")
    {
        return;
    }
    yir.resources.push(Resource {
        name: "network0".to_owned(),
        kind: ResourceKind::parse("network.io"),
    });
}

pub(super) fn push_dep_edges(state: &mut LoweringState<'_>, from: &str, to: &str) {
    let from_node = state.yir.nodes.iter().find(|node| node.name == from);
    let to_node = state.yir.nodes.iter().find(|node| node.name == to);
    let (Some(from_node), Some(to_node)) = (from_node, to_node) else {
        return;
    };
    if from_node.resource != to_node.resource {
        push_xfer_edge(state, from, to);
        return;
    }
    push_unique_edge(state, EdgeKind::Dep, from, to);
}

pub(super) fn push_xfer_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    push_unique_edge(state, EdgeKind::CrossDomainExchange, from, to);
}

pub(super) fn push_lifetime_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    push_unique_edge(state, EdgeKind::Lifetime, from, to);
}

fn push_unique_edge(state: &mut LoweringState<'_>, kind: EdgeKind, from: &str, to: &str) {
    let exists = state
        .yir
        .edges
        .iter()
        .any(|edge| edge.kind == kind && edge.from == from && edge.to == to);
    if exists {
        return;
    }
    state.yir.edges.push(Edge {
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

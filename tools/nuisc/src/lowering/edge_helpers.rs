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
    sync_node_resource_index(state);
    let crosses_resource = match (state.node_resources.get(from), state.node_resources.get(to)) {
        (Some(from_resource), Some(to_resource)) => from_resource != to_resource,
        _ => return,
    };
    if crosses_resource {
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

pub(super) fn push_effect_edge(state: &mut LoweringState<'_>, from: &str, to: &str) {
    push_unique_edge(state, EdgeKind::Effect, from, to);
}

pub(super) fn invalidate_graph_indexes(state: &mut LoweringState<'_>) {
    state.node_resources.clear();
    state.indexed_node_count = 0;
    state.edge_index.clear();
    state.indexed_edge_count = 0;
}

fn push_unique_edge(state: &mut LoweringState<'_>, kind: EdgeKind, from: &str, to: &str) {
    sync_edge_index(state);
    let key = (from.to_owned(), to.to_owned(), kind.as_str());
    if !state.edge_index.insert(key) {
        return;
    }
    state.yir.edges.push(Edge {
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
    });
    state.indexed_edge_count += 1;
}

fn sync_node_resource_index(state: &mut LoweringState<'_>) {
    for node in state.yir.nodes.iter().skip(state.indexed_node_count) {
        state
            .node_resources
            .entry(node.name.clone())
            .or_insert_with(|| node.resource.clone());
    }
    state.indexed_node_count = state.yir.nodes.len();
}

fn sync_edge_index(state: &mut LoweringState<'_>) {
    for edge in state.yir.edges.iter().skip(state.indexed_edge_count) {
        state
            .edge_index
            .insert((edge.from.clone(), edge.to.clone(), edge.kind.as_str()));
    }
    state.indexed_edge_count = state.yir.edges.len();
}

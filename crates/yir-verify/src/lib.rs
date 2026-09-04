use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use yir_core::{
    glm_profile_for_operation, DataMod, EdgeKind, GlmEffect, GlmUseMode, LegacyFabricMod,
    ModRegistry, Node, Resource, ResourceKind, SemanticOp, YirModule,
};

mod cpu_heap;
mod cpu_heap_checks;
mod cpu_heap_state;
mod function_contracts;
mod graph;
mod nustar_provider;
mod project_abi_contracts;
mod project_contracts;
mod project_target_contracts;
mod result_state;
mod scheduler_contracts;
mod scheduler_lane_contracts;
mod scheduler_observer_contracts;

use cpu_heap::verify_cpu_heap_protocol;
use function_contracts::verify_function_table;
use graph::{ensure_acyclic, topological_order};
use project_contracts::{verify_lowering_contract_nodes, verify_project_type_contract_nodes};
use result_state::verify_result_state_nodes;
use scheduler_contracts::verify_scheduler_contract_nodes;

pub use nustar_provider::{
    register_static_nustar_semantics, static_nustar_semantic_providers,
    StaticNustarSemanticProvider,
};

pub fn default_registry() -> ModRegistry {
    let mut registry = ModRegistry::new();
    registry.register(DataMod);
    registry.register(LegacyFabricMod);
    registry.register(yir_domain_cffi::CffiMod);
    registry.register(yir_domain_cpu::CpuMod);
    registry.register(yir_domain_kernel::KernelMod);
    registry.register(yir_domain_kernel::LegacyNpuMod);
    registry.register(yir_domain_network::NetworkMod);
    registry.register(yir_domain_npu::NpuMod);
    registry.register(yir_domain_shader::ShaderMod);
    registry
}

pub fn verify_module(module: &YirModule) -> Result<(), String> {
    let registry = default_registry();
    verify_module_with_registry(module, &registry)
}

pub fn verify_module_with_registry(
    module: &YirModule,
    registry: &ModRegistry,
) -> Result<(), String> {
    if module.version.is_empty() {
        return Err("module version must not be empty".to_owned());
    }

    let mut resources = HashMap::with_capacity(module.resources.len());
    for resource in &module.resources {
        if resources.insert(resource.name.as_str(), resource).is_some() {
            return Err(format!("duplicate resource `{}`", resource.name));
        }
    }

    let mut nodes = HashMap::with_capacity(module.nodes.len());
    for node in &module.nodes {
        if nodes.insert(node.name.as_str(), node).is_some() {
            return Err(format!("duplicate node `{}`", node.name));
        }
    }
    let node_indices = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.as_str(), index))
        .collect::<HashMap<_, _>>();

    let mut edge_index = HashSet::with_capacity(module.edges.len());
    for edge in &module.edges {
        let Some(&source_index) = node_indices.get(edge.from.as_str()) else {
            return Err(format!(
                "edge `{}` {} `{}` references unknown source node",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        };

        let Some(&target_index) = node_indices.get(edge.to.as_str()) else {
            return Err(format!(
                "edge `{}` {} `{}` references unknown target node",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        };

        if !edge_index.insert((source_index, target_index, edge.kind.as_str())) {
            return Err(format!(
                "duplicate edge `{}` {} `{}`",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }
    }

    verify_function_table(module, &nodes)?;

    for (node_index, node) in module.nodes.iter().enumerate() {
        let resource = resources
            .get(node.resource.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "node `{}` references unknown resource `{}`",
                    node.name, node.resource
                )
            })?;

        let module_impl = registry.lookup(&node.op.module).ok_or_else(|| {
            format!(
                "node `{}` references unregistered mod `{}`",
                node.name, node.op.module
            )
        })?;

        let semantics = match registry.describe_branch_effect_node(node)? {
            Some(semantics) => semantics,
            None => module_impl.describe(node, resource)?,
        };

        for dependency in semantics.dependencies {
            if dependency == node.name {
                return Err(format!("node `{}` may not depend on itself", node.name));
            }

            let source_index = node_indices
                .get(dependency.as_str())
                .copied()
                .ok_or_else(|| {
                    format!(
                        "node `{}` depends on unknown node `{dependency}`",
                        node.name
                    )
                })?;
            let source = &module.nodes[source_index];

            let source_resource = resources
                .get(source.resource.as_str())
                .copied()
                .ok_or_else(|| {
                    format!(
                        "node `{}` depends on `{dependency}` with unknown resource `{}`",
                        node.name, source.resource
                    )
                })?;

            let required_kind =
                required_dependency_edge_kind(&source_resource.kind, &resource.kind);
            let key = (source_index, node_index, required_kind.as_str());

            if !edge_index.contains(&key) {
                return Err(format!(
                    "node `{}` requires `{}` edge from `{}` to `{}`",
                    node.name,
                    required_kind.as_str(),
                    dependency,
                    node.name
                ));
            }
        }
    }

    ensure_acyclic(module)?;
    verify_glm_protocol(module)?;
    verify_data_fabric_protocol(module, &resources)?;
    verify_result_state_nodes(module)?;
    verify_scheduler_contract_nodes(module, &resources, &nodes)?;
    verify_project_type_contract_nodes(module)?;
    verify_lowering_contract_nodes(module)?;
    verify_cpu_heap_protocol(module)?;
    Ok(())
}

fn verify_glm_protocol(module: &YirModule) -> Result<(), String> {
    let node_indices = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.as_str(), index))
        .collect::<HashMap<_, _>>();
    let dependency_edges = module
        .edges
        .iter()
        .filter(|edge| matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange))
        .filter_map(|edge| {
            Some((
                *node_indices.get(edge.from.as_str())?,
                *node_indices.get(edge.to.as_str())?,
            ))
        })
        .collect::<HashSet<_>>();
    let lifetime_edges = module
        .edges
        .iter()
        .filter(|edge| matches!(edge.kind, EdgeKind::Lifetime))
        .filter_map(|edge| {
            Some((
                *node_indices.get(edge.from.as_str())?,
                *node_indices.get(edge.to.as_str())?,
            ))
        })
        .collect::<HashSet<_>>();
    let mut reverse_edges = vec![Vec::<usize>::new(); module.nodes.len()];
    for edge in &module.edges {
        if let (Some(source), Some(target)) = (
            node_indices.get(edge.from.as_str()),
            node_indices.get(edge.to.as_str()),
        ) {
            reverse_edges[*target].push(*source);
        }
    }
    let mut consumers =
        vec![Vec::<(usize, yir_core::GlmValueClass, GlmUseMode)>::new(); module.nodes.len()];

    for (node_index, node) in module.nodes.iter().enumerate() {
        let profile = glm_profile_for_operation(&node.op);
        for access in &profile.accesses {
            let Some(source_index) = node_indices.get(access.input.as_str()).copied() else {
                continue;
            };
            consumers[source_index].push((node_index, access.class, access.mode));
            let has_dep = dependency_edges.contains(&(source_index, node_index));
            if !has_dep {
                return Err(format!(
                    "GLM: node `{}` uses `{}` as {} {} without dep/xfer edge",
                    node.name, access.input, access.class, access.mode
                ));
            }
            if matches!(access.class, yir_core::GlmValueClass::Res)
                && matches!(access.mode, GlmUseMode::Own | GlmUseMode::Write)
            {
                let has_lifetime = lifetime_edges.contains(&(source_index, node_index));
                if !has_lifetime {
                    return Err(format!(
                        "GLM: node `{}` requires lifetime edge from `{}` for {} {} access",
                        node.name, access.input, access.class, access.mode
                    ));
                }
            }
        }

        match profile.effect {
            GlmEffect::DomainMove | GlmEffect::LifetimeEnd => {
                if let Some(primary) = profile.accesses.first() {
                    if matches!(primary.class, yir_core::GlmValueClass::Res) {
                        let ordered = node_indices
                            .get(primary.input.as_str())
                            .is_some_and(|source| lifetime_edges.contains(&(*source, node_index)));
                        if !ordered {
                            return Err(format!(
                                "GLM: node `{}` must be ordered by lifetime from `{}`",
                                node.name, primary.input
                            ));
                        }
                    }
                }
            }
            GlmEffect::None => {}
        }
    }

    let mut consumed_sources = consumers
        .iter()
        .enumerate()
        .filter_map(|(index, values)| (!values.is_empty()).then_some(index))
        .collect::<Vec<_>>();
    consumed_sources
        .sort_unstable_by(|lhs, rhs| module.nodes[*lhs].name.cmp(&module.nodes[*rhs].name));
    for source_index in consumed_sources {
        let source = &module.nodes[source_index].name;
        let consumers_for_source = &consumers[source_index];
        for (owner_index, class, mode) in consumers_for_source {
            if !matches!(mode, GlmUseMode::Own) {
                continue;
            }
            let owner_node = &module.nodes[*owner_index].name;
            let ordered_before_owner = reverse_reachable_nodes(&reverse_edges, *owner_index);
            for (other_index, _, _) in consumers_for_source {
                if other_index == owner_index {
                    continue;
                }
                if !ordered_before_owner.contains(other_index) {
                    let other_node = &module.nodes[*other_index].name;
                    return Err(format!(
                        "GLM: node `{}` consumes {} `{}` with Own, but `{}` is not ordered before that consume",
                        owner_node, class, source, other_node
                    ));
                }
            }
        }
    }

    Ok(())
}

fn reverse_reachable_nodes(reverse_edges: &[Vec<usize>], target: usize) -> HashSet<usize> {
    let mut stack = reverse_edges[target].clone();
    let mut reachable = HashSet::new();
    while let Some(current) = stack.pop() {
        if !reachable.insert(current) {
            continue;
        }
        stack.extend(reverse_edges[current].iter().copied());
    }
    reachable
}

fn required_dependency_edge_kind(source: &ResourceKind, target: &ResourceKind) -> EdgeKind {
    if source.family() == target.family() {
        EdgeKind::Dep
    } else {
        EdgeKind::CrossDomainExchange
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DataValueKind {
    Other,
    PipeOutput,
    PipeInput,
    WindowMutable,
    WindowImmutable,
    Marker,
    HandleTable,
    CoreBinding,
}

fn verify_data_fabric_protocol(
    module: &YirModule,
    resources: &HashMap<&str, &Resource>,
) -> Result<(), String> {
    let order = topological_order(module)?;
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut value_kinds = BTreeMap::<String, DataValueKind>::new();

    for node_name in order {
        let node = nodes
            .get(node_name.as_str())
            .copied()
            .ok_or_else(|| format!("verification order references unknown node `{node_name}`"))?;

        let kind = if node.op.is_data_domain_family() {
            match node.op.semantic_op() {
                SemanticOp::DataMove => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source != DataValueKind::Other {
                        return Err(format!(
                            "node `{}` cannot use data.move on non-Value payload `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::Other
                }
                SemanticOp::DataOutputPipe => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source == DataValueKind::WindowMutable {
                        return Err(format!(
                            "node `{}` cannot send mutable window payload `{}` across data pipe",
                            node.name, node.op.args[0]
                        ));
                    }
                    if source == DataValueKind::PipeOutput || source == DataValueKind::PipeInput {
                        return Err(format!(
                            "node `{}` creates nested pipe value from `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::PipeOutput
                }
                SemanticOp::DataInputPipe => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source != DataValueKind::PipeOutput {
                        return Err(format!(
                            "node `{}` expects output_pipe input, got `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::Other
                }
                SemanticOp::DataCopyWindow | SemanticOp::DataImmutableWindow => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if matches!(
                        source,
                        DataValueKind::WindowMutable
                            | DataValueKind::WindowImmutable
                            | DataValueKind::PipeOutput
                            | DataValueKind::PipeInput
                            | DataValueKind::Marker
                            | DataValueKind::HandleTable
                    ) {
                        return Err(format!(
                            "node `{}` cannot create nested/illegal window from `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    match node.op.semantic_op() {
                        SemanticOp::DataCopyWindow => DataValueKind::WindowMutable,
                        SemanticOp::DataImmutableWindow => DataValueKind::WindowImmutable,
                        _ => unreachable!(),
                    }
                }
                SemanticOp::DataReadWindow => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if !matches!(
                        source,
                        DataValueKind::WindowMutable | DataValueKind::WindowImmutable
                    ) {
                        return Err(format!(
                            "node `{}` expects window input for read_window, got `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::Other
                }
                SemanticOp::DataWriteWindow => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source != DataValueKind::WindowMutable {
                        return Err(format!(
                            "node `{}` expects mutable window input for write_window, got `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::WindowMutable
                }
                SemanticOp::DataValue => {
                    infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0])
                }
                SemanticOp::DataObserve => {
                    if node.op.args.get(1).is_some_and(|state| state == "windowed") {
                        infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0])
                    } else {
                        DataValueKind::Other
                    }
                }
                SemanticOp::DataFreezeWindow => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if !matches!(
                        source,
                        DataValueKind::WindowMutable | DataValueKind::WindowImmutable
                    ) {
                        return Err(format!(
                            "node `{}` expects window input for freeze_window, got `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::WindowImmutable
                }
                SemanticOp::DataMarker => DataValueKind::Marker,
                SemanticOp::DataHandleTable => {
                    let mut seen_slots = BTreeSet::new();
                    for entry in &node.op.args {
                        let Some((slot, resource_name)) = entry.split_once('=') else {
                            return Err(format!(
                                "node `{}` has invalid handle-table entry `{}`",
                                node.name, entry
                            ));
                        };
                        let slot = slot.trim();
                        let resource_name = resource_name.trim();
                        if slot.is_empty() || resource_name.is_empty() {
                            return Err(format!(
                                "node `{}` has empty handle-table slot/resource in `{}`",
                                node.name, entry
                            ));
                        }
                        if !seen_slots.insert(slot.to_owned()) {
                            return Err(format!(
                                "node `{}` has duplicate handle-table slot `{}`",
                                node.name, slot
                            ));
                        }
                        if !resources.contains_key(resource_name) {
                            return Err(format!(
                                "node `{}` references unknown resource `{}` in handle table",
                                node.name, resource_name
                            ));
                        }
                    }
                    DataValueKind::HandleTable
                }
                SemanticOp::DataBindCore => {
                    if node.op.args[0].parse::<usize>().is_err() {
                        return Err(format!(
                            "node `{}` has invalid fabric core index `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::CoreBinding
                }
                _ => DataValueKind::Other,
            }
        } else {
            DataValueKind::Other
        };

        value_kinds.insert(node.name.clone(), kind);
    }

    Ok(())
}

fn infer_data_value_kind(
    value_kinds: &BTreeMap<String, DataValueKind>,
    nodes: &BTreeMap<&str, &Node>,
    name: &str,
) -> DataValueKind {
    value_kinds.get(name).copied().unwrap_or_else(|| {
        nodes
            .get(name)
            .map(|node| match node.op.semantic_op() {
                SemanticOp::DataMarker => DataValueKind::Marker,
                SemanticOp::DataHandleTable => DataValueKind::HandleTable,
                SemanticOp::DataBindCore => DataValueKind::CoreBinding,
                SemanticOp::DataOutputPipe => DataValueKind::PipeOutput,
                SemanticOp::DataValue => {
                    infer_data_value_kind(value_kinds, nodes, &node.op.args[0])
                }
                SemanticOp::DataObserve => {
                    if node.op.args.get(1).is_some_and(|state| state == "windowed") {
                        infer_data_value_kind(value_kinds, nodes, &node.op.args[0])
                    } else {
                        DataValueKind::Other
                    }
                }
                SemanticOp::DataInputPipe | SemanticOp::DataMove | SemanticOp::DataReadWindow => {
                    DataValueKind::Other
                }
                SemanticOp::DataCopyWindow => DataValueKind::WindowMutable,
                SemanticOp::DataWriteWindow => DataValueKind::WindowMutable,
                SemanticOp::DataFreezeWindow => DataValueKind::WindowImmutable,
                SemanticOp::DataImmutableWindow => DataValueKind::WindowImmutable,
                _ => DataValueKind::Other,
            })
            .unwrap_or(DataValueKind::Other)
    })
}

#[cfg(test)]
mod tests;

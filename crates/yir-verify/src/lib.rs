use std::collections::{BTreeMap, BTreeSet};

use yir_core::{
    glm_profile_for_operation, DataMod, EdgeKind, GlmEffect, GlmUseMode, LegacyFabricMod,
    ModRegistry, Node, Resource, ResourceKind, SemanticOp, YirModule,
};

pub fn default_registry() -> ModRegistry {
    let mut registry = ModRegistry::new();
    registry.register(DataMod);
    registry.register(LegacyFabricMod);
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

    let mut resources = BTreeMap::<String, &Resource>::new();
    for resource in &module.resources {
        if resources.insert(resource.name.clone(), resource).is_some() {
            return Err(format!("duplicate resource `{}`", resource.name));
        }
    }

    let mut nodes = BTreeMap::<String, &Node>::new();
    for node in &module.nodes {
        if nodes.insert(node.name.clone(), node).is_some() {
            return Err(format!("duplicate node `{}`", node.name));
        }
    }

    let mut edge_index = BTreeSet::<(String, String, &'static str)>::new();
    for edge in &module.edges {
        if !nodes.contains_key(&edge.from) {
            return Err(format!(
                "edge `{}` {} `{}` references unknown source node",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }

        if !nodes.contains_key(&edge.to) {
            return Err(format!(
                "edge `{}` {} `{}` references unknown target node",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }

        if !edge_index.insert((edge.from.clone(), edge.to.clone(), edge.kind.as_str())) {
            return Err(format!(
                "duplicate edge `{}` {} `{}`",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }
    }

    for node in &module.nodes {
        let resource = resources.get(&node.resource).copied().ok_or_else(|| {
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

        let semantics = module_impl.describe(node, resource)?;

        for dependency in semantics.dependencies {
            if dependency == node.name {
                return Err(format!("node `{}` may not depend on itself", node.name));
            }

            let source = nodes.get(&dependency).copied().ok_or_else(|| {
                format!(
                    "node `{}` depends on unknown node `{dependency}`",
                    node.name
                )
            })?;

            let source_resource = resources.get(&source.resource).copied().ok_or_else(|| {
                format!(
                    "node `{}` depends on `{dependency}` with unknown resource `{}`",
                    node.name, source.resource
                )
            })?;

            let required_kind =
                required_dependency_edge_kind(&source_resource.kind, &resource.kind);
            let key = (
                dependency.clone(),
                node.name.clone(),
                required_kind.as_str(),
            );

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
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut consumers =
        BTreeMap::<String, Vec<(String, yir_core::GlmValueClass, GlmUseMode)>>::new();

    for node in &module.nodes {
        let profile = glm_profile_for_operation(&node.op);
        for access in &profile.accesses {
            if !nodes.contains_key(access.input.as_str()) {
                continue;
            }
            consumers.entry(access.input.clone()).or_default().push((
                node.name.clone(),
                access.class,
                access.mode,
            ));
            let has_dep = module.edges.iter().any(|edge| {
                edge.from == access.input
                    && edge.to == node.name
                    && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
            });
            if !has_dep {
                return Err(format!(
                    "GLM: node `{}` uses `{}` as {} {} without dep/xfer edge",
                    node.name, access.input, access.class, access.mode
                ));
            }
            if matches!(access.class, yir_core::GlmValueClass::Res)
                && matches!(access.mode, GlmUseMode::Own | GlmUseMode::Write)
            {
                let has_lifetime = module.edges.iter().any(|edge| {
                    edge.from == access.input
                        && edge.to == node.name
                        && matches!(edge.kind, EdgeKind::Lifetime)
                });
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
                        let lifetime_count = module
                            .edges
                            .iter()
                            .filter(|edge| {
                                edge.from == primary.input
                                    && edge.to == node.name
                                    && matches!(edge.kind, EdgeKind::Lifetime)
                            })
                            .count();
                        if lifetime_count == 0 {
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

    for (source, consumers_for_source) in &consumers {
        for (owner_node, class, mode) in consumers_for_source {
            if !matches!(mode, GlmUseMode::Own) {
                continue;
            }
            for (other_node, _, _) in consumers_for_source {
                if other_node == owner_node {
                    continue;
                }
                if !path_exists(module, other_node, owner_node) {
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

fn required_dependency_edge_kind(source: &ResourceKind, target: &ResourceKind) -> EdgeKind {
    if source.family() == target.family() {
        EdgeKind::Dep
    } else {
        EdgeKind::CrossDomainExchange
    }
}

fn ensure_acyclic(module: &YirModule) -> Result<(), String> {
    let mut adjacency = BTreeMap::<&str, Vec<&str>>::new();
    let mut indegree = BTreeMap::<&str, usize>::new();

    for node in &module.nodes {
        adjacency.entry(node.name.as_str()).or_default();
        indegree.entry(node.name.as_str()).or_insert(0);
    }

    for edge in &module.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        *indegree.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(*name))
        .collect::<Vec<_>>();
    let mut visited = 0usize;

    while let Some(node) = ready.pop() {
        visited += 1;
        if let Some(targets) = adjacency.get(node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target);
                    }
                }
            }
        }
    }

    if visited != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum PointerState {
    Null,
    Owned(usize),
    Borrowed(usize),
    Unknown,
}

#[derive(Clone, Copy)]
enum HeapObjectKind {
    Node { next: PointerState },
    Buffer { len: Option<usize> },
}

#[derive(Clone, Copy)]
struct HeapBinding {
    live: bool,
    kind: HeapObjectKind,
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
    resources: &BTreeMap<String, &Resource>,
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

fn verify_project_type_contract_nodes(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        if node.op.module != "cpu" || node.op.instruction != "text" {
            continue;
        }

        let Some(contract) = classify_project_contract_node(node.name.as_str()) else {
            continue;
        };
        let value = node
            .op
            .args
            .first()
            .map(|value| value.trim())
            .ok_or_else(|| {
                format!(
                    "project contract node `{}` must carry a canonical type payload",
                    node.name
                )
            })?;
        if value.is_empty() {
            return Err(format!(
                "project contract node `{}` must carry a non-empty canonical type payload",
                node.name
            ));
        }

        let target = nodes
            .get(contract.target.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "project contract node `{}` references unknown target `{}`",
                    node.name, contract.target
                )
            })?;
        let has_link = module.edges.iter().any(|edge| {
            edge.from == node.name
                && edge.to == contract.target
                && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
        });
        if !has_link {
            return Err(format!(
                "project contract node `{}` requires dep/xfer edge into `{}`",
                node.name, contract.target
            ));
        }

        match contract.kind {
            ProjectContractKind::AbiGraphSummary => {
                verify_abi_graph_summary_text(node.name.as_str(), value, target)?;
            }
            ProjectContractKind::AbiSelectionSummary => {
                verify_abi_selection_summary_text(node.name.as_str(), value, target)?;
            }
            ProjectContractKind::DataPayloadClass | ProjectContractKind::ShaderPacketClass => {
                require_prefixed_contract_value(node.name.as_str(), value, "PayloadClass")?;
            }
            ProjectContractKind::DataPayloadShape | ProjectContractKind::ShaderPacketShape => {
                require_prefixed_contract_value(node.name.as_str(), value, "PayloadShape")?;
            }
            ProjectContractKind::DataHandleTableSchema | ProjectContractKind::ShaderPacketType => {}
            ProjectContractKind::BridgeStageContract => {
                verify_bridge_stage_contract_text(node.name.as_str(), value)?;
            }
            ProjectContractKind::BridgePayloadContract(direction) => {
                verify_bridge_payload_contract_text(
                    &nodes,
                    node.name.as_str(),
                    value,
                    target,
                    direction,
                )?;
            }
            ProjectContractKind::KernelSlotContract => {
                verify_kernel_slot_contract_text(node.name.as_str(), value, target)?;
            }
            ProjectContractKind::KernelTargetContract => {
                verify_target_contract_text(node.name.as_str(), value, target, "kernel")?;
            }
            ProjectContractKind::KernelAbiSelectionContract => {
                verify_abi_selection_contract_text(node.name.as_str(), value, target, "kernel")?;
            }
            ProjectContractKind::ShaderTargetContract => {
                verify_target_contract_text(node.name.as_str(), value, target, "shader")?;
            }
            ProjectContractKind::ShaderAbiSelectionContract => {
                verify_abi_selection_contract_text(node.name.as_str(), value, target, "shader")?;
            }
            ProjectContractKind::NetworkTargetContract => {
                verify_target_contract_text(node.name.as_str(), value, target, "network")?;
            }
            ProjectContractKind::NetworkAbiSelectionContract => {
                verify_abi_selection_contract_text(node.name.as_str(), value, target, "network")?;
            }
        }
    }

    Ok(())
}

fn verify_lowering_contract_nodes(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        if node.op.module != "cpu" || node.op.instruction != "text" {
            continue;
        }
        if node.name != "lowering_cpu_target_contract_type" {
            continue;
        }
        let value = node
            .op
            .args
            .first()
            .map(|value| value.trim())
            .ok_or_else(|| {
                format!(
                    "lowering contract node `{}` must carry a canonical text payload",
                    node.name
                )
            })?;
        if value.is_empty() {
            return Err(format!(
                "lowering contract node `{}` must carry a non-empty canonical text payload",
                node.name
            ));
        }
        let target_name = "lowering_cpu_target_config";
        let target = nodes.get(target_name).copied().ok_or_else(|| {
            format!(
                "lowering contract node `{}` references unknown target `{target_name}`",
                node.name
            )
        })?;
        let has_link = module.edges.iter().any(|edge| {
            edge.from == node.name
                && edge.to == target_name
                && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
        });
        if !has_link {
            return Err(format!(
                "lowering contract node `{}` requires dep/xfer edge into `{target_name}`",
                node.name
            ));
        }
        verify_cpu_target_contract_text(node.name.as_str(), value, target)?;
    }

    Ok(())
}

fn verify_scheduler_contract_nodes(
    module: &YirModule,
    resources: &BTreeMap<String, &Resource>,
    nodes: &BTreeMap<String, &Node>,
) -> Result<(), String> {
    for node in &module.nodes {
        if node.op.module != "cpu" || node.op.instruction != "text" {
            continue;
        }
        let Some(contract) = classify_scheduler_contract_node(node.name.as_str()) else {
            continue;
        };
        let value = node
            .op
            .args
            .first()
            .map(|value| value.trim())
            .ok_or_else(|| {
                format!(
                    "scheduler contract node `{}` must carry a canonical text payload",
                    node.name
                )
            })?;
        if value.is_empty() {
            return Err(format!(
                "scheduler contract node `{}` must carry a non-empty canonical text payload",
                node.name
            ));
        }
        let targets = module
            .edges
            .iter()
            .filter(|edge| {
                edge.from == node.name
                    && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
            })
            .map(|edge| edge.to.as_str())
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return Err(format!(
                "scheduler contract node `{}` requires at least one dep/xfer edge into its domain anchor",
                node.name
            ));
        }
        for target_name in &targets {
            let target = nodes.get(*target_name).copied().ok_or_else(|| {
                format!(
                    "scheduler contract node `{}` references unknown target `{}`",
                    node.name, target_name
                )
            })?;
            let target_resource = resources.get(&target.resource).copied().ok_or_else(|| {
                format!(
                    "scheduler contract node `{}` references target `{}` with unknown resource `{}`",
                    node.name, target.name, target.resource
                )
            })?;
            if target_resource.kind.family() != contract.family {
                return Err(format!(
                    "scheduler contract node `{}` is declared for `{}`, but targets `{}` on `{}`",
                    node.name,
                    contract.family,
                    target.name,
                    target_resource.kind.family()
                ));
            }
        }

        match contract.kind {
            SchedulerContractKind::LanePolicy => {
                verify_scheduler_lane_contract_text(node.name.as_str(), contract.family, value)?
            }
            SchedulerContractKind::LaneCapability => {
                verify_scheduler_lane_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::BridgeCapability => {
                verify_scheduler_bridge_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::Clock => {
                verify_scheduler_clock_contract_text(node.name.as_str(), contract.family, value)?
            }
            SchedulerContractKind::ResultLane => verify_scheduler_result_lane_contract_text(
                nodes,
                node.name.as_str(),
                contract.family,
                value,
            )?,
            SchedulerContractKind::ResultCapability => {
                verify_scheduler_result_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverRoleVariant => {
                verify_scheduler_observer_role_variant_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::SummaryCapability => {
                verify_scheduler_summary_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::SummaryClass => verify_scheduler_summary_class_contract_text(
                nodes,
                node.name.as_str(),
                contract.family,
                value,
            )?,
            SchedulerContractKind::ObserverSourceClass => {
                verify_scheduler_observer_source_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverStageClass => {
                verify_scheduler_observer_stage_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverScopeClass => {
                verify_scheduler_observer_scope_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverBranchClass => {
                verify_scheduler_observer_branch_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum SchedulerContractKind {
    LanePolicy,
    LaneCapability,
    BridgeCapability,
    Clock,
    ResultLane,
    ResultCapability,
    ObserverRoleVariant,
    SummaryCapability,
    SummaryClass,
    ObserverSourceClass,
    ObserverStageClass,
    ObserverScopeClass,
    ObserverBranchClass,
}

struct SchedulerContract<'a> {
    family: &'a str,
    kind: SchedulerContractKind,
}

fn classify_scheduler_contract_node(name: &str) -> Option<SchedulerContract<'_>> {
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_lane_policy_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::LanePolicy,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_lane_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::LaneCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_bridge_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::BridgeCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_clock_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::Clock,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_result_lane_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ResultLane,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_result_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ResultCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_role_variant_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverRoleVariant,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_summary_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::SummaryCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_summary_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::SummaryClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_source_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverSourceClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_stage_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverStageClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_scope_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverScopeClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_branch_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverBranchClass,
        });
    }
    None
}

fn verify_scheduler_lane_contract_text(
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler lane contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lanes = fields
        .get("lanes")
        .ok_or_else(|| format!("scheduler contract node `{node_name}` is missing `lanes` field"))?;
    let defaults = fields.get("defaults").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `defaults` field")
    })?;
    let parsed_lanes = lanes
        .split(',')
        .map(str::trim)
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    if parsed_lanes.is_empty() {
        return Err(format!(
            "scheduler contract node `{node_name}` requires at least one declared lane"
        ));
    }
    let mut lanes_from_defaults = BTreeSet::<&str>::new();
    for entry in defaults.split('|') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((pattern, lane)) = entry.split_once('=') else {
            return Err(format!(
                "scheduler contract node `{node_name}` has invalid default lane entry `{entry}`"
            ));
        };
        let pattern = pattern.trim();
        let lane = lane.trim();
        if pattern.is_empty() || lane.is_empty() {
            return Err(format!(
                "scheduler contract node `{node_name}` has invalid default lane entry `{entry}`"
            ));
        }
        if !parsed_lanes.contains(lane) {
            return Err(format!(
                "scheduler contract node `{node_name}` declares default lane `{lane}` outside `{lanes}`"
            ));
        }
        lanes_from_defaults.insert(lane);
    }
    if lanes_from_defaults != parsed_lanes {
        return Err(format!(
            "scheduler contract node `{node_name}` declares lanes `{lanes}` but defaults cover `{}`",
            lanes_from_defaults.into_iter().collect::<Vec<_>>().join(",")
        ));
    }
    Ok(())
}

fn verify_scheduler_clock_contract_text(
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler clock contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    for key in ["domain", "kind", "epoch", "resolution", "bridge"] {
        let value = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if value.trim().is_empty() {
            return Err(format!(
                "scheduler contract node `{node_name}` requires non-empty `{key}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_lane_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler lane capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lane_policy_name = format!("scheduler_contract_{family}_lane_policy_type");
    let lane_policy_node = nodes.get(lane_policy_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling lane policy node `{lane_policy_name}`"
        )
    })?;
    let lane_policy_value = lane_policy_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{lane_policy_name}` must carry a canonical text payload"
            )
        })?;
    let lane_policy_fields = parse_semicolon_kv_contract(
        lane_policy_name.as_str(),
        lane_policy_value,
        "scheduler lane contract",
    )?;
    let declared_lanes = lane_policy_fields
        .get("lanes")
        .ok_or_else(|| {
            format!("scheduler contract node `{lane_policy_name}` is missing `lanes` field")
        })?
        .split(',')
        .map(str::trim)
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    let declared_lane_list = declared_lanes.iter().copied().collect::<Vec<_>>().join(",");
    let capability_lanes = fields
        .iter()
        .filter_map(|(key, value)| (*key != "family").then_some((*key, *value)))
        .collect::<BTreeMap<_, _>>();
    if capability_lanes.is_empty() {
        return Err(format!(
            "scheduler contract node `{node_name}` requires at least one lane capability entry"
        ));
    }
    for lane in &declared_lanes {
        let capability = capability_lanes.get(lane).ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` is missing capability for declared lane `{lane}`"
            )
        })?;
        if capability.trim().is_empty() {
            return Err(format!(
                "scheduler contract node `{node_name}` requires non-empty capability for lane `{lane}`"
            ));
        }
    }
    for lane in capability_lanes.keys() {
        if !declared_lanes.contains(*lane) {
            return Err(format!(
                "scheduler contract node `{node_name}` declares capability for lane `{lane}` outside `{declared_lane_list}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_bridge_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler bridge capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lane_bridge = fields.get("lane_bridge").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `lane_bridge` field")
    })?;
    let clock_bridge = fields.get("clock_bridge").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `clock_bridge` field")
    })?;
    if lane_bridge.trim().is_empty() || clock_bridge.trim().is_empty() {
        return Err(format!(
            "scheduler contract node `{node_name}` requires non-empty `lane_bridge` and `clock_bridge`"
        ));
    }
    if family == "cpu" && *lane_bridge != "cpu_bind_core_lane:host_main_lane|worker_lane" {
        return Err(format!(
            "scheduler contract node `{node_name}` currently expects CPU lane bridge `cpu_bind_core_lane:host_main_lane|worker_lane`, got `{lane_bridge}`"
        ));
    }
    if family != "cpu" && *lane_bridge != "none" {
        return Err(format!(
            "scheduler contract node `{node_name}` currently expects non-CPU lane bridge `none`, got `{lane_bridge}`"
        ));
    }
    let clock_contract_name = format!("scheduler_contract_{family}_clock_type");
    let clock_contract_node = nodes.get(clock_contract_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling clock node `{clock_contract_name}`"
        )
    })?;
    let clock_contract_value = clock_contract_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{clock_contract_name}` must carry a canonical text payload"
            )
        })?;
    let clock_contract_fields = parse_semicolon_kv_contract(
        clock_contract_name.as_str(),
        clock_contract_value,
        "scheduler clock contract",
    )?;
    let declared_clock_bridge = clock_contract_fields.get("bridge").ok_or_else(|| {
        format!("scheduler contract node `{clock_contract_name}` is missing `bridge` field")
    })?;
    if *declared_clock_bridge != *clock_bridge {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `clock_bridge={clock_bridge}`, but `{clock_contract_name}` uses `{declared_clock_bridge}`"
        ));
    }
    Ok(())
}

fn verify_scheduler_result_lane_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler result lane contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let lane_policy_name = format!("scheduler_contract_{family}_lane_policy_type");
    let lane_policy_node = nodes.get(lane_policy_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling lane policy node `{lane_policy_name}`"
        )
    })?;
    let lane_policy_value = lane_policy_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{lane_policy_name}` must carry a canonical text payload"
            )
        })?;
    let lane_policy_fields = parse_semicolon_kv_contract(
        lane_policy_name.as_str(),
        lane_policy_value,
        "scheduler lane contract",
    )?;
    let declared_lanes = lane_policy_fields
        .get("lanes")
        .ok_or_else(|| {
            format!("scheduler contract node `{lane_policy_name}` is missing `lanes` field")
        })?
        .split(',')
        .map(str::trim)
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    let declared_lane_list = declared_lanes.iter().copied().collect::<Vec<_>>().join(",");
    for key in ["entry", "probe", "value"] {
        let lane = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if !declared_lanes.contains(*lane) {
            return Err(format!(
                "scheduler contract node `{node_name}` declares result lane `{lane}` for `{key}` outside `{declared_lane_list}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_result_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler result capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let result_lane_name = format!("scheduler_contract_{family}_result_lane_type");
    let result_lane_node = nodes.get(result_lane_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling result lane node `{result_lane_name}`"
        )
    })?;
    let result_lane_value = result_lane_node
        .op
        .args
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{result_lane_name}` must carry a canonical text payload"
            )
        })?;
    let result_lane_fields = parse_semicolon_kv_contract(
        result_lane_name.as_str(),
        result_lane_value,
        "scheduler result lane contract",
    )?;
    for key in ["entry", "probe", "value"] {
        if !result_lane_fields.contains_key(key) {
            return Err(format!(
                "scheduler contract node `{result_lane_name}` is missing `{key}` field"
            ));
        }
        let capability = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        let expected = match key {
            "entry" => "result-entry",
            "probe" => "result-ready-probe",
            "value" => "result-payload-value",
            _ => unreachable!(),
        };
        if *capability != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={capability}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_observer_role_variant_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer role variant contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let result_capability_name = format!("scheduler_contract_{family}_result_capability_type");
    let _result_capability_node = nodes
        .get(result_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling result capability node `{result_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("config_ready", "config-ready-observer"),
        ("send_ready", "send-ready-observer"),
        ("recv_ready", "recv-ready-observer"),
        ("connect_ready", "connect-ready-observer"),
        ("accept_ready", "accept-ready-observer"),
        ("closed", "closed-observer"),
    ] {
        let variant = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *variant != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={variant}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_summary_capability_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler summary capability contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let result_capability_name = format!("scheduler_contract_{family}_result_capability_type");
    let _result_capability_node = nodes
        .get(result_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling result capability node `{result_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("policy", "async-policy-summary"),
        ("batch", "async-batch-summary"),
        ("windowed", "async-windowed-summary"),
    ] {
        let capability = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *capability != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={capability}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_summary_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "scheduler summary class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let summary_capability_name = format!("scheduler_contract_{family}_summary_capability_type");
    let _summary_capability_node = nodes
        .get(summary_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling summary capability node `{summary_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("transport_split", "transport-split-summary"),
        (
            "transport_windowed_split",
            "transport-windowed-split-summary",
        ),
        (
            "transport_session_bridge_split",
            "transport-session-bridge-split-summary",
        ),
        ("control_split", "control-split-summary"),
        ("control_windowed", "control-windowed-summary"),
        ("control_session_bridge", "control-session-bridge-summary"),
    ] {
        let summary_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *summary_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={summary_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_observer_source_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer source class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let summary_capability_name = format!("scheduler_contract_{family}_summary_capability_type");
    let _summary_capability_node = nodes
        .get(summary_capability_name.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "scheduler contract node `{node_name}` requires sibling summary capability node `{summary_capability_name}`"
            )
        })?;
    for (key, expected) in [
        ("profile", "profile-backed"),
        ("result", "result-backed"),
        ("summary", "summary-backed"),
    ] {
        let source_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *source_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={source_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_observer_stage_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer stage class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let source_class_name = format!("scheduler_contract_{family}_observer_source_class_type");
    let _source_class_node = nodes.get(source_class_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling observer source class node `{source_class_name}`"
        )
    })?;
    for (key, expected) in [
        ("entry", "observer-entry-stage"),
        ("ready", "observer-ready-stage"),
        ("payload", "observer-payload-stage"),
        ("policy", "observer-policy-stage"),
        ("batch", "observer-batch-stage"),
        ("windowed", "observer-windowed-stage"),
    ] {
        let stage_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *stage_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={stage_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_observer_scope_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer scope class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let stage_class_name = format!("scheduler_contract_{family}_observer_stage_class_type");
    let _stage_class_node = nodes.get(stage_class_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling observer stage class node `{stage_class_name}`"
        )
    })?;
    for (key, expected) in [
        ("local", "local-scope"),
        ("cross_lane", "cross-lane-scope"),
        ("cross_domain", "cross-domain-scope"),
        ("bridge_visible", "bridge-visible-scope"),
    ] {
        let scope_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *scope_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={scope_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn verify_scheduler_observer_branch_class_contract_text(
    nodes: &BTreeMap<String, &Node>,
    node_name: &str,
    family: &str,
    value: &str,
) -> Result<(), String> {
    let fields =
        parse_semicolon_kv_contract(node_name, value, "scheduler observer branch class contract")?;
    let declared_family = fields.get("family").ok_or_else(|| {
        format!("scheduler contract node `{node_name}` is missing `family` field")
    })?;
    if *declared_family != family {
        return Err(format!(
            "scheduler contract node `{node_name}` declares `family={declared_family}`, expected `{family}`"
        ));
    }
    let scope_class_name = format!("scheduler_contract_{family}_observer_scope_class_type");
    let _scope_class_node = nodes.get(scope_class_name.as_str()).copied().ok_or_else(|| {
        format!(
            "scheduler contract node `{node_name}` requires sibling observer scope class node `{scope_class_name}`"
        )
    })?;
    for (key, expected) in [
        ("primary", "primary-branch"),
        ("secondary", "secondary-branch"),
        ("fallback", "fallback-branch"),
        ("send", "send-branch"),
        ("recv", "recv-branch"),
    ] {
        let branch_class = fields.get(key).ok_or_else(|| {
            format!("scheduler contract node `{node_name}` is missing `{key}` field")
        })?;
        if *branch_class != expected {
            return Err(format!(
                "scheduler contract node `{node_name}` declares `{key}={branch_class}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum ProjectContractKind {
    AbiGraphSummary,
    AbiSelectionSummary,
    DataPayloadClass,
    DataPayloadShape,
    DataHandleTableSchema,
    ShaderPacketType,
    ShaderPacketClass,
    ShaderPacketShape,
    BridgeStageContract,
    BridgePayloadContract(BridgePayloadDirection),
    KernelSlotContract,
    KernelTargetContract,
    KernelAbiSelectionContract,
    ShaderTargetContract,
    ShaderAbiSelectionContract,
    NetworkTargetContract,
    NetworkAbiSelectionContract,
}

#[derive(Clone, Copy)]
enum BridgePayloadDirection {
    Uplink,
    Downlink,
}

struct ProjectContract<'a> {
    kind: ProjectContractKind,
    target: String,
    _unit: &'a str,
}

fn classify_project_contract_node(name: &str) -> Option<ProjectContract<'_>> {
    if name == "project_abi_graph_summary_type" {
        return Some(ProjectContract {
            kind: ProjectContractKind::AbiGraphSummary,
            target: "project_abi_graph_summary_entry".to_owned(),
            _unit: "graph",
        });
    }
    if let Some(domain) = name
        .strip_prefix("project_abi_")
        .and_then(|suffix| suffix.strip_suffix("_selection_summary_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::AbiSelectionSummary,
            target: format!("project_abi_{domain}_selection_entry"),
            _unit: domain,
        });
    }
    if let Some(id) = name
        .strip_prefix("project_link_")
        .and_then(|suffix| suffix.strip_suffix("_bridge_stage_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::BridgeStageContract,
            target: project_link_bridge_contract_target(id, true),
            _unit: id,
        });
    }
    if let Some(id) = name
        .strip_prefix("project_link_")
        .and_then(|suffix| suffix.strip_suffix("_uplink_bridge_payload_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::BridgePayloadContract(BridgePayloadDirection::Uplink),
            target: project_link_bridge_payload_contract_target(id, BridgePayloadDirection::Uplink),
            _unit: id,
        });
    }
    if let Some(id) = name
        .strip_prefix("project_link_")
        .and_then(|suffix| suffix.strip_suffix("_downlink_bridge_payload_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::BridgePayloadContract(BridgePayloadDirection::Downlink),
            target: project_link_bridge_payload_contract_target(
                id,
                BridgePayloadDirection::Downlink,
            ),
            _unit: id,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_uplink_payload_class_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadClass,
            target: format!("project_profile_data_{unit}_uplink_payload_class"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_uplink_payload_shape_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadShape,
            target: format!("project_profile_data_{unit}_uplink_payload_shape"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_downlink_payload_class_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadClass,
            target: format!("project_profile_data_{unit}_downlink_payload_class"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_downlink_payload_shape_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadShape,
            target: format!("project_profile_data_{unit}_downlink_payload_shape"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_handle_table_schema_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataHandleTableSchema,
            target: format!("project_profile_data_{unit}_profile_handles"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_packet_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderPacketType,
            target: format!("project_profile_shader_{unit}_packet_field_count"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_packet_class_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderPacketClass,
            target: format!("project_profile_shader_{unit}_packet_field_count"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_packet_shape_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderPacketShape,
            target: format!("project_profile_shader_{unit}_packet_field_count"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_kernel_")
        .and_then(|suffix| suffix.strip_suffix("_slot_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::KernelSlotContract,
            target: format!("project_profile_kernel_{unit}_profile_entry"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_kernel_")
        .and_then(|suffix| suffix.strip_suffix("_target_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::KernelTargetContract,
            target: format!("project_profile_kernel_{unit}_kernel_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_kernel_")
        .and_then(|suffix| suffix.strip_suffix("_abi_selection_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::KernelAbiSelectionContract,
            target: format!("project_profile_kernel_{unit}_kernel_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_target_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderTargetContract,
            target: format!("project_profile_shader_{unit}_shader_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_abi_selection_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderAbiSelectionContract,
            target: format!("project_profile_shader_{unit}_shader_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_network_")
        .and_then(|suffix| suffix.strip_suffix("_target_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::NetworkTargetContract,
            target: format!("project_profile_network_{unit}_network_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_network_")
        .and_then(|suffix| suffix.strip_suffix("_abi_selection_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::NetworkAbiSelectionContract,
            target: format!("project_profile_network_{unit}_network_target_config_auto"),
            _unit: unit,
        });
    }
    None
}

fn project_link_bridge_contract_target(id: &str, stage: bool) -> String {
    let marker = if stage {
        "uplink_window_policy"
    } else {
        "uplink_payload_shape"
    };
    if let Some(data_unit) = id.split("_via_data_").nth(1) {
        return format!("project_profile_data_{data_unit}_{marker}");
    }
    format!("project_link_{id}_missing_target")
}

fn project_link_bridge_payload_contract_target(
    id: &str,
    direction: BridgePayloadDirection,
) -> String {
    let marker = match direction {
        BridgePayloadDirection::Uplink => "uplink_payload_shape",
        BridgePayloadDirection::Downlink => "downlink_payload_shape",
    };
    if let Some(data_unit) = id.split("_via_data_").nth(1) {
        return format!("project_profile_data_{data_unit}_{marker}");
    }
    format!("project_link_{id}_missing_target")
}

fn require_prefixed_contract_value(
    node_name: &str,
    value: &str,
    prefix: &str,
) -> Result<(), String> {
    if !value.starts_with(prefix) {
        return Err(format!(
            "project contract node `{node_name}` must use `{prefix}...`, got `{value}`"
        ));
    }
    Ok(())
}

fn verify_bridge_stage_contract_text(node_name: &str, value: &str) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "bridge stage")?;
    let uplink = fields.get("uplink").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing bridge `uplink` stage")
    })?;
    let downlink = fields.get("downlink").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing bridge `downlink` stage")
    })?;
    if *uplink != "windowed" || *downlink != "windowed" {
        return Err(format!(
            "project contract node `{node_name}` currently expects `uplink=windowed;downlink=windowed`, got `{value}`"
        ));
    }
    Ok(())
}

fn verify_bridge_payload_contract_text(
    nodes: &BTreeMap<&str, &Node>,
    node_name: &str,
    value: &str,
    target: &Node,
    direction: BridgePayloadDirection,
) -> Result<(), String> {
    if value.is_empty() || value == "unknown" {
        return Err(format!(
            "project contract node `{node_name}` requires non-empty bridge payload, got `{value}`"
        ));
    }
    if !value.starts_with("Window<") {
        return Err(format!(
            "project contract node `{node_name}` currently expects bridge payload to be `Window<...>`, got `{value}`"
        ));
    }
    let expected_shape = payload_shape_contract_for_bridge_payload(value).ok_or_else(|| {
        format!("project contract node `{node_name}` could not derive payload shape from `{value}`")
    })?;
    let target_shape_node_name = format!("{}_type", target.name);
    let target_shape_node = nodes
        .get(target_shape_node_name.as_str())
        .copied()
        .unwrap_or(target);
    let target_shape = target_shape_node
        .op
        .args
        .first()
        .map(|value| value.as_str())
        .ok_or_else(|| {
            format!(
                "project contract node `{node_name}` targets `{}` without payload shape text",
                target_shape_node.name
            )
        })?;
    if target_shape != expected_shape {
        let direction = match direction {
            BridgePayloadDirection::Uplink => "uplink",
            BridgePayloadDirection::Downlink => "downlink",
        };
        return Err(format!(
            "project contract node `{node_name}` has {direction} bridge payload `{value}` requiring `{expected_shape}`, but target `{}` encodes `{target_shape}`",
            target_shape_node.name
        ));
    }
    Ok(())
}

fn payload_shape_contract_for_bridge_payload(value: &str) -> Option<String> {
    let normalized = value.replace(['<', '>'], "");
    Some(format!(
        "PayloadShape{}",
        sanitize_contract_type_fragment(&normalized)
    ))
}

fn sanitize_contract_type_fragment(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

fn parse_semicolon_kv_contract<'a>(
    node_name: &str,
    value: &'a str,
    label: &str,
) -> Result<BTreeMap<&'a str, &'a str>, String> {
    value
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| {
            let (key, raw) = entry.split_once('=').ok_or_else(|| {
                format!("project contract node `{node_name}` has invalid {label} field `{entry}`")
            })?;
            Ok((key.trim(), raw.trim()))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()
}

fn verify_kernel_slot_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    let fields = value
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| {
            let (key, raw) = entry.split_once('=').ok_or_else(|| {
                format!(
                    "project contract node `{node_name}` has invalid kernel slot field `{entry}`"
                )
            })?;
            let raw = raw.strip_prefix("i64:").ok_or_else(|| {
                format!(
                    "project contract node `{node_name}` expects i64-encoded kernel slot value in `{entry}`"
                )
            })?;
            let parsed = raw.parse::<i64>().map_err(|_| {
                format!(
                    "project contract node `{node_name}` has non-integer kernel slot value `{entry}`"
                )
            })?;
            Ok((key.trim(), parsed))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;

    let bind_core = *fields.get("bind_core").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `bind_core` field")
    })?;
    let queue_depth = *fields.get("queue_depth").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `queue_depth` field")
    })?;
    let batch_lanes = *fields.get("batch_lanes").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `batch_lanes` field")
    })?;

    if bind_core < 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `bind_core >= 0`, got `{bind_core}`"
        ));
    }
    if queue_depth <= 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `queue_depth > 0`, got `{queue_depth}`"
        ));
    }
    if batch_lanes <= 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `batch_lanes > 0`, got `{batch_lanes}`"
        ));
    }

    let target_batch_lanes = target
        .op
        .args
        .last()
        .ok_or_else(|| {
            format!(
                "project contract node `{node_name}` references kernel profile `{}` without target_config args",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "project contract node `{node_name}` references kernel profile `{}` with non-integer batch lanes",
                target.name
            )
        })?;

    if target_batch_lanes != batch_lanes {
        return Err(format!(
            "project contract node `{node_name}` encodes `batch_lanes={batch_lanes}`, but `{}` uses `{target_batch_lanes}`",
            target.name
        ));
    }

    Ok(())
}

fn verify_target_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
    domain: &str,
) -> Result<(), String> {
    if target.op.module != domain || target.op.instruction != "target_config" {
        return Err(format!(
            "project contract node `{node_name}` must target `{domain}.target_config`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let fields = parse_semicolon_kv_contract(node_name, value, "target contract")?;
    let arch = fields
        .get("arch")
        .copied()
        .ok_or_else(|| format!("project contract node `{node_name}` is missing `arch` field"))?;
    let runtime = fields
        .get("runtime")
        .copied()
        .ok_or_else(|| format!("project contract node `{node_name}` is missing `runtime` field"))?;
    let lane_width = fields.get("lane_width").copied().ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `lane_width` field")
    })?;
    let arch = arch
        .strip_prefix("symbol:")
        .ok_or_else(|| format!("project contract node `{node_name}` expects `arch=symbol:...`"))?;
    let runtime = runtime.strip_prefix("symbol:").ok_or_else(|| {
        format!("project contract node `{node_name}` expects `runtime=symbol:...`")
    })?;
    let lane_width = lane_width.strip_prefix("i64:").ok_or_else(|| {
        format!("project contract node `{node_name}` expects `lane_width=i64:...`")
    })?;
    let lane_width = lane_width
        .parse::<i64>()
        .map_err(|_| format!("project contract node `{node_name}` has non-integer lane_width"))?;
    if lane_width <= 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `lane_width > 0`, got `{lane_width}`"
        ));
    }
    let target_arch = target.op.args.first().map(String::as_str).ok_or_else(|| {
        format!(
            "project contract node `{node_name}` references `{}` without arch arg",
            target.name
        )
    })?;
    let target_runtime = target.op.args.get(1).map(String::as_str).ok_or_else(|| {
        format!(
            "project contract node `{node_name}` references `{}` without runtime arg",
            target.name
        )
    })?;
    let target_lane_width = target
        .op
        .args
        .get(2)
        .ok_or_else(|| {
            format!(
                "project contract node `{node_name}` references `{}` without lane width arg",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "project contract node `{node_name}` references `{}` with non-integer lane width",
                target.name
            )
        })?;
    if target_arch != arch {
        return Err(format!(
            "project contract node `{node_name}` encodes `arch={arch}`, but `{}` uses `{target_arch}`",
            target.name
        ));
    }
    if target_runtime != runtime {
        return Err(format!(
            "project contract node `{node_name}` encodes `runtime={runtime}`, but `{}` uses `{target_runtime}`",
            target.name
        ));
    }
    if target_lane_width != lane_width {
        return Err(format!(
            "project contract node `{node_name}` encodes `lane_width={lane_width}`, but `{}` uses `{target_lane_width}`",
            target.name
        ));
    }
    Ok(())
}

fn verify_cpu_target_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    if target.op.module != "cpu" || target.op.instruction != "target_config" {
        return Err(format!(
            "lowering contract node `{node_name}` must target `cpu.target_config`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let fields = parse_semicolon_kv_contract(node_name, value, "cpu target contract")?;
    let arch = fields
        .get("arch")
        .copied()
        .ok_or_else(|| format!("lowering contract node `{node_name}` is missing `arch` field"))?;
    let abi = fields
        .get("abi")
        .copied()
        .ok_or_else(|| format!("lowering contract node `{node_name}` is missing `abi` field"))?;
    let vector_bits = fields.get("vector_bits").copied().ok_or_else(|| {
        format!("lowering contract node `{node_name}` is missing `vector_bits` field")
    })?;
    let arch = arch
        .strip_prefix("symbol:")
        .ok_or_else(|| format!("lowering contract node `{node_name}` expects `arch=symbol:...`"))?;
    let abi = abi
        .strip_prefix("symbol:")
        .ok_or_else(|| format!("lowering contract node `{node_name}` expects `abi=symbol:...`"))?;
    let vector_bits = vector_bits.strip_prefix("i64:").ok_or_else(|| {
        format!("lowering contract node `{node_name}` expects `vector_bits=i64:...`")
    })?;
    let vector_bits = vector_bits
        .parse::<i64>()
        .map_err(|_| format!("lowering contract node `{node_name}` has non-integer vector_bits"))?;
    if vector_bits <= 0 {
        return Err(format!(
            "lowering contract node `{node_name}` requires `vector_bits > 0`, got `{vector_bits}`"
        ));
    }
    let target_arch = target.op.args.first().map(String::as_str).ok_or_else(|| {
        format!(
            "lowering contract node `{node_name}` references `{}` without arch arg",
            target.name
        )
    })?;
    let target_abi = target.op.args.get(1).map(String::as_str).ok_or_else(|| {
        format!(
            "lowering contract node `{node_name}` references `{}` without abi arg",
            target.name
        )
    })?;
    let target_vector_bits = target
        .op
        .args
        .get(2)
        .ok_or_else(|| {
            format!(
                "lowering contract node `{node_name}` references `{}` without vector_bits arg",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "lowering contract node `{node_name}` references `{}` with non-integer vector_bits",
                target.name
            )
        })?;
    if target_arch != arch {
        return Err(format!(
            "lowering contract node `{node_name}` encodes `arch={arch}`, but `{}` uses `{target_arch}`",
            target.name
        ));
    }
    if target_abi != abi {
        return Err(format!(
            "lowering contract node `{node_name}` encodes `abi={abi}`, but `{}` uses `{target_abi}`",
            target.name
        ));
    }
    if target_vector_bits != vector_bits {
        return Err(format!(
            "lowering contract node `{node_name}` encodes `vector_bits={vector_bits}`, but `{}` uses `{target_vector_bits}`",
            target.name
        ));
    }
    Ok(())
}

fn verify_abi_selection_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
    domain: &str,
) -> Result<(), String> {
    if target.op.module != domain || target.op.instruction != "target_config" {
        return Err(format!(
            "project ABI selection contract node `{node_name}` must target `{domain}.target_config`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let fields = parse_semicolon_kv_contract(node_name, value, "ABI selection contract")?;
    let mode = fields.get("mode").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `mode` field")
    })?;
    let abi = fields.get("abi").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `abi` field")
    })?;
    let arch = fields.get("arch").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `arch` field")
    })?;
    let runtime = fields.get("runtime").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `runtime` field")
    })?;
    let lane_width = fields.get("lane_width").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `lane_width` field")
    })?;
    let mode = mode.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `mode=symbol:...`")
    })?;
    if mode != "explicit" && mode != "auto" {
        return Err(format!(
            "project ABI selection contract node `{node_name}` requires `mode` to be `explicit` or `auto`, got `{mode}`"
        ));
    }
    let abi = abi.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `abi=symbol:...`")
    })?;
    if abi.is_empty() {
        return Err(format!(
            "project ABI selection contract node `{node_name}` requires non-empty `abi`"
        ));
    }
    let arch = arch.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `arch=symbol:...`")
    })?;
    let runtime = runtime.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `runtime=symbol:...`")
    })?;
    let lane_width = lane_width.strip_prefix("i64:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `lane_width=i64:...`")
    })?;
    let lane_width = lane_width.parse::<i64>().map_err(|_| {
        format!("project ABI selection contract node `{node_name}` has non-integer lane_width")
    })?;
    if lane_width <= 0 {
        return Err(format!(
            "project ABI selection contract node `{node_name}` requires `lane_width > 0`, got `{lane_width}`"
        ));
    }
    let target_arch = target.op.args.first().map(String::as_str).ok_or_else(|| {
        format!(
            "project ABI selection contract node `{node_name}` references `{}` without arch arg",
            target.name
        )
    })?;
    let target_runtime = target.op.args.get(1).map(String::as_str).ok_or_else(|| {
        format!(
            "project ABI selection contract node `{node_name}` references `{}` without runtime arg",
            target.name
        )
    })?;
    let target_lane_width = target
        .op
        .args
        .get(2)
        .ok_or_else(|| {
            format!(
                "project ABI selection contract node `{node_name}` references `{}` without lane width arg",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "project ABI selection contract node `{node_name}` references `{}` with non-integer lane width",
                target.name
            )
        })?;
    if target_arch != arch {
        return Err(format!(
            "project ABI selection contract node `{node_name}` encodes `arch={arch}`, but `{}` uses `{target_arch}`",
            target.name
        ));
    }
    if target_runtime != runtime {
        return Err(format!(
            "project ABI selection contract node `{node_name}` encodes `runtime={runtime}`, but `{}` uses `{target_runtime}`",
            target.name
        ));
    }
    if target_lane_width != lane_width {
        return Err(format!(
            "project ABI selection contract node `{node_name}` encodes `lane_width={lane_width}`, but `{}` uses `{target_lane_width}`",
            target.name
        ));
    }
    Ok(())
}

fn verify_abi_selection_summary_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    if target.op.module != "cpu" || target.op.instruction != "text" {
        return Err(format!(
            "project ABI summary node `{node_name}` must target `cpu.text`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let target_value = target
        .op
        .args
        .first()
        .map(|item| item.trim())
        .ok_or_else(|| {
            format!(
                "project ABI summary node `{node_name}` references `{}` without summary payload",
                target.name
            )
        })?;
    let fields = parse_semicolon_kv_contract(node_name, value, "ABI summary contract")?;
    for key in ["mode", "abi", "arch", "os", "object", "calling", "backend"] {
        let raw = fields.get(key).copied().ok_or_else(|| {
            format!("project ABI summary node `{node_name}` is missing `{key}` field")
        })?;
        let parsed = raw.strip_prefix("symbol:").ok_or_else(|| {
            format!("project ABI summary node `{node_name}` expects `{key}=symbol:...`")
        })?;
        if parsed.is_empty() {
            return Err(format!(
                "project ABI summary node `{node_name}` requires non-empty `{key}`"
            ));
        }
    }
    let mode = fields
        .get("mode")
        .and_then(|value| value.strip_prefix("symbol:"))
        .unwrap_or_default();
    if mode != "explicit" && mode != "auto" {
        return Err(format!(
            "project ABI summary node `{node_name}` requires `mode` to be `explicit` or `auto`, got `{mode}`"
        ));
    }
    if value != target_value {
        return Err(format!(
            "project ABI summary node `{node_name}` encodes `{value}`, but `{}` uses `{target_value}`",
            target.name
        ));
    }
    Ok(())
}

fn verify_abi_graph_summary_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    if target.op.module != "cpu" || target.op.instruction != "text" {
        return Err(format!(
            "project ABI graph summary node `{node_name}` must target `cpu.text`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let target_value = target
        .op
        .args
        .first()
        .map(|item| item.trim())
        .ok_or_else(|| {
            format!(
                "project ABI graph summary node `{node_name}` references `{}` without summary payload",
                target.name
            )
        })?;
    let fields = parse_semicolon_kv_contract(node_name, value, "ABI graph summary")?;
    for key in [
        "mode",
        "domains",
        "cpu_summary",
        "data_summary",
        "kernel_target",
        "shader_target",
        "network_target",
    ] {
        let raw = fields.get(key).copied().ok_or_else(|| {
            format!("project ABI graph summary node `{node_name}` is missing `{key}` field")
        })?;
        let parsed = raw.strip_prefix("symbol:").ok_or_else(|| {
            format!("project ABI graph summary node `{node_name}` expects `{key}=symbol:...`")
        })?;
        if parsed.is_empty() {
            return Err(format!(
                "project ABI graph summary node `{node_name}` requires non-empty `{key}`"
            ));
        }
    }
    let mode = fields
        .get("mode")
        .and_then(|value| value.strip_prefix("symbol:"))
        .unwrap_or_default();
    if mode != "explicit" && mode != "auto" {
        return Err(format!(
            "project ABI graph summary node `{node_name}` requires `mode` to be `explicit` or `auto`, got `{mode}`"
        ));
    }
    if value != target_value {
        return Err(format!(
            "project ABI graph summary node `{node_name}` encodes `{value}`, but `{}` uses `{target_value}`",
            target.name
        ));
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

fn verify_cpu_heap_protocol(module: &YirModule) -> Result<(), String> {
    let order = topological_order(module)?;
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let order_index = order
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let borrow_scope_ends = infer_borrow_scope_ends(module, &order_index);

    let mut values = BTreeMap::<String, PointerState>::new();
    let mut heap = BTreeMap::<usize, HeapBinding>::new();
    let mut borrow_counts = BTreeMap::<usize, usize>::new();
    let mut borrow_owner = BTreeMap::<String, usize>::new();
    let mut next_id = 1usize;
    let mut moved_names = BTreeSet::<String>::new();

    for (current_index, node_name) in order.into_iter().enumerate() {
        let node = nodes
            .get(node_name.as_str())
            .copied()
            .ok_or_else(|| format!("verification order references unknown node `{node_name}`"))?;

        if node.op.module != "cpu" {
            continue;
        }

        for arg in &node.op.args {
            if moved_names.contains(arg) {
                return Err(format!(
                    "node `{}` uses moved pointer value `{}`",
                    node.name, arg
                ));
            }
        }

        match node.op.instruction.as_str() {
            "null" => {
                values.insert(node.name.clone(), PointerState::Null);
            }
            "alloc_node" => {
                let next = values
                    .get(&node.op.args[1])
                    .copied()
                    .unwrap_or(PointerState::Unknown);
                if let PointerState::Borrowed(next_id) = next {
                    return Err(format!(
                        "node `{}` cannot capture borrowed pointer `&{}` as linked-list next pointer",
                        node.name, next_id
                    ));
                }
                let id = next_id;
                next_id += 1;
                heap.insert(
                    id,
                    HeapBinding {
                        live: true,
                        kind: HeapObjectKind::Node { next },
                    },
                );
                values.insert(node.name.clone(), PointerState::Owned(id));
            }
            "alloc_buffer" => {
                let id = next_id;
                next_id += 1;
                let len = known_non_negative_int(&nodes, &node.op.args[0])?;
                heap.insert(
                    id,
                    HeapBinding {
                        live: true,
                        kind: HeapObjectKind::Buffer { len },
                    },
                );
                values.insert(node.name.clone(), PointerState::Owned(id));
            }
            "borrow" => {
                let source = pointer_arg(&values, &node.op.args[0]);
                match source {
                    PointerState::Owned(id) | PointerState::Borrowed(id) => {
                        ensure_live_heap(&heap, id, node)?;
                        *borrow_counts.entry(id).or_insert(0) += 1;
                        values.insert(node.name.clone(), PointerState::Borrowed(id));
                        borrow_owner.insert(node.name.clone(), id);
                    }
                    PointerState::Null => {
                        values.insert(node.name.clone(), PointerState::Null);
                    }
                    PointerState::Unknown => {
                        values.insert(node.name.clone(), PointerState::Unknown);
                    }
                }
            }
            "borrow_end" => {
                let borrow_name = &node.op.args[0];
                match pointer_arg(&values, borrow_name) {
                    PointerState::Borrowed(id) => {
                        ensure_live_heap(&heap, id, node)?;
                        release_named_borrow(borrow_name, id, &borrow_owner, &mut borrow_counts)?;
                    }
                    PointerState::Null | PointerState::Unknown => {}
                    PointerState::Owned(_) => {
                        return Err(format!(
                            "node `{}` expects borrowed pointer `{}` for cpu.borrow_end",
                            node.name, borrow_name
                        ));
                    }
                }
            }
            "move_ptr" => {
                let source_name = &node.op.args[0];
                let source = pointer_arg(&values, source_name);
                match source {
                    PointerState::Owned(id) => {
                        ensure_live_heap(&heap, id, node)?;
                        ensure_no_active_borrows(&borrow_counts, id, node, "move")?;
                        values.insert(node.name.clone(), PointerState::Owned(id));
                        moved_names.insert(source_name.clone());
                    }
                    PointerState::Borrowed(_) => {
                        return Err(format!(
                            "node `{}` cannot move borrowed pointer `{}`",
                            node.name, source_name
                        ));
                    }
                    PointerState::Null => {
                        values.insert(node.name.clone(), PointerState::Null);
                    }
                    PointerState::Unknown => {
                        values.insert(node.name.clone(), PointerState::Unknown);
                    }
                }
            }
            "load_value" => {
                ensure_node_readable(pointer_arg(&values, &node.op.args[0]), &heap, node)?;
            }
            "load_next" => {
                let pointer = pointer_arg(&values, &node.op.args[0]);
                let next = match pointer {
                    PointerState::Owned(id) | PointerState::Borrowed(id) => {
                        ensure_live_heap(&heap, id, node)?;
                        match heap.get(&id).map(|binding| binding.kind) {
                            Some(HeapObjectKind::Node { next }) => next,
                            Some(HeapObjectKind::Buffer { .. }) => {
                                return Err(format!(
                                    "node `{}` uses buffer object `&{id}` as linked-list node",
                                    node.name
                                ));
                            }
                            None => PointerState::Unknown,
                        }
                    }
                    PointerState::Null => {
                        return Err(format!("node `{}` dereferences null pointer", node.name));
                    }
                    PointerState::Unknown => PointerState::Unknown,
                };
                values.insert(node.name.clone(), next);
            }
            "buffer_len" => {
                let pointer = pointer_arg(&values, &node.op.args[0]);
                ensure_buffer_readable(pointer, &heap, node)?;
            }
            "load_at" => {
                let pointer = pointer_arg(&values, &node.op.args[0]);
                ensure_buffer_readable(pointer, &heap, node)?;
                ensure_buffer_index_in_bounds(pointer, &heap, &nodes, &node.op.args[1], node)?;
            }
            "store_value" => {
                ensure_node_writable(
                    pointer_arg(&values, &node.op.args[0]),
                    &heap,
                    &borrow_counts,
                    node,
                )?;
            }
            "store_next" => {
                let dest = pointer_arg(&values, &node.op.args[0]);
                ensure_node_writable(dest, &heap, &borrow_counts, node)?;
                let next = pointer_arg(&values, &node.op.args[1]);
                if let PointerState::Borrowed(next_id) = next {
                    return Err(format!(
                        "node `{}` cannot write borrowed pointer `&{}` into linked-list next field",
                        node.name, next_id
                    ));
                }
                if let PointerState::Owned(id) = dest {
                    if let Some(binding) = heap.get_mut(&id) {
                        match &mut binding.kind {
                            HeapObjectKind::Node { next: binding_next } => {
                                *binding_next = next;
                            }
                            HeapObjectKind::Buffer { .. } => {
                                return Err(format!(
                                    "node `{}` uses buffer object `&{id}` as linked-list node",
                                    node.name
                                ));
                            }
                        }
                    }
                }
            }
            "store_at" => {
                ensure_buffer_writable(
                    pointer_arg(&values, &node.op.args[0]),
                    &heap,
                    &borrow_counts,
                    node,
                )?;
                ensure_buffer_index_in_bounds(
                    pointer_arg(&values, &node.op.args[0]),
                    &heap,
                    &nodes,
                    &node.op.args[1],
                    node,
                )?;
            }
            "free" => {
                let source_name = &node.op.args[0];
                match pointer_arg(&values, source_name) {
                    PointerState::Owned(id) => {
                        ensure_live_heap(&heap, id, node)?;
                        ensure_no_active_borrows(&borrow_counts, id, node, "free")?;
                        ensure_no_live_heap_aliases(&heap, id, node)?;
                        if let Some(binding) = heap.get_mut(&id) {
                            binding.live = false;
                        }
                        moved_names.insert(source_name.clone());
                    }
                    PointerState::Borrowed(_) => {
                        return Err(format!(
                            "node `{}` cannot free borrowed pointer `{}`",
                            node.name, source_name
                        ));
                    }
                    PointerState::Null => {
                        return Err(format!("node `{}` cannot free null pointer", node.name));
                    }
                    PointerState::Unknown => {}
                }
            }
            _ => {}
        }

        release_completed_borrows(
            current_index,
            &borrow_scope_ends,
            &borrow_owner,
            &mut borrow_counts,
        );
    }

    Ok(())
}

fn verify_result_state_nodes(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        match node.op.semantic_op() {
            SemanticOp::DataObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                if !node.op.observe_state_matches_source(&source.op, actual)? {
                    return Err(format!(
                        "node `{}` observes data state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::DataValue => {
                require_observe_source(&nodes, node, SemanticOp::DataObserve)?;
            }
            SemanticOp::ShaderObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                if !node.op.observe_state_matches_source(&source.op, actual)? {
                    return Err(format!(
                        "node `{}` observes shader state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::ShaderIsPassReady
            | SemanticOp::ShaderIsFrameReady
            | SemanticOp::ShaderValue => {
                require_observe_source(&nodes, node, SemanticOp::ShaderObserve)?;
            }
            SemanticOp::KernelObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                let direct_project_ref =
                    source.op.semantic_op() == SemanticOp::CpuProjectProfileRef;
                let resolved_kernel_profile_slot = is_resolved_kernel_profile_slot(source);
                let direct_kernel_scalar_source = is_direct_kernel_scalar_source(source);
                if !direct_project_ref
                    && !resolved_kernel_profile_slot
                    && !direct_kernel_scalar_source
                {
                    return Err(format!(
                        "node `{}` expects cpu.project_profile_ref or direct kernel scalar input for kernel observe, got `{}`",
                        node.name,
                        source.op.full_name()
                    ));
                }
                let state_matches = if resolved_kernel_profile_slot || direct_kernel_scalar_source {
                    actual == "config_ready"
                } else {
                    node.op.observe_state_matches_source(&source.op, actual)?
                };
                if !state_matches {
                    return Err(format!(
                        "node `{}` observes kernel state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::KernelIsConfigReady | SemanticOp::KernelValue => {
                require_observe_source(&nodes, node, SemanticOp::KernelObserve)?;
            }
            SemanticOp::NetworkObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                let direct_project_ref =
                    source.op.semantic_op() == SemanticOp::CpuProjectProfileRef;
                let resolved_network_profile_slot = is_resolved_network_profile_slot(source);
                let host_network_transport_probe = is_host_network_transport_probe_source(source);
                if !direct_project_ref
                    && !resolved_network_profile_slot
                    && !host_network_transport_probe
                {
                    return Err(format!(
                        "node `{}` expects cpu.project_profile_ref or host network transport probe input for network observe, got `{}`",
                        node.name,
                        source.op.full_name()
                    ));
                }
                let state_matches = if resolved_network_profile_slot {
                    actual == "config_ready"
                } else if host_network_transport_probe {
                    match source.op.args[1].as_str() {
                        "host_network_send_probe" => actual == "send_ready",
                        "host_network_send_owned" => actual == "send_ready",
                        "host_network_accept_probe" => actual == "accept_ready",
                        "host_network_accept_owned" => actual == "accept_ready",
                        "host_network_recv_probe" => actual == "recv_ready",
                        "host_network_recv_owned" => actual == "recv_ready",
                        "host_network_recv_http_status_owned" => actual == "recv_ready",
                        "host_network_close" => actual == "closed",
                        _ => false,
                    }
                } else {
                    node.op.observe_state_matches_source(&source.op, actual)?
                };
                if !state_matches {
                    return Err(format!(
                        "node `{}` observes network state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::NetworkIsConfigReady => {
                require_observe_source(&nodes, node, SemanticOp::NetworkObserve)?;
            }
            SemanticOp::NetworkIsSendReady => {
                require_observe_source(&nodes, node, SemanticOp::NetworkObserve)?;
            }
            SemanticOp::NetworkIsRecvReady => {
                require_observe_source(&nodes, node, SemanticOp::NetworkObserve)?;
            }
            _ if node.op.result_source_semantic_op().is_some() => {
                require_expected_result_source(&nodes, node)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn is_resolved_kernel_profile_slot(node: &Node) -> bool {
    node.name.starts_with("project_profile_kernel_")
        && node.op.module == "cpu"
        && node.op.instruction == "const_i64"
}

fn is_resolved_network_profile_slot(node: &Node) -> bool {
    node.name.starts_with("project_profile_network_")
        && node.op.module == "network"
        && node.op.instruction == "const_i64"
}

fn is_host_network_transport_probe_source(node: &Node) -> bool {
    node.op.module == "cpu"
        && node.op.instruction == "extern_call_i64"
        && node.op.args.len() >= 2
        && matches!(
            node.op.args[1].as_str(),
            "host_network_accept_probe"
                | "host_network_accept_owned"
                | "host_network_send_probe"
                | "host_network_send_owned"
                | "host_network_recv_probe"
                | "host_network_recv_owned"
                | "host_network_recv_http_status_owned"
                | "host_network_close"
        )
}

fn is_direct_kernel_scalar_source(node: &Node) -> bool {
    node.op.module == "kernel"
        && matches!(
            node.op.instruction.as_str(),
            "reduce_sum" | "reduce_max" | "reduce_mean" | "argmax" | "argmin"
        )
}

fn require_expected_result_source(
    nodes: &BTreeMap<&str, &Node>,
    node: &Node,
) -> Result<(), String> {
    if node.op.semantic_op() == SemanticOp::NetworkValue {
        let source = node
            .op
            .args
            .first()
            .ok_or_else(|| format!("node `{}` is missing result source arg", node.name))
            .and_then(|name| {
                nodes.get(name.as_str()).copied().ok_or_else(|| {
                    format!(
                        "node `{}` references unknown result source `{name}`",
                        node.name
                    )
                })
            })?;
        let actual = source.op.semantic_op();
        if matches!(
            actual,
            SemanticOp::NetworkObserve
                | SemanticOp::NetworkConnect
                | SemanticOp::NetworkAccept
                | SemanticOp::NetworkClose
        ) {
            return Ok(());
        }
        return Err(format!(
            "node `{}` expects one of `network.observe`, `network.connect`, `network.accept`, or `network.close`, got `{}`",
            node.name,
            source.op.full_name()
        ));
    }
    let expected = node.op.result_source_semantic_op().ok_or_else(|| {
        format!(
            "node `{}` has no expected result source contract",
            node.name
        )
    })?;
    require_observe_source(nodes, node, expected)
}

fn observe_source_node<'a>(
    nodes: &'a BTreeMap<&str, &'a Node>,
    node: &Node,
) -> Result<&'a Node, String> {
    let source_name = node
        .op
        .args
        .first()
        .ok_or_else(|| format!("node `{}` is missing observe source arg", node.name))?;
    nodes.get(source_name.as_str()).copied().ok_or_else(|| {
        format!(
            "node `{}` references unknown observe source `{source_name}`",
            node.name
        )
    })
}

fn observe_state_arg<'a>(node: &'a Node) -> Result<&'a str, String> {
    node.op
        .args
        .get(1)
        .map(|value| value.as_str())
        .ok_or_else(|| format!("node `{}` is missing observe state arg", node.name))
}

fn require_observe_source(
    nodes: &BTreeMap<&str, &Node>,
    node: &Node,
    expected: SemanticOp,
) -> Result<(), String> {
    let source = node
        .op
        .args
        .first()
        .ok_or_else(|| format!("node `{}` is missing result source arg", node.name))
        .and_then(|name| {
            nodes.get(name.as_str()).copied().ok_or_else(|| {
                format!(
                    "node `{}` references unknown result source `{name}`",
                    node.name
                )
            })
        })?;
    if source.op.semantic_op() != expected {
        return Err(format!(
            "node `{}` expects `{}` input, got `{}`",
            node.name,
            semantic_op_name(expected),
            source.op.full_name()
        ));
    }
    Ok(())
}

fn semantic_op_name(op: SemanticOp) -> &'static str {
    match op {
        SemanticOp::CpuProjectProfileRef => "cpu.project_profile_ref",
        SemanticOp::CpuJoinResult => "cpu.join_result",
        SemanticOp::CpuTaskCompleted => "cpu.task_completed",
        SemanticOp::CpuTaskTimedOut => "cpu.task_timed_out",
        SemanticOp::CpuTaskCancelled => "cpu.task_cancelled",
        SemanticOp::CpuTaskValue => "cpu.task_value",
        SemanticOp::DataObserve => "data.observe",
        SemanticOp::DataIsReady => "data.is_ready",
        SemanticOp::DataIsMoved => "data.is_moved",
        SemanticOp::DataIsWindowed => "data.is_windowed",
        SemanticOp::DataValue => "data.value",
        SemanticOp::ShaderObserve => "shader.observe",
        SemanticOp::ShaderIsPassReady => "shader.is_pass_ready",
        SemanticOp::ShaderIsFrameReady => "shader.is_frame_ready",
        SemanticOp::ShaderValue => "shader.value",
        SemanticOp::KernelObserve => "kernel.observe",
        SemanticOp::KernelIsConfigReady => "kernel.is_config_ready",
        SemanticOp::KernelValue => "kernel.value",
        SemanticOp::NetworkObserve => "network.observe",
        SemanticOp::NetworkConnect => "network.connect",
        SemanticOp::NetworkAccept => "network.accept",
        SemanticOp::NetworkClose => "network.close",
        SemanticOp::NetworkIsConfigReady => "network.is_config_ready",
        SemanticOp::NetworkIsConnectReady => "network.is_connect_ready",
        SemanticOp::NetworkIsAcceptReady => "network.is_accept_ready",
        SemanticOp::NetworkIsClosed => "network.is_closed",
        SemanticOp::NetworkValue => "network.value",
        SemanticOp::DataBindCore => "data.bind_core",
        SemanticOp::DataMarker => "data.marker",
        SemanticOp::DataHandleTable => "data.handle_table",
        SemanticOp::DataOutputPipe => "data.output_pipe",
        SemanticOp::DataInputPipe => "data.input_pipe",
        SemanticOp::DataCopyWindow => "data.copy_window",
        SemanticOp::DataReadWindow => "data.read_window",
        SemanticOp::DataWriteWindow => "data.write_window",
        SemanticOp::DataImmutableWindow => "data.immutable_window",
        SemanticOp::ShaderBeginPass => "shader.begin_pass",
        SemanticOp::ShaderDrawInstanced => "shader.draw_instanced",
        _ => "other",
    }
}

fn infer_borrow_scope_ends(
    module: &YirModule,
    order_index: &BTreeMap<String, usize>,
) -> BTreeMap<String, usize> {
    let mut scope_ends = BTreeMap::<String, usize>::new();
    for node in &module.nodes {
        if node.op.instruction != "borrow" || node.op.module != "cpu" {
            continue;
        }
        let Some(start_index) = order_index.get(&node.name).copied() else {
            continue;
        };
        let mut end_index = start_index;
        for consumer in &module.nodes {
            if consumer.name == node.name {
                continue;
            }
            if consumer.op.args.iter().any(|arg| arg == &node.name) {
                if let Some(index) = order_index.get(&consumer.name).copied() {
                    end_index = end_index.max(index);
                }
            }
        }
        scope_ends.insert(node.name.clone(), end_index);
    }
    scope_ends
}

fn release_completed_borrows(
    current_index: usize,
    borrow_scope_ends: &BTreeMap<String, usize>,
    borrow_owner: &BTreeMap<String, usize>,
    borrow_counts: &mut BTreeMap<usize, usize>,
) {
    for (borrow_name, end_index) in borrow_scope_ends {
        if *end_index != current_index {
            continue;
        }
        let Some(owner_id) = borrow_owner.get(borrow_name).copied() else {
            continue;
        };
        if let Some(count) = borrow_counts.get_mut(&owner_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                borrow_counts.remove(&owner_id);
            }
        }
    }
}

fn release_named_borrow(
    borrow_name: &str,
    owner_id: usize,
    borrow_owner: &BTreeMap<String, usize>,
    borrow_counts: &mut BTreeMap<usize, usize>,
) -> Result<(), String> {
    let Some(recorded_owner) = borrow_owner.get(borrow_name).copied() else {
        return Err(format!(
            "borrow `{}` has no active owner record for release",
            borrow_name
        ));
    };
    if recorded_owner != owner_id {
        return Err(format!(
            "borrow `{}` release owner mismatch: expected `&{}`, got `&{}`",
            borrow_name, recorded_owner, owner_id
        ));
    }
    if let Some(count) = borrow_counts.get_mut(&owner_id) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            borrow_counts.remove(&owner_id);
        }
    }
    Ok(())
}

fn pointer_arg(values: &BTreeMap<String, PointerState>, name: &str) -> PointerState {
    values.get(name).copied().unwrap_or(PointerState::Unknown)
}

fn known_non_negative_int(
    nodes: &BTreeMap<&str, &Node>,
    name: &str,
) -> Result<Option<usize>, String> {
    let Some(node) = nodes.get(name).copied() else {
        return Ok(None);
    };

    if node.op.module == "cpu" && node.op.instruction == "const" {
        let value = node.op.args[0].parse::<i64>().map_err(|_| {
            format!(
                "node `{}` has invalid integer literal `{}`",
                node.name, node.op.args[0]
            )
        })?;
        if value < 0 {
            return Err(format!(
                "node `{}` uses negative integer `{}` where non-negative value is required",
                node.name, value
            ));
        }
        return Ok(Some(value as usize));
    }

    Ok(None)
}

fn ensure_buffer_index_in_bounds(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    nodes: &BTreeMap<&str, &Node>,
    index_name: &str,
    node: &Node,
) -> Result<(), String> {
    let Some(index) = known_non_negative_int(nodes, index_name)? else {
        return Ok(());
    };

    let object_id = match pointer {
        PointerState::Owned(id) | PointerState::Borrowed(id) => id,
        PointerState::Null | PointerState::Unknown => return Ok(()),
    };

    if let Some(HeapBinding {
        kind: HeapObjectKind::Buffer { len: Some(len) },
        ..
    }) = heap.get(&object_id)
    {
        if index >= *len {
            return Err(format!(
                "node `{}` indexes buffer `&{object_id}` out of bounds: index {} >= len {}",
                node.name, index, len
            ));
        }
    }

    Ok(())
}

fn ensure_live_heap(
    heap: &BTreeMap<usize, HeapBinding>,
    id: usize,
    node: &Node,
) -> Result<(), String> {
    let binding = heap.get(&id).ok_or_else(|| {
        format!(
            "node `{}` references unknown heap object `&{id}`",
            node.name
        )
    })?;
    if binding.live {
        Ok(())
    } else {
        Err(format!(
            "node `{}` dereferences freed heap object `&{id}`",
            node.name
        ))
    }
}

fn ensure_pointer_readable(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    node: &Node,
) -> Result<(), String> {
    match pointer {
        PointerState::Owned(id) | PointerState::Borrowed(id) => ensure_live_heap(heap, id, node),
        PointerState::Null => Err(format!("node `{}` dereferences null pointer", node.name)),
        PointerState::Unknown => Ok(()),
    }
}

fn ensure_pointer_writable(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    borrow_counts: &BTreeMap<usize, usize>,
    node: &Node,
) -> Result<(), String> {
    match pointer {
        PointerState::Owned(id) => {
            ensure_live_heap(heap, id, node)?;
            ensure_no_active_borrows(borrow_counts, id, node, "write")?;
            Ok(())
        }
        PointerState::Borrowed(_) => Err(format!(
            "node `{}` writes through borrowed pointer",
            node.name
        )),
        PointerState::Null => Err(format!("node `{}` writes through null pointer", node.name)),
        PointerState::Unknown => Ok(()),
    }
}

fn ensure_node_readable(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    node: &Node,
) -> Result<(), String> {
    ensure_pointer_readable(pointer, heap, node)?;
    match pointer {
        PointerState::Owned(id) | PointerState::Borrowed(id) => {
            match heap.get(&id).map(|binding| binding.kind) {
                Some(HeapObjectKind::Node { .. }) => Ok(()),
                Some(HeapObjectKind::Buffer { .. }) => Err(format!(
                    "node `{}` uses buffer object `&{id}` as linked-list node",
                    node.name
                )),
                None => Ok(()),
            }
        }
        PointerState::Null | PointerState::Unknown => Ok(()),
    }
}

fn ensure_node_writable(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    borrow_counts: &BTreeMap<usize, usize>,
    node: &Node,
) -> Result<(), String> {
    ensure_pointer_writable(pointer, heap, borrow_counts, node)?;
    match pointer {
        PointerState::Owned(id) => match heap.get(&id).map(|binding| binding.kind) {
            Some(HeapObjectKind::Node { .. }) => Ok(()),
            Some(HeapObjectKind::Buffer { .. }) => Err(format!(
                "node `{}` uses buffer object `&{id}` as linked-list node",
                node.name
            )),
            None => Ok(()),
        },
        PointerState::Borrowed(_) | PointerState::Null | PointerState::Unknown => Ok(()),
    }
}

fn ensure_buffer_readable(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    node: &Node,
) -> Result<(), String> {
    match pointer {
        PointerState::Owned(id) | PointerState::Borrowed(id) => {
            ensure_live_heap(heap, id, node)?;
            match heap.get(&id).map(|binding| binding.kind) {
                Some(HeapObjectKind::Buffer { .. }) => Ok(()),
                Some(HeapObjectKind::Node { .. }) => Err(format!(
                    "node `{}` uses linked-list node `&{id}` as buffer",
                    node.name
                )),
                None => Ok(()),
            }
        }
        PointerState::Null => Err(format!("node `{}` dereferences null pointer", node.name)),
        PointerState::Unknown => Ok(()),
    }
}

fn ensure_buffer_writable(
    pointer: PointerState,
    heap: &BTreeMap<usize, HeapBinding>,
    borrow_counts: &BTreeMap<usize, usize>,
    node: &Node,
) -> Result<(), String> {
    match pointer {
        PointerState::Owned(id) => {
            ensure_live_heap(heap, id, node)?;
            ensure_no_active_borrows(borrow_counts, id, node, "write")?;
            match heap.get(&id).map(|binding| binding.kind) {
                Some(HeapObjectKind::Buffer { .. }) => Ok(()),
                Some(HeapObjectKind::Node { .. }) => Err(format!(
                    "node `{}` uses linked-list node `&{id}` as buffer",
                    node.name
                )),
                None => Ok(()),
            }
        }
        PointerState::Borrowed(id) => match heap.get(&id).map(|binding| binding.kind) {
            Some(HeapObjectKind::Buffer { .. }) => Err(format!(
                "node `{}` writes through borrowed pointer",
                node.name
            )),
            Some(HeapObjectKind::Node { .. }) => Err(format!(
                "node `{}` uses linked-list node `&{id}` as buffer",
                node.name
            )),
            None => Err(format!(
                "node `{}` writes through borrowed pointer",
                node.name
            )),
        },
        PointerState::Null => Err(format!("node `{}` writes through null pointer", node.name)),
        PointerState::Unknown => Ok(()),
    }
}

fn ensure_no_active_borrows(
    borrow_counts: &BTreeMap<usize, usize>,
    id: usize,
    node: &Node,
    action: &str,
) -> Result<(), String> {
    let active = borrow_counts.get(&id).copied().unwrap_or(0);
    if active == 0 {
        return Ok(());
    }

    Err(format!(
        "node `{}` cannot {} `&{}` while {} borrow(s) are active",
        node.name, action, id, active
    ))
}

fn ensure_no_live_heap_aliases(
    heap: &BTreeMap<usize, HeapBinding>,
    id: usize,
    node: &Node,
) -> Result<(), String> {
    for (owner_id, binding) in heap {
        if !binding.live || *owner_id == id {
            continue;
        }
        let HeapObjectKind::Node { next } = binding.kind else {
            continue;
        };
        if matches!(next, PointerState::Owned(next_id) | PointerState::Borrowed(next_id) if next_id == id)
        {
            return Err(format!(
                "node `{}` cannot free `&{}` while live node `&{}` still links to it",
                node.name, id, owner_id
            ));
        }
    }

    Ok(())
}

fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        *indegree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut order = Vec::with_capacity(module.nodes.len());

    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort();
                    }
                }
            }
        }
    }

    if order.len() != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(order)
}

fn path_exists(module: &YirModule, from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }

    let mut adjacency = BTreeMap::<&str, Vec<&str>>::new();
    for edge in &module.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    let mut stack = vec![from];
    let mut visited = BTreeSet::<&str>::new();

    while let Some(current) = stack.pop() {
        if !visited.insert(current) {
            continue;
        }
        if let Some(targets) = adjacency.get(current) {
            for target in targets {
                if *target == to {
                    return true;
                }
                stack.push(target);
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::verify_module;
    use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

    fn node(name: &str, resource: &str, op: &str, args: &[&str]) -> Node {
        Node {
            name: name.to_owned(),
            resource: resource.to_owned(),
            op: Operation::parse(op, args.iter().map(|item| (*item).to_owned()).collect()).unwrap(),
        }
    }

    fn dep(from: &str, to: &str) -> Edge {
        Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    fn effect(from: &str, to: &str) -> Edge {
        Edge {
            kind: EdgeKind::Effect,
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    fn lifetime(from: &str, to: &str) -> Edge {
        Edge {
            kind: EdgeKind::Lifetime,
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    fn xfer(from: &str, to: &str) -> Edge {
        Edge {
            kind: EdgeKind::CrossDomainExchange,
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    #[test]
    fn owner_write_after_last_borrow_use_is_allowed() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("nil", "cpu0", "cpu.null", &[]),
                node("v1", "cpu0", "cpu.const", &["10"]),
                node("v2", "cpu0", "cpu.const", &["99"]),
                node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "nil"]),
                node("head", "cpu0", "cpu.move_ptr", &["head_raw"]),
                node("head_ref", "cpu0", "cpu.borrow", &["head"]),
                node("read_head", "cpu0", "cpu.load_value", &["head_ref"]),
                node("write_head", "cpu0", "cpu.store_value", &["head", "v2"]),
            ],
            edges: vec![
                dep("v1", "head_raw"),
                dep("nil", "head_raw"),
                dep("head_raw", "head"),
                lifetime("head_raw", "head"),
                dep("head", "head_ref"),
                dep("head_ref", "read_head"),
                effect("head_ref", "read_head"),
                dep("head", "write_head"),
                dep("v2", "write_head"),
                effect("read_head", "write_head"),
                lifetime("head", "write_head"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn owner_free_after_last_borrow_use_is_allowed() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("len", "cpu0", "cpu.const", &["4"]),
                node("fill", "cpu0", "cpu.const", &["7"]),
                node("idx1", "cpu0", "cpu.const", &["1"]),
                node("buf_raw", "cpu0", "cpu.alloc_buffer", &["len", "fill"]),
                node("buf", "cpu0", "cpu.move_ptr", &["buf_raw"]),
                node("buf_ref", "cpu0", "cpu.borrow", &["buf"]),
                node("read_slot", "cpu0", "cpu.load_at", &["buf_ref", "idx1"]),
                node("drop_buf", "cpu0", "cpu.free", &["buf"]),
            ],
            edges: vec![
                dep("len", "buf_raw"),
                dep("fill", "buf_raw"),
                dep("buf_raw", "buf"),
                lifetime("buf_raw", "buf"),
                dep("buf", "buf_ref"),
                dep("buf_ref", "read_slot"),
                dep("idx1", "read_slot"),
                effect("buf_ref", "read_slot"),
                dep("buf", "drop_buf"),
                effect("read_slot", "drop_buf"),
                lifetime("buf", "drop_buf"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn explicit_borrow_end_allows_owner_write() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("nil", "cpu0", "cpu.null", &[]),
                node("v1", "cpu0", "cpu.const", &["10"]),
                node("v2", "cpu0", "cpu.const", &["99"]),
                node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "nil"]),
                node("head", "cpu0", "cpu.move_ptr", &["head_raw"]),
                node("head_ref", "cpu0", "cpu.borrow", &["head"]),
                node("end_ref", "cpu0", "cpu.borrow_end", &["head_ref"]),
                node("write_head", "cpu0", "cpu.store_value", &["head", "v2"]),
            ],
            edges: vec![
                dep("v1", "head_raw"),
                dep("nil", "head_raw"),
                dep("head_raw", "head"),
                lifetime("head_raw", "head"),
                dep("head", "head_ref"),
                dep("head_ref", "end_ref"),
                effect("head_ref", "end_ref"),
                dep("head", "write_head"),
                dep("v2", "write_head"),
                effect("end_ref", "write_head"),
                lifetime("head", "write_head"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn alloc_node_with_borrowed_next_is_rejected() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("nil", "cpu0", "cpu.null", &[]),
                node("v1", "cpu0", "cpu.const", &["10"]),
                node("v2", "cpu0", "cpu.const", &["20"]),
                node("tail_raw", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
                node("tail", "cpu0", "cpu.move_ptr", &["tail_raw"]),
                node("tail_ref", "cpu0", "cpu.borrow", &["tail"]),
                node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "tail_ref"]),
            ],
            edges: vec![
                dep("v2", "tail_raw"),
                dep("nil", "tail_raw"),
                dep("tail_raw", "tail"),
                lifetime("tail_raw", "tail"),
                dep("tail", "tail_ref"),
                dep("v1", "head_raw"),
                dep("tail_ref", "head_raw"),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("cannot capture borrowed pointer"));
    }

    #[test]
    fn store_next_with_borrowed_pointer_is_rejected() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("nil", "cpu0", "cpu.null", &[]),
                node("v1", "cpu0", "cpu.const", &["10"]),
                node("v2", "cpu0", "cpu.const", &["20"]),
                node("tail_raw", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
                node("tail", "cpu0", "cpu.move_ptr", &["tail_raw"]),
                node("head_raw", "cpu0", "cpu.alloc_node", &["v1", "nil"]),
                node("head", "cpu0", "cpu.move_ptr", &["head_raw"]),
                node("tail_ref", "cpu0", "cpu.borrow", &["tail"]),
                node("link_tail", "cpu0", "cpu.store_next", &["head", "tail_ref"]),
            ],
            edges: vec![
                dep("v2", "tail_raw"),
                dep("nil", "tail_raw"),
                dep("tail_raw", "tail"),
                lifetime("tail_raw", "tail"),
                dep("v1", "head_raw"),
                dep("nil", "head_raw"),
                dep("head_raw", "head"),
                lifetime("head_raw", "head"),
                dep("tail", "tail_ref"),
                dep("head", "link_tail"),
                dep("tail_ref", "link_tail"),
                lifetime("head", "link_tail"),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("cannot write borrowed pointer"));
    }

    #[test]
    fn freeing_live_link_target_is_rejected() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("nil", "cpu0", "cpu.null", &[]),
                node("v2", "cpu0", "cpu.const", &["20"]),
                node("v1", "cpu0", "cpu.const", &["10"]),
                node("tail", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
                node("head", "cpu0", "cpu.alloc_node", &["v1", "tail"]),
                node("drop_tail", "cpu0", "cpu.free", &["tail"]),
            ],
            edges: vec![
                dep("v2", "tail"),
                dep("nil", "tail"),
                dep("v1", "head"),
                dep("tail", "head"),
                dep("tail", "drop_tail"),
                effect("head", "drop_tail"),
                lifetime("tail", "drop_tail"),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("still links to it"));
    }

    #[test]
    fn freeing_detached_link_target_is_allowed() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("nil", "cpu0", "cpu.null", &[]),
                node("v2", "cpu0", "cpu.const", &["20"]),
                node("v1", "cpu0", "cpu.const", &["10"]),
                node("tail", "cpu0", "cpu.alloc_node", &["v2", "nil"]),
                node("head", "cpu0", "cpu.alloc_node", &["v1", "tail"]),
                node("detach_tail", "cpu0", "cpu.store_next", &["head", "nil"]),
                node("drop_tail", "cpu0", "cpu.free", &["tail"]),
            ],
            edges: vec![
                dep("v2", "tail"),
                dep("nil", "tail"),
                dep("v1", "head"),
                dep("tail", "head"),
                dep("head", "detach_tail"),
                dep("nil", "detach_tail"),
                effect("head", "detach_tail"),
                lifetime("head", "detach_tail"),
                dep("tail", "drop_tail"),
                effect("detach_tail", "drop_tail"),
                lifetime("tail", "drop_tail"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_validate_data_shader_and_kernel_links() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.apple"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_data_FabricPlane_uplink_payload_class_type",
                    "cpu0",
                    "cpu.text",
                    &["PayloadClassWindow"],
                ),
                node(
                    "project_profile_data_FabricPlane_uplink_payload_class",
                    "fabric0",
                    "data.marker",
                    &["uplink_payload_class"],
                ),
                node(
                    "project_profile_data_FabricPlane_handle_table_schema_type",
                    "cpu0",
                    "cpu.text",
                    &["FabricPlaneBindings"],
                ),
                node(
                    "project_profile_data_FabricPlane_profile_handles",
                    "fabric0",
                    "data.handle_table",
                    &["color=shader0"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_packet_type",
                    "cpu0",
                    "cpu.text",
                    &["SurfaceShaderPacket"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_packet_class_type",
                    "cpu0",
                    "cpu.text",
                    &["PayloadClassValue"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_packet_shape_type",
                    "cpu0",
                    "cpu.text",
                    &["PayloadShapeSurfaceShaderPacket"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_packet_field_count",
                    "cpu0",
                    "cpu.const_i64",
                    &["3"],
                ),
                node(
                    "project_profile_kernel_KernelUnit_slot_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["bind_core=i64:2;queue_depth=i64:8;batch_lanes=i64:16"],
                ),
                node(
                    "project_profile_kernel_KernelUnit_profile_entry",
                    "kernel0",
                    "kernel.target_config",
                    &["apple_ane", "coreml", "16"],
                ),
            ],
            edges: vec![
                xfer(
                    "project_profile_data_FabricPlane_uplink_payload_class_type",
                    "project_profile_data_FabricPlane_uplink_payload_class",
                ),
                xfer(
                    "project_profile_data_FabricPlane_handle_table_schema_type",
                    "project_profile_data_FabricPlane_profile_handles",
                ),
                dep(
                    "project_profile_shader_SurfaceShader_packet_type",
                    "project_profile_shader_SurfaceShader_packet_field_count",
                ),
                dep(
                    "project_profile_shader_SurfaceShader_packet_class_type",
                    "project_profile_shader_SurfaceShader_packet_field_count",
                ),
                dep(
                    "project_profile_shader_SurfaceShader_packet_shape_type",
                    "project_profile_shader_SurfaceShader_packet_field_count",
                ),
                xfer(
                    "project_profile_kernel_KernelUnit_slot_contract_type",
                    "project_profile_kernel_KernelUnit_profile_entry",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_require_contract_edge() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_data_FabricPlane_uplink_payload_shape_type",
                    "cpu0",
                    "cpu.text",
                    &["PayloadShapeWindowFrame"],
                ),
                node(
                    "project_profile_data_FabricPlane_uplink_payload_shape",
                    "fabric0",
                    "data.marker",
                    &["uplink_payload_shape"],
                ),
            ],
            edges: vec![],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("requires dep/xfer edge"));
    }

    #[test]
    fn project_contract_nodes_reject_kernel_slot_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.apple"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_kernel_KernelUnit_slot_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["bind_core=i64:2;queue_depth=i64:8;batch_lanes=i64:12"],
                ),
                node(
                    "project_profile_kernel_KernelUnit_profile_entry",
                    "kernel0",
                    "kernel.target_config",
                    &["apple_ane", "coreml", "16"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_kernel_KernelUnit_slot_contract_type",
                "project_profile_kernel_KernelUnit_profile_entry",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("encodes `batch_lanes=12`"));
    }

    #[test]
    fn project_contract_nodes_validate_kernel_target_config() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.apple"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_kernel_KernelUnit_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:apple_ane;runtime=symbol:coreml;lane_width=i64:1"],
                ),
                node(
                    "project_profile_kernel_KernelUnit_kernel_target_config_auto",
                    "kernel0",
                    "kernel.target_config",
                    &["apple_ane", "coreml", "1"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_kernel_KernelUnit_target_contract_type",
                "project_profile_kernel_KernelUnit_kernel_target_config_auto",
            )],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_reject_kernel_target_runtime_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.apple"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_kernel_KernelUnit_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:apple_ane;runtime=symbol:mlx;lane_width=i64:1"],
                ),
                node(
                    "project_profile_kernel_KernelUnit_kernel_target_config_auto",
                    "kernel0",
                    "kernel.target_config",
                    &["apple_ane", "coreml", "1"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_kernel_KernelUnit_target_contract_type",
                "project_profile_kernel_KernelUnit_kernel_target_config_auto",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("encodes `runtime=mlx`"));
    }

    #[test]
    fn project_contract_nodes_validate_shader_and_network_target_configs() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
                Resource {
                    name: "network0".to_owned(),
                    kind: ResourceKind::parse("network.urlsession"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_shader_SurfaceShader_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_shader_target_config_auto",
                    "shader0",
                    "shader.target_config",
                    &["arm64", "metal", "1"],
                ),
                node(
                    "project_profile_network_HttpLink_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:arm64;runtime=symbol:urlsession;lane_width=i64:1"],
                ),
                node(
                    "project_profile_network_HttpLink_network_target_config_auto",
                    "network0",
                    "network.target_config",
                    &["arm64", "urlsession", "1"],
                ),
            ],
            edges: vec![
                xfer(
                    "project_profile_shader_SurfaceShader_target_contract_type",
                    "project_profile_shader_SurfaceShader_shader_target_config_auto",
                ),
                xfer(
                    "project_profile_network_HttpLink_target_contract_type",
                    "project_profile_network_HttpLink_network_target_config_auto",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_validate_shader_abi_selection_contract() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_shader_SurfaceShader_abi_selection_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["mode=symbol:explicit;abi=symbol:shader.metal.msl2_4;arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_shader_target_config_auto",
                    "shader0",
                    "shader.target_config",
                    &["arm64", "metal", "1"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_shader_SurfaceShader_abi_selection_contract_type",
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
            )],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_validate_project_cpu_abi_summary() {
        let payload = "mode=symbol:explicit;abi=symbol:cpu.arm64.apple_aapcs64;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:none";
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node(
                    "project_abi_cpu_selection_summary_type",
                    "cpu0",
                    "cpu.text",
                    &[payload],
                ),
                node(
                    "project_abi_cpu_selection_entry",
                    "cpu0",
                    "cpu.text",
                    &[payload],
                ),
            ],
            edges: vec![dep(
                "project_abi_cpu_selection_summary_type",
                "project_abi_cpu_selection_entry",
            )],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_validate_project_abi_graph_summary() {
        let payload = "mode=symbol:explicit;domains=symbol:cpu,data;cpu_summary=symbol:present;data_summary=symbol:present;kernel_target=symbol:absent;shader_target=symbol:absent;network_target=symbol:absent";
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node(
                    "project_abi_graph_summary_type",
                    "cpu0",
                    "cpu.text",
                    &[payload],
                ),
                node(
                    "project_abi_graph_summary_entry",
                    "cpu0",
                    "cpu.text",
                    &[payload],
                ),
            ],
            edges: vec![dep(
                "project_abi_graph_summary_type",
                "project_abi_graph_summary_entry",
            )],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn project_contract_nodes_reject_project_data_abi_summary_invalid_mode() {
        let bad = "mode=symbol:recommended;abi=symbol:data.fabric.host-match.v1;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:none";
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node(
                    "project_abi_data_selection_summary_type",
                    "cpu0",
                    "cpu.text",
                    &[bad],
                ),
                node(
                    "project_abi_data_selection_entry",
                    "cpu0",
                    "cpu.text",
                    &[bad],
                ),
            ],
            edges: vec![dep(
                "project_abi_data_selection_summary_type",
                "project_abi_data_selection_entry",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("requires `mode` to be `explicit` or `auto`"));
    }

    #[test]
    fn project_contract_nodes_reject_shader_abi_selection_mode_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_shader_SurfaceShader_abi_selection_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["mode=symbol:recommended;abi=symbol:shader.metal.msl2_4;arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_shader_target_config_auto",
                    "shader0",
                    "shader.target_config",
                    &["arm64", "metal", "1"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_shader_SurfaceShader_abi_selection_contract_type",
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("requires `mode` to be `explicit` or `auto`"));
    }

    #[test]
    fn project_contract_nodes_reject_shader_target_runtime_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_shader_SurfaceShader_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:arm64;runtime=symbol:vulkan;lane_width=i64:1"],
                ),
                node(
                    "project_profile_shader_SurfaceShader_shader_target_config_auto",
                    "shader0",
                    "shader.target_config",
                    &["arm64", "metal", "1"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_shader_SurfaceShader_target_contract_type",
                "project_profile_shader_SurfaceShader_shader_target_config_auto",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("encodes `runtime=vulkan`"));
    }

    #[test]
    fn project_contract_nodes_reject_network_target_lane_width_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "network0".to_owned(),
                    kind: ResourceKind::parse("network.urlsession"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_network_HttpLink_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:arm64;runtime=symbol:urlsession;lane_width=i64:4"],
                ),
                node(
                    "project_profile_network_HttpLink_network_target_config_auto",
                    "network0",
                    "network.target_config",
                    &["arm64", "urlsession", "1"],
                ),
            ],
            edges: vec![xfer(
                "project_profile_network_HttpLink_target_contract_type",
                "project_profile_network_HttpLink_network_target_config_auto",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("encodes `lane_width=4`"));
    }

    #[test]
    fn lowering_contract_nodes_validate_cpu_target_config() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.x86_64"),
            }],
            nodes: vec![
                node(
                    "lowering_cpu_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:x86_64;abi=symbol:cpu.x86_64.sysv64;vector_bits=i64:128"],
                ),
                node(
                    "lowering_cpu_target_config",
                    "cpu0",
                    "cpu.target_config",
                    &["x86_64", "cpu.x86_64.sysv64", "128"],
                ),
            ],
            edges: vec![dep(
                "lowering_cpu_target_contract_type",
                "lowering_cpu_target_config",
            )],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn lowering_contract_nodes_reject_cpu_target_vector_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.x86_64"),
            }],
            nodes: vec![
                node(
                    "lowering_cpu_target_contract_type",
                    "cpu0",
                    "cpu.text",
                    &["arch=symbol:x86_64;abi=symbol:cpu.x86_64.sysv64;vector_bits=i64:256"],
                ),
                node(
                    "lowering_cpu_target_config",
                    "cpu0",
                    "cpu.target_config",
                    &["x86_64", "cpu.x86_64.sysv64", "128"],
                ),
            ],
            edges: vec![dep(
                "lowering_cpu_target_contract_type",
                "lowering_cpu_target_config",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("encodes `vector_bits=256`"));
    }

    #[test]
    fn scheduler_contract_nodes_validate_lane_and_clock_registration() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_lane_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;render=render-pass;setup=render-setup"#],
                ),
                node(
                    "scheduler_contract_shader_bridge_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;lane_bridge=none;clock_bridge=global->frame:bridge"#],
                ),
                node(
                    "scheduler_contract_shader_clock_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;domain=shader.clock.frame.v1;kind=frame-monotonic;epoch=frame-epoch;resolution=render-pass-step;bridge=global->frame:bridge"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_role_variant_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;config_ready=config-ready-observer;send_ready=send-ready-observer;recv_ready=recv-ready-observer;connect_ready=connect-ready-observer;accept_ready=accept-ready-observer;closed=closed-observer"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_source_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;profile=profile-backed;result=result-backed;summary=summary-backed"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=observer-entry-stage;ready=observer-ready-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_scope_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;local=local-scope;cross_lane=cross-lane-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_lane_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_bridge_capability_type",
                    "shader_target",
                ),
                dep("scheduler_contract_shader_clock_type", "shader_target"),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_role_variant_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_source_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_scope_class_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn scheduler_contract_nodes_reject_lane_defaults_outside_declared_set() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.apple"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_kernel_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=kernel;lanes=compute;defaults=kernel.tensor=compute|kernel.print=main"#,
                    ],
                ),
                node(
                    "kernel_entry",
                    "kernel0",
                    "kernel.target_config",
                    &["apple_ane", "coreml", "16"],
                ),
            ],
            edges: vec![dep(
                "scheduler_contract_kernel_lane_policy_type",
                "kernel_entry",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("declares default lane `main` outside"));
    }

    #[test]
    fn scheduler_contract_nodes_reject_result_lane_outside_declared_set() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=render;value=main"#],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("declares result lane `main`"));
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_result_capability_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-state-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `result-ready-probe`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_observer_role_variant_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_role_variant_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;config_ready=config-ready-observer;send_ready=send-ready-observer;recv_ready=recv-observer;connect_ready=connect-ready-observer;accept_ready=accept-ready-observer;closed=closed-observer"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_role_variant_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `recv-ready-observer`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_summary_capability_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-fan-in-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `async-batch-summary`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_summary_class_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;transport_split=transport-summary;transport_windowed_split=transport-windowed-split-summary;transport_session_bridge_split=transport-session-bridge-split-summary;control_split=control-split-summary;control_windowed=control-windowed-summary;control_session_bridge=control-session-bridge-summary"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_class_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(
            error.contains("expected `transport-split-summary`"),
            "{error}"
        );
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_observer_source_class_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_source_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;profile=profile-source;result=result-backed;summary=summary-backed"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_source_class_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `profile-backed`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_observer_stage_class_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_source_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;profile=profile-backed;result=result-backed;summary=summary-backed"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=observer-entry-stage;ready=observer-state-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_source_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `observer-ready-stage`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_observer_scope_class_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_source_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;profile=profile-backed;result=result-backed;summary=summary-backed"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=observer-entry-stage;ready=observer-ready-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_scope_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;local=local-scope;cross_lane=lane-crossing-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_branch_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;primary=primary-branch;secondary=secondary-branch;fallback=fallback-branch;send=send-branch;recv=recv-branch"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_source_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_scope_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_branch_class_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `cross-lane-scope`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_observer_branch_class_label() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_result_lane_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
                ),
                node(
                    "scheduler_contract_shader_result_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_summary_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_source_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;profile=profile-backed;result=result-backed;summary=summary-backed"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;entry=observer-entry-stage;ready=observer-ready-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_scope_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;local=local-scope;cross_lane=cross-lane-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_observer_branch_class_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;primary=primary-branch;secondary=secondary-branch;fallback=default-branch;send=send-branch;recv=recv-branch"#,
                    ],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_lane_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_result_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_summary_capability_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_source_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_stage_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_scope_class_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_observer_branch_class_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expected `fallback-branch`"), "{error}");
    }

    #[test]
    fn scheduler_contract_nodes_reject_lane_capability_outside_declared_set() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "shader0".to_owned(),
                    kind: ResourceKind::parse("shader.metal"),
                },
            ],
            nodes: vec![
                node(
                    "scheduler_contract_shader_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                    ],
                ),
                node(
                    "scheduler_contract_shader_lane_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=shader;render=render-pass;setup=render-setup;main=host-entry"#],
                ),
                node(
                    "shader_target",
                    "shader0",
                    "shader.target",
                    &["rgba8_unorm", "160", "120"],
                ),
            ],
            edges: vec![
                dep(
                    "scheduler_contract_shader_lane_policy_type",
                    "shader_target",
                ),
                dep(
                    "scheduler_contract_shader_lane_capability_type",
                    "shader_target",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("declares capability for lane `main`"));
    }

    #[test]
    fn scheduler_contract_nodes_reject_invalid_cpu_bridge_capability() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node(
                    "scheduler_contract_cpu_lane_policy_type",
                    "cpu0",
                    "cpu.text",
                    &[r#"family=cpu;lanes=main,mem;defaults=cpu.print=main|cpu.alloc_node=mem"#],
                ),
                node(
                    "scheduler_contract_cpu_clock_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=cpu;domain=cpu.clock.host.v1;kind=host-monotonic;epoch=host-epoch;resolution=cpu.tick_i64;bridge=global->monotonic:bridge"#,
                    ],
                ),
                node(
                    "scheduler_contract_cpu_bridge_capability_type",
                    "cpu0",
                    "cpu.text",
                    &[
                        r#"family=cpu;lane_bridge=host_main_lane;clock_bridge=global->monotonic:bridge"#,
                    ],
                ),
                node("seed", "cpu0", "cpu.const", &["7"]),
                node("cpu_entry", "cpu0", "cpu.print", &["seed"]),
            ],
            edges: vec![
                dep("scheduler_contract_cpu_lane_policy_type", "cpu_entry"),
                dep("scheduler_contract_cpu_clock_type", "cpu_entry"),
                dep("scheduler_contract_cpu_bridge_capability_type", "cpu_entry"),
                dep("seed", "cpu_entry"),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(
            error.contains("currently expects CPU lane bridge"),
            "{error}"
        );
    }

    #[test]
    fn rejects_mismatched_data_observe_state() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node("value", "cpu0", "cpu.const", &["7"]),
                node("pipe", "fabric0", "data.output_pipe", &["value"]),
                node("result", "fabric0", "data.observe", &["pipe", "ready"]),
            ],
            edges: vec![xfer("value", "pipe"), dep("pipe", "result")],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("does not support that state"));
    }

    #[test]
    fn accepts_kernel_result_observe_from_project_profile_ref() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.compute"),
                },
            ],
            nodes: vec![
                node(
                    "queue_depth",
                    "cpu0",
                    "cpu.project_profile_ref",
                    &["kernel", "KernelUnit", "queue_depth"],
                ),
                node(
                    "kernel_result",
                    "kernel0",
                    "kernel.observe",
                    &["queue_depth", "config_ready"],
                ),
                node(
                    "kernel_ready",
                    "kernel0",
                    "kernel.is_config_ready",
                    &["kernel_result"],
                ),
            ],
            edges: vec![
                xfer("queue_depth", "kernel_result"),
                dep("kernel_result", "kernel_ready"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn accepts_kernel_result_observe_from_resolved_project_profile_slot() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "kernel0".to_owned(),
                    kind: ResourceKind::parse("kernel.compute"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_kernel_KernelUnit_batch_lanes",
                    "cpu0",
                    "cpu.const_i64",
                    &["16"],
                ),
                node(
                    "kernel_result",
                    "kernel0",
                    "kernel.observe",
                    &[
                        "project_profile_kernel_KernelUnit_batch_lanes",
                        "config_ready",
                    ],
                ),
                node(
                    "kernel_ready",
                    "kernel0",
                    "kernel.is_config_ready",
                    &["kernel_result"],
                ),
            ],
            edges: vec![
                xfer(
                    "project_profile_kernel_KernelUnit_batch_lanes",
                    "kernel_result",
                ),
                dep("kernel_result", "kernel_ready"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn accepts_network_control_result_probes() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "network0".to_owned(),
                    kind: ResourceKind::parse("network.io"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_network_NetworkUnit_local_port",
                    "cpu0",
                    "cpu.const_i64",
                    &["7001"],
                ),
                node(
                    "project_profile_network_NetworkUnit_remote_port",
                    "cpu0",
                    "cpu.const_i64",
                    &["7443"],
                ),
                node(
                    "project_profile_network_NetworkUnit_connect_timeout_ms",
                    "cpu0",
                    "cpu.const_i64",
                    &["1500"],
                ),
                node(
                    "project_profile_network_NetworkUnit_read_timeout_ms",
                    "cpu0",
                    "cpu.const_i64",
                    &["800"],
                ),
                node(
                    "project_profile_network_NetworkUnit_write_timeout_ms",
                    "cpu0",
                    "cpu.const_i64",
                    &["900"],
                ),
                node(
                    "local_port_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_local_port",
                        "config_ready",
                    ],
                ),
                node(
                    "remote_port_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_remote_port",
                        "config_ready",
                    ],
                ),
                node(
                    "connect_timeout_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_connect_timeout_ms",
                        "config_ready",
                    ],
                ),
                node(
                    "read_timeout_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_read_timeout_ms",
                        "config_ready",
                    ],
                ),
                node(
                    "write_timeout_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_write_timeout_ms",
                        "config_ready",
                    ],
                ),
                node(
                    "local_port",
                    "network0",
                    "network.value",
                    &["local_port_seed"],
                ),
                node(
                    "remote_port",
                    "network0",
                    "network.value",
                    &["remote_port_seed"],
                ),
                node(
                    "connect_timeout",
                    "network0",
                    "network.value",
                    &["connect_timeout_seed"],
                ),
                node(
                    "read_timeout",
                    "network0",
                    "network.value",
                    &["read_timeout_seed"],
                ),
                node(
                    "write_timeout",
                    "network0",
                    "network.value",
                    &["write_timeout_seed"],
                ),
                node(
                    "socket_handle",
                    "network0",
                    "network.value",
                    &["local_port_seed"],
                ),
                node(
                    "connect_result",
                    "network0",
                    "network.connect",
                    &["local_port", "remote_port", "connect_timeout"],
                ),
                node(
                    "accept_probe",
                    "cpu0",
                    "cpu.extern_call_i64",
                    &[
                        "c",
                        "host_network_accept_probe",
                        "local_port",
                        "read_timeout",
                        "write_timeout",
                    ],
                ),
                node(
                    "accept_result",
                    "network0",
                    "network.observe",
                    &["accept_probe", "accept_ready"],
                ),
                node(
                    "close_result",
                    "network0",
                    "network.close",
                    &["socket_handle"],
                ),
                node(
                    "connect_ready",
                    "network0",
                    "network.is_connect_ready",
                    &["connect_result"],
                ),
                node(
                    "accept_ready_probe",
                    "network0",
                    "network.is_accept_ready",
                    &["accept_result"],
                ),
                node("closed", "network0", "network.is_closed", &["close_result"]),
            ],
            edges: vec![
                xfer(
                    "project_profile_network_NetworkUnit_local_port",
                    "local_port_seed",
                ),
                xfer(
                    "project_profile_network_NetworkUnit_remote_port",
                    "remote_port_seed",
                ),
                xfer(
                    "project_profile_network_NetworkUnit_connect_timeout_ms",
                    "connect_timeout_seed",
                ),
                xfer(
                    "project_profile_network_NetworkUnit_read_timeout_ms",
                    "read_timeout_seed",
                ),
                xfer(
                    "project_profile_network_NetworkUnit_write_timeout_ms",
                    "write_timeout_seed",
                ),
                dep("local_port_seed", "local_port"),
                dep("remote_port_seed", "remote_port"),
                dep("connect_timeout_seed", "connect_timeout"),
                dep("read_timeout_seed", "read_timeout"),
                dep("write_timeout_seed", "write_timeout"),
                dep("local_port_seed", "socket_handle"),
                dep("local_port", "connect_result"),
                dep("remote_port", "connect_result"),
                dep("connect_timeout", "connect_result"),
                xfer("local_port", "accept_probe"),
                xfer("read_timeout", "accept_probe"),
                xfer("write_timeout", "accept_probe"),
                xfer("accept_probe", "accept_result"),
                dep("socket_handle", "close_result"),
                dep("connect_result", "connect_ready"),
                dep("accept_result", "accept_ready_probe"),
                dep("close_result", "closed"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn accepts_network_value_from_connect_result() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "network0".to_owned(),
                    kind: ResourceKind::parse("network.io"),
                },
            ],
            nodes: vec![
                node(
                    "project_profile_network_NetworkUnit_local_port",
                    "cpu0",
                    "cpu.const_i64",
                    &["7001"],
                ),
                node(
                    "project_profile_network_NetworkUnit_remote_port",
                    "cpu0",
                    "cpu.const_i64",
                    &["7443"],
                ),
                node(
                    "project_profile_network_NetworkUnit_connect_timeout_ms",
                    "cpu0",
                    "cpu.const_i64",
                    &["1500"],
                ),
                node(
                    "local_port_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_local_port",
                        "config_ready",
                    ],
                ),
                node(
                    "remote_port_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_remote_port",
                        "config_ready",
                    ],
                ),
                node(
                    "connect_timeout_seed",
                    "network0",
                    "network.observe",
                    &[
                        "project_profile_network_NetworkUnit_connect_timeout_ms",
                        "config_ready",
                    ],
                ),
                node(
                    "local_port",
                    "network0",
                    "network.value",
                    &["local_port_seed"],
                ),
                node(
                    "remote_port",
                    "network0",
                    "network.value",
                    &["remote_port_seed"],
                ),
                node(
                    "connect_timeout",
                    "network0",
                    "network.value",
                    &["connect_timeout_seed"],
                ),
                node(
                    "connect_result",
                    "network0",
                    "network.connect",
                    &["local_port", "remote_port", "connect_timeout"],
                ),
                node(
                    "connect_value",
                    "network0",
                    "network.value",
                    &["connect_result"],
                ),
            ],
            edges: vec![
                xfer(
                    "project_profile_network_NetworkUnit_local_port",
                    "local_port_seed",
                ),
                xfer(
                    "project_profile_network_NetworkUnit_remote_port",
                    "remote_port_seed",
                ),
                xfer(
                    "project_profile_network_NetworkUnit_connect_timeout_ms",
                    "connect_timeout_seed",
                ),
                dep("local_port_seed", "local_port"),
                dep("remote_port_seed", "remote_port"),
                dep("connect_timeout_seed", "connect_timeout"),
                dep("local_port", "connect_result"),
                dep("remote_port", "connect_result"),
                dep("connect_timeout", "connect_result"),
                dep("connect_result", "connect_value"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn accepts_network_observe_from_host_transport_probe() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "network0".to_owned(),
                    kind: ResourceKind::parse("network.io"),
                },
            ],
            nodes: vec![
                node("stream_window", "cpu0", "cpu.const_i64", &["64"]),
                node("send_window", "cpu0", "cpu.const_i64", &["32"]),
                node("recv_window", "cpu0", "cpu.const_i64", &["32"]),
                node("local_port", "cpu0", "cpu.const_i64", &["9000"]),
                node("remote_port", "cpu0", "cpu.const_i64", &["443"]),
                node(
                    "send_probe",
                    "cpu0",
                    "cpu.extern_call_i64",
                    &[
                        "c",
                        "host_network_send_probe",
                        "stream_window",
                        "send_window",
                        "remote_port",
                    ],
                ),
                node(
                    "send_owned",
                    "cpu0",
                    "cpu.extern_call_i64",
                    &[
                        "c",
                        "host_network_send_owned",
                        "remote_port",
                        "stream_window",
                        "send_window",
                    ],
                ),
                node(
                    "recv_probe",
                    "cpu0",
                    "cpu.extern_call_i64",
                    &[
                        "c",
                        "host_network_recv_probe",
                        "stream_window",
                        "recv_window",
                        "local_port",
                    ],
                ),
                node(
                    "recv_owned",
                    "cpu0",
                    "cpu.extern_call_i64",
                    &[
                        "c",
                        "host_network_recv_owned",
                        "local_port",
                        "stream_window",
                        "recv_window",
                    ],
                ),
                node(
                    "send_seed",
                    "network0",
                    "network.observe",
                    &["send_probe", "send_ready"],
                ),
                node(
                    "send_owned_seed",
                    "network0",
                    "network.observe",
                    &["send_owned", "send_ready"],
                ),
                node(
                    "close_probe",
                    "cpu0",
                    "cpu.extern_call_i64",
                    &["c", "host_network_close", "local_port"],
                ),
                node(
                    "close_seed",
                    "network0",
                    "network.observe",
                    &["close_probe", "closed"],
                ),
                node(
                    "recv_seed",
                    "network0",
                    "network.observe",
                    &["recv_probe", "recv_ready"],
                ),
                node(
                    "recv_owned_seed",
                    "network0",
                    "network.observe",
                    &["recv_owned", "recv_ready"],
                ),
                node(
                    "send_ready_probe",
                    "network0",
                    "network.is_send_ready",
                    &["send_seed"],
                ),
                node(
                    "recv_ready_probe",
                    "network0",
                    "network.is_recv_ready",
                    &["recv_seed"],
                ),
                node("send_value", "network0", "network.value", &["send_seed"]),
                node("recv_value", "network0", "network.value", &["recv_seed"]),
            ],
            edges: vec![
                dep("stream_window", "send_probe"),
                dep("send_window", "send_probe"),
                dep("remote_port", "send_probe"),
                dep("remote_port", "send_owned"),
                dep("stream_window", "send_owned"),
                dep("send_window", "send_owned"),
                dep("stream_window", "recv_probe"),
                dep("recv_window", "recv_probe"),
                dep("local_port", "recv_probe"),
                dep("local_port", "recv_owned"),
                dep("stream_window", "recv_owned"),
                dep("recv_window", "recv_owned"),
                dep("local_port", "close_probe"),
                xfer("send_probe", "send_seed"),
                xfer("send_owned", "send_owned_seed"),
                xfer("close_probe", "close_seed"),
                xfer("recv_probe", "recv_seed"),
                xfer("recv_owned", "recv_owned_seed"),
                dep("send_seed", "send_ready_probe"),
                dep("recv_seed", "recv_ready_probe"),
                dep("send_seed", "send_value"),
                dep("recv_seed", "recv_value"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn rejects_task_value_without_join_result_source() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node("value", "cpu0", "cpu.const", &["7"]),
                node("task", "cpu0", "cpu.spawn_task", &["ping", "value"]),
                node("invalid", "cpu0", "cpu.task_value", &["task"]),
            ],
            edges: vec![dep("value", "task"), dep("task", "invalid")],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("expects `cpu.join_result` input"));
    }

    #[test]
    fn rejects_invalid_project_bridge_stage_contract() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node(
                    "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_bridge_stage_type",
                    "cpu0",
                    "cpu.text",
                    &["uplink=ready;downlink=windowed"],
                ),
                node(
                    "project_profile_data_FabricPlane_uplink_window_policy",
                    "cpu0",
                    "cpu.text",
                    &["marker"],
                ),
            ],
            edges: vec![dep(
                "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_bridge_stage_type",
                "project_profile_data_FabricPlane_uplink_window_policy",
            )],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("uplink=windowed;downlink=windowed"));
    }

    #[test]
    fn rejects_nested_data_window_values() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node("seed", "cpu0", "cpu.const", &["7"]),
                node("value", "fabric0", "data.move", &["seed", "cpu0"]),
                node(
                    "window0",
                    "fabric0",
                    "data.immutable_window",
                    &["value", "0", "1"],
                ),
                node(
                    "window1",
                    "fabric0",
                    "data.copy_window",
                    &["window0", "0", "1"],
                ),
            ],
            edges: vec![
                xfer("seed", "value"),
                dep("value", "window0"),
                dep("window0", "window1"),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("cannot create nested/illegal window"));
    }

    #[test]
    fn rejects_mutable_window_payload_across_data_pipe() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node("seed", "cpu0", "cpu.const", &["7"]),
                node("value", "fabric0", "data.move", &["seed", "cpu0"]),
                node(
                    "window0",
                    "fabric0",
                    "data.copy_window",
                    &["value", "0", "1"],
                ),
                node("pipe", "fabric0", "data.output_pipe", &["window0"]),
            ],
            edges: vec![
                xfer("seed", "value"),
                dep("value", "window0"),
                dep("window0", "pipe"),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("cannot send mutable window payload"));
    }

    #[test]
    fn accepts_frozen_window_payload_across_data_pipe() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node("seed", "cpu0", "cpu.const", &["7"]),
                node("value", "fabric0", "data.move", &["seed", "cpu0"]),
                node(
                    "window0",
                    "fabric0",
                    "data.copy_window",
                    &["value", "0", "1"],
                ),
                node("frozen", "fabric0", "data.freeze_window", &["window0"]),
                node("pipe", "fabric0", "data.output_pipe", &["frozen"]),
            ],
            edges: vec![
                xfer("seed", "value"),
                dep("value", "window0"),
                dep("window0", "frozen"),
                dep("frozen", "pipe"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn rejects_write_window_on_immutable_input() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node("seed", "cpu0", "cpu.const", &["7"]),
                node("value", "fabric0", "data.move", &["seed", "cpu0"]),
                node(
                    "window0",
                    "fabric0",
                    "data.immutable_window",
                    &["value", "0", "1"],
                ),
                node(
                    "updated",
                    "fabric0",
                    "data.write_window",
                    &["window0", "0", "value"],
                ),
            ],
            edges: vec![
                xfer("seed", "value"),
                dep("value", "window0"),
                dep("window0", "updated"),
                dep("value", "updated"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap_err();
    }

    #[test]
    fn accepts_read_window_on_immutable_input() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![
                Resource {
                    name: "cpu0".to_owned(),
                    kind: ResourceKind::parse("cpu.arm64"),
                },
                Resource {
                    name: "fabric0".to_owned(),
                    kind: ResourceKind::parse("data.fabric"),
                },
            ],
            nodes: vec![
                node("seed", "cpu0", "cpu.const", &["7"]),
                node("value", "fabric0", "data.move", &["seed", "cpu0"]),
                node(
                    "window0",
                    "fabric0",
                    "data.immutable_window",
                    &["value", "0", "1"],
                ),
                node("read", "fabric0", "data.read_window", &["window0", "0"]),
            ],
            edges: vec![
                xfer("seed", "value"),
                dep("value", "window0"),
                dep("window0", "read"),
            ],
            node_lanes: BTreeMap::new(),
        };

        verify_module(&module).unwrap();
    }

    #[test]
    fn rejects_bridge_payload_shape_mismatch() {
        let module = YirModule {
            version: "0.1".to_owned(),
            resources: vec![Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            }],
            nodes: vec![
                node(
                    "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_uplink_bridge_payload_type",
                    "cpu0",
                    "cpu.text",
                    &["Window<SurfaceShaderPacket>"],
                ),
                node(
                    "project_profile_data_FabricPlane_uplink_payload_shape",
                    "cpu0",
                    "cpu.text",
                    &["uplink_payload_shape"],
                ),
                node(
                    "project_profile_data_FabricPlane_uplink_payload_shape_type",
                    "cpu0",
                    "cpu.text",
                    &["PayloadShapeWindowFrame"],
                ),
            ],
            edges: vec![
                dep(
                    "project_link_cpu_Main_to_shader_SurfaceShader_via_data_FabricPlane_uplink_bridge_payload_type",
                    "project_profile_data_FabricPlane_uplink_payload_shape",
                ),
                dep(
                    "project_profile_data_FabricPlane_uplink_payload_shape_type",
                    "project_profile_data_FabricPlane_uplink_payload_shape",
                ),
            ],
            node_lanes: BTreeMap::new(),
        };

        let error = verify_module(&module).unwrap_err();
        assert!(error.contains("PayloadShapeWindowSurfaceShaderPacket"));
        assert!(error.contains("PayloadShapeWindowFrame"));
    }
}

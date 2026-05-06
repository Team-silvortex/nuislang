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
    verify_project_type_contract_nodes(module)?;
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
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum ProjectContractKind {
    DataPayloadClass,
    DataPayloadShape,
    DataHandleTableSchema,
    ShaderPacketType,
    ShaderPacketClass,
    ShaderPacketShape,
    BridgeStageContract,
    BridgePayloadContract(BridgePayloadDirection),
    KernelSlotContract,
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

fn project_link_bridge_payload_contract_target(id: &str, direction: BridgePayloadDirection) -> String {
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
    Some(format!("PayloadShape{}", sanitize_contract_type_fragment(&normalized)))
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
            SemanticOp::ShaderIsPassReady | SemanticOp::ShaderIsFrameReady | SemanticOp::ShaderValue => {
                require_observe_source(&nodes, node, SemanticOp::ShaderObserve)?;
            }
            SemanticOp::KernelObserve => {
                let source = observe_source_node(&nodes, node)?;
                let actual = observe_state_arg(node)?;
                if source.op.semantic_op() != SemanticOp::CpuProjectProfileRef {
                    return Err(format!(
                        "node `{}` expects cpu.project_profile_ref input for kernel observe, got `{}`",
                        node.name,
                        source.op.full_name()
                    ));
                }
                if !node.op.observe_state_matches_source(&source.op, actual)? {
                    return Err(format!(
                        "node `{}` observes kernel state `{actual}`, but `{}` does not support that state",
                        node.name, source.name
                    ));
                }
            }
            SemanticOp::KernelIsConfigReady | SemanticOp::KernelValue => {
                require_observe_source(&nodes, node, SemanticOp::KernelObserve)?;
            }
            _ if node.op.result_source_semantic_op().is_some() => {
                require_expected_result_source(&nodes, node)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn require_expected_result_source(
    nodes: &BTreeMap<&str, &Node>,
    node: &Node,
) -> Result<(), String> {
    let expected = node
        .op
        .result_source_semantic_op()
        .ok_or_else(|| format!("node `{}` has no expected result source contract", node.name))?;
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
            edges: vec![xfer("queue_depth", "kernel_result"), dep("kernel_result", "kernel_ready")],
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
                node("window0", "fabric0", "data.immutable_window", &["value", "0", "1"]),
                node("window1", "fabric0", "data.copy_window", &["window0", "0", "1"]),
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
                node("window0", "fabric0", "data.copy_window", &["value", "0", "1"]),
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
                node("window0", "fabric0", "data.copy_window", &["value", "0", "1"]),
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
                node("window0", "fabric0", "data.immutable_window", &["value", "0", "1"]),
                node("updated", "fabric0", "data.write_window", &["window0", "0", "value"]),
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
                node("window0", "fabric0", "data.immutable_window", &["value", "0", "1"]),
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

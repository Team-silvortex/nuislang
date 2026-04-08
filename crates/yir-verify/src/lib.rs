use std::collections::{BTreeMap, BTreeSet};

use yir_core::{
    glm_profile_for_operation, DataMod, EdgeKind, GlmEffect, GlmUseMode, LegacyFabricMod,
    ModRegistry, Node, Resource, ResourceKind, YirModule,
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
    Window,
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

        let kind = if node.op.module == "data" || node.op.module == "fabric" {
            match node.op.instruction.as_str() {
                "move" => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source != DataValueKind::Other {
                        return Err(format!(
                            "node `{}` cannot use data.move on non-Value payload `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::Other
                }
                "output_pipe" => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source == DataValueKind::PipeOutput || source == DataValueKind::PipeInput {
                        return Err(format!(
                            "node `{}` creates nested pipe value from `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::PipeOutput
                }
                "input_pipe" => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if source != DataValueKind::PipeOutput {
                        return Err(format!(
                            "node `{}` expects output_pipe input, got `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::Other
                }
                "copy_window" | "immutable_window" => {
                    let source = infer_data_value_kind(&value_kinds, &nodes, &node.op.args[0]);
                    if matches!(
                        source,
                        DataValueKind::PipeOutput
                            | DataValueKind::PipeInput
                            | DataValueKind::Marker
                            | DataValueKind::HandleTable
                    ) {
                        return Err(format!(
                            "node `{}` cannot create window from `{}`",
                            node.name, node.op.args[0]
                        ));
                    }
                    DataValueKind::Window
                }
                "marker" => DataValueKind::Marker,
                "handle_table" => {
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
                "bind_core" => {
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
            .map(
                |node| match (node.op.module.as_str(), node.op.instruction.as_str()) {
                    ("data" | "fabric", "marker") => DataValueKind::Marker,
                    ("data" | "fabric", "handle_table") => DataValueKind::HandleTable,
                    ("data" | "fabric", "bind_core") => DataValueKind::CoreBinding,
                    ("data" | "fabric", "output_pipe") => DataValueKind::PipeOutput,
                    ("data" | "fabric", "input_pipe") => DataValueKind::Other,
                    ("data" | "fabric", "copy_window" | "immutable_window") => {
                        DataValueKind::Window
                    }
                    _ => DataValueKind::Other,
                },
            )
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

    let mut values = BTreeMap::<String, PointerState>::new();
    let mut heap = BTreeMap::<usize, HeapBinding>::new();
    let mut borrow_counts = BTreeMap::<usize, usize>::new();
    let mut next_id = 1usize;
    let mut moved_names = BTreeSet::<String>::new();

    for node_name in order {
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
                    }
                    PointerState::Null => {
                        values.insert(node.name.clone(), PointerState::Null);
                    }
                    PointerState::Unknown => {
                        values.insert(node.name.clone(), PointerState::Unknown);
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

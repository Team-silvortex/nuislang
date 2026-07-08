use std::collections::{BTreeMap, BTreeSet};

use yir_core::{Node, YirModule};

use crate::cpu_heap_state::{HeapBinding, HeapObjectKind, PointerState};
use crate::graph::topological_order;

pub(crate) fn verify_cpu_heap_protocol(module: &YirModule) -> Result<(), String> {
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

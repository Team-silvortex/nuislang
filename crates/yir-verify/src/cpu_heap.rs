use std::collections::{BTreeMap, BTreeSet};

use yir_core::{parse_branch_effect_args, BranchEffectAccess, BranchEffectResult, Node, YirModule};

use crate::cpu_heap_checks::{
    ensure_buffer_index_in_bounds, ensure_buffer_readable, ensure_buffer_writable,
    ensure_live_heap, ensure_no_active_borrows, ensure_no_live_heap_aliases, ensure_node_readable,
    ensure_node_writable, infer_borrow_scope_ends, known_non_negative_int, pointer_arg,
    release_completed_borrows, release_named_borrow,
};
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
            "branch_effect" => {
                verify_owned_pointer_branch_merge(
                    node,
                    &mut values,
                    &heap,
                    &borrow_counts,
                    &mut moved_names,
                )?;
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

fn verify_owned_pointer_branch_merge(
    node: &Node,
    values: &mut BTreeMap<String, PointerState>,
    heap: &BTreeMap<usize, HeapBinding>,
    borrow_counts: &BTreeMap<usize, usize>,
    moved_names: &mut BTreeSet<String>,
) -> Result<(), String> {
    let Some(args) = parse_branch_effect_args(&node.op.args) else {
        return Ok(());
    };
    if args.merge_result != BranchEffectResult::OwnedPointer {
        return Ok(());
    }
    let owned_inputs = |actions: &[yir_core::BranchEffectAction<'_>]| {
        actions
            .iter()
            .flat_map(|action| &action.operands)
            .filter(|operand| operand.access == BranchEffectAccess::ResourceOwn)
            .map(|operand| operand.value.to_owned())
            .collect::<BTreeSet<_>>()
    };
    let then_owned = owned_inputs(&args.then_actions);
    let else_owned = owned_inputs(&args.else_actions);
    if then_owned != else_owned || then_owned.len() != 2 {
        return Err(format!(
            "node `{}` owned pointer branch must consume the same two owners on both paths",
            node.name
        ));
    }

    let mut heap_ids = BTreeSet::new();
    for source_name in then_owned {
        match pointer_arg(values, &source_name) {
            PointerState::Owned(id) => {
                ensure_live_heap(heap, id, node)?;
                ensure_no_active_borrows(borrow_counts, id, node, "branch pointer merge")?;
                if let Some(address_kind) = args.address_kind {
                    let kind_matches = matches!(
                        (address_kind, heap.get(&id).map(|binding| binding.kind)),
                        ("node", Some(HeapObjectKind::Node { .. }))
                            | ("buffer", Some(HeapObjectKind::Buffer { .. }))
                    );
                    if !kind_matches {
                        return Err(format!(
                            "node `{}` owned pointer branch address kind `{address_kind}` does not match owner `{source_name}`",
                            node.name
                        ));
                    }
                }
                if !heap_ids.insert(id) {
                    return Err(format!(
                        "node `{}` owned pointer branch aliases owner `{source_name}`",
                        node.name
                    ));
                }
                moved_names.insert(source_name);
            }
            PointerState::Borrowed(_) => {
                return Err(format!(
                    "node `{}` cannot merge borrowed pointer `{source_name}` as an owner",
                    node.name
                ));
            }
            PointerState::Null => {
                return Err(format!(
                    "node `{}` cannot merge null pointer `{source_name}` as an owner",
                    node.name
                ));
            }
            PointerState::Unknown => {
                return Err(format!(
                    "node `{}` cannot prove ownership of branch pointer `{source_name}`",
                    node.name
                ));
            }
        }
    }
    values.insert(node.name.clone(), PointerState::Unknown);
    Ok(())
}

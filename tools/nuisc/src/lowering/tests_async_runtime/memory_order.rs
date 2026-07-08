use super::*;

#[test]
fn sequences_borrow_end_before_free_in_expr_stmt_order() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let head: ref Node = move(alloc_node(10, null()));
            let head_ref: ref Node = borrow(head);
            let current: i64 = load_value(head_ref);
            borrow_end(head_ref);
            free(head);
            return current;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let borrow_end = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .unwrap();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();
    assert!(yir.edges.iter().any(|edge| {
        edge.from == borrow_end.name
            && edge.to == free.name
            && matches!(edge.kind, EdgeKind::Effect)
    }));
}

#[test]
fn sequences_store_at_before_free_in_expr_stmt_order() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(3, 0);
            store_at(buffer, 1, 7);
            let value: i64 = load_at(buffer, 1);
            free(buffer);
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let store = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "store_at")
        .unwrap();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();
    assert!(path_exists(&yir, &store.name, &free.name));
}

#[test]
fn sequences_borrowed_next_traversal_before_borrow_end_and_free() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let tail: ref Node = move(alloc_node(30, null()));
            let head: ref Node = alloc_node(10, tail);
            let head_ref: ref Node = borrow(head);
            let next_ptr: ref Node = load_next(head_ref);
            let tail_ref: ref Node = borrow(next_ptr);
            let current: i64 = load_value(tail_ref);
            borrow_end(tail_ref);
            borrow_end(head_ref);
            free(head);
            return current;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let load_next = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "load_next")
        .unwrap();
    let load_value = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "load_value")
        .unwrap();
    let borrow_ends = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();

    assert!(path_exists(&yir, &load_next.name, &load_value.name));
    assert!(borrow_ends
        .iter()
        .all(|borrow_end| path_exists(&yir, &load_value.name, borrow_end)));
    assert!(borrow_ends
        .iter()
        .all(|borrow_end| path_exists(&yir, borrow_end, &free.name)));
}

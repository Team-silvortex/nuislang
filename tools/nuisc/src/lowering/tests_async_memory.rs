use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

fn path_exists(yir: &yir_core::YirModule, from: &str, to: &str) -> bool {
    let mut frontier = vec![from.to_owned()];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(current) = frontier.pop() {
        if current == to {
            return true;
        }
        if !seen.insert(current.clone()) {
            continue;
        }
        for edge in &yir.edges {
            if edge.from == current {
                frontier.push(edge.to.clone());
            }
        }
    }
    false
}

#[test]
fn sequences_memory_lifecycle_around_task_result_observation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn amplify(seed: i64) -> i64 {
            return seed + 30;
          }

          fn main() -> i64 {
            let head: ref Node = move(alloc_node(5, null()));
            let head_ref: ref Node = borrow(head);
            let seed: i64 = load_value(head_ref);
            borrow_end(head_ref);

            let scratch: ref Buffer = alloc_buffer(2, 0);
            store_at(scratch, 0, seed);

            let task: Task<i64> = spawn(amplify(seed));
            let observed: i64 = join(task);
            store_at(scratch, 1, observed);
            let replay: i64 = load_at(scratch, 0) + load_at(scratch, 1);
            free(scratch);
            free(head);
            return replay;
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
    let spawn = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .unwrap();
    let join = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "join")
        .unwrap();
    let stores = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "store_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let frees = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let load_ats = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    assert!(path_exists(&yir, &borrow_end.name, &spawn.name));
    assert!(path_exists(&yir, &spawn.name, &join.name));
    assert!(stores
        .iter()
        .any(|store| path_exists(&yir, &join.name, store)));
    assert!(load_ats
        .iter()
        .all(|load| frees.iter().any(|free| path_exists(&yir, load, free))));
    assert!(frees
        .iter()
        .all(|free| stores.iter().any(|store| path_exists(&yir, store, free))));
}

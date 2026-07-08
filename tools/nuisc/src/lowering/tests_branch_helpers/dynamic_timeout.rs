use super::*;

#[test]
fn lowers_dynamic_if_binding_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(spawn(ping(5)), 16)
            } else {
              timeout(spawn(ping(9)), 32)
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              let task: Task<i64> = timeout(spawn(ping(5)), 16);
              let alias: Task<i64> = task;
              alias
            } else {
              let task: Task<i64> = timeout(spawn(ping(9)), 32);
              let alias: Task<i64> = task;
              alias
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(spawn(ping(5)), 16);
                task
              }
              _ => {
                let task: Task<i64> = timeout(spawn(ping(9)), 32);
                task
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(spawn(ping(5)), 16);
                let alias: Task<i64> = task;
                alias
              }
              _ => {
                let task: Task<i64> = timeout(spawn(ping(9)), 32);
                let alias: Task<i64> = task;
                alias
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

// Thread handle production through branch-local selection.

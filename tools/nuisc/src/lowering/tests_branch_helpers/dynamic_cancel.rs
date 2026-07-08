use super::*;

#[test]
fn lowers_dynamic_if_binding_by_selecting_cancel_input_before_cancel() {
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
              cancel(spawn(ping(5)))
            } else {
              cancel(spawn(ping(9)))
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
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(cancel_count, 1, "expected one post-select cancel");
    assert!(select_count >= 1, "expected selected cancel input");
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_cancel_input_before_cancel() {
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
                let task: Task<i64> = cancel(spawn(ping(5)));
                task
              }
              _ => {
                let task: Task<i64> = cancel(spawn(ping(9)));
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
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(cancel_count, 1, "expected one post-select cancel");
    assert!(select_count >= 1, "expected selected cancel input");
}

// Nested result-producing runtime consumers.
#[test]
fn lowers_dynamic_if_return_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return join_result(spawn(ping(5)));
            } else {
              return join_result(spawn(ping(9)));
            }
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return join_result(spawn(ping(5)));
              }
              _ => {
                return join_result(spawn(ping(9)));
              }
            }
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let joined: TaskResult<i64> = join_result(spawn(ping(5)));
              let alias: TaskResult<i64> = joined;
              return alias;
            } else {
              let joined: TaskResult<i64> = join_result(spawn(ping(9)));
              let alias: TaskResult<i64> = joined;
              return alias;
            }
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                let joined: TaskResult<i64> = join_result(spawn(ping(5)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = join_result(spawn(ping(9)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
            }
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

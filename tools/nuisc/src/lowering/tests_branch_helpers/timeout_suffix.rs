use super::*;

#[test]
fn lowers_dynamic_if_timeout_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(spawn(ping(5)), 16)
            } else {
              timeout(spawn(ping(9)), 32)
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_match_timeout_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
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
            let joined: TaskResult<i64> = join_result(chosen);
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_if_timeout_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(spawn(ping(5)), 16)
            } else {
              timeout(spawn(ping(9)), 32)
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_timeout_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
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
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
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
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

// Nested direct-return timeout -> join_result recursion.
#[test]
fn lowers_dynamic_if_return_by_selecting_timeout_spawn_input_before_join_result() {
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
              return join_result(timeout(spawn(ping(5)), 16));
            } else {
              return join_result(timeout(spawn(ping(9)), 32));
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
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_timeout_spawn_input_before_join_result() {
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
                return join_result(timeout(spawn(ping(5)), 16));
              }
              _ => {
                return join_result(timeout(spawn(ping(9)), 32));
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
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_timeout_spawn_input_before_join_result() {
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
              let joined: TaskResult<i64> = join_result(timeout(spawn(ping(5)), 16));
              let alias: TaskResult<i64> = joined;
              return alias;
            } else {
              let joined: TaskResult<i64> = join_result(timeout(spawn(ping(9)), 32));
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
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_timeout_spawn_input_before_join_result() {
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
                let joined: TaskResult<i64> = join_result(timeout(spawn(ping(5)), 16));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = join_result(timeout(spawn(ping(9)), 32));
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
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

// Shared observer and pure-suffix paths for Thread<T>.

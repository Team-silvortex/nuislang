use super::*;

#[test]
fn lowers_dynamic_if_task_result_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let joined: TaskResult<i64> = if argc < 2 {
              join_result(spawn(ping(5)))
            } else {
              join_result(spawn(ping(9)))
            };
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
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(
        select_count >= 1,
        "expected select before shared observer suffix"
    );
}

#[test]
fn lowers_dynamic_match_task_result_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let joined: TaskResult<i64> = match arm {
              0 => {
                let result: TaskResult<i64> = join_result(spawn(ping(5)));
                result
              }
              _ => {
                let result: TaskResult<i64> = join_result(spawn(ping(9)));
                result
              }
            };
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
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(
        select_count >= 1,
        "expected select before shared observer suffix"
    );
}

#[test]
fn lowers_dynamic_if_task_result_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let joined: TaskResult<i64> = if argc < 2 {
              join_result(spawn(ping(5)))
            } else {
              join_result(spawn(ping(9)))
            };
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
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_task_result_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let joined: TaskResult<i64> = match arm {
              0 => {
                let result: TaskResult<i64> = join_result(spawn(ping(5)));
                result
              }
              _ => {
                let result: TaskResult<i64> = join_result(spawn(ping(9)));
                result
              }
            };
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
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_if_task_result_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let joined: TaskResult<i64> = if argc < 2 {
              join_result(spawn(ping(5)))
            } else {
              join_result(spawn(ping(9)))
            };
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            let widened: i64 = resolved + 1;
            return widened + 2;
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
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

#[test]
fn lowers_dynamic_match_task_result_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let joined: TaskResult<i64> = match arm {
              0 => {
                let result: TaskResult<i64> = join_result(spawn(ping(5)));
                result
              }
              _ => {
                let result: TaskResult<i64> = join_result(spawn(ping(9)));
                result
              }
            };
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            let widened: i64 = resolved + 1;
            return widened + 2;
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
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

// Shared observer and pure-suffix paths for timeout(task, limit).

use super::*;

#[test]
fn lowers_dynamic_if_return_by_selecting_spawn_input_before_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return join(spawn(ping(5)));
            } else {
              return join(spawn(ping(9)));
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
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select join");
    assert!(select_count >= 1, "expected selected join input");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_spawn_input_before_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return join(spawn(ping(5)));
              }
              _ => {
                return join(spawn(ping(9)));
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
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select join");
    assert!(select_count >= 1, "expected selected join input");
}

#[test]
fn lowers_dynamic_if_return_by_selecting_thread_spawn_input_before_thread_join_result() {
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
              return thread_join_result(thread_spawn(ping(5)));
            } else {
              return thread_join_result(thread_spawn(ping(9)));
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_thread_spawn_input_before_thread_join_result() {
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
              let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(5)));
              let alias: TaskResult<i64> = joined;
              return alias;
            } else {
              let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(9)));
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

#[test]
fn lowers_dynamic_if_return_by_selecting_thread_spawn_input_before_thread_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return thread_join(thread_spawn(ping(5)));
            } else {
              return thread_join(thread_spawn(ping(9)));
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select thread_join");
    assert!(select_count >= 1, "expected selected thread_join input");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_thread_spawn_input_before_thread_join_result() {
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
                return thread_join_result(thread_spawn(ping(5)));
              }
              _ => {
                return thread_join_result(thread_spawn(ping(9)));
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_thread_spawn_input_before_thread_join_result() {
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
                let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(5)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(9)));
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

// Shared observer and pure-suffix paths for TaskResult<T>.

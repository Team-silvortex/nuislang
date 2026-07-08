use super::*;

#[test]
fn lowers_dynamic_if_binding_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Thread<i64> = if argc < 2 {
              thread_spawn(ping(5))
            } else {
              thread_spawn(ping(9))
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Thread<i64> = if argc < 2 {
              let thread: Thread<i64> = thread_spawn(ping(5));
              let alias: Thread<i64> = thread;
              alias
            } else {
              let thread: Thread<i64> = thread_spawn(ping(9));
              let alias: Thread<i64> = thread;
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Thread<i64> = match arm {
              0 => {
                let thread: Thread<i64> = thread_spawn(ping(5));
                thread
              }
              _ => {
                let thread: Thread<i64> = thread_spawn(ping(9));
                thread
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Thread<i64> = match arm {
              0 => {
                let thread: Thread<i64> = thread_spawn(ping(5));
                let alias: Thread<i64> = thread;
                alias
              }
              _ => {
                let thread: Thread<i64> = thread_spawn(ping(9));
                let alias: Thread<i64> = thread;
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
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

// Cancel(task) branch-local selection.

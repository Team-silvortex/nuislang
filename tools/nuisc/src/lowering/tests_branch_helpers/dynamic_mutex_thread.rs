use super::*;

#[test]
fn lowers_dynamic_if_binding_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
    assert!(select_count >= 1, "expected selected mutex input");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_dynamic_if_return_by_selecting_thread_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            let left: Thread<i64> = thread_spawn(ping(5));
            let right: Thread<i64> = thread_spawn(ping(9));
            if argc < 2 {
              return thread_join_result(left);
            } else {
              return thread_join_result(right);
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

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

    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(select_count >= 1, "expected selected thread input");
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = if argc < 2 {
              let guard: MutexGuard<i64> = mutex_lock(left);
              guard
            } else {
              let guard: MutexGuard<i64> = mutex_lock(right);
              guard
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
    assert!(select_count >= 1, "expected selected mutex input");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_thread_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            let left: Thread<i64> = thread_spawn(ping(5));
            let right: Thread<i64> = thread_spawn(ping(9));
            if argc < 2 {
              let joined: TaskResult<i64> = thread_join_result(left);
              return joined;
            } else {
              let joined: TaskResult<i64> = thread_join_result(right);
              return joined;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

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

    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(select_count >= 1, "expected selected thread input");
}

#[test]
fn lowers_dynamic_if_long_alias_chain_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = if argc < 2 {
              let guard: MutexGuard<i64> = mutex_lock(left);
              let alias0: MutexGuard<i64> = guard;
              let alias1: MutexGuard<i64> = alias0;
              alias1
            } else {
              let guard: MutexGuard<i64> = mutex_lock(right);
              let alias0: MutexGuard<i64> = guard;
              let alias1: MutexGuard<i64> = alias0;
              alias1
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = match arm {
              0 => {
                let guard: MutexGuard<i64> = mutex_lock(left);
                let alias: MutexGuard<i64> = guard;
                alias
              }
              _ => {
                let guard: MutexGuard<i64> = mutex_lock(right);
                let alias: MutexGuard<i64> = guard;
                alias
              }
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
    assert!(select_count >= 1, "expected selected match-arm input");
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_thread_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            let left: Thread<i64> = thread_spawn(ping(5));
            let right: Thread<i64> = thread_spawn(ping(9));
            match arm {
              0 => {
                let joined: TaskResult<i64> = thread_join_result(left);
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = thread_join_result(right);
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

    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(select_count >= 1, "expected selected match-arm input");
}

// Spawn/task-result branch selection.

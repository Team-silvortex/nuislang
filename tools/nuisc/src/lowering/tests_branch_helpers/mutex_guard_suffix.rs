use super::*;

#[test]
fn lowers_dynamic_if_mutex_guard_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            return mutex_value(guard);
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
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(
        select_count >= 1,
        "expected select before shared mutex observer"
    );
}

#[test]
fn lowers_dynamic_if_mutex_guard_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            let value: i64 = mutex_value(guard);
            return value + 1;
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
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_if_mutex_guard_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            let value: i64 = mutex_value(guard);
            let widened: i64 = value + 1;
            return widened + 2;
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
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

#[test]
fn lowers_dynamic_match_mutex_guard_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = match arm {
              0 => {
                let locked: MutexGuard<i64> = mutex_lock(left);
                locked
              }
              _ => {
                let locked: MutexGuard<i64> = mutex_lock(right);
                locked
              }
            };
            return mutex_value(guard);
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
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(
        select_count >= 1,
        "expected select before shared mutex observer"
    );
}

#[test]
fn lowers_dynamic_match_mutex_guard_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = match arm {
              0 => {
                let locked: MutexGuard<i64> = mutex_lock(left);
                locked
              }
              _ => {
                let locked: MutexGuard<i64> = mutex_lock(right);
                locked
              }
            };
            let value: i64 = mutex_value(guard);
            return value + 1;
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
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_mutex_guard_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = match arm {
              0 => {
                let locked: MutexGuard<i64> = mutex_lock(left);
                locked
              }
              _ => {
                let locked: MutexGuard<i64> = mutex_lock(right);
                locked
              }
            };
            let value: i64 = mutex_value(guard);
            let widened: i64 = value + 1;
            return widened + 2;
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
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_thread_spawn_input_before_thread_join() {
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
                return thread_join(thread_spawn(ping(5)));
              }
              _ => {
                return thread_join(thread_spawn(ping(9)));
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

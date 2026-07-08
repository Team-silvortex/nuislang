use super::*;

#[test]
fn lowers_if_expression_with_branch_local_task_observer_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let joined: TaskResult<i64> = thread_join_result(worker);
            let current: i64 = if true {
              let done: bool = task_completed(joined);
              if done {
                let observed: i64 = task_value(joined);
                observed
              } else {
                0
              }
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
            .count()
            >= 2
    );
}

#[test]
fn lowers_match_expression_with_branch_local_mutex_observer_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let current: i64 = match 1 {
              1 => {
                let observed: i64 = mutex_value(guard);
                observed
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_match_expression_with_branch_local_task_observer_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let joined: TaskResult<i64> = thread_join_result(worker);
            let current: i64 = match 1 {
              1 => {
                let done: bool = task_completed(joined);
                if done {
                  task_value(joined)
                } else {
                  0
                }
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
            .count()
            >= 2
    );
}

#[test]
fn lowers_effectful_thread_branch_when_constant_condition_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let current: i64 = if true {
              let joined: TaskResult<i64> = thread_join_result(worker);
              let resolved: i64 = if task_completed(joined) {
                task_value(joined)
              } else {
                0
              };
              resolved
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_effectful_mutex_branch_when_constant_condition_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let current: i64 = if true {
              let guard: MutexGuard<i64> = mutex_lock(lock);
              let value: i64 = mutex_value(guard);
              value
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_new"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_mutex_unlock_branch_when_constant_condition_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let current: i64 = if true {
              let reopened: Mutex<i64> = mutex_unlock(guard);
              let reopened_guard: MutexGuard<i64> = mutex_lock(reopened);
              mutex_value(reopened_guard)
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_unlock"));
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
            .count()
            >= 2
    );
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_thread_match_arm_when_constant_scrutinee_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let current: i64 = match 1 {
              1 => {
                let joined: TaskResult<i64> = thread_join_result(worker);
                let resolved: i64 = if task_completed(joined) {
                  task_value(joined)
                } else {
                  0
                };
                resolved
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_effectful_mutex_match_arm_when_constant_scrutinee_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let current: i64 = match 1 {
              1 => {
                let guard: MutexGuard<i64> = mutex_lock(lock);
                let value: i64 = mutex_value(guard);
                value
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_new"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_mutex_unlock_match_arm_when_constant_scrutinee_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let current: i64 = match 1 {
              1 => {
                let reopened: Mutex<i64> = mutex_unlock(guard);
                let reopened_guard: MutexGuard<i64> = mutex_lock(reopened);
                mutex_value(reopened_guard)
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_unlock"));
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
            .count()
            >= 2
    );
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_if_branch_when_constant_binding_controls_condition() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let enabled: bool = 1 < 2;
            let lock: Mutex<i64> = mutex_new(11);
            let current: i64 = if enabled {
              let guard: MutexGuard<i64> = mutex_lock(lock);
              mutex_value(guard)
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_match_arm_when_constant_binding_controls_scrutinee() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = 1;
            let worker: Thread<i64> = thread_spawn(ping(5));
            let current: i64 = match arm {
              1 => {
                let joined: TaskResult<i64> = thread_join_result(worker);
                let resolved: i64 = if task_completed(joined) {
                  task_value(joined)
                } else {
                  0
                };
                resolved
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
}

// Dynamic branch-local runtime tests are grouped below by the runtime family
// or suffix shape they are defending.

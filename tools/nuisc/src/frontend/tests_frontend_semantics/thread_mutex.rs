use super::*;
use nuis_semantics::model::NirMutexCapabilityOp;

#[test]
fn lowers_explicit_timeout_on_task_handle() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = timeout(spawn(ping()), 16);
            return join(task);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuTimeout { .. },
            ..
        }) if ty.render() == "Task<i64>"
    ));
}

#[test]
fn lowers_explicit_ready_delay_on_task_handle() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = ready_after(spawn(ping()), 4);
            return join(task);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuReadyAfter { .. },
            ..
        }) if ty.render() == "Task<i64>"
    ));
}

#[test]
fn lowers_explicit_join_result_and_task_state_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = timeout(spawn(ping()), 16);
            let result: TaskResult<i64> = join_result(task);
            if task_completed(result) {
              return task_value(result);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuJoinResult(_),
            ..
        }) if ty.render() == "TaskResult<i64>"
    ));
}

#[test]
fn lowers_thread_and_mutex_builtins_with_expected_surface_types() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping());
            let joined: TaskResult<i64> = thread_join_result(worker);
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let value: i64 = mutex_value(guard);
            return value + task_value(joined);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuThreadSpawn { .. },
            ..
        }) if ty.render() == "Thread<i64>"
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuThreadJoinResult(_),
            ..
        }) if ty.render() == "TaskResult<i64>"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuMutexNew(_),
            ..
        }) if ty.render() == "Mutex<i64>"
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuMutexLock(_),
            ..
        }) if ty.render() == "MutexGuard<i64>"
    ));
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuMutexValue(_),
            ..
        }) if ty.render() == "i64"
    ));
}

#[test]
fn lowers_shared_mutex_permits_into_two_async_workers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn observe(permit: MutexPermit<i64>) -> i64 {
            let lease: MutexLease<i64> = mutex_permit_lock(permit);
            let value: i64 = mutex_lease_value(lease);
            let released: i64 = mutex_lease_unlock(lease);
            return value + released - 1;
          }

          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(17);
            let shared: SharedMutex<i64> = mutex_share(lock);
            let left: Task<i64> = spawn(observe(mutex_permit(shared, 0)));
            let right: Task<i64> = spawn(observe(mutex_permit(shared, 1)));
            let left_value: i64 = join(left);
            let right_value: i64 = join(right);
            let revoked: i64 = mutex_shared_close(shared);
            return left_value + right_value + revoked;
          }
        }
        "#,
    )
    .unwrap();

    let observe = module
        .functions
        .iter()
        .find(|function| function.name == "observe")
        .unwrap();
    assert_eq!(observe.params[0].ty.render(), "MutexPermit<i64>");
    assert!(matches!(
        observe.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::PermitLock,
                ..
            },
            ..
        }) if ty.render() == "MutexLease<i64>"
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::Share,
                ..
            },
            ..
        }) if ty.render() == "SharedMutex<i64>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            value: NirExpr::CpuSpawn { args, .. },
            ..
        }) if matches!(
            args.as_slice(),
            [NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::Permit,
                ..
            }]
        )
    ));
    assert!(matches!(
        main.body.get(6),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::SharedClose,
                ..
            },
            ..
        }) if ty.render() == "i64"
    ));
}

#[test]
fn rejects_dynamic_or_out_of_range_mutex_permit_lanes() {
    for lane in ["lane", "2"] {
        let source = format!(
            r#"
            mod cpu Main {{
              fn main() -> i64 {{
                let lane: i64 = 0;
                let lock: Mutex<i64> = mutex_new(17);
                let shared: SharedMutex<i64> = mutex_share(lock);
                let permit: MutexPermit<i64> = mutex_permit(shared, {lane});
                return 0;
              }}
            }}
            "#
        );
        let error = parse_nuis_module(&source).unwrap_err();
        assert!(error.contains("requires a unique literal lane `0` or `1`"));
    }
}

#[test]
fn rejects_copying_a_stored_mutex_permit_into_spawn() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn observe(permit: MutexPermit<i64>) -> i64 {
            let lease: MutexLease<i64> = mutex_permit_lock(permit);
            let value: i64 = mutex_lease_value(lease);
            mutex_lease_unlock(lease);
            return value;
          }

          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(17);
            let shared: SharedMutex<i64> = mutex_share(lock);
            let permit: MutexPermit<i64> = mutex_permit(shared, 0);
            let task: Task<i64> = spawn(observe(permit));
            return join(task);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("requires a freshly issued inline MutexPermit"));
    assert!(error.contains("cannot be copied across task boundaries"));
}

#[test]
fn rejects_thread_spawn_with_sync_callee() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping());
            return thread_join(worker);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("thread_spawn(...) expects async function call"));
    assert!(error.contains("sync function `ping`"));
}

#[test]
fn rejects_thread_join_of_non_thread_value() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return thread_join(7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("thread_join(...) expects `Thread<...>`"));
    assert!(error.contains("found `i64`"));
}

#[test]
fn rejects_thread_join_result_of_non_thread_value() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let joined: TaskResult<i64> = thread_join_result(7);
            return task_value(joined);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("thread_join_result(...) expects `Thread<...>`"));
    assert!(error.contains("found `i64`"));
}

#[test]
fn rejects_mutex_lock_of_non_mutex_value() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let guard: MutexGuard<i64> = mutex_lock(7);
            return mutex_value(guard);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("mutex_lock(...) expects `Mutex<...>`"));
    assert!(error.contains("found `i64`"));
}

#[test]
fn rejects_mutex_unlock_of_non_guard_value() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_unlock(7);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            return mutex_value(guard);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("mutex_unlock(...) expects `MutexGuard<...>`"));
    assert!(error.contains("found `i64`"));
}

#[test]
fn rejects_mutex_value_of_non_guard_value() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return mutex_value(7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("mutex_value(...) expects `MutexGuard<...>`"));
    assert!(error.contains("found `i64`"));
}

#[test]
fn rejects_mutex_new_of_result_family_payload() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(ping());
            let joined: TaskResult<i64> = join_result(task);
            let lock: Mutex<TaskResult<i64>> = mutex_new(joined);
            let guard: MutexGuard<TaskResult<i64>> = mutex_lock(lock);
            let observed: TaskResult<i64> = mutex_value(guard);
            return task_value(observed);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("mutex_new(...) expects a staged mutex payload value")
            || error.contains("`Mutex<...>` expects a staged value payload"),
        "{error}"
    );
    assert!(error.contains("TaskResult<i64>"), "{error}");
}

#[test]
fn rejects_timeout_with_non_integer_limit() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = timeout(spawn(ping()), "slow");
            return join(task);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("expects integer limit"));
}

#[test]
fn rejects_ready_after_with_non_integer_delay() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = ready_after(spawn(ping()), "slow");
            return join(task);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("expects integer delay"));
}

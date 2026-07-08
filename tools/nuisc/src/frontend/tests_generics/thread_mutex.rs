use super::*;

#[test]
fn monomorphizes_generic_thread_and_mutex_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct MutexSnapshot<T> {
            value: T,
            lock: Mutex<T>,
          }

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn mutex_snapshot<T>(lock: Mutex<T>) -> MutexSnapshot<T> {
            let guard: MutexGuard<T> = mutex_lock(lock);
            let value: T = mutex_value(guard);
            let reopened: Mutex<T> = mutex_unlock(guard);
            return MutexSnapshot {
              value: value,
              lock: reopened,
            };
          }

          fn join_thread_result<T>(worker: Thread<T>) -> TaskResult<T> {
            return thread_join_result(worker);
          }

          fn main() -> i64 {
            let snapshot: MutexSnapshot<i64> = mutex_snapshot(mutex_new(7));
            let joined: TaskResult<i64> =
              join_thread_result(thread_spawn(ping(snapshot.value)));
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(stmt_tree_contains_call(&main.body, &|callee, _| callee == "mutex_snapshot__i64"));
    assert!(stmt_tree_contains_call(&main.body, &|callee, _| {
        callee == "join_thread_result__i64"
    }));

    let snapshot = module
        .functions
        .iter()
        .find(|function| function.name == "mutex_snapshot__i64")
        .expect("expected specialized mutex snapshot helper");
    assert!(snapshot.generic_params.is_empty());
    assert!(snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexLock(_),
            } if name == "guard" && ty.render() == "MutexGuard<i64>"
        )
    }));
    assert!(snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexValue(_),
            } if name == "value" && ty.render() == "i64"
        )
    }));
    assert!(snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexUnlock(_),
            } if name == "reopened" && ty.render() == "Mutex<i64>"
        )
    }));

    let joiner = module
        .functions
        .iter()
        .find(|function| function.name == "join_thread_result__i64")
        .expect("expected specialized thread join helper");
    assert!(joiner.generic_params.is_empty());
    assert!(matches!(
        joiner.body.last(),
        Some(NirStmt::Return(Some(NirExpr::CpuThreadJoinResult(value))))
            if matches!(value.as_ref(), NirExpr::Var(name) if name == "worker")
    ));
}

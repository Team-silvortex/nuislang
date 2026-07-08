use super::*;

#[test]
fn lowers_explicit_task_primitives_into_cpu_effect_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(ping());
            cancel(task);
            return join(task);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
}

#[test]
fn lowers_explicit_timeout_primitive_into_cpu_effect_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
}

#[test]
fn lowers_recursive_async_result_family_observation_path_into_cpu_effect_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_down(seed: i64, remaining: i64) -> i64 {
            if remaining == 0 {
              return seed;
            }
            return await sum_down(seed + 1, remaining - 1);
          }

          fn encode_timed_out(result: TaskResult<i64>) -> i64 {
            if task_timed_out(result) {
              return 1;
            }
            return 0;
          }

          fn encode_cancelled(result: TaskResult<i64>) -> i64 {
            if task_cancelled(result) {
              return 1;
            }
            return 0;
          }

          fn encode_value(result: TaskResult<i64>) -> i64 {
            if task_completed(result) {
              return task_value(result);
            }
            return 0;
          }

          fn main() -> i64 {
            let completed_result: TaskResult<i64> = join_result(spawn(sum_down(7, 4)));
            let timed_result: TaskResult<i64> =
              join_result(timeout(spawn(sum_down(7, 4)), 0));
            let cancelled_result: TaskResult<i64> =
              join_result(cancel(spawn(sum_down(7, 4))));

            return encode_value(completed_result)
              + encode_timed_out(timed_result)
              + encode_cancelled(cancelled_result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_timed_out"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
}

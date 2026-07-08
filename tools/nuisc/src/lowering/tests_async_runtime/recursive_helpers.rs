use super::*;

#[test]
fn rejects_async_while_with_await_step_and_task_observer_flow_control() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
              let result: TaskResult<i64> = join_result(spawn(step(value)));
              if task_completed(result) {
                break;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();

    assert!(error.contains(
        "structured `while` lowering recognized loop state `value` and a loop-control `if`"
    ));
    assert!(error.contains(
        "control condition is not reducible to supported loop-state/carry boolean tests"
    ));
}

#[test]
fn lowers_mutually_recursive_async_calls_into_schedule_boundaries_and_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return await even(value - 1);
          }

          async fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return await odd(value - 1);
          }

          async fn main() -> i64 {
            return await even(4);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let call_i64_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even")
        })
        .count();
    assert!(
        async_call_count >= 3,
        "expected async mutual recursion to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        call_i64_count >= 3,
        "expected async mutual recursion to emit helper-lowered calls, found {call_i64_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:odd"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:even"));
}

#[test]
fn lowers_generic_recursive_async_call_into_schedule_boundary_and_specialized_helper_lane() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn bounce<T>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return value;
            }
            return await bounce(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await bounce(7, 4);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let specialized_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("bounce__i64"))
        })
        .count();
    assert!(
        async_call_count >= 2,
        "expected generic recursive async lowering to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        specialized_call_count >= 2,
        "expected generic recursive async lowering to emit specialized helper calls, found {specialized_call_count}"
    );
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:bounce__i64")));
}

#[test]
fn lowers_generic_mutually_recursive_async_calls_into_schedule_boundaries_and_specialized_helper_lanes(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn odd<T>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return value;
            }
            return await even(value, remaining - 1);
          }

          async fn even<T>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return value;
            }
            return await odd(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await even(7, 4);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let specialized_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("odd__i64") || name.starts_with("even__i64")
                })
        })
        .count();
    assert!(
        async_call_count >= 3,
        "expected generic async mutual recursion to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        specialized_call_count >= 3,
        "expected generic async mutual recursion to emit specialized helper calls, found {specialized_call_count}"
    );
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:odd__i64")));
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:even__i64")));
}

#[test]
fn lowers_recursive_async_with_generic_payload_alias_higher_order_body_into_specialized_helper_lanes(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          async fn climb(value: i64, remaining: i64) -> i64 {
            if remaining == 0 {
              return apply_payload(
                JustAlias<i64>(value),
                |x: i64| -> i64 { return x.add(1); }
              );
            }
            return await climb(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await climb(7, 4);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let recursive_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "climb")
        })
        .count();
    assert!(
        async_call_count >= 2,
        "expected async recursive higher-order lowering to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        recursive_call_count >= 2,
        "expected async recursive higher-order lowering to emit recursive helper calls, found {recursive_call_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:climb"));
}

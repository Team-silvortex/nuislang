use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_recursive_component_reachable_fn2_lambda_higher_order_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return f(x, y);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply2(value, 1, |x: i64, y: i64| -> i64 { return x - y; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply2(value, 1, |x: i64, y: i64| -> i64 { return x - y; }));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let hof_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply2_"))
        .count();
    assert!(
        hof_lane_count >= 1,
        "expected helper-lowered Fn2 higher-order specialization lane, found {hof_lane_count}"
    );
    let lambda_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected helper-lowered Fn2 lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("__hof_apply2_"))
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected recursive calls into helper-lowered Fn2 specialization, found {hof_calls}"
    );
    let lambda_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        lambda_calls >= 2,
        "expected helper-lowered Fn2 higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_generic_fn2_lambda_higher_order_helpers_into_helper_lanes()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply2<T: Addable>(x: T, y: T, f: Fn2<T, T, T>) -> T {
            return f(x, y);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply2(value, 1, |x: i64, y: i64| -> i64 { return x - y; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply2(value, 1, |x: i64, y: i64| -> i64 { return x - y; }));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let hof_i64_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply2_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_i64_lane_count >= 1,
        "expected helper-lowered generic Fn2 higher-order specialization lane, found {hof_i64_lane_count}"
    );
    let lambda_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected helper-lowered generic Fn2 lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__hof_apply2_") && name.ends_with("__i64")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic Fn2 helper-lowered higher-order specialization, found {hof_calls}"
    );
    let lambda_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        lambda_calls >= 2,
        "expected generic Fn2 helper-lowered higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_generic_fn3_lambda_higher_order_helpers_into_helper_lanes()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply3(value, 0, 1, |x: i64, _y: i64, z: i64| -> i64 { return x - z; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply3(value, 0, 1, |x: i64, _y: i64, z: i64| -> i64 { return x - z; }));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let hof_i64_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply3_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_i64_lane_count >= 1,
        "expected helper-lowered generic Fn3 higher-order specialization lane, found {hof_i64_lane_count}"
    );
    let lambda_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected helper-lowered generic Fn3 lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__hof_apply3_") && name.ends_with("__i64")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic Fn3 helper-lowered higher-order specialization, found {hof_calls}"
    );
    let lambda_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        lambda_calls >= 2,
        "expected generic Fn3 helper-lowered higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_generic_fn3_alias_lambda_higher_order_helpers_into_helper_lanes(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Reducer<T>) -> T {
            return f(x, y, z);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply3(value, 0, 1, |x: i64, _y: i64, z: i64| -> i64 { return x - z; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply3(value, 0, 1, |x: i64, _y: i64, z: i64| -> i64 { return x - z; }));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let hof_i64_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply3_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_i64_lane_count >= 1,
        "expected helper-lowered generic alias Fn3 higher-order specialization lane, found {hof_i64_lane_count}"
    );
    let lambda_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected helper-lowered generic alias Fn3 lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__hof_apply3_") && name.ends_with("__i64")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic alias Fn3 helper-lowered higher-order specialization, found {hof_calls}"
    );
    let lambda_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        lambda_calls >= 2,
        "expected generic alias Fn3 helper-lowered higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

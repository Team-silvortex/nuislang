use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_recursive_component_reachable_higher_order_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn dec(value: i64) -> i64 {
            return value - 1;
          }

          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply(value, dec));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply(value, dec));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:dec"));
    let hof_lane = yir
        .node_lanes
        .values()
        .find(|lane| lane.starts_with("fn:__hof_apply_"))
        .cloned();
    assert!(
        hof_lane.is_some(),
        "expected helper-lowered higher-order specialization lane"
    );
    let hof_target = yir
        .nodes
        .iter()
        .find_map(|node| {
            (node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("__hof_apply_")))
            .then(|| node.op.args[0].clone())
        })
        .expect("expected recursive calls into higher-order specialization");
    let dec_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "dec")
        })
        .count();
    let recursive_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even" || name == &hof_target)
        })
        .count();
    assert!(
        dec_calls >= 1,
        "expected helper-lowered higher-order body to call `dec`, found {dec_calls}"
    );
    assert!(
        recursive_calls >= 2,
        "expected recursive calls through helper-lowered higher-order specialization, found {recursive_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_lambda_higher_order_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let lambda_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected helper-lowered lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_targets = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("__hof_apply_"))
        })
        .map(|node| node.op.args[0].clone())
        .collect::<Vec<_>>();
    assert!(
        hof_targets.len() >= 2,
        "expected recursive calls into helper-lowered higher-order specialization, found {}",
        hof_targets.len()
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
        "expected helper-lowered higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_generic_lambda_higher_order_helpers_into_helper_lanes() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply(value, |v: i64| -> i64 { return v - 1; }));
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
        .filter(|lane| lane.starts_with("fn:__hof_apply_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_i64_lane_count >= 1,
        "expected helper-lowered generic higher-order specialization lane, found {hof_i64_lane_count}"
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
        "expected helper-lowered generic lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_calls =
        yir.nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "call_i64"
                    && node.op.args.first().is_some_and(|name| {
                        name.starts_with("__hof_apply_") && name.ends_with("__i64")
                    })
            })
            .count();
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic helper-lowered higher-order specialization, found {hof_calls}"
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
        "expected generic helper-lowered higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_generic_alias_lambda_higher_order_helpers_into_helper_lanes(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: Mapper<T>) -> T {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(apply(value, |v: i64| -> i64 { return v - 1; }));
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
        .filter(|lane| lane.starts_with("fn:__hof_apply_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_i64_lane_count >= 1,
        "expected helper-lowered generic alias higher-order specialization lane, found {hof_i64_lane_count}"
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
        "expected helper-lowered generic alias lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let hof_calls =
        yir.nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "call_i64"
                    && node.op.args.first().is_some_and(|name| {
                        name.starts_with("__hof_apply_") && name.ends_with("__i64")
                    })
            })
            .count();
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic alias helper-lowered higher-order specialization, found {hof_calls}"
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
        "expected generic alias helper-lowered higher-order body to call synthesized lambdas, found {lambda_calls}"
    );
}

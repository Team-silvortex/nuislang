use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_recursive_component_reachable_mixed_scalar_higher_order_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step(value: i32) -> i32 {
            return value - i32_from_i64(1);
          }

          fn near_zero(value: i64) -> bool {
            let narrowed: i32 = i32_from_i64(value);
            if step(narrowed) == i32_from_i64(0) {
              return true;
            }
            return false;
          }

          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if near_zero(value) == true {
              return 0;
            }
            return even(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn even(value: i64) -> i64 {
            if near_zero(value) == true {
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
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:near_zero"));
    let hof_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply_"))
        .count();
    assert!(
        hof_lane_count >= 1,
        "expected helper-lowered higher-order specialization lane, found {hof_lane_count}"
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
        "expected helper-lowered lambda lanes for odd/even bodies, found {lambda_lane_count}"
    );
    let bool_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "near_zero")
        })
        .count();
    let i32_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
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
                    .is_some_and(|name| name.starts_with("__hof_apply_"))
        })
        .count();
    assert!(
        bool_calls >= 2,
        "expected calls into helper-lowered `near_zero`, found {bool_calls}"
    );
    assert!(
        i32_calls >= 1,
        "expected calls into helper-lowered `step`, found {i32_calls}"
    );
    assert!(
        hof_calls >= 2,
        "expected recursive calls into helper-lowered higher-order specialization, found {hof_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_mixed_scalar_generic_higher_order_helpers_into_helper_lanes(
) {
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

          fn step(value: i32) -> i32 {
            return value - i32_from_i64(1);
          }

          fn near_zero(value: i64) -> bool {
            let narrowed: i32 = i32_from_i64(value);
            if step(narrowed) == i32_from_i64(0) {
              return true;
            }
            return false;
          }

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if near_zero(value) == true {
              return 0;
            }
            return even(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn even(value: i64) -> i64 {
            if near_zero(value) == true {
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
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:near_zero"));
    let hof_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_lane_count >= 1,
        "expected helper-lowered generic higher-order specialization lane, found {hof_lane_count}"
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
    let bool_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "near_zero")
        })
        .count();
    let i32_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
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
        bool_calls >= 2,
        "expected calls into helper-lowered `near_zero`, found {bool_calls}"
    );
    assert!(
        i32_calls >= 1,
        "expected calls into helper-lowered `step`, found {i32_calls}"
    );
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic helper-lowered higher-order specialization, found {hof_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_mixed_scalar_generic_alias_higher_order_helpers_into_helper_lanes(
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

          fn step(value: i32) -> i32 {
            return value - i32_from_i64(1);
          }

          fn near_zero(value: i64) -> bool {
            let narrowed: i32 = i32_from_i64(value);
            if step(narrowed) == i32_from_i64(0) {
              return true;
            }
            return false;
          }

          fn apply<T: Addable>(x: T, f: Mapper<T>) -> T {
            return f(x);
          }

          fn odd(value: i64) -> i64 {
            if near_zero(value) == true {
              return 0;
            }
            return even(apply(value, |v: i64| -> i64 { return v - 1; }));
          }

          fn even(value: i64) -> i64 {
            if near_zero(value) == true {
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
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:near_zero"));
    let hof_lane_count = yir
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:__hof_apply_") && lane.ends_with("__i64"))
        .count();
    assert!(
        hof_lane_count >= 1,
        "expected helper-lowered generic alias higher-order specialization lane, found {hof_lane_count}"
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
    let bool_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "near_zero")
        })
        .count();
    let i32_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
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
        bool_calls >= 2,
        "expected calls into helper-lowered `near_zero`, found {bool_calls}"
    );
    assert!(
        i32_calls >= 1,
        "expected calls into helper-lowered `step`, found {i32_calls}"
    );
    assert!(
        hof_calls >= 2,
        "expected recursive calls into generic alias helper-lowered higher-order specialization, found {hof_calls}"
    );
}

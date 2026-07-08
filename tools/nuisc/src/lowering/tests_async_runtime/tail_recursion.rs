use super::*;

#[test]
fn lowers_recursive_async_call_into_schedule_boundary_and_helper_lane() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_down(current: i64) -> i64 {
            if current == 0 {
              return 0;
            }
            let tail: i64 = await sum_down(current - 1);
            return current + tail;
          }

          async fn main() -> i64 {
            return await sum_down(4);
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
                && node.op.args.first().is_some_and(|name| name == "sum_down")
        })
        .count();
    assert!(
        async_call_count >= 2,
        "expected recursive async lowering to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        call_i64_count >= 2,
        "expected recursive async lowering to emit helper-lowered calls, found {call_i64_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:sum_down"));
}

#[test]
fn lowers_self_tail_recursive_async_function_into_loop_while_i64_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_next(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            return await sum_next(current - 1, acc + (current - 1));
          }

          async fn main() -> i64 {
            return await sum_next(4, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_next")
        })
        .count();
    assert_eq!(
        self_async_call_count, 1,
        "expected only the outer entry async call to remain after self tail-recursive async rewrite"
    );
}

#[test]
fn lowers_branching_self_tail_recursive_async_function_into_loop_while_i64_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_selected(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return await sum_selected(current - 1, acc + (current - 1));
            } else {
              return await sum_selected(current - 1, acc + 0);
            }
          }

          async fn main() -> i64 {
            return await sum_selected(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "sum_selected")
        })
        .count();
    assert_eq!(
        self_async_call_count, 1,
        "expected only the outer entry async call to remain after branching self tail-recursive async rewrite"
    );
}

#[test]
fn lowers_multi_carry_prev_current_self_tail_recursive_async_function_into_loop_while_i64_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            return await accumulate(current - 1, sum + current, prod * current);
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_prev_current");
    assert_eq!(loop_node.op.args[8], "mul_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_carry_condition_branching_multi_carry_prev_current_self_tail_recursive_async_function_into_loop_while_i64_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if sum > 3 {
              return await accumulate(current - 1, sum + 0, prod + current);
            } else {
              return await accumulate(current - 1, sum + current, prod * current);
            }
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_carry0_gt");
    assert_eq!(loop_node.op.args[8], "keep");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    assert_eq!(loop_node.op.args[11], "prev_carry0_gt");
    assert_eq!(loop_node.op.args[13], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "mul_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_cross_prev_carry_self_tail_recursive_async_function_into_loop_while_i64_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            return await accumulate(current - 1, sum + prod, prod + current);
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_prev_carry1");
    assert_eq!(loop_node.op.args[8], "add_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_branching_cross_prev_carry_self_tail_recursive_async_function_into_loop_while_i64_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if current > 2 {
              return await accumulate(current - 1, sum + prod, prod + current);
            } else {
              return await accumulate(current - 1, sum + 0, prod + sum);
            }
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_prev_carry1");
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "prev_current_gt");
    assert_eq!(loop_node.op.args[13], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "add_prev_carry0");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_identity_branching_self_tail_recursive_async_function_into_loop_while_i64_cond_chain_with_keep_prev_carry(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            if current > 2 {
              return await accumulate(current - 1, acc + current);
            } else {
              return await accumulate(current - 1, acc);
            }
          }

          async fn main() -> i64 {
            return await accumulate(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_prev_current");
    assert_eq!(loop_node.op.args[9], "keep_prev_carry");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

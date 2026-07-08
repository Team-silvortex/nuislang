use super::*;

#[test]
fn lowers_early_break_self_tail_recursive_async_function_into_loop_while_scalar_flow_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return acc;
            } else {
              return await sum_until(current - 1, acc + current);
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0);
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "prev_current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_early_break_branching_self_tail_recursive_async_function_into_loop_while_scalar_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return acc;
            } else {
              if current > 1 {
                return await sum_until(current - 1, acc + current);
              } else {
                return await sum_until(current - 1, acc + 0);
              }
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0);
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "prev_current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "prev_current_gt");
    assert_eq!(loop_node.op.args[11], "add_prev_current");
    assert_eq!(loop_node.op.args[12], "keep");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 5 {
              return acc + current;
            }
            return await sum_until(current - 1, acc + current);
          }

          async fn main() -> i64 {
            return await sum_until(4, 0);
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected loop_while_scalar_post_flow_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_branching_aux_carry_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 5 {
              return acc + current;
            }
            if current > 1 {
              return await sum_until(current - 1, acc + current, flag + current);
            } else {
              return await sum_until(current - 1, acc + current, flag + 0);
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let lowered_ops = yir
        .nodes
        .iter()
        .map(|node| format!("{}::{}", node.op.module, node.op.instruction))
        .collect::<Vec<_>>();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .unwrap_or_else(|| {
            panic!("expected loop_while_scalar_post_flow_cond_chain node, got {lowered_ops:?}")
        });
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "always");
    assert_eq!(loop_node.op.args[11], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "prev_current_gt");
    assert_eq!(loop_node.op.args[16], "add_prev_current");
    assert_eq!(loop_node.op.args[17], "keep");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_identity_branching_aux_carry_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_cond_chain_with_keep_prev_carry(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 5 {
              return acc + current;
            }
            if current > 2 {
              return await sum_until(current - 1, acc + current, flag + current);
            } else {
              return await sum_until(current - 1, acc + current, flag);
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0, 0);
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep_prev_carry"));
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_nested_branching_aux_carry_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 6 {
              return acc + current;
            }
            if current > 3 {
              return await sum_until(current - 1, acc + current, flag + current);
            } else {
              if current > 1 {
                return await sum_until(current - 1, acc + current, flag + current);
              } else {
                return await sum_until(current - 1, acc + current, flag + 0);
              }
            }
          }

          async fn main() -> i64 {
            return await sum_until(5, 0, 0);
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert!(loop_node.op.args.iter().any(|arg| arg == "or"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

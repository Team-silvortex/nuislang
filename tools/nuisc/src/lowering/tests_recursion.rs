use super::loop_purity::collect_pure_helper_functions;
use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_self_tail_recursive_function_into_loop_while_scalar_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_next(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            return sum_next(current - 1, acc + (current - 1));
          }

          fn main() -> i64 {
            return sum_next(4, 1);
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
}

#[test]
fn lowers_branching_self_tail_recursive_function_into_loop_while_scalar_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_selected(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return sum_selected(current - 1, acc + (current - 1));
            } else {
              return sum_selected(current - 1, acc + 0);
            }
          }

          fn main() -> i64 {
            return sum_selected(4, 0);
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
}

#[test]
fn lowers_multiplicative_self_tail_recursive_function_into_loop_while_scalar_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn fact(current: i64, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            return fact(current - 1, acc * current);
          }

          fn main() -> i64 {
            return fact(4, 1);
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
    assert_eq!(loop_node.op.args[6], "mul_prev_current");
}

#[test]
fn lowers_multi_carry_prev_current_self_tail_recursive_function_into_loop_while_scalar_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            return accumulate(current - 1, sum + current, prod * current);
          }

          fn main() -> i64 {
            return accumulate(4, 0, 1);
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
}

#[test]
fn lowers_tail_recursive_dynamic_buffer_index_read_from_prev_current_into_loop_while_scalar_chain()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, buffer: ref Buffer, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            return accumulate(current - 1, buffer, acc + load_at(buffer, current));
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = accumulate(4, buffer, 0);
            free(buffer);
            return acc;
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
    assert_eq!(loop_node.op.args[6], "add_read_at_dynamic_prev_current");
}

#[test]
fn lowers_tail_recursive_dynamic_buffer_index_read_from_prev_carry_into_loop_while_scalar_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, buffer: ref Buffer, slot: i64, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            if current > 2 {
              return accumulate(current - 1, buffer, slot + current, acc + load_at(buffer, slot));
            } else {
              return accumulate(current - 1, buffer, slot + 0, acc + 0);
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = accumulate(4, buffer, 1, 0);
            free(buffer);
            return acc;
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
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "prev_current_gt");
    assert_eq!(loop_node.op.args[13], "add_read_at_dynamic_prev_carry0");
    assert!(loop_node.op.args[14].starts_with("alloc_buffer_"));
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn lowers_early_break_tail_recursive_dynamic_buffer_index_read_into_loop_while_scalar_flow_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_until(current: i64, buffer: ref Buffer, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return acc;
            } else {
              return sum_until(current - 1, buffer, acc + load_at(buffer, current));
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = sum_until(4, buffer, 0);
            free(buffer);
            return acc;
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
    assert_eq!(loop_node.op.args[9], "add_read_at_dynamic_prev_current");
}

#[test]
fn lowers_post_flow_break_tail_recursive_dynamic_buffer_index_read_into_loop_while_scalar_post_flow_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_until(current: i64, buffer: ref Buffer, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + load_at(buffer, current) > 5 {
              return acc + load_at(buffer, current);
            }
            return sum_until(current - 1, buffer, acc + load_at(buffer, current));
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = sum_until(4, buffer, 0);
            free(buffer);
            return acc;
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
    assert_eq!(loop_node.op.args[9], "add_read_at_dynamic_prev_current");
}

#[test]
fn lowers_post_flow_break_branching_tail_recursive_dynamic_prev_carry_index_read_into_loop_while_scalar_post_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_until(current: i64, buffer: ref Buffer, acc: i64, slot: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + load_at(buffer, slot) > 5 {
              return acc + load_at(buffer, slot);
            }
            if current > 1 {
              return sum_until(current - 1, buffer, acc + load_at(buffer, slot), slot + current);
            } else {
              return sum_until(current - 1, buffer, acc + load_at(buffer, slot), slot + 0);
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = sum_until(4, buffer, 0, 1);
            free(buffer);
            return acc;
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
    assert_eq!(loop_node.op.args[9], "always");
    assert_eq!(loop_node.op.args[11], "add_read_at_dynamic_prev_carry1");
    assert!(loop_node.op.args[12].starts_with("alloc_buffer_"));
    assert_eq!(loop_node.op.args[13], "add_read_at_dynamic_prev_carry1");
    assert!(loop_node.op.args[14].starts_with("alloc_buffer_"));
    assert_eq!(loop_node.op.args[16], "prev_current_gt");
    assert_eq!(loop_node.op.args[18], "add_prev_current");
    assert_eq!(loop_node.op.args[19], "keep");
}

#[test]
fn lowers_post_flow_break_nested_branching_tail_recursive_dynamic_prev_carry_index_read_into_loop_while_scalar_post_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_until(current: i64, buffer: ref Buffer, acc: i64, slot: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + load_at(buffer, slot) > 6 {
              return acc + load_at(buffer, slot);
            }
            if current > 3 {
              return sum_until(current - 1, buffer, acc + load_at(buffer, slot), slot + current);
            } else {
              if current > 1 {
                return sum_until(current - 1, buffer, acc + load_at(buffer, slot), slot + current);
              } else {
                return sum_until(current - 1, buffer, acc + load_at(buffer, slot), slot + 0);
              }
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 9);
            let acc: i64 = sum_until(5, buffer, 0, 1);
            free(buffer);
            return acc;
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
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_read_at_dynamic_prev_carry1"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg.starts_with("alloc_buffer_")));
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
}

#[test]
fn lowers_branching_multi_carry_prev_current_self_tail_recursive_function_into_loop_while_scalar_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if current > 2 {
              return accumulate(current - 1, sum + current, prod * current);
            } else {
              return accumulate(current - 1, sum + 0, prod + current);
            }
          }

          fn main() -> i64 {
            return accumulate(4, 0, 1);
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
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "prev_current_gt");
    assert_eq!(loop_node.op.args[13], "mul_prev_current");
    assert_eq!(loop_node.op.args[14], "add_prev_current");
}

#[test]
fn lowers_carry_condition_branching_multi_carry_prev_current_self_tail_recursive_function_into_loop_while_scalar_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if sum > 3 {
              return accumulate(current - 1, sum + 0, prod + current);
            } else {
              return accumulate(current - 1, sum + current, prod * current);
            }
          }

          fn main() -> i64 {
            return accumulate(4, 0, 1);
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
}
#[test]
fn lowers_cross_prev_carry_self_tail_recursive_function_into_loop_while_scalar_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            return accumulate(current - 1, sum + prod, prod + current);
          }

          fn main() -> i64 {
            return accumulate(4, 0, 1);
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
}

#[test]
fn lowers_branching_cross_prev_carry_self_tail_recursive_function_into_loop_while_scalar_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if current > 2 {
              return accumulate(current - 1, sum + prod, prod + current);
            } else {
              return accumulate(current - 1, sum + 0, prod + sum);
            }
          }

          fn main() -> i64 {
            return accumulate(4, 0, 1);
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
}

#[test]
fn lowers_identity_branching_self_tail_recursive_function_into_loop_while_scalar_cond_chain_with_keep_prev_carry(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            if current > 2 {
              return accumulate(current - 1, acc + current);
            } else {
              return accumulate(current - 1, acc);
            }
          }

          fn main() -> i64 {
            return accumulate(4, 0);
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
}

#[test]
fn lowers_f64_self_tail_recursive_function_into_loop_while_scalar_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_next(current: f64, acc: f64) -> f64 {
            if current == 0.0 {
              return acc;
            }
            return sum_next(current - 1.0, acc + current);
          }

          fn main() -> i64 {
            let result: f64 = sum_next(4.0, 1.0);
            if result > 1.0 {
              return 7;
            }
            return 9;
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
    assert_eq!(loop_node.op.args[6], "add_prev_current");
}

#[test]
fn lowers_branching_f64_self_tail_recursive_function_into_loop_while_scalar_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn walk(current: f64, acc: f64) -> f64 {
            if current <= 0.0 {
              return acc;
            }
            if current > 2.0 {
              return walk(current - 1.0, acc + current);
            } else {
              return walk(current - 1.0, acc * current);
            }
          }

          fn main() -> i64 {
            let result: f64 = walk(4.0, 1.0);
            if result > 1.0 {
              return 7;
            }
            return 9;
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
    assert_eq!(loop_node.op.args[9], "mul_prev_current");
}

#[test]
fn lowers_fallthrough_branching_self_tail_recursive_function_into_loop_while_scalar_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn sum_selected(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return sum_selected(current - 1, acc + (current - 1));
            }
            return sum_selected(current - 1, acc + 0);
          }

          fn main() -> i64 {
            return sum_selected(4, 0);
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
}

#[test]
fn lowers_fallthrough_branching_f64_self_tail_recursive_function_into_loop_while_scalar_cond_chain()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn walk(current: f64, acc: f64) -> f64 {
            if current <= 0.0 {
              return acc;
            }
            if current > 2.0 {
              return walk(current - 1.0, acc + current);
            }
            return walk(current - 1.0, acc * current);
          }

          fn main() -> i64 {
            let result: f64 = walk(4.0, 1.0);
            if result > 1.0 {
              return 7;
            }
            return 9;
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
    assert_eq!(loop_node.op.args[9], "mul_prev_current");
}

#[test]
fn lowers_tail_recursive_function_with_prelude_bindings_into_loop_while_scalar_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn accumulate(current: i64, acc: i64) -> i64 {
            if current <= 1 {
              let done: i64 = acc;
              return done;
            }
            if current > 2 {
              let bonus: i64 = current - 1;
              return accumulate(current - 1, acc + bonus);
            } else {
              let bonus: i64 = 0;
              return accumulate(current - 1, acc + bonus);
            }
          }

          fn main() -> i64 {
            return accumulate(4, 0);
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
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn recognizes_pure_helper_with_prelude_and_if_control_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn bonus(current: i64) -> i64 {
            let down: i64 = current - 1;
            if current > 2 {
              return down;
            }
            return 0;
          }

          fn main() -> i64 {
            return bonus(4);
          }
        }
        "#,
    )
    .unwrap();

    let pure_helpers = collect_pure_helper_functions(&module);
    assert!(pure_helpers.contains("bonus"));
}

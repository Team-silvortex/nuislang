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

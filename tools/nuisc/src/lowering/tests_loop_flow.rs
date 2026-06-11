use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_chained_accumulating_while_into_loop_while_i64_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let weighted: i64 = 0;
            while value < 4 {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              let weighted: i64 = weighted + acc;
            }
            return weighted;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_chain")
        .expect("expected loop_while_i64_chain node");
    assert_eq!(loop_node.op.args[6], "add_current");
    assert_eq!(loop_node.op.args[8], "add_carry0");
}

#[test]
fn lowers_branching_chained_while_into_loop_while_i64_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn lowers_match_branching_while_into_loop_while_i64_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              match value {
                3 => {
                  let acc: i64 = acc + value;
                },
                _ => {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_eq");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn lowers_chained_while_with_inlineable_pure_helper_wrapped_step_and_carry_into_loop_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step_value(value: i64) -> i64 {
            let one: i64 = 1;
            return value + one;
          }

          fn add_value(acc: i64, value: i64) -> i64 {
            let delta: i64 = value;
            return acc + delta;
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = step_value(value);
              let acc: i64 = add_value(acc, value);
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_chain")
        .expect("expected loop_while_i64_chain node");
    assert_eq!(loop_node.op.args[4], "add");
    assert_eq!(loop_node.op.args[6], "add_current");
}

#[test]
fn lowers_chained_while_with_conditional_pure_helper_wrapped_carry_into_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step_value(value: i64) -> i64 {
            return value + 1;
          }

          fn update_acc(acc: i64, value: i64) -> i64 {
            if value > 2 {
              return acc + value;
            } else {
              return acc + 0;
            }
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = step_value(value);
              let acc: i64 = update_acc(acc, value);
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[4], "add");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn lowers_chained_while_with_prelude_conditional_pure_helper_wrapped_carry_into_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step_value(value: i64) -> i64 {
            return value + 1;
          }

          fn update_acc(acc: i64, value: i64) -> i64 {
            let high: bool = value > 2;
            if high == true {
              return acc + value;
            } else {
              return acc + 0;
            }
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = step_value(value);
              let acc: i64 = update_acc(acc, value);
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[4], "add");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn lowers_multi_arm_match_inside_guarded_while_into_guard_return() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match state {
                1 => { return 7; },
                2 => { return 8; },
                _ => { return 9; }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let guard_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "guard_return")
        .expect("expected guard_return node");
    assert_eq!(guard_node.op.args.len(), 2);
}

#[test]
fn lowers_bool_match_branching_while_into_loop_while_i64_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              match value > 2 {
                true => {
                  let acc: i64 = acc + value;
                },
                _ => {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn lowers_inline_helper_bool_match_branching_while_into_loop_while_i64_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          @inline
          fn hot(value: i64) -> bool {
            return value > 2;
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              match hot(value) {
                true => {
                  let acc: i64 = acc + value;
                },
                _ => {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn lowers_flow_breaking_while_into_loop_while_i64_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 8 {
              let value: i64 = value + 1;
              if value > 4 {
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
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_chain")
        .expect("expected loop_while_i64_flow_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_continuing_while_into_loop_while_i64_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value < 3 {
                continue;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_chain")
        .expect("expected loop_while_i64_flow_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_breaking_while_on_carried_state_into_loop_while_i64_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 8 {
              let value: i64 = value + 1;
              if acc > 6 {
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
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_chain")
        .expect("expected loop_while_i64_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_breaking_then_branching_carry_while_into_loop_while_i64_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value > 4 {
                break;
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_flow_continuing_then_branching_carry_while_into_loop_while_i64_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value < 3 {
                continue;
              }
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_flow_continuing_then_eq_branching_carry_while_into_loop_while_i64_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value != 3 {
                continue;
              }
              if value == 3 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_ne");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_eq");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_match_prefixed_flow_control_then_branching_carry_into_loop_while_i64_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          @inline
          fn hot(value: i64) -> bool {
            return value < 3;
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              match hot(value) {
                true => { continue; },
                _ => { }
              }
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_bool_or_helper_match_flow_control_then_branching_carry_into_loop_while_i64_flow_cond_chain(
) {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          @inline
          fn hot(value: i64) -> bool {
            return value == 1 || value == 2;
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              match hot(value) {
                true => { continue; },
                _ => {}
              }
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "or");
    assert_eq!(loop_node.op.args[6], "current_eq");
    assert_eq!(loop_node.op.args[8], "current_eq");
    assert_eq!(loop_node.op.args[10], "continue");
    assert_eq!(loop_node.op.args[12], "current_gt");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn lowers_nested_if_break_then_branching_carry_into_loop_while_i64_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value > 1 {
                if value > 4 {
                  break;
                } else {
                }
              } else {
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "break");
    assert_eq!(loop_node.op.args[12], "current_gt");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn lowers_nested_match_continue_then_branching_carry_into_loop_while_i64_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              match value {
                1 => {
                  continue;
                }
                _ => {
                  if value > 4 {
                    let acc: i64 = acc + value;
                  } else {
                    let acc: i64 = acc + 0;
                  }
                }
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_flow_cond_chain"
        })
        .expect("expected loop_while_i64_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_eq");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

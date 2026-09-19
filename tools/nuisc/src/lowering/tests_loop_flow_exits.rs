use super::{lower_nir_to_yir_builtin_cpu, parse_nuis_module};

#[test]
fn lowers_flow_breaking_while_into_loop_while_scalar_flow_chain() {
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_continuing_while_into_loop_while_scalar_flow_chain() {
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_breaking_while_with_else_only_control_into_loop_while_scalar_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value <= 3 {
              } else {
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_breaking_while_on_carried_state_into_loop_while_scalar_flow_chain() {
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_breaking_while_with_pure_control_temp_chain_into_loop_while_scalar_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 8 {
              let value: i64 = value + 1;
              let stop_now: bool = value > 4;
              let should_break: bool = stop_now;
              if should_break {
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_flow_breaking_then_branching_carry_while_into_loop_while_scalar_flow_cond_chain() {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_flow_continuing_then_branching_carry_while_into_loop_while_scalar_flow_cond_chain() {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_flow_continuing_then_eq_branching_carry_while_into_loop_while_scalar_flow_cond_chain() {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_ne");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_eq");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_match_prefixed_flow_control_then_branching_carry_into_loop_while_scalar_flow_cond_chain()
{
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_bool_or_helper_match_flow_control_then_branching_carry_into_loop_while_scalar_flow_cond_chain(
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "or");
    assert_eq!(loop_node.op.args[6], "current_eq");
    assert_eq!(loop_node.op.args[8], "current_eq");
    assert_eq!(loop_node.op.args[10], "continue");
    assert_eq!(loop_node.op.args[12], "current_gt");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn lowers_nested_if_break_then_branching_carry_into_loop_while_scalar_flow_cond_chain() {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "break");
    assert_eq!(loop_node.op.args[12], "current_gt");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn lowers_nested_if_break_in_then_and_direct_break_in_else_into_loop_while_scalar_flow_cond_chain()
{
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "or");
    assert!(loop_node.op.args.iter().any(|arg| arg == "current_gt"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "current_le"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "break"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "add_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
}

#[test]
fn lowers_nested_match_continue_then_branching_carry_into_loop_while_scalar_flow_cond_chain() {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_eq");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_flow_continue_then_multi_arm_match_helper_carry_into_loop_while_scalar_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn update_acc(acc: i64, value: i64) -> i64 {
            match value {
              2 => { return acc + value; },
              3 => { return acc + value; },
              _ => { return acc + 0; }
            }
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value < 2 {
                continue;
              }
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "or");
    assert_eq!(loop_node.op.args[10], "current_eq");
    assert_eq!(loop_node.op.args[12], "current_eq");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn lowers_recursive_boolean_break_then_branching_carry_into_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = value + 1;
              if value > 1 && value > 3 && value < 6 {
                break;
              } else {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "and");
    assert_eq!(loop_node.op.args[7], "current_gt");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "current_lt");
    assert_eq!(loop_node.op.args[13], "break");
    assert_eq!(loop_node.op.args[15], "current_gt");
    assert_eq!(loop_node.op.args[17], "add_current");
    assert_eq!(loop_node.op.args[18], "keep");
}

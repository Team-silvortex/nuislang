use super::*;

#[test]
fn lowers_async_while_with_compound_flow_control_and_conditional_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "and");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "current_gt");
    assert_eq!(loop_node.op.args[9], "break");
    assert_eq!(loop_node.op.args[11], "current_gt");
    assert_eq!(loop_node.op.args[13], "add_current");
    assert_eq!(loop_node.op.args[14], "keep");
}

#[test]
fn lowers_async_recursive_boolean_break_then_branching_carry_into_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "and");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "current_lt");
    assert_eq!(loop_node.op.args[12], "break");
    assert_eq!(loop_node.op.args[14], "current_gt");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert_eq!(loop_node.op.args[17], "keep");
}

#[test]
fn lowers_async_mixed_break_continue_control_into_recursive_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 3 {
                break;
              } else {
                if value < 2 {
                  continue;
                } else {
                  break;
                }
              }
            }
            return value;
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "flow_or");
    assert_eq!(loop_node.op.args[5], "flow_break");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "flow_or");
    assert_eq!(loop_node.op.args[9], "flow_continue");
    assert_eq!(loop_node.op.args[10], "current_lt");
    assert_eq!(loop_node.op.args[12], "flow_break");
}

#[test]
fn lowers_async_while_with_await_step_and_post_flow_break() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 8 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > 6 {
                break;
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry0_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_post_flow_conditional_break() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 5 {
                break;
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry0_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
}

#[test]
fn lowers_async_while_with_compound_post_flow_control_and_conditional_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              match acc {
                5 => { continue; },
                _ => {
                  if acc < 6 {
                    continue;
                  } else {
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
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "or");
    assert_eq!(loop_node.op.args[5], "carry0_eq");
    assert_eq!(loop_node.op.args[7], "carry0_lt");
    assert_eq!(loop_node.op.args[9], "continue");
    assert_eq!(loop_node.op.args[11], "current_gt");
    assert_eq!(loop_node.op.args[13], "add_current");
    assert_eq!(loop_node.op.args[14], "keep");
}

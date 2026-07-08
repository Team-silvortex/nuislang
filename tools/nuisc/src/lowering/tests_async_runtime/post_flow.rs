use super::*;

#[test]
fn rejects_async_chained_while_with_future_sibling_carry_dependency() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn seed_slot() -> i64 {
            return 0;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let slot: i64 = await seed_slot();
            let buffer: ref Buffer = alloc_buffer(4, 9);
            while value < 3 {
              let value: i64 = await step(value);
              let acc: i64 = acc + load_at(buffer, slot);
              if value > 1 {
                let slot: i64 = slot + value;
              } else {
                let slot: i64 = slot + 0;
              }
            }
            return acc + slot;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error
        .contains("references sibling carry `slot` before that carry is updated in the loop body"));
}

#[test]
fn lowers_async_post_flow_continue_with_recursive_boolean_condition_into_post_flow_cond_chain() {
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
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 1 && acc > 3 && acc < 10 {
                continue;
              } else {
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
    assert_eq!(loop_node.op.args[4], "and");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "carry0_gt");
    assert_eq!(loop_node.op.args[8], "carry0_gt");
    assert_eq!(loop_node.op.args[10], "carry0_lt");
    assert_eq!(loop_node.op.args[12], "continue");
    assert_eq!(loop_node.op.args[14], "current_gt");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert_eq!(loop_node.op.args[17], "keep");
}

#[test]
fn lowers_async_mixed_break_continue_post_flow_control_into_recursive_cond_chain() {
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
              } else {
                if acc < 3 {
                  continue;
                } else {
                  break;
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
    assert_eq!(loop_node.op.args[4], "flow_or");
    assert_eq!(loop_node.op.args[5], "flow_break");
    assert_eq!(loop_node.op.args[6], "carry0_gt");
    assert_eq!(loop_node.op.args[8], "flow_or");
    assert_eq!(loop_node.op.args[9], "flow_continue");
    assert_eq!(loop_node.op.args[10], "carry0_lt");
    assert_eq!(loop_node.op.args[12], "flow_break");
}

#[test]
fn lowers_async_kernel_observer_step_into_async_post_flow_break_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            let probe: KernelResult<i64> =
              kernel_result(kernel_profile_queue_depth("KernelUnit"));
            if kernel_config_ready(probe) {
              return value + kernel_value(probe);
            }
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > 8 {
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
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "value"));
}

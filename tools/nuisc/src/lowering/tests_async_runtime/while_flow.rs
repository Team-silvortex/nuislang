use super::*;

#[test]
fn lowers_async_while_with_await_step_and_pure_carry_into_async_loop_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_chain"
        })
        .expect("expected loop_while_scalar_async_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[5], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_dynamic_buffer_index_read_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let buffer: ref Buffer = alloc_buffer(8, 9);
            while value < 3 {
              let value: i64 = await step(value);
              let acc: i64 = acc + load_at(buffer, value);
            }
            free(buffer);
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_chain"
        })
        .expect("expected loop_while_scalar_async_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[5], "add_read_at_dynamic_current");
}

#[test]
fn lowers_async_while_with_await_step_and_break_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
              if value > 1 {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_flow_chain"
        })
        .expect("expected loop_while_scalar_async_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "current_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_continue_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = await step(value);
              if value == 2 {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_flow_chain"
        })
        .expect("expected loop_while_scalar_async_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "current_eq");
    assert_eq!(loop_node.op.args[6], "continue");
    assert_eq!(loop_node.op.args[8], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_conditional_carry_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 3 {
                continue;
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
    assert_eq!(loop_node.op.args[4], "current_gt");
    assert_eq!(loop_node.op.args[6], "continue");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
}

#[test]
fn lowers_mutable_async_while_with_memory_carry_and_continue_into_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let mut value: i64 = 0;
            let mut acc: i64 = 0;
            let buffer: ref Buffer = alloc_buffer(8, 9);
            while value < 5 {
              value = await step(value);
              if value > 3 {
                continue;
              }
              if value > 2 {
                acc += load_at(buffer, value);
              } else {
                acc += 0;
              }
            }
            free(buffer);
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
    assert_eq!(loop_node.op.args[4], "current_gt");
    assert_eq!(loop_node.op.args[6], "continue");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_read_at_dynamic_current");
    assert!(loop_node.op.args[11].starts_with("alloc_buffer_"));
    assert_eq!(loop_node.op.args[12], "keep");

    let load_ats = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .expect("expected free node");
    assert!(
        load_ats.is_empty(),
        "expected dynamic buffer loads to be absorbed into the structured loop node"
    );
    let buffer_name = loop_node.op.args[11].clone();
    let outgoing = yir
        .edges
        .iter()
        .filter(|edge| edge.from == loop_node.name)
        .collect::<Vec<_>>();
    let incoming_free = yir
        .edges
        .iter()
        .filter(|edge| edge.to == free.name)
        .collect::<Vec<_>>();
    assert!(outgoing
        .iter()
        .any(|edge| edge.to.starts_with("loop_carry_")));
    assert!(incoming_free.iter().any(|edge| edge.from == buffer_name));
    assert!(path_exists(&yir, &buffer_name, &free.name));
}

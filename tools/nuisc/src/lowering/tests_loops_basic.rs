use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_guard_break_only_while_into_noop_loop_guard() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              break;
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(!yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "guard_print"));
}

#[test]
fn lowers_guard_print_break_while_into_guard_print() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while true {
              print(7);
              break;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_print"));
}

#[test]
fn lowers_guard_print_continue_while_into_guard_print() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while true {
              let shown: i64 = 7 + 0;
              print(shown);
              continue;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_print"));
}

#[test]
fn lowers_guarded_branching_while_into_select_plus_guard_print() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while true {
              if 2 < 5 {
                print(7);
                break;
              } else {
                print(9);
                continue;
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_print"));
}

#[test]
fn lowers_guarded_while_return_body_into_guard_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while 2 < 5 {
              return 9;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn lowers_guarded_while_with_structural_load_value_into_guard_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let head: ref Node = move(alloc_node(17, null()));
            while true {
              let current: i64 = load_value(head);
              return current;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn lowers_guarded_while_with_buffer_load_at_into_guard_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 9);
            while true {
              let current: i64 = load_at(buffer, 0);
              return current;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn lowers_guarded_branching_while_into_select_plus_guard_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while true {
              if 2 < 5 {
                return 7;
              } else {
                return 9;
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn rejects_bare_break_with_structured_loop_control_diagnostic() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            break;
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("`break` is currently lowered only as terminal loop control"));
    assert!(error.contains("recognized `while` flow shapes"));
}

#[test]
fn rejects_bare_continue_with_structured_loop_control_diagnostic() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            continue;
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("`continue` is currently lowered only as terminal loop control"));
    assert!(error.contains("recognized `while` flow shapes"));
}

#[test]
fn rejects_general_sync_while_with_structured_shape_diagnostic() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              let value: i64 = value + 1;
              print(value);
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("structured `while` lowering recognized loop state `value` and its step")
    );
    assert!(error.contains("remaining body still contains arbitrary executable statements"));
}

#[test]
fn rejects_structured_while_with_unsupported_first_step_binding_diagnostic() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              let next: i64 = value + 1;
              let value: i64 = next;
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("the first body binding `next` is not a supported step"));
    assert!(error.contains("loop state `value`"));
}

#[test]
fn rejects_structured_while_with_unsupported_control_condition_diagnostic() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let head: ref Node = move(alloc_node(7, null()));
            while value < 3 {
              let value: i64 = value + 1;
              if load_value(head) > 0 {
                break;
              }
              let acc: i64 = value + 0;
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("a loop-control `if`"));
    assert!(error.contains(
        "control condition is not reducible to supported loop-state/carry boolean tests"
    ));
}

#[test]
fn lowers_structured_while_with_mixed_break_continue_control_into_recursive_flow_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              let value: i64 = value + 1;
              if value > 1 {
                break;
              } else {
                if value < 1 {
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "flow_or");
    assert_eq!(loop_node.op.args[6], "flow_break");
    assert_eq!(loop_node.op.args[7], "current_gt");
    assert_eq!(loop_node.op.args[9], "flow_or");
    assert_eq!(loop_node.op.args[10], "flow_continue");
    assert_eq!(loop_node.op.args[11], "current_lt");
    assert_eq!(loop_node.op.args[13], "flow_break");
}

#[test]
fn lowers_guarded_branching_while_into_select_plus_guard_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while true {
              if 2 < 5 {
                print(7);
                return 17;
              } else {
                print(9);
                return 19;
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_print_return" }));
}

#[test]
fn lowers_simple_counted_while_into_loop_while_i64() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              let value: i64 = value + 1;
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64")
        .expect("expected loop_while_i64 node");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "add");
}

#[test]
fn lowers_descending_counted_while_into_loop_while_i64() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 7;
            while value > 1 {
              let value: i64 = value - 2;
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64")
        .expect("expected loop_while_i64 node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
}

#[test]
fn lowers_equality_counted_while_into_loop_while_i64() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value == 0 {
              let value: i64 = value + 1;
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64")
        .expect("expected loop_while_i64 node");
    assert_eq!(loop_node.op.args[3], "eq");
    assert_eq!(loop_node.op.args[4], "add");
}

#[test]
fn lowers_inequality_counted_while_into_loop_while_i64() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value != 1 {
              let value: i64 = value + 1;
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64")
        .expect("expected loop_while_i64 node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "add");
}

#[test]
fn lowers_accumulating_counted_while_into_loop_while_scalar_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = value + 1;
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "add");
    assert_eq!(loop_node.op.args[6], "add_current");
}

use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_two_branching_carries_into_loop_while_i64_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let weighted: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value > 1 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc == 5 {
                let weighted: i64 = weighted + acc;
              } else {
                let weighted: i64 = weighted + 0;
              }
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "carry0_eq");
    assert_eq!(loop_node.op.args[13], "add_carry0");
    assert_eq!(loop_node.op.args[14], "keep");
}

#[test]
fn lowers_post_flow_breaking_while_on_updated_carry_into_loop_while_i64_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 8 {
              let value: i64 = value + 1;
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_chain"
        })
        .expect("expected loop_while_i64_post_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_post_flow_breaking_while_with_le_ge_into_loop_while_i64_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value <= 3 {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              if acc >= 6 {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_chain"
        })
        .expect("expected loop_while_i64_post_flow_chain node");
    assert_eq!(loop_node.op.args[3], "le");
    assert_eq!(loop_node.op.args[5], "carry0_ge");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_post_flow_breaking_while_with_eq_into_loop_while_i64_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              if acc == 6 {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_chain"
        })
        .expect("expected loop_while_i64_post_flow_chain node");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[5], "carry0_eq");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_post_flow_breaking_while_with_ne_into_loop_while_i64_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              if acc != 6 {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_chain"
        })
        .expect("expected loop_while_i64_post_flow_chain node");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[5], "carry0_ne");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_post_flow_continuing_while_on_updated_carry_into_loop_while_i64_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              if acc < 3 {
                continue;
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_chain"
        })
        .expect("expected loop_while_i64_post_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn lowers_post_flow_breaking_after_branching_carry_into_loop_while_i64_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_post_flow_continuing_after_branching_carry_into_loop_while_i64_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc < 6 {
                continue;
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_match_prefixed_post_flow_control_after_branching_carry_into_loop_while_i64_post_flow_cond_chain(
) {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          @inline
          fn hot(acc: i64) -> bool {
            return acc < 6;
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = value + 1;
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              match hot(acc) {
                true => { continue; },
                _ => { }
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_post_flow_breaking_after_eq_branching_carry_into_loop_while_i64_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value == 3 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc != 3 {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_ne");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_eq");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn lowers_post_flow_breaking_after_two_branching_carries_into_loop_while_i64_post_flow_cond_chain()
{
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let weighted: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value > 1 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc == 5 {
                let weighted: i64 = weighted + acc;
              } else {
                let weighted: i64 = weighted + 0;
              }
              if weighted >= 5 {
                break;
              }
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
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_i64_post_flow_cond_chain"
        })
        .expect("expected loop_while_i64_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry1_ge");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
    assert_eq!(loop_node.op.args[14], "carry0_eq");
    assert_eq!(loop_node.op.args[16], "add_carry0");
    assert_eq!(loop_node.op.args[17], "keep");
}

#[test]
fn rejects_general_iterative_while_until_loop_lowering_exists() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              print(value);
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("guard-style `while` loops or simple counted `while` loops"));
}

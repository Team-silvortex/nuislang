use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_mixed_continue_break_return_tree_into_distinct_terminal_effects() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn classify(value: i64) -> i64 {
            while true {
              if value < 0 {
                continue;
              } else if value == 0 {
                break;
              } else {
                return 7;
              }
            }
            return 3;
          }

          fn main() -> i64 {
            return classify(0) + classify(1);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_loop_continue" }));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
    assert!(yir
        .functions
        .iter()
        .any(|function| function.name == "classify" && function.domain == "cpu"));
    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "call_i64"
            && node
                .op
                .args
                .first()
                .is_some_and(|callee| callee == "classify")
    }));
}

#[test]
fn lowers_state_carrying_loop_into_unbounded_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            loop {
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
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected unbounded post-flow loop node");
    assert_eq!(loop_node.op.args[3], "always");
    assert_eq!(loop_node.op.args[5], "carry0_ge");
    assert_eq!(loop_node.op.args[7], "break");
}

#[test]
fn lowers_invariant_while_let_payload_into_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn main() -> i64 {
            let selected: Option = Option.Some(2);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > payload {
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
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "variant_is"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "variant_field"));
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected pattern-gated post-flow loop node");
    assert_eq!(loop_node.op.args[3], "invariant_true");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
}

#[test]
fn rejects_dynamically_rebound_while_let_scrutinee_precisely() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn main() -> i64 {
            let selected: Option = Option.Some(2);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              let selected: Option = Option.None;
              if cursor > 2 {
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

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("requires a loop-invariant enum scrutinee"),
        "{error}"
    );
    assert!(
        error.contains("dynamic variant-state carry contract"),
        "{error}"
    );
}

#[test]
fn runtime_none_argument_skips_invariant_pattern_loop_in_llvm() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn consume(selected: Option) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > payload {
                break;
              }
            }
            return acc;
          }

          fn main() -> i64 {
            return consume(Option.None);
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(llvm_ir.contains("zext i1 false to i64"));
    assert!(!llvm_ir.contains("loop_while_scalar_post_flow_chain_cond."));
    assert!(!llvm_ir.contains("deferred lowering"));
}

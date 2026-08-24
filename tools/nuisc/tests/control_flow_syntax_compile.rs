#[test]
fn loop_and_else_if_syntax_cross_the_full_compile_pipeline() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn loop_value() -> i64 {
            loop {
              return 7;
            }
            return 0;
          }

          fn classify_statement(value: i64) -> i64 {
            if value < 0 {
              return 10;
            } else if value == 0 {
              return 20;
            } else {
              return 30;
            }
          }

          fn classify_expression(value: i64) -> i64 {
            return if value < 0 {
              100
            } else if value == 0 {
              200
            } else {
              300
            };
          }

          fn option_statement(value: Option) -> i64 {
            if let Option.Some(payload) = value {
              return payload;
            } else if let Option.None = value {
              return 0;
            } else {
              return -1;
            }
          }

          fn option_expression(value: Option) -> i64 {
            return if let Option.Some(payload) = value {
              payload + 1
            } else {
              0
            };
          }

          fn mixed_terminal(value: i64) -> i64 {
            loop {
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
            return loop_value()
              + classify_statement(0)
              + classify_expression(1)
              + option_statement(Option.Some(4))
              + option_expression(Option.None)
              + mixed_terminal(0)
              + mixed_terminal(1);
          }
        }
        "#,
    )
    .expect("loop and else-if syntax should compile");

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_loop_continue" }));
    assert!(artifacts.llvm_ir.contains("guard_loop_continue_repeat."));
    assert!(artifacts.llvm_ir.contains("define "));
    assert!(!artifacts.llvm_ir.trim().is_empty());
    assert!(!artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "deferred"));
}

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

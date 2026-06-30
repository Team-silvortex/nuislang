use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn rewrites_async_nested_post_flow_branching_tail_recursion_into_while() {
    let module = parse_nuis_module(
        r#"
            mod cpu Main {
              async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
                if current == 0 {
                  return acc;
                }
                if acc + current > 6 {
                  return acc + current;
                }
                if current > 3 {
                  return await sum_until(current - 1, acc + current, flag + current);
                } else {
                  if current > 1 {
                    return await sum_until(current - 1, acc + current, flag + current);
                  } else {
                    return await sum_until(current - 1, acc + current, flag + 0);
                  }
                }
              }

              async fn main() -> i64 {
                return await sum_until(5, 0, 0);
              }
            }
            "#,
    )
    .unwrap();

    let pure_helpers = collect_pure_helper_functions(&module);
    let inlineable_pure_helpers = collect_inlineable_pure_helper_exprs(&module);
    let pure_helper_blocks = collect_pure_helper_blocks(&module);
    let original = module
        .functions
        .iter()
        .find(|function| function.name == "sum_until")
        .expect("expected sum_until");
    let (recurse_condition, base_return, recursive_step) =
        extract_self_tail_recursive_shape(original, &pure_helpers)
            .expect("expected recursive shape to be recognized");
    let loop_body = rewrite_self_tail_recursive_loop_body(original, recursive_step)
        .expect("expected recursive loop body rewrite");
    assert!(
            is_self_tail_recursive_loop_shape(
                &recurse_condition,
                &loop_body,
                &pure_helpers,
                &inlineable_pure_helpers,
                &pure_helper_blocks,
            ),
            "expected rewritten loop body to satisfy self-tail-recursive loop shape; base_return={base_return:?}, body={loop_body:?}"
        );

    let rewritten = rewrite_self_tail_recursive_functions(&module);
    let sum_until = rewritten
        .functions
        .iter()
        .find(|function| function.name == "sum_until")
        .expect("expected sum_until");
    assert!(
        matches!(sum_until.body.first(), Some(NirStmt::While { .. })),
        "expected self tail recursion rewrite to produce a while loop, got {:?}",
        sum_until.body
    );
}

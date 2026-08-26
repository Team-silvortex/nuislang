use super::*;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn lowers_match_expression_with_shared_borrow_lifecycle_and_shared_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let head: ref Node = move(alloc_node(10, null()));
            let current: i64 = match 1 {
              1 => {
                let head_ref: ref Node = borrow(head);
                let value: i64 = head_ref.value;
                borrow_end(head_ref);
                let widened: i64 = value + 3;
                widened
              }
              _ => {
                let head_ref: ref Node = borrow(head);
                let value: i64 = head_ref.value + 1;
                borrow_end(head_ref);
                let widened: i64 = value + 3;
                widened
              }
            };
            head.value = current + 67;
            return head.value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let borrows = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow")
        .collect::<Vec<_>>();
    let borrow_ends = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .collect::<Vec<_>>();
    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    let adds = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .collect::<Vec<_>>();
    let store_value = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "store_value")
        .expect("expected owner store_value after match expression");

    assert_eq!(
        borrows.len(),
        1,
        "expected shared borrow hoisted out of branches"
    );
    assert_eq!(
        borrow_ends.len(),
        1,
        "expected shared borrow_end hoisted out of branches"
    );
    assert!(
        !selects.is_empty(),
        "expected select-based branch value lowering for match expression"
    );
    assert!(
        adds.len() >= 3,
        "expected branch-local add, shared-suffix add, and owner-write add"
    );
    assert!(path_exists(&yir, &selects[0].name, &store_value.name));
    assert!(path_exists(&yir, &borrow_ends[0].name, &store_value.name));
}

#[test]
fn lowers_nested_pure_helper_call_chain_into_branch_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cffi Main {
          extern "c" fn host_argv_count() -> i64;

          struct ExitSummary {
            message: String,
            code: i64
          }

          fn usage_message() -> String {
            return "usage";
          }

          fn usage_exit_code() -> i64 {
            return 60 + 4;
          }

          fn ok_message() -> String {
            return "ok";
          }

          fn ok_exit_code() -> i64 {
            return 0 + 0;
          }

          fn render_summary(message: String, code: i64) -> ExitSummary {
            return ExitSummary {
              message: message,
              code: code
            };
          }

          fn usage_summary() -> ExitSummary {
            return render_summary(usage_message(), usage_exit_code());
          }

          fn ok_summary() -> ExitSummary {
            return render_summary(ok_message(), ok_exit_code());
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let summary: ExitSummary = usage_summary();
              print(summary.message);
              return summary.code;
            } else {
              let summary: ExitSummary = ok_summary();
              print(summary.message);
              return summary.code;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "branch_print_return" }));
}

#[test]
fn lowers_nested_pure_helper_param_passthrough_into_branch_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cffi Main {
          extern "c" fn host_argv_count() -> i64;

          struct ExitSummary {
            message: String,
            code: i64
          }

          fn usage_message() -> String {
            return "usage";
          }

          fn ok_message() -> String {
            return "ok";
          }

          fn pass_text(message: String) -> String {
            return message;
          }

          fn usage_exit_code() -> i64 {
            return 60 + 4;
          }

          fn ok_exit_code() -> i64 {
            return 0 + 0;
          }

          fn render_summary(message: String, code: i64) -> ExitSummary {
            return ExitSummary {
              message: message,
              code: code
            };
          }

          fn usage_summary() -> ExitSummary {
            return render_summary(pass_text(usage_message()), usage_exit_code());
          }

          fn ok_summary() -> ExitSummary {
            return render_summary(pass_text(ok_message()), ok_exit_code());
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let summary: ExitSummary = usage_summary();
              print(summary.message);
              return summary.code;
            } else {
              let summary: ExitSummary = ok_summary();
              print(summary.message);
              return summary.code;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "branch_print_return" }));
}

#[test]
fn lowers_nested_if_expression_chain_with_tail_expr_branches_into_selects() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let current: i64 = if true {
              if false {
                4
              } else {
                7
              }
            } else {
              1
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    assert!(
        selects.len() >= 2,
        "expected nested tail-expression `if` chain to lower through select nodes"
    );
}

#[test]
fn lowers_match_expression_arm_with_nested_if_tail_expr_into_selects() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let current: i64 = match 1 {
              1 => {
                if false {
                  9
                } else {
                  4
                }
              }
              _ => {
                0
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    assert!(
        !selects.is_empty(),
        "expected match arm tail-expression lowering to produce select nodes"
    );
}

#[test]
fn lowers_nested_if_return_chain_with_pure_local_bindings_into_selects() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            if true {
              let seed: i64 = 4;
              if false {
                let widened: i64 = seed + 3;
                return widened;
              } else {
                let widened: i64 = seed + 6;
                return widened;
              }
            } else {
              let fallback: i64 = 1;
              return fallback;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    assert!(
        selects.len() >= 2,
        "expected pure local bindings in nested return chain to still lower through selects"
    );
}

#[test]
fn lowers_multi_arm_match_return_chain_with_pure_local_bindings_into_selects() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            match 2 {
              1 => {
                let seed: i64 = 4;
                let widened: i64 = seed + 3;
                return widened;
              }
              2 => {
                let seed: i64 = 5;
                let widened: i64 = seed + 6;
                return widened;
              }
              _ => {
                let fallback: i64 = 1;
                return fallback;
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    assert!(
        selects.len() >= 2,
        "expected lowered multi-arm match return chain with pure local bindings to use nested selects"
    );
}

#[test]
fn lowers_guard_style_helper_return_chain_into_selects_without_guard_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn min2(lhs: i64, rhs: i64) -> i64 {
            if lhs < rhs {
              return lhs;
            }
            return rhs;
          }

          fn clamp3(first: i64, second: i64, third: i64) -> i64 {
            return min2(min2(first, second), third);
          }

          fn main() -> i64 {
            return clamp3(7, 4, 9);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    let guard_returns = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "guard_return")
        .collect::<Vec<_>>();
    assert!(
        !selects.is_empty(),
        "expected guard-style helper return chain to lower through select nodes"
    );
    assert!(
        guard_returns.is_empty(),
        "expected helper guard-return chain to avoid guard_return nodes, found {}",
        guard_returns.len()
    );
}

#[test]
fn lowers_if_return_chain_with_shared_suffix_after_branch_local_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            if true {
              let value: i64 = 4;
              let widened: i64 = value + 3;
              return widened;
            } else {
              let value: i64 = 8;
              let widened: i64 = value + 3;
              return widened;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    let adds = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .collect::<Vec<_>>();
    assert!(
        !selects.is_empty(),
        "expected shared-suffix return chain to select the branch-local binding"
    );
    assert!(
        !adds.is_empty(),
        "expected shared suffix arithmetic to remain lowered after branch selection"
    );
}

#[test]
fn lowers_match_return_chain_with_shared_suffix_after_branch_local_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            match 2 {
              1 => {
                let value: i64 = 4;
                let widened: i64 = value + 3;
                return widened;
              }
              _ => {
                let value: i64 = 8;
                let widened: i64 = value + 3;
                return widened;
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .collect::<Vec<_>>();
    let adds = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .collect::<Vec<_>>();
    assert!(
        !selects.is_empty(),
        "expected lowered match shared-suffix return chain to select the branch-local binding"
    );
    assert!(
        !adds.is_empty(),
        "expected shared suffix arithmetic to survive match lowering"
    );
}

#[test]
fn lowers_try_propagation_inside_terminal_if_branch() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn checked<T>(value: T) -> Result<T, i64> {
            return Result.Ok(value);
          }

          fn encode(value: i64) -> Result<i64, i64> {
            if value > 0 {
              let accepted: i64 = checked<i64>(value)?;
              let widened: i64 = checked<i64>(accepted + 1)?;
              return Result.Ok(widened + 1);
            }
            return Result.Err(3);
          }

          fn main() -> i64 {
            let ignored: Result<i64, i64> = encode(4);
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let pure_helpers = super::super::loop_purity::collect_pure_helper_functions(&module);
    assert!(
        pure_helpers.contains("checked__i64"),
        "expected generic try helper to be pure: {pure_helpers:?}"
    );
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn compiler_data_model_helpers_remain_pure() {
    let module = parse_nuis_module(include_str!(
        "../../../../../stdlib/std/lib/language_core.ns"
    ))
    .unwrap();
    let pure_helpers = super::super::loop_purity::collect_pure_helper_functions(&module);

    for expected in [
        "compiler_vector_push",
        "compiler_text_append_byte",
        "compiler_text_push_scalar",
    ] {
        assert!(
            pure_helpers.contains(expected),
            "expected `{expected}` to be pure: {pure_helpers:?}"
        );
    }
}

#[test]
fn carries_match_binding_through_return_chain_after_pure_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            Some(T),
            None,
          }

          fn observe(value: i64) -> Option<i64> {
            return Option.Some(value);
          }

          fn compile_component(seed: i64) -> i64 {
            let observed: i64 = match observe(seed) {
              Option.Some(value) => {
                value
              }
              Option.None => {
                0
              }
            };
            if seed < 0 {
              return 80;
            }
            return observed + 1;
          }

          fn main() -> i64 {
            return compile_component(12);
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
}

#[test]
fn branch_substitution_stops_at_a_nested_shadowing_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 17;
            if true {
              let value: i64 = 0;
              if value != 0 {
                return 1;
              }
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let outer_if = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .and_then(|function| function.body.get(1))
        .expect("outer if statement");
    let substituted = super::super::loop_purity::substitute_stmt_bindings(
        outer_if,
        &[("value".to_owned(), NirExpr::Int(99))],
    );
    let NirStmt::If { then_body, .. } = substituted else {
        panic!("expected substituted outer if");
    };
    let Some(NirStmt::If { condition, .. }) = then_body.get(1) else {
        panic!("expected inner if after shadowing declaration");
    };
    let NirExpr::Binary { lhs, .. } = condition else {
        panic!("expected inner comparison");
    };

    assert!(matches!(lhs.as_ref(), NirExpr::Var(name) if name == "value"));
}

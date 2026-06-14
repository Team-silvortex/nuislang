use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

fn path_exists(yir: &yir_core::YirModule, from: &str, to: &str) -> bool {
    let mut frontier = vec![from.to_owned()];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(current) = frontier.pop() {
        if current == to {
            return true;
        }
        if !seen.insert(current.clone()) {
            continue;
        }
        for edge in &yir.edges {
            if edge.from == current {
                frontier.push(edge.to.clone());
            }
        }
    }
    false
}

#[test]
fn lowers_join_result_and_task_state_primitives_into_cpu_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = timeout(spawn(ping()), 16);
            let result: TaskResult<i64> = join_result(task);
            let done: bool = task_completed(result);
            let timed_out: bool = task_timed_out(result);
            let value: i64 = task_value(result);
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let join_result = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .unwrap();
    let completed = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .unwrap();
    let timed_out = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "task_timed_out")
        .unwrap();
    let value = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .unwrap();

    assert_eq!(completed.op.args, vec![join_result.name.clone()]);
    assert_eq!(timed_out.op.args, vec![join_result.name.clone()]);
    assert_eq!(value.op.args, vec![join_result.name.clone()]);
}

#[test]
fn lowers_thread_and_mutex_primitives_into_cpu_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping());
            let joined: TaskResult<i64> = thread_join_result(worker);
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let value: i64 = mutex_value(guard);
            let unlocked: Mutex<i64> = mutex_unlock(guard);
            if task_completed(joined) {
              return value + thread_join(worker) + mutex_value(mutex_lock(unlocked));
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_new"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_unlock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_pure_branch_local_binding_into_guard_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let usage_text = "usage";
              let usage: String = usage_text;
              let exit_base: i64 = 60;
              let exit_code: i64 = exit_base + 4;
              print(usage);
              return exit_code;
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
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_print_return" }));
}

#[test]
fn lowers_pure_helper_call_binding_into_guard_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn usage_exit_code() -> i64 {
            return 60 + 4;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let usage_text = "usage";
              let usage: String = usage_text;
              let exit_code: i64 = usage_exit_code();
              print(usage);
              return exit_code;
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
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_print_return" }));
}

#[test]
fn lowers_pure_text_helper_call_binding_into_guard_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn usage_message() -> String {
            return "usage";
          }

          fn usage_exit_code() -> i64 {
            return 60 + 4;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let usage: String = usage_message();
              let exit_code: i64 = usage_exit_code();
              print(usage);
              return exit_code;
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
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_print_return" }));
}

#[test]
fn lowers_pure_struct_helper_call_binding_into_branch_print_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          struct ExitSummary {
            message: String,
            code: i64
          }

          fn usage_summary() -> ExitSummary {
            return ExitSummary {
              message: "usage",
              code: 60 + 4
            };
          }

          fn ok_summary() -> ExitSummary {
            return ExitSummary {
              message: "ok",
              code: 0 + 0
            };
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
fn lowers_pure_binding_chain_into_shared_branch_binding_select() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let seed: i64 = 1;
            let result: i64 = 0;
            if seed < 2 {
              let base: i64 = 40;
              let result: i64 = base + 2;
            } else {
              let base: i64 = 10;
              let result: i64 = base + 5;
            }
            return result;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let select_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .expect("expected select node for shared branch binding");
    assert_eq!(select_node.op.args.len(), 3);
}

#[test]
fn lowers_match_expression_with_shared_borrow_lifecycle_into_select_then_owner_write() {
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
                value
              }
              _ => {
                let head_ref: ref Node = borrow(head);
                let value: i64 = head_ref.value + 1;
                borrow_end(head_ref);
                value
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
    assert!(path_exists(&yir, &borrow_ends[0].name, &store_value.name));
}

#[test]
fn lowers_if_expression_with_shared_borrow_lifecycle_into_select_then_owner_write() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let head: ref Node = move(alloc_node(10, null()));
            let current: i64 = if true {
              let head_ref: ref Node = borrow(head);
              let value: i64 = head_ref.value;
              borrow_end(head_ref);
              value
            } else {
              let head_ref: ref Node = borrow(head);
              let value: i64 = head_ref.value + 1;
              borrow_end(head_ref);
              value
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
    let store_value = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "store_value")
        .expect("expected owner store_value after if expression");

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
        "expected select-based branch value lowering for if expression"
    );
    assert!(path_exists(&yir, &borrow_ends[0].name, &store_value.name));
}

#[test]
fn lowers_match_expression_with_thread_result_observation_into_select() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let joined: TaskResult<i64> = thread_join_result(worker);
            let done: bool = task_completed(joined);
            let current: i64 = match done {
              true => {
                1
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

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_match_expression_with_mutex_branch_lifecycle_into_select() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let first: i64 = mutex_value(guard);
            let reopened: Mutex<i64> = mutex_unlock(guard);
            let reopened_guard: MutexGuard<i64> = mutex_lock(reopened);
            let current: i64 = match 1 {
              1 => {
                mutex_value(reopened_guard)
              }
              _ => {
                first + mutex_value(reopened_guard)
              }
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_new"));
    assert!(yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count()
        >= 2);
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_unlock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

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
        mod cpu Main {
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
        mod cpu Main {
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
        adds.len() >= 1,
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
        adds.len() >= 1,
        "expected shared suffix arithmetic to survive match lowering"
    );
}

#[test]
fn lowers_if_expression_with_branch_local_task_observer_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let joined: TaskResult<i64> = thread_join_result(worker);
            let current: i64 = if true {
              let done: bool = task_completed(joined);
              if done {
                let observed: i64 = task_value(joined);
                observed
              } else {
                0
              }
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count()
        >= 2);
}

#[test]
fn lowers_match_expression_with_branch_local_mutex_observer_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let current: i64 = match 1 {
              1 => {
                let observed: i64 = mutex_value(guard);
                observed
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

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_match_expression_with_branch_local_task_observer_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let joined: TaskResult<i64> = thread_join_result(worker);
            let current: i64 = match 1 {
              1 => {
                let done: bool = task_completed(joined);
                if done {
                  task_value(joined)
                } else {
                  0
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

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count()
        >= 2);
}

#[test]
fn reports_effectful_thread_branch_lowering_gap_explicitly() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let current: i64 = if true {
              let joined: TaskResult<i64> = thread_join_result(worker);
              let resolved: i64 = if task_completed(joined) {
                task_value(joined)
              } else {
                0
              };
              resolved
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("conditional `if`/lowered-`match` lowering"));
    assert!(error.contains("consuming task/thread/mutex runtime primitives"));
    assert!(error.contains("join-result"));
    assert!(error.contains("pure/select-compatible values"));
}

#[test]
fn reports_effectful_mutex_branch_lowering_gap_explicitly() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let current: i64 = if true {
              let guard: MutexGuard<i64> = mutex_lock(lock);
              let value: i64 = mutex_value(guard);
              value
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("conditional `if`/lowered-`match` lowering"));
    assert!(error.contains("consuming task/thread/mutex runtime primitives"));
    assert!(error.contains("lock"));
}

#[test]
fn reports_effectful_mutex_unlock_branch_lowering_gap_explicitly() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let current: i64 = if true {
              let reopened: Mutex<i64> = mutex_unlock(guard);
              let reopened_guard: MutexGuard<i64> = mutex_lock(reopened);
              mutex_value(reopened_guard)
            } else {
              0
            };
            return current + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains("conditional `if`/lowered-`match` lowering"));
    assert!(error.contains("consuming task/thread/mutex runtime primitives"));
    assert!(error.contains("unlock"));
}

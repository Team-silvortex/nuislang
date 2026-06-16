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
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
            .count()
            >= 2
    );
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
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
            .count()
            >= 2
    );
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
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
            .count()
            >= 2
    );
}

#[test]
fn lowers_effectful_thread_branch_when_constant_condition_selects_active_path() {
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_effectful_mutex_branch_when_constant_condition_selects_active_path() {
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_mutex_unlock_branch_when_constant_condition_selects_active_path() {
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_unlock"));
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
            .count()
            >= 2
    );
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_thread_match_arm_when_constant_scrutinee_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let worker: Thread<i64> = thread_spawn(ping(5));
            let current: i64 = match 1 {
              1 => {
                let joined: TaskResult<i64> = thread_join_result(worker);
                let resolved: i64 = if task_completed(joined) {
                  task_value(joined)
                } else {
                  0
                };
                resolved
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "select"));
}

#[test]
fn lowers_effectful_mutex_match_arm_when_constant_scrutinee_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let current: i64 = match 1 {
              1 => {
                let guard: MutexGuard<i64> = mutex_lock(lock);
                let value: i64 = mutex_value(guard);
                value
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_new"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_mutex_unlock_match_arm_when_constant_scrutinee_selects_active_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lock: Mutex<i64> = mutex_new(11);
            let guard: MutexGuard<i64> = mutex_lock(lock);
            let current: i64 = match 1 {
              1 => {
                let reopened: Mutex<i64> = mutex_unlock(guard);
                let reopened_guard: MutexGuard<i64> = mutex_lock(reopened);
                mutex_value(reopened_guard)
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_unlock"));
    assert!(
        yir.nodes
            .iter()
            .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
            .count()
            >= 2
    );
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_if_branch_when_constant_binding_controls_condition() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let enabled: bool = 1 < 2;
            let lock: Mutex<i64> = mutex_new(11);
            let current: i64 = if enabled {
              let guard: MutexGuard<i64> = mutex_lock(lock);
              mutex_value(guard)
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_effectful_match_arm_when_constant_binding_controls_scrutinee() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = 1;
            let worker: Thread<i64> = thread_spawn(ping(5));
            let current: i64 = match arm {
              1 => {
                let joined: TaskResult<i64> = thread_join_result(worker);
                let resolved: i64 = if task_completed(joined) {
                  task_value(joined)
                } else {
                  0
                };
                resolved
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
}

// Dynamic branch-local runtime tests are grouped below by the runtime family
// or suffix shape they are defending.
#[test]
fn lowers_dynamic_if_binding_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
    assert!(select_count >= 1, "expected selected mutex input");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_dynamic_if_return_by_selecting_thread_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            let left: Thread<i64> = thread_spawn(ping(5));
            let right: Thread<i64> = thread_spawn(ping(9));
            if argc < 2 {
              return thread_join_result(left);
            } else {
              return thread_join_result(right);
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(select_count >= 1, "expected selected thread input");
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = if argc < 2 {
              let guard: MutexGuard<i64> = mutex_lock(left);
              guard
            } else {
              let guard: MutexGuard<i64> = mutex_lock(right);
              guard
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
    assert!(select_count >= 1, "expected selected mutex input");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value"));
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_thread_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            let left: Thread<i64> = thread_spawn(ping(5));
            let right: Thread<i64> = thread_spawn(ping(9));
            if argc < 2 {
              let joined: TaskResult<i64> = thread_join_result(left);
              return joined;
            } else {
              let joined: TaskResult<i64> = thread_join_result(right);
              return joined;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(select_count >= 1, "expected selected thread input");
}

#[test]
fn lowers_dynamic_if_long_alias_chain_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = if argc < 2 {
              let guard: MutexGuard<i64> = mutex_lock(left);
              let alias0: MutexGuard<i64> = guard;
              let alias1: MutexGuard<i64> = alias0;
              alias1
            } else {
              let guard: MutexGuard<i64> = mutex_lock(right);
              let alias0: MutexGuard<i64> = guard;
              let alias1: MutexGuard<i64> = alias0;
              alias1
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_mutex_input_before_lock() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let chosen: MutexGuard<i64> = match arm {
              0 => {
                let guard: MutexGuard<i64> = mutex_lock(left);
                let alias: MutexGuard<i64> = guard;
                alias
              }
              _ => {
                let guard: MutexGuard<i64> = mutex_lock(right);
                let alias: MutexGuard<i64> = guard;
                alias
              }
            };
            return mutex_value(chosen);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one post-select mutex_lock");
    assert!(select_count >= 1, "expected selected match-arm input");
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_thread_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            let left: Thread<i64> = thread_spawn(ping(5));
            let right: Thread<i64> = thread_spawn(ping(9));
            match arm {
              0 => {
                let joined: TaskResult<i64> = thread_join_result(left);
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = thread_join_result(right);
                let alias: TaskResult<i64> = joined;
                return alias;
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(select_count >= 1, "expected selected match-arm input");
}

// Spawn/task-result branch selection.
#[test]
fn lowers_dynamic_if_binding_by_selecting_spawn_args_before_spawn_task() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              spawn(ping(5))
            } else {
              spawn(ping(9))
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected spawn argument");
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_spawn_args_before_spawn_task() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = spawn(ping(5));
                task
              }
              _ => {
                let task: Task<i64> = spawn(ping(9));
                task
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected spawn argument");
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_spawn_args_before_spawn_task() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              let task: Task<i64> = spawn(ping(5));
              let alias: Task<i64> = task;
              alias
            } else {
              let task: Task<i64> = spawn(ping(9));
              let alias: Task<i64> = task;
              alias
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected spawn argument");
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_spawn_args_before_spawn_task() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = spawn(ping(5));
                let alias: Task<i64> = task;
                alias
              }
              _ => {
                let task: Task<i64> = spawn(ping(9));
                let alias: Task<i64> = task;
                alias
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected spawn argument");
}

// Timeout over task handles, including alias-chain forms.
#[test]
fn lowers_dynamic_if_binding_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(spawn(ping(5)), 16)
            } else {
              timeout(spawn(ping(9)), 32)
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              let task: Task<i64> = timeout(spawn(ping(5)), 16);
              let alias: Task<i64> = task;
              alias
            } else {
              let task: Task<i64> = timeout(spawn(ping(9)), 32);
              let alias: Task<i64> = task;
              alias
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(spawn(ping(5)), 16);
                task
              }
              _ => {
                let task: Task<i64> = timeout(spawn(ping(9)), 32);
                task
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_timeout_inputs_before_timeout() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(spawn(ping(5)), 16);
                let alias: Task<i64> = task;
                alias
              }
              _ => {
                let task: Task<i64> = timeout(spawn(ping(9)), 32);
                let alias: Task<i64> = task;
                alias
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(timeout_count, 1, "expected one post-select timeout");
    assert!(
        select_count >= 2,
        "expected selected task and timeout inputs"
    );
}

// Thread handle production through branch-local selection.
#[test]
fn lowers_dynamic_if_binding_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Thread<i64> = if argc < 2 {
              thread_spawn(ping(5))
            } else {
              thread_spawn(ping(9))
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

#[test]
fn lowers_dynamic_if_binding_chain_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Thread<i64> = if argc < 2 {
              let thread: Thread<i64> = thread_spawn(ping(5));
              let alias: Thread<i64> = thread;
              alias
            } else {
              let thread: Thread<i64> = thread_spawn(ping(9));
              let alias: Thread<i64> = thread;
              alias
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Thread<i64> = match arm {
              0 => {
                let thread: Thread<i64> = thread_spawn(ping(5));
                thread
              }
              _ => {
                let thread: Thread<i64> = thread_spawn(ping(9));
                thread
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

#[test]
fn lowers_dynamic_match_binding_chain_by_selecting_thread_spawn_args_before_spawn_thread() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Thread<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Thread<i64> = match arm {
              0 => {
                let thread: Thread<i64> = thread_spawn(ping(5));
                let alias: Thread<i64> = thread;
                alias
              }
              _ => {
                let thread: Thread<i64> = thread_spawn(ping(9));
                let alias: Thread<i64> = thread;
                alias
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert!(select_count >= 1, "expected selected thread_spawn argument");
}

// Cancel(task) branch-local selection.
#[test]
fn lowers_dynamic_if_binding_by_selecting_cancel_input_before_cancel() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              cancel(spawn(ping(5)))
            } else {
              cancel(spawn(ping(9)))
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(cancel_count, 1, "expected one post-select cancel");
    assert!(select_count >= 1, "expected selected cancel input");
}

#[test]
fn lowers_dynamic_match_binding_by_selecting_cancel_input_before_cancel() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> Task<i64> {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = cancel(spawn(ping(5)));
                task
              }
              _ => {
                let task: Task<i64> = cancel(spawn(ping(9)));
                task
              }
            };
            return chosen;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(cancel_count, 1, "expected one post-select cancel");
    assert!(select_count >= 1, "expected selected cancel input");
}

// Nested result-producing runtime consumers.
#[test]
fn lowers_dynamic_if_return_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return join_result(spawn(ping(5)));
            } else {
              return join_result(spawn(ping(9)));
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return join_result(spawn(ping(5)));
              }
              _ => {
                return join_result(spawn(ping(9)));
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let joined: TaskResult<i64> = join_result(spawn(ping(5)));
              let alias: TaskResult<i64> = joined;
              return alias;
            } else {
              let joined: TaskResult<i64> = join_result(spawn(ping(9)));
              let alias: TaskResult<i64> = joined;
              return alias;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                let joined: TaskResult<i64> = join_result(spawn(ping(5)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = join_result(spawn(ping(9)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_result_count, 1, "expected one post-select join_result");
    assert!(select_count >= 1, "expected selected join_result input");
}

#[test]
fn lowers_dynamic_if_return_by_selecting_spawn_input_before_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return join(spawn(ping(5)));
            } else {
              return join(spawn(ping(9)));
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select join");
    assert!(select_count >= 1, "expected selected join input");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_spawn_input_before_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return join(spawn(ping(5)));
              }
              _ => {
                return join(spawn(ping(9)));
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_task");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select join");
    assert!(select_count >= 1, "expected selected join input");
}

#[test]
fn lowers_dynamic_if_return_by_selecting_thread_spawn_input_before_thread_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return thread_join_result(thread_spawn(ping(5)));
            } else {
              return thread_join_result(thread_spawn(ping(9)));
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_thread_spawn_input_before_thread_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(5)));
              let alias: TaskResult<i64> = joined;
              return alias;
            } else {
              let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(9)));
              let alias: TaskResult<i64> = joined;
              return alias;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

#[test]
fn lowers_dynamic_if_return_by_selecting_thread_spawn_input_before_thread_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return thread_join(thread_spawn(ping(5)));
            } else {
              return thread_join(thread_spawn(ping(9)));
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select thread_join");
    assert!(select_count >= 1, "expected selected thread_join input");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_thread_spawn_input_before_thread_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return thread_join_result(thread_spawn(ping(5)));
              }
              _ => {
                return thread_join_result(thread_spawn(ping(9)));
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_thread_spawn_input_before_thread_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(5)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = thread_join_result(thread_spawn(ping(9)));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one post-select thread_join_result"
    );
    assert!(
        select_count >= 1,
        "expected selected thread_join_result input"
    );
}

// Shared observer and pure-suffix paths for TaskResult<T>.
#[test]
fn lowers_dynamic_if_task_result_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let joined: TaskResult<i64> = if argc < 2 {
              join_result(spawn(ping(5)))
            } else {
              join_result(spawn(ping(9)))
            };
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(
        select_count >= 1,
        "expected select before shared observer suffix"
    );
}

#[test]
fn lowers_dynamic_match_task_result_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let joined: TaskResult<i64> = match arm {
              0 => {
                let result: TaskResult<i64> = join_result(spawn(ping(5)));
                result
              }
              _ => {
                let result: TaskResult<i64> = join_result(spawn(ping(9)));
                result
              }
            };
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(
        select_count >= 1,
        "expected select before shared observer suffix"
    );
}

#[test]
fn lowers_dynamic_if_task_result_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let joined: TaskResult<i64> = if argc < 2 {
              join_result(spawn(ping(5)))
            } else {
              join_result(spawn(ping(9)))
            };
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_task_result_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let joined: TaskResult<i64> = match arm {
              0 => {
                let result: TaskResult<i64> = join_result(spawn(ping(5)));
                result
              }
              _ => {
                let result: TaskResult<i64> = join_result(spawn(ping(9)));
                result
              }
            };
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_if_task_result_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let joined: TaskResult<i64> = if argc < 2 {
              join_result(spawn(ping(5)))
            } else {
              join_result(spawn(ping(9)))
            };
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            let widened: i64 = resolved + 1;
            return widened + 2;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

#[test]
fn lowers_dynamic_match_task_result_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let joined: TaskResult<i64> = match arm {
              0 => {
                let result: TaskResult<i64> = join_result(spawn(ping(5)));
                result
              }
              _ => {
                let result: TaskResult<i64> = join_result(spawn(ping(9)));
                result
              }
            };
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            let widened: i64 = resolved + 1;
            return widened + 2;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

// Shared observer and pure-suffix paths for timeout(task, limit).
#[test]
fn lowers_dynamic_if_timeout_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(spawn(ping(5)), 16)
            } else {
              timeout(spawn(ping(9)), 32)
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_match_timeout_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(spawn(ping(5)), 16);
                task
              }
              _ => {
                let task: Task<i64> = timeout(spawn(ping(9)), 32);
                task
              }
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_if_timeout_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(spawn(ping(5)), 16)
            } else {
              timeout(spawn(ping(9)), 32)
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_timeout_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(spawn(ping(5)), 16);
                task
              }
              _ => {
                let task: Task<i64> = timeout(spawn(ping(9)), 32);
                task
              }
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

// Nested direct-return timeout -> join_result recursion.
#[test]
fn lowers_dynamic_if_return_by_selecting_timeout_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              return join_result(timeout(spawn(ping(5)), 16));
            } else {
              return join_result(timeout(spawn(ping(9)), 32));
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_timeout_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return join_result(timeout(spawn(ping(5)), 16));
              }
              _ => {
                return join_result(timeout(spawn(ping(9)), 32));
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

#[test]
fn lowers_dynamic_if_return_chain_by_selecting_timeout_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let joined: TaskResult<i64> = join_result(timeout(spawn(ping(5)), 16));
              let alias: TaskResult<i64> = joined;
              return alias;
            } else {
              let joined: TaskResult<i64> = join_result(timeout(spawn(ping(9)), 32));
              let alias: TaskResult<i64> = joined;
              return alias;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

#[test]
fn lowers_dynamic_match_return_chain_by_selecting_timeout_spawn_input_before_join_result() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> TaskResult<i64> {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                let joined: TaskResult<i64> = join_result(timeout(spawn(ping(5)), 16));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
              _ => {
                let joined: TaskResult<i64> = join_result(timeout(spawn(ping(9)), 32));
                let alias: TaskResult<i64> = joined;
                return alias;
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
}

// Shared observer and pure-suffix paths for Thread<T>.
#[test]
fn lowers_dynamic_if_thread_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Thread<i64> = if argc < 2 {
              thread_spawn(ping(5))
            } else {
              thread_spawn(ping(9))
            };
            let joined: TaskResult<i64> = thread_join_result(chosen);
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_thread");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one shared thread_join_result"
    );
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_match_thread_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Thread<i64> = match arm {
              0 => {
                let thread: Thread<i64> = thread_spawn(ping(5));
                thread
              }
              _ => {
                let thread: Thread<i64> = thread_spawn(ping(9));
                thread
              }
            };
            let joined: TaskResult<i64> = thread_join_result(chosen);
            if task_completed(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_thread");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one shared thread_join_result"
    );
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_if_thread_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Thread<i64> = if argc < 2 {
              thread_spawn(ping(5))
            } else {
              thread_spawn(ping(9))
            };
            let joined: TaskResult<i64> = thread_join_result(chosen);
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_thread");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one shared thread_join_result"
    );
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_thread_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Thread<i64> = match arm {
              0 => {
                let thread: Thread<i64> = thread_spawn(ping(5));
                thread
              }
              _ => {
                let thread: Thread<i64> = thread_spawn(ping(9));
                thread
              }
            };
            let joined: TaskResult<i64> = thread_join_result(chosen);
            let resolved: i64 = if task_completed(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join_result")
        .count();
    let completed_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_completed")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_thread");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(
        join_result_count, 1,
        "expected one shared thread_join_result"
    );
    assert_eq!(completed_count, 1, "expected one shared task_completed");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

// Shared observer and pure-suffix paths for cancel(task).
#[test]
fn lowers_dynamic_if_cancelled_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              cancel(spawn(ping(5)))
            } else {
              cancel(spawn(ping(9)))
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_cancelled(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_match_cancelled_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = cancel(spawn(ping(5)));
                task
              }
              _ => {
                let task: Task<i64> = cancel(spawn(ping(9)));
                task
              }
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_cancelled(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_if_cancelled_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              cancel(spawn(ping(5)))
            } else {
              cancel(spawn(ping(9)))
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_cancelled(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_cancelled_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = cancel(spawn(ping(5)));
                task
              }
              _ => {
                let task: Task<i64> = cancel(spawn(ping(9)));
                task
              }
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_cancelled(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

// Mixed nested binary(unary(call)) recursion: timeout(cancel(spawn(...)), limit).
#[test]
fn lowers_dynamic_if_timeout_cancelled_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(cancel(spawn(ping(5))), 16)
            } else {
              timeout(cancel(spawn(ping(9))), 32)
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_cancelled(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_match_timeout_cancelled_task_binding_into_shared_result_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(cancel(spawn(ping(5))), 16);
                task
              }
              _ => {
                let task: Task<i64> = timeout(cancel(spawn(ping(9))), 32);
                task
              }
            };
            let joined: TaskResult<i64> = join_result(chosen);
            if task_cancelled(joined) {
              return task_value(joined);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
}

#[test]
fn lowers_dynamic_if_timeout_cancelled_task_binding_into_shared_result_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let chosen: Task<i64> = if argc < 2 {
              timeout(cancel(spawn(ping(5))), 16)
            } else {
              timeout(cancel(spawn(ping(9))), 32)
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_cancelled(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_timeout_cancelled_task_binding_into_shared_result_observer_and_pure_suffix()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let chosen: Task<i64> = match arm {
              0 => {
                let task: Task<i64> = timeout(cancel(spawn(ping(5))), 16);
                task
              }
              _ => {
                let task: Task<i64> = timeout(cancel(spawn(ping(9))), 32);
                task
              }
            };
            let joined: TaskResult<i64> = join_result(chosen);
            let resolved: i64 = if task_cancelled(joined) {
              task_value(joined)
            } else {
              0
            };
            return resolved + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let cancel_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "cancel")
        .count();
    let timeout_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "timeout")
        .count();
    let join_result_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "join_result")
        .count();
    let cancelled_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "task_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(spawn_count, 1, "expected one shared spawn_task");
    assert_eq!(async_call_count, 1, "expected one shared async_call");
    assert_eq!(cancel_count, 1, "expected one shared cancel");
    assert_eq!(timeout_count, 1, "expected one shared timeout");
    assert_eq!(join_result_count, 1, "expected one shared join_result");
    assert_eq!(cancelled_count, 1, "expected one shared task_cancelled");
    assert_eq!(value_count, 1, "expected one shared task_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

// Shared observer and pure-suffix paths for MutexGuard<T>.
#[test]
fn lowers_dynamic_if_mutex_guard_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            return mutex_value(guard);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(
        select_count >= 1,
        "expected select before shared mutex observer"
    );
}

#[test]
fn lowers_dynamic_if_mutex_guard_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            let value: i64 = mutex_value(guard);
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_if_mutex_guard_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = if argc < 2 {
              mutex_lock(left)
            } else {
              mutex_lock(right)
            };
            let value: i64 = mutex_value(guard);
            let widened: i64 = value + 1;
            return widened + 2;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

#[test]
fn lowers_dynamic_match_mutex_guard_binding_into_shared_observer_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = match arm {
              0 => {
                let locked: MutexGuard<i64> = mutex_lock(left);
                locked
              }
              _ => {
                let locked: MutexGuard<i64> = mutex_lock(right);
                locked
              }
            };
            return mutex_value(guard);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(
        select_count >= 1,
        "expected select before shared mutex observer"
    );
}

#[test]
fn lowers_dynamic_match_mutex_guard_binding_into_shared_observer_and_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = match arm {
              0 => {
                let locked: MutexGuard<i64> = mutex_lock(left);
                locked
              }
              _ => {
                let locked: MutexGuard<i64> = mutex_lock(right);
                locked
              }
            };
            let value: i64 = mutex_value(guard);
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 1, "expected shared pure suffix add");
}

#[test]
fn lowers_dynamic_match_mutex_guard_binding_into_two_stage_shared_pure_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            let left: Mutex<i64> = mutex_new(11);
            let right: Mutex<i64> = mutex_new(19);
            let guard: MutexGuard<i64> = match arm {
              0 => {
                let locked: MutexGuard<i64> = mutex_lock(left);
                locked
              }
              _ => {
                let locked: MutexGuard<i64> = mutex_lock(right);
                locked
              }
            };
            let value: i64 = mutex_value(guard);
            let widened: i64 = value + 1;
            return widened + 2;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lock_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_lock")
        .count();
    let value_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "mutex_value")
        .count();
    let add_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "add")
        .count();

    assert_eq!(lock_count, 1, "expected one shared mutex_lock");
    assert_eq!(value_count, 1, "expected one shared mutex_value");
    assert!(add_count >= 2, "expected two-stage shared pure suffix adds");
}

#[test]
fn lowers_dynamic_match_return_by_selecting_thread_spawn_input_before_thread_join() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn main() -> i64 {
            let arm: i64 = host_argv_count();
            match arm {
              0 => {
                return thread_join(thread_spawn(ping(5)));
              }
              _ => {
                return thread_join(thread_spawn(ping(9)));
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_thread")
        .count();
    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let join_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "thread_join")
        .count();
    let select_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();

    assert_eq!(spawn_count, 1, "expected one post-select spawn_thread");
    assert_eq!(async_call_count, 1, "expected one post-select async_call");
    assert_eq!(join_count, 1, "expected one post-select thread_join");
    assert!(select_count >= 1, "expected selected thread_join input");
}

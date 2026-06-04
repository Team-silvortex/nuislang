use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_branch_local_binding_into_pure_helper_param_chain() {
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

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let base_usage: String = usage_message();
              let summary: ExitSummary = render_summary(pass_text(base_usage), usage_exit_code());
              print(summary.message);
              return summary.code;
            } else {
              let base_ok: String = ok_message();
              let summary: ExitSummary = render_summary(pass_text(base_ok), ok_exit_code());
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
fn lowers_multi_step_summary_helpers_inside_branch() {
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

          fn attach_message(summary: ExitSummary, message: String) -> ExitSummary {
            return ExitSummary {
              message: message,
              code: summary.code
            };
          }

          fn attach_code(summary: ExitSummary, code: i64) -> ExitSummary {
            return ExitSummary {
              message: summary.message,
              code: code
            };
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let base_usage: String = usage_message();
              let empty_summary: ExitSummary = render_summary("", 0);
              let message_summary: ExitSummary = attach_message(empty_summary, pass_text(base_usage));
              let summary: ExitSummary = attach_code(message_summary, usage_exit_code());
              print(summary.message);
              return summary.code;
            } else {
              let base_ok: String = ok_message();
              let empty_summary: ExitSummary = render_summary("", 0);
              let message_summary: ExitSummary = attach_message(empty_summary, pass_text(base_ok));
              let summary: ExitSummary = attach_code(message_summary, ok_exit_code());
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
fn lowers_shared_branch_bindings_into_multiple_pure_helpers() {
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

          fn attach_message(summary: ExitSummary, message: String) -> ExitSummary {
            return ExitSummary {
              message: message,
              code: summary.code
            };
          }

          fn attach_code(summary: ExitSummary, code: i64) -> ExitSummary {
            return ExitSummary {
              message: summary.message,
              code: code
            };
          }

          fn amplify_code(base: i64) -> i64 {
            return base + 0;
          }

          fn decorate_message(message: String) -> String {
            return pass_text(message);
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let base_usage: String = usage_message();
              let base_code: i64 = usage_exit_code();
              let empty_summary: ExitSummary = render_summary("", 0);
              let message_summary: ExitSummary = attach_message(empty_summary, decorate_message(base_usage));
              let summary: ExitSummary = attach_code(message_summary, amplify_code(base_code));
              print(summary.message);
              return summary.code;
            } else {
              let base_ok: String = ok_message();
              let base_code: i64 = ok_exit_code();
              let empty_summary: ExitSummary = render_summary("", 0);
              let message_summary: ExitSummary = attach_message(empty_summary, decorate_message(base_ok));
              let summary: ExitSummary = attach_code(message_summary, amplify_code(base_code));
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
fn lowers_branch_local_task_value_binding_before_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(seed: i64) -> i64 {
            return seed + 7;
          }

          struct SampleSummary {
            value: i64,
            branch: i64
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(ping(5));
            let result: TaskResult<i64> = join_result(task);
            if task_completed(result) {
              let observed: i64 = task_value(result);
              return SampleSummary {
                value: observed,
                branch: 1
              }.value;
            } else {
              return 0;
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
}

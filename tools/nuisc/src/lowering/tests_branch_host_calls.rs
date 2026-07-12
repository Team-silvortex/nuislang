use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_let_bound_host_call_chain_into_guard_host_call_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;
          extern "c" fn host_stdout_write(text_handle: i64) -> i64;
          extern "c" fn host_stdout_flush() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let wrote: i64 = host_stdout_write(1901);
              let flushed: i64 = host_stdout_flush();
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let guard = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "guard_host_call_return")
        .expect("expected branch-local host call chain guard");
    assert_eq!(guard.op.args[1], "value");
    assert_eq!(guard.op.args[4], "2");
    assert!(guard
        .op
        .args
        .windows(2)
        .any(|window| window == ["host_stdout_flush", "0"]));
}

#[test]
fn lowers_host_call_chain_write_flush_exit_code_as_computed_guard_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;
          extern "c" fn host_stdout_write(text_handle: i64) -> i64;
          extern "c" fn host_stdout_flush() -> i64;

          fn write_flush_exit_code(write_count: i64, flush_status: i64) -> i64 {
            if write_count < 0 {
              return 1;
            }
            if flush_status < 0 {
              return 1;
            }
            return 0;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let wrote: i64 = host_stdout_write(1901);
              let flushed: i64 = host_stdout_flush();
              return write_flush_exit_code(wrote, flushed);
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let guard = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "guard_host_call_return")
        .expect("expected branch-local computed host call return");
    assert_eq!(guard.op.args[1], "write_flush_exit_code");
    assert_eq!(guard.op.args[3], "wrote");
    assert_eq!(guard.op.args[4], "flushed");
    assert_eq!(guard.op.args[5], "0");
}

#[test]
fn lowers_diag_host_call_chain_into_guard_host_call_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;
          extern "c" fn host_diag_label(message_handle: i64) -> i64;
          extern "c" fn host_diag_span(start: i64, end: i64) -> i64;
          extern "c" fn host_diag_emit(label_handle: i64, span_handle: i64, severity: i64) -> i64;
          extern "c" fn host_stderr_write(text_handle: i64) -> i64;
          extern "c" fn host_stderr_flush() -> i64;

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc < 2 {
              let diag: i64 = host_diag_emit(1901, 4, 2);
              let wrote: i64 = host_stderr_write(1903);
              let flushed: i64 = host_stderr_flush();
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let guard = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "guard_host_call_return")
        .expect("expected guarded diagnostic host call chain");
    assert!(guard
        .op
        .args
        .windows(2)
        .any(|window| window == ["host_diag_emit", "3"]));
    assert!(guard
        .op
        .args
        .windows(2)
        .any(|window| window == ["host_stderr_flush", "0"]));
}

#[test]
fn lowers_two_way_host_call_branches_into_branch_host_call_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;
          extern "c" fn host_stdout_write(text_handle: i64) -> i64;
          extern "c" fn host_stdout_flush() -> i64;
          extern "c" fn host_stderr_write(text_handle: i64) -> i64;
          extern "c" fn host_stderr_flush() -> i64;

          fn write_flush_exit_code(write_count: i64, flush_status: i64) -> i64 {
            if write_count < 0 {
              return 1;
            }
            if flush_status < 0 {
              return 1;
            }
            return 0;
          }

          fn main() -> i64 {
            let argc: i64 = host_argv_count();
            if argc > 1 {
              let wrote: i64 = host_stdout_write(1901);
              let flushed: i64 = host_stdout_flush();
              return write_flush_exit_code(wrote, flushed);
            } else {
              let err_wrote: i64 = host_stderr_write(1903);
              let err_flushed: i64 = host_stderr_flush();
              return write_flush_exit_code(err_wrote, err_flushed) + 1;
            }
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let branch = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "branch_host_call_return")
        .expect("expected two-way branch-local host call return");
    assert_eq!(branch.op.args[1], "write_flush_exit_code");
    assert!(branch.op.args.contains(&"host_stderr_write".to_owned()));
    assert!(branch.op.args.contains(&"1".to_owned()));
}

use super::*;

#[test]
fn rejects_spawn_of_sync_function() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn ping() -> i64 {
            return 7;
          }

          fn main() {
            let task: Task<i64> = spawn(ping());
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("spawn(...) expects async function call"));
}

#[test]
fn rejects_join_of_non_task_value() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return join(7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("expects `Task<...>`"));
}

#[test]
fn rejects_spawn_of_borrowed_input() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(head_ref: ref Node) -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let head: ref Node = alloc_node(1, null());
            let task: Task<i64> = spawn(ping(borrow(head)));
            return join(task);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("does not currently allow borrowed task inputs"));
}

#[test]
fn rejects_spawn_of_ref_typed_input() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(head: ref Node) -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let head: ref Node = alloc_node(1, null());
            let task: Task<i64> = spawn(ping(head));
            return join(task);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("does not currently allow `ref` task inputs"));
}

#[test]
fn rejects_async_function_ref_parameter_boundary() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping(head: ref Node) -> i64 {
            return 7;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot cross async boundary"));
    assert!(error.contains("`Task<...>`"));
}

#[test]
fn rejects_async_function_result_family_return_boundary() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> TaskResult<i64> {
            return join_result(timeout(spawn(pong()), 16));
          }

          async fn pong() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot return `TaskResult<i64>` across async boundary"));
    assert!(error.contains("*Result<...>"));
}

#[test]
fn accepts_host_buffer_handle_to_extern_i64_host_handle_bridge() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_stdin_read(buffer_handle: i64, len: i64) -> i64;

          fn main() -> i64 {
            let backing: ref Buffer = alloc_buffer(8, 0);
            return host_stdin_read(host_buffer_handle(backing), 8);
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.functions.len(), 1);
}

#[test]
fn rejects_ref_node_to_extern_i64_host_handle_slot() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_stdin_read(buffer_handle: i64, len: i64) -> i64;

          fn main() -> i64 {
            let head: ref Node = alloc_node(1, null());
            return host_stdin_read(head, 8);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("function `host_stdin_read` argument 1 expects `i64`, found `ref Node`"));
    assert!(error.contains("`ref Buffer -> i64`"));
}

#[test]
fn accepts_ref_buffer_parameter_to_extern_host_handle_bridge() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_stdin_read(buffer: ref Buffer, len: i64) -> i64;

          fn main() -> i64 {
            let backing: ref Buffer = alloc_buffer(8, 0);
            return host_stdin_read(backing, 8);
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::CpuExternCall { args, .. })))
            if matches!(args.first(), Some(NirExpr::HostBufferHandle(_)))
    ));
}

#[test]
fn rejects_task_completed_on_raw_task_input() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> bool {
            let task: Task<i64> = spawn(ping());
            return task_completed(task);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("task_completed(...) expects `TaskResult<...>`"));
    assert!(error.contains("found `Task<i64>`"));
}

#[test]
fn rejects_task_value_on_join_payload_input() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(ping());
            let value: i64 = join(task);
            return task_value(value);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("task_value(...) expects `TaskResult<...>`"));
    assert!(error.contains("found `i64`"));
}

#[test]
fn rejects_network_style_sync_summary_calling_async_helper_directly() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct NetHttpClientExchangeSummary {
            exchange_value: i64,
          }

          struct NetSessionSummary {
            summary: NetHttpClientExchangeSummary,
            session_value: i64,
          }

          async fn capture_net_http_client_exchange_summary() -> NetHttpClientExchangeSummary {
            return NetHttpClientExchangeSummary { exchange_value: 41 };
          }

          fn capture_net_session_summary() -> NetSessionSummary {
            return NetSessionSummary {
              summary: capture_net_http_client_exchange_summary(),
              session_value: 99,
            };
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("can only be called inside `async fn`"));
    assert!(error.contains("capture_net_http_client_exchange_summary"));
}

#[test]
fn rejects_network_style_spawn_of_sync_summary_builder() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct NetSessionSummary {
            session_value: i64,
          }

          fn capture_net_session_summary() -> NetSessionSummary {
            return NetSessionSummary { session_value: 99 };
          }

          fn main() -> i64 {
            let task: Task<NetSessionSummary> = spawn(capture_net_session_summary());
            return join(task).session_value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("spawn(...) expects async function call"));
    assert!(error.contains("found sync function `capture_net_session_summary`"));
}

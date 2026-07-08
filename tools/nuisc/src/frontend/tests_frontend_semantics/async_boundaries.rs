use super::*;

#[test]
fn rejects_await_inside_sync_function() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn ping() -> i64 {
            return 7;
          }

          fn main() {
            await ping();
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("`await`"));
    assert!(error.contains("async fn"));
}

#[test]
fn rejects_async_function_returning_ref_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn head() -> ref Node {
            return null();
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot return"));
    assert!(error.contains("ref Node"));
    assert!(error.contains("async boundary"));
}

#[test]
fn rejects_async_function_returning_result_family() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn main() -> DataResult<i64> {
            return data_result(data_input_pipe(data_output_pipe(7)));
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("DataResult<i64>"));
    assert!(error.contains("async boundary"));
}

#[test]
fn rejects_async_function_taking_instance_param() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn render(shader: Instance<SurfaceShader>) {
            return;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `shader`"));
    assert!(error.contains("Instance<SurfaceShader>"));
    assert!(error.contains("async boundary"));
}

#[test]
fn accepts_async_function_taking_shader_result_family_param() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume(result: ShaderResult<Frame>) -> i64 {
            if shader_frame_ready(result) {
              return 1;
            }
            return 0;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "consume")
        .unwrap();
    assert_eq!(function.params[0].ty.render(), "ShaderResult<Frame>");
}

#[test]
fn accepts_async_function_taking_kernel_result_family_param() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) {
              return kernel_value(result);
            }
            return 0;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "consume")
        .unwrap();
    assert_eq!(function.params[0].ty.render(), "KernelResult<i64>");
}

#[test]
fn rejects_async_function_taking_struct_with_nested_ref_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct RefPacket {
            head: ref Node
          }

          async fn consume(packet: RefPacket) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `packet`"));
    assert!(error.contains("RefPacket"));
    assert!(error.contains("nested field `RefPacket.head`"));
    assert!(error.contains("ref Node"));
    assert!(error.contains("async boundary"));
}

#[test]
fn rejects_async_function_returning_struct_with_nested_ref_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct RefPacket {
            head: ref Node
          }

          async fn emit() -> RefPacket {
            return RefPacket { head: null() };
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot return `RefPacket` across async boundary"));
    assert!(error.contains("nested field `RefPacket.head`"));
    assert!(error.contains("ref Node"));
}

#[test]
fn rejects_async_function_taking_struct_with_nested_optional_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct OptionalPacket {
            value: i64?
          }

          async fn consume(packet: OptionalPacket) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `packet`"));
    assert!(error.contains("OptionalPacket"));
    assert!(error.contains("nested field `OptionalPacket.value`"));
    assert!(error.contains("i64?"));
}

#[test]
fn rejects_async_function_taking_struct_with_nested_instance_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct ShaderPacket {
            shader: Instance<SurfaceShader>
          }

          async fn consume(packet: ShaderPacket) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `packet`"));
    assert!(error.contains("ShaderPacket"));
    assert!(error.contains("nested field `ShaderPacket.shader`"));
    assert!(error.contains("Instance<SurfaceShader>"));
}

#[test]
fn rejects_async_function_taking_struct_with_nested_result_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct ResultPacket {
            result: TaskResult<i64>
          }

          async fn consume(packet: ResultPacket) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `packet`"));
    assert!(error.contains("ResultPacket"));
    assert!(error.contains("nested field `ResultPacket.result`"));
    assert!(error.contains("TaskResult<i64>"));
}

#[test]
fn rejects_async_function_taking_window_param() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume(window: Window<i64>) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `window`"));
    assert!(error.contains("Window<i64>"));
    assert!(error.contains("resource-bearing"));
}

#[test]
fn accepts_sync_function_declaring_staged_thread_and_mutex_types() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn hold(worker: Thread<i64>, lock: Mutex<i64>, guard: MutexGuard<i64>) -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "hold")
        .unwrap();
    assert_eq!(function.params[0].ty.render(), "Thread<i64>");
    assert_eq!(function.params[1].ty.render(), "Mutex<i64>");
    assert_eq!(function.params[2].ty.render(), "MutexGuard<i64>");
}

#[test]
fn rejects_async_function_taking_thread_param() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume(worker: Thread<i64>) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `worker`"));
    assert!(error.contains("Thread<i64>"));
    assert!(error.contains("async boundary"));
}

#[test]
fn rejects_async_function_taking_mutex_param() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume(lock: Mutex<i64>) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `lock`"));
    assert!(error.contains("Mutex<i64>"));
    assert!(error.contains("async boundary"));
}

#[test]
fn rejects_async_function_taking_struct_with_nested_marker_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct MarkerPacket {
            ready: Marker<CpuToShader>
          }

          async fn consume(packet: MarkerPacket) -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameter `packet`"));
    assert!(error.contains("MarkerPacket"));
    assert!(error.contains("nested field `MarkerPacket.ready`"));
    assert!(error.contains("resource-bearing `Marker<CpuToShader>`"));
}

#[test]
fn allows_async_function_taking_nested_scalar_struct_payload() {
    parse_nuis_module(
        r#"
        mod cpu Main {
          struct ScalarPair {
            lhs: i64,
            rhs: i64
          }

          struct NestedPacket {
            pair: ScalarPair,
            bias: i64
          }

          async fn add(packet: NestedPacket) -> i64 {
            return packet.pair.lhs + packet.pair.rhs + packet.bias;
          }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn allows_async_function_taking_nested_text_struct_payload() {
    parse_nuis_module(
        r#"
        mod cpu Main {
          struct MessagePacket {
            message: String
          }

          struct LabeledMessage {
            packet: MessagePacket,
            label: String
          }

          async fn show(input: LabeledMessage) -> i64 {
            return 5;
          }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn rejects_async_shader_function_for_now() {
    let error = parse_nuis_module(
        r#"
        mod shader SurfaceShader {
          async fn profile() {
            return;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("mod shader SurfaceShader"));
    assert!(error.contains("async fn profile"));
    assert!(error.contains("only supported in `mod cpu`"));
}

#[test]
fn rejects_async_data_function_for_now() {
    let error = parse_nuis_module(
        r#"
        mod data FabricPlane {
          async fn profile() {
            return;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("mod data FabricPlane"));
    assert!(error.contains("only supported in `mod cpu`"));
}

#[test]
fn rejects_async_kernel_function_for_now() {
    let error = parse_nuis_module(
        r#"
        mod kernel KernelUnit {
          async fn profile() {
            return;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("mod kernel KernelUnit"));
    assert!(error.contains("only supported in `mod cpu`"));
}

#[test]
fn rejects_async_main_with_parameters() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn main(seed: i64) {
            print(seed);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("async entry"));
    assert!(error.contains("Main::main"));
    assert!(error.contains("cannot take parameters"));
}

#[test]
fn rejects_async_call_without_await() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          async fn main() -> i64 {
            return ping();
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("must be used under `await`"));
}

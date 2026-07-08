use super::*;

#[test]
fn monomorphizes_generic_function_from_pipe_shaped_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn roundtrip_pipe<T>(pipe: Pipe<T>) -> T {
            return data_input_pipe(pipe);
          }

          fn main() -> i64 {
            return roundtrip_pipe(data_output_pipe(7));
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "roundtrip_pipe__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "roundtrip_pipe__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("Pipe<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_window_shaped_argument_and_expected_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep_window<T>(window: Window<T>) -> Window<T> {
            return window;
          }

          fn main() -> i64 {
            let frozen: Window<i64> = keep_window(data_freeze_window(data_copy_window(7, 0, 1)));
            return data_read_window(frozen, 0);
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
        main.body.first(),
        Some(NirStmt::Let { value: NirExpr::Call { callee, .. }, .. })
            if callee == "keep_window__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_window__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("Window<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Window<i64>"
    ));
}

#[test]
fn monomorphizes_generic_function_from_task_shaped_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn keep_task<T>(task: Task<T>) -> Task<T> {
            return task;
          }

          fn main() -> i64 {
            let task: Task<i64> = keep_task(spawn(ping()));
            return join(task);
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
        main.body.first(),
        Some(NirStmt::Let { value: NirExpr::Call { callee, .. }, .. })
            if callee == "keep_task__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_task__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("Task<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Task<i64>"
    ));
}

#[test]
fn monomorphizes_generic_function_from_data_result_shaped_argument() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn keep_data<T>(result: DataResult<T>) -> DataResult<T> {
            return result;
          }

          fn main() -> i64 {
            let result: DataResult<i64> = keep_data(data_result(data_input_pipe(data_output_pipe(7))));
            return data_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let module = lower_ast_to_nir(&ast).unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(main.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        } if name == "result" && callee == "keep_data__i64"
    )));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_data__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("DataResult<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "DataResult<i64>"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_async_function_from_await_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn typed_zero<T>() -> T {
            return 0;
          }

          async fn main() -> i64 {
            return await typed_zero();
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
        Some(NirStmt::Return(Some(NirExpr::Await(value))))
            if matches!(
                value.as_ref(),
                NirExpr::Call { callee, .. } if callee == "typed_zero__i64"
            )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_async_function_through_await_into_alias_payload_call_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          async fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_payload(value: Just<i64>) -> i64 {
            return value.value;
          }

          async fn main() -> i64 {
            return takes_payload(JustAlias(await typed_zero()));
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "takes_payload"
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { fields, .. }]
                        if matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Await(value))]
                                if field == "value"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Call { callee, .. } if callee == "typed_zero__i64"
                                    )
                        )
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn monomorphizes_generic_function_from_data_result_shaped_argument_in_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep_data<T>(result: DataResult<T>) -> DataResult<T> {
            return result;
          }

          fn produce() -> DataResult<i64> {
            return keep_data(data_result(data_input_pipe(data_output_pipe(7))));
          }

          fn main() -> i64 {
            return data_value(produce());
          }
        }
        "#,
    )
    .unwrap();

    let produce = module
        .functions
        .iter()
        .find(|function| function.name == "produce")
        .unwrap();
    assert!(matches!(
        produce.body.first(),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::DataResult { .. },
        }) if name == "__nuis_generic_return_arg_0" && ty.render() == "DataResult<i64>"
    ));
    assert!(matches!(
        produce.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "keep_data__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_nested_alias_shaped_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Frozen<T> = Window<T>;
          type Wrapped<T> = DataResult<Frozen<T>>;

          fn keep_wrapped<T>(wrapped: Wrapped<T>) -> Wrapped<T> {
            return wrapped;
          }

          fn main() -> i64 {
            let wrapped: Wrapped<i64> =
              keep_wrapped(data_result(data_freeze_window(data_copy_window(7, 0, 1))));
            return data_read_window(data_value(wrapped), 0);
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
    assert!(main.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        } if name == "wrapped" && callee == "keep_wrapped__i64"
    )));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_wrapped__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("DataResult<Window<i64>>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "DataResult<Window<i64>>"
    ));
}

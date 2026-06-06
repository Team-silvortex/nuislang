use super::lower_ast_to_nir;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn monomorphizes_generic_function_call_into_concrete_nir_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs.add(rhs);
          }

          fn main() -> i64 {
            return sum_two(1, 2);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "sum_two__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "sum_two__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "impl.Addable.for.i64.add"
    ));
}

#[test]
fn monomorphizes_multi_generic_function_call_into_concrete_nir_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Keepable {
            fn keep(lhs: Self, rhs: Self) -> Self;
          }

          impl Keepable for i64 {
            fn keep(lhs: i64, rhs: i64) -> i64 {
              return lhs;
            }
          }

          impl Keepable for bool {
            fn keep(lhs: bool, rhs: bool) -> bool {
              return rhs;
            }
          }

          fn choose_second<A: Keepable, B: Keepable>(a0: A, a1: A, b0: B, b1: B) -> B {
            return b0.keep(b1);
          }

          fn main() -> bool {
            return choose_second(1, 2, true, false);
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
            if callee == "choose_second__i64__bool"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_second__i64__bool")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.name.as_str()),
        Some("bool")
    ));
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "impl.Keepable.for.bool.keep"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_local_type_annotation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let value: i64 = typed_zero();
            return value;
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
            if callee == "typed_zero__i64"
    ));
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.name.as_str()),
        Some("i64")
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            return typed_zero();
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
            if callee == "typed_zero__i64"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_nested_call_parameter_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_i64(value: i64) -> i64 {
            return value;
          }

          fn main() -> i64 {
            return takes_i64(typed_zero());
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
            if callee == "takes_i64"
                && matches!(args.as_slice(), [NirExpr::Call { callee, .. }] if callee == "typed_zero__i64")
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_struct_field_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: typed_zero() };
            return boxed.value;
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
        Some(NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        }) if matches!(
            fields.as_slice(),
            [(field, NirExpr::Call { callee, .. })] if field == "value" && callee == "typed_zero__i64"
        )
    ));
}

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
    assert!(matches!(
        main.body.iter().find(|stmt| matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "result" && callee == "keep_data__i64"
        )),
        Some(_)
    ));

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
    assert!(matches!(
        main.body.iter().find(|stmt| matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "wrapped" && callee == "keep_wrapped__i64"
        )),
        Some(_)
    ));

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

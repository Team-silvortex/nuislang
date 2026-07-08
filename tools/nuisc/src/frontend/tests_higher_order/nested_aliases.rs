use super::*;

#[test]
fn lowers_option_map_with_direct_some_constructor_and_generic_named_nested_result_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Option<T> {
            None,
            Some(T),
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn wrap_ok<T, E>(value: T) -> Result<T, E> {
            return Result.Ok(value);
          }

          fn option_map<T, R>(value: Option<T>, mapper: Fn1<T, R>) -> Option<R> {
            match value {
              Option.Some(payload) => {
                return Option.Some(mapper(payload));
              }
              Option.None => {
                return Option.None;
              }
            }
          }

          fn main() -> i64 {
            let mapped: Option<Result<i64, CoreError>> = option_map(Option.Some(7), wrap_ok);
            match mapped {
              Option.Some(Result.Ok(value)) => {
                return value;
              }
              _ => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_option_map_wrap_ok__i64__Result_i64__CoreError_")
        .expect("expected monomorphized option_map helper for nested result callable");
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "wrap_ok__i64__Result_i64__CoreError_")
        .expect("expected generic nested result callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_result_and_then_with_direct_ok_constructor_and_generic_named_result_task_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            Invalid,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          async fn emit<T>(value: T) -> T {
            return value;
          }

          fn spawn_wrap<T, E>(value: T) -> Result<Task<T>, E> {
            return Result.Ok(spawn(emit(value)));
          }

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return mapper(value);
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          async fn main() -> Result<i64, Error> {
            let task: Task<i64> = result_and_then(Result.Ok(7), spawn_wrap)?;
            let value: i64 = await task;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_result_and_then_spawn_wrap__i64__Task_i64___Error")
        .expect("expected monomorphized result_and_then helper for result-task callable");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Task<i64>, Error>"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "spawn_wrap__i64__Task_i64___Error")
        .expect("expected generic result-task callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_option_map_with_direct_some_constructor_and_alias_chain_nested_result_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Option<T> {
            None,
            Some(T),
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          type AppError = CoreError;
          type AppResult<T> = Result<T, AppError>;

          fn wrap_app_ok<T>(value: T) -> AppResult<T> {
            return Result.Ok(value);
          }

          fn option_map<T, R>(value: Option<T>, mapper: Fn1<T, R>) -> Option<R> {
            match value {
              Option.Some(payload) => {
                return Option.Some(mapper(payload));
              }
              Option.None => {
                return Option.None;
              }
            }
          }

          fn main() -> i64 {
            let mapped: Option<AppResult<i64>> = option_map(Option.Some(7), wrap_app_ok);
            match mapped {
              Option.Some(Result.Ok(value)) => {
                return value;
              }
              _ => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_option_map_wrap_app_ok__")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Option<Result<i64, CoreError>>"
                )
        })
        .expect("expected monomorphized option_map helper for alias-chain nested result callable");
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("wrap_app_ok__")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Result<i64, CoreError>"
                )
        })
        .expect("expected generic alias-chain nested result callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_result_and_then_with_direct_ok_constructor_and_alias_chain_result_task_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            Invalid,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          type AppError = Error;
          type AsyncResult<T> = Result<Task<T>, AppError>;

          async fn emit<T>(value: T) -> T {
            return value;
          }

          fn spawn_wrap_alias<T>(value: T) -> AsyncResult<T> {
            return Result.Ok(spawn(emit(value)));
          }

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return mapper(value);
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          async fn main() -> Result<i64, Error> {
            let task: Task<i64> = result_and_then(Result.Ok(7), spawn_wrap_alias)?;
            let value: i64 = await task;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_result_and_then_spawn_wrap_alias__")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Result<Task<i64>, Error>"
                )
        })
        .expect(
            "expected monomorphized result_and_then helper for alias-chain result-task callable",
        );
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("spawn_wrap_alias__")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Result<Task<i64>, Error>"
                )
        })
        .expect("expected generic alias-chain result-task callable specialization");
    assert!(specialized.generic_params.is_empty());
}

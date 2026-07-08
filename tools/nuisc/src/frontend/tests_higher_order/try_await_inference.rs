use super::*;

#[test]
fn lowers_higher_order_result_and_then_without_explicit_lambda_return_type_in_try_result_context() {
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

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
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

          fn ok_branch(x: i64) -> Result<Outer<i64, String>, Error> {
            return Result.Ok(Outer {
              inner: Phantom { value: x, tag: 1 },
              meta: "ok",
            });
          }

          fn fallback_branch(x: i64) -> Result<Outer<i64, String>, Error> {
            return Result.Ok(Outer {
              inner: Phantom { value: x, tag: 2 },
              meta: "fallback",
            });
          }

          fn main() -> Result<i64, Error> {
            let input: Result<i64, Error> = Result.Ok(7);
            let outer: Outer<i64, String> = result_and_then(input, |x: i64| {
              if x == 7 {
                ok_branch(x)
              } else {
                fallback_branch(x)
              }
            })?;
            return Result.Ok(outer.inner.value);
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_result_and_then_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Outer<i64, String>, Error>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_main_")
                && function.name.contains("Outer_i64__String___Error")
        })
        .expect("expected specialized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Outer<i64, String>, Error>"
    ));
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_in_await_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          fn enqueue<T, R>(value: T, mapper: Fn1<T, Task<R>>) -> Task<R> {
            return mapper(value);
          }

          async fn main() -> i64 {
            return await enqueue(7, |x: i64| {
              spawn(work(x))
            });
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_enqueue_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Task<i64>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_main_") && function.name.ends_with("__i64__i64")
        })
        .expect("expected specialized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Task<i64>"
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(value))))
            if matches!(value.as_ref(), NirExpr::Call { callee, .. } if callee == &hof.name)
    ));
}

#[test]
fn lowers_higher_order_result_and_then_without_explicit_lambda_return_type_in_try_await_chain() {
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

          async fn work(seed: i64) -> i64 {
            return seed + 1;
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

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            return Result.Ok(spawn(work(seed)));
          }

          async fn main() -> Result<i64, Error> {
            let input: Result<i64, Error> = Result.Ok(7);
            let task: Task<i64> = result_and_then(input, |x: i64| {
              if x == 7 {
                fetch(x)
              } else {
                fetch(0)
              }
            })?;
            let value: i64 = await task;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_result_and_then_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Task<i64>, Error>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_main_")
                && function.name.contains("Task_i64___Error")
        })
        .expect("expected specialized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Task<i64>, Error>"
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(main.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_higher_order_result_and_then_without_explicit_lambda_return_type_in_if_try_await_chain() {
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

          async fn work(seed: i64) -> i64 {
            return seed + 1;
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

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            return Result.Ok(spawn(work(seed)));
          }

          async fn main() -> Result<i64, Error> {
            let input: Result<i64, Error> = if true {
              Result.Ok(7)
            } else {
              Result.Ok(0)
            };
            let task: Task<i64> = result_and_then(input, |x: i64| {
              if x == 7 {
                fetch(x)
              } else {
                fetch(0)
              }
            })?;
            let value: i64 = await task;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_result_and_then_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(stmt_tree_contains_call(&main.body, &|callee, _| callee == hof.name));
    assert!(matches!(main.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_higher_order_result_and_then_without_explicit_lambda_return_type_with_inner_match_try_await_chain(
) {
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

          async fn work(seed: i64) -> i64 {
            return seed + 1;
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

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            return Result.Ok(spawn(work(seed)));
          }

          async fn main() -> Result<i64, Error> {
            let input: Result<i64, Error> = Result.Ok(7);
            let task: Task<i64> = result_and_then(input, |x: i64| {
              match x {
                7 => { fetch(x) }
                _ => { fetch(0) }
              }
            })?;
            let value: i64 = await task;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_result_and_then_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Task<i64>, Error>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_main_")
                && function.name.contains("Task_i64___Error")
        })
        .expect("expected specialized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<Task<i64>, Error>"
    ));
}

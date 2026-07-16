use super::*;

#[test]
fn monomorphizes_zero_arg_generic_async_function_through_await_into_nested_alias_wrapper_argument()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
          }

          async fn typed_box<T>() -> Boxed<T> {
            return Boxed(7);
          }

          fn keep_response<T>(response: Response<T>) -> Response<T> {
            return response;
          }

          async fn main() -> i64 {
            let response: Response<i64> = keep_response(Response(await typed_box()));
            return response.payload.value;
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
            name,
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "response"
            && callee == "keep_response__i64"
            && matches!(
                args.as_slice(),
                [NirExpr::StructLiteral { type_name, type_args, fields }]
                    if type_name == "Envelope"
                        && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
                        && matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Await(value))]
                                if field == "payload"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Call { callee, .. } if callee == "typed_box__i64"
                                    )
                        )
            )
    ));

    let box_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_box__i64")
        .unwrap();
    assert!(box_specialized.is_async);
    assert!(box_specialized.generic_params.is_empty());
    assert!(matches!(
        box_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Boxed<i64>"
    ));

    let response_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_response__i64")
        .unwrap();
    assert!(response_specialized.generic_params.is_empty());
    assert_eq!(
        response_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("Envelope<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        response_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));
}

#[test]
fn monomorphizes_generic_nested_alias_task_join_through_if_branch() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response(Boxed(7));
          }

          fn keep_response<T>(response: Response<T>) -> Response<T> {
            return response;
          }

          fn choose(flag: bool, task: Task<Response<i64>>) -> Response<i64> {
            if flag {
              return keep_response(join(task));
            } else {
              return Response(Boxed(9));
            }
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let response: Response<i64> = choose(true, task);
            return response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let produce_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "produce_response__i64")
        .unwrap();
    assert!(produce_specialized.is_async);
    assert!(produce_specialized.generic_params.is_empty());
    assert!(matches!(
        produce_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));

    let keep_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_response__i64")
        .unwrap();
    assert!(keep_specialized.generic_params.is_empty());
    assert_eq!(
        keep_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("Envelope<Boxed<i64>>".to_owned())
    );

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task" && ty.render() == "Task<Envelope<Boxed<i64>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "response"
            && ty.render() == "Envelope<Boxed<i64>>"
            && callee == "choose"
    ));

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .unwrap();
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "keep_response__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                    if type_name == "Envelope"
                        && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
            )
    ));
}

#[test]
fn monomorphizes_generic_response_unwrap_through_task_join_and_branch_constructors() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn unwrap_response<T>(response: Response<T>) -> T {
            match response {
              Response<T> { payload: { value: body }, ready: true } => {
                return body;
              }
              _ => {
                return response.payload.value;
              }
            }
          }

          fn consume(flag: bool, task: Task<Response<i64>>) -> i64 {
            if flag {
              return unwrap_response(join(task));
            } else {
              return unwrap_response(Response { payload: Boxed(9), ready: false });
            }
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            return consume(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let produced = module
        .functions
        .iter()
        .find(|function| function.name == "produce_response__i64")
        .unwrap();
    assert!(produced.is_async);
    assert!(produced.generic_params.is_empty());
    assert!(matches!(
        produced.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));

    let unwrapped = module
        .functions
        .iter()
        .find(|function| function.name == "unwrap_response__i64")
        .unwrap();
    assert!(unwrapped.generic_params.is_empty());
    assert_eq!(
        unwrapped.params.first().map(|param| param.ty.render()),
        Some("Envelope<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        unwrapped.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let consume = module
        .functions
        .iter()
        .find(|function| function.name == "consume")
        .unwrap();
    assert!(matches!(
        consume.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "unwrap_response__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "unwrap_response__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "Envelope"
                                    && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
                        )
            )
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task" && ty.render() == "Task<Envelope<Boxed<i64>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "consume"
    ));
}

#[test]
fn monomorphizes_no_annotation_try_await_result_binding_through_higher_order_mapper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn map_result<T, U, E>(result: Result<T, E>, mapper: Fn1<T, U>) -> Result<U, E> {
            match result {
              Result.Ok(value) => {
                return Result.Ok(mapper(value));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(work(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn compute(seed: i64) -> Result<i64, Error> {
            let mapped = map_result(
              Result.Ok(await fetch(seed)?),
              |value: i64| -> i64 { return value + 1; }
            );
            return mapped;
          }

          async fn main() -> Result<i64, Error> {
            return await compute(3);
          }
        }
        "#,
    )
    .unwrap();

    let compute = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    assert!(compute
        .body
        .iter()
        .any(|stmt| mapped_binding_is_concrete_result(stmt)));
    assert!(matches!(
        compute.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "mapped"
    ));

    let mapper = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_map_result___lambda_compute_0")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Result<i64, Error>"
                )
        })
        .unwrap();
    assert!(mapper.generic_params.is_empty());
    assert!(matches!(
        mapper.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Result<i64, Error>"
    ));
}

fn mapped_binding_is_concrete_result(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        } => {
            name == "mapped"
                && ty.render() == "Result<i64, Error>"
                && callee == "__hof_map_result___lambda_compute_0__i64__i64__Error"
        }
        NirStmt::If { else_body, .. } => else_body.iter().any(mapped_binding_is_concrete_result),
        _ => false,
    }
}

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

use super::*;

#[test]
fn monomorphizes_network_shaped_generic_task_exchange_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Request<T> = HttpRequest<Boxed<T>>;
          type Response<T> = HttpResponse<Boxed<T>>;
          type HttpResult<T> = ResultEnvelope<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry: bool,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            ok: bool,
          }

          fn keep_request<T>(request: Request<T>) -> Request<T> {
            return request;
          }

          async fn exchange<T>(request: Request<T>) -> HttpResult<T> {
            return HttpResult {
              response: Response { body: request.body, status: 200 },
              ok: true,
            };
          }

          fn read_body<T>(result: HttpResult<T>) -> T {
            match result {
              HttpResult<T> { response: { body: { value: payload }, status: 200 }, ok: true } => {
                return payload;
              }
              _ => {
                return result.response.body.value;
              }
            }
          }

          fn serve(flag: bool, task: Task<HttpResult<i64>>) -> i64 {
            if flag {
              return read_body(join(task));
            } else {
              return read_body(HttpResult {
                response: Response { body: Boxed(9), status: 503 },
                ok: false,
              });
            }
          }

          fn main() -> i64 {
            let request: Request<i64> = keep_request(Request { body: Boxed(7), retry: false });
            let task: Task<HttpResult<i64>> = spawn(exchange(request));
            return serve(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let keep_request = module
        .functions
        .iter()
        .find(|function| function.name == "keep_request__i64")
        .unwrap();
    assert!(keep_request.generic_params.is_empty());
    assert_eq!(
        keep_request.params.first().map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        keep_request.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "HttpRequest<Boxed<i64>>"
    ));

    let exchange = module
        .functions
        .iter()
        .find(|function| function.name == "exchange__i64")
        .unwrap();
    assert!(exchange.is_async);
    assert!(exchange.generic_params.is_empty());
    assert_eq!(
        exchange.params.first().map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "ResultEnvelope<HttpResponse<Boxed<i64>>>"
    ));

    let read_body = module
        .functions
        .iter()
        .find(|function| function.name == "read_body__i64")
        .unwrap();
    assert!(read_body.generic_params.is_empty());
    assert_eq!(
        read_body.params.first().map(|param| param.ty.render()),
        Some("ResultEnvelope<HttpResponse<Boxed<i64>>>".to_owned())
    );
    assert!(matches!(
        read_body.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let serve = module
        .functions
        .iter()
        .find(|function| function.name == "serve")
        .unwrap();
    assert!(matches!(
        serve.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "read_body__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "read_body__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "ResultEnvelope"
                                    && matches!(type_args.as_slice(), [ty] if ty.render() == "HttpResponse<Boxed<i64>>")
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
            value: NirExpr::Call { callee, .. },
        }) if name == "request"
            && ty.render() == "HttpRequest<Boxed<i64>>"
            && callee == "keep_request__i64"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render() == "Task<ResultEnvelope<HttpResponse<Boxed<i64>>>>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "serve"
    ));
}

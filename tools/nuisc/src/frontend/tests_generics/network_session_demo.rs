use super::*;

#[test]
fn monomorphizes_std_net_facade_shaped_http_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          fn net_http_request<T>(request: NetHttpRequest<T>) -> NetHttpRequest<T> {
            return request;
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchange<T> {
            return NetHttpClientExchange {
              result: NetResult {
                response: NetHttpResponse {
                  body: request.body,
                  status: 200,
                },
                recv_ready: true,
              },
              attempts: request.retry_budget,
            };
          }

          async fn net_session<T>(request: NetHttpRequest<T>) -> NetSession<T> {
            return NetSession {
              exchange: await net_http_client_exchange(request),
              open: true,
            };
          }

          fn net_http_response_value<T>(session: NetSession<T>) -> T {
            match session {
              NetSession<T> {
                exchange: {
                  result: {
                    response: { body: { value: payload }, status: 200 },
                    recv_ready: true,
                  },
                  attempts: 2,
                },
                open: true,
              } => {
                return payload;
              }
              _ => {
                return session.exchange.result.response.body.value;
              }
            }
          }

          fn serve(flag: bool, task: Task<NetSession<i64>>) -> i64 {
            if flag {
              return net_http_response_value(join(task));
            } else {
              return net_http_response_value(NetSession {
                exchange: NetHttpClientExchange {
                  result: NetResult {
                    response: NetHttpResponse { body: Boxed(9), status: 503 },
                    recv_ready: false,
                  },
                  attempts: 1,
                },
                open: false,
              });
            }
          }

          fn main() -> i64 {
            let request: NetHttpRequest<i64> = net_http_request(NetHttpRequest {
              body: Boxed(7),
              retry_budget: 2,
            });
            let task: Task<NetSession<i64>> = spawn(net_session(request));
            return serve(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let request_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_request__i64")
        .unwrap();
    assert!(request_specialized.generic_params.is_empty());
    assert_eq!(
        request_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        request_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "HttpRequest<Boxed<i64>>"
    ));

    let exchange_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_client_exchange__i64")
        .unwrap();
    assert!(exchange_specialized.is_async);
    assert!(exchange_specialized.generic_params.is_empty());
    assert_eq!(
        exchange_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange_specialized
            .return_type
            .as_ref()
            .map(|ty| ty.render()),
        Some(rendered) if rendered == "ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>"
    ));

    let session_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_session__i64")
        .unwrap();
    assert!(session_specialized.is_async);
    assert!(session_specialized.generic_params.is_empty());
    assert_eq!(
        session_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        session_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>"
    ));

    let value_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_response_value__i64")
        .unwrap();
    assert!(value_specialized.generic_params.is_empty());
    assert_eq!(
        value_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>".to_owned())
    );
    assert!(matches!(
        value_specialized.return_type.as_ref().map(|ty| ty.render()),
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
                    if callee == "net_http_response_value__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "net_http_response_value__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "SessionLane"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty]
                                            if ty.render()
                                                == "ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>"
                                    )
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
            && callee == "net_http_request__i64"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render()
                == "Task<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "serve"
    ));
}

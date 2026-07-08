use super::*;

#[test]
fn monomorphizes_std_net_demo_shaped_summary_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;
          type NetHttpClientExchangeSummary<T> = ExchangeSummary<NetSession<T>>;
          type NetSessionSummary<T> = SessionSummary<NetHttpClientExchangeSummary<T>>;

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

          struct ExchangeSummary<T> {
            session: T,
            exchange_value: i64,
          }

          struct SessionSummary<T> {
            summary: T,
            session_value: i64,
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

          async fn capture_net_http_client_exchange_summary<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return NetHttpClientExchangeSummary {
              session: await net_session(request),
              exchange_value: 41,
            };
          }

          async fn capture_net_session_summary<T>(
            request: NetHttpRequest<T>
          ) -> NetSessionSummary<T> {
            return SessionSummary {
              summary: await capture_net_http_client_exchange_summary(request),
              session_value: 99,
            };
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            match summary {
              NetSessionSummary<T> {
                summary: {
                  session: {
                    exchange: {
                      result: {
                        response: { body: { value: payload }, status: 200 },
                        recv_ready: true,
                      },
                      attempts: 2,
                    },
                    open: true,
                  },
                  exchange_value: 41,
                },
                session_value: 99,
              } => {
                return payload;
              }
              _ => {
                return summary.summary.session.exchange.result.response.body.value;
              }
            }
          }

          fn serve(flag: bool, task: Task<NetSessionSummary<i64>>) -> i64 {
            if flag {
              return summarize_net_session(join(task));
            } else {
              return summarize_net_session(SessionSummary {
                summary: ExchangeSummary {
                  session: SessionLane {
                    exchange: ExchangeLane {
                      result: ResultEnvelope {
                        response: HttpResponse { body: Boxed(9), status: 503 },
                        recv_ready: false,
                      },
                      attempts: 1,
                    },
                    open: false,
                  },
                  exchange_value: 40,
                },
                session_value: 98,
              });
            }
          }

          fn main() -> i64 {
            let request: NetHttpRequest<i64> = NetHttpRequest {
              body: Boxed(7),
              retry_budget: 2,
            };
            let task: Task<NetSessionSummary<i64>> = spawn(capture_net_session_summary(request));
            return serve(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let exchange_summary = module
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_client_exchange_summary__i64")
        .unwrap();
    assert!(exchange_summary.is_async);
    assert!(exchange_summary.generic_params.is_empty());
    assert_eq!(
        exchange_summary
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange_summary.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
    ));

    let session_summary = module
        .functions
        .iter()
        .find(|function| function.name == "capture_net_session_summary__i64")
        .unwrap();
    assert!(session_summary.is_async);
    assert!(session_summary.generic_params.is_empty());
    assert_eq!(
        session_summary
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        session_summary.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
    ));

    let summarize = module
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_session__i64")
        .unwrap();
    assert!(summarize.generic_params.is_empty());
    assert_eq!(
        summarize.params.first().map(|param| param.ty.render()),
        Some(
            "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
                .to_owned()
        )
    );
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
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
                    if callee == "summarize_net_session__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "summarize_net_session__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty]
                                            if ty.render()
                                                == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
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
            value: NirExpr::StructLiteral { type_name, type_args, .. },
        }) if name == "request"
            && ty.render() == "HttpRequest<Boxed<i64>>"
            && type_name == "HttpRequest"
            && matches!(type_args.as_slice(), [arg] if arg.render() == "Boxed<i64>")
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render()
                == "Task<SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>>"
    ));
}

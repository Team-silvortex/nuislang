use super::*;

#[test]
fn monomorphizes_match_arm_std_net_summary_session_flow() {
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

          fn choose_summary(
            mode: i64,
            task: Task<NetSessionSummary<i64>>,
            summary_task: Task<NetHttpClientExchangeSummary<i64>>
          ) -> NetSessionSummary<i64> {
            match mode {
              0 => {
                return join(task);
              }
              1 => {
                return SessionSummary {
                  summary: join(summary_task),
                  session_value: 99,
                };
              }
              _ => {
                return SessionSummary {
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
                };
              }
            }
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            return summary.summary.session.exchange.result.response.body.value;
          }

          fn main() -> i64 {
            let summary_task: Task<NetHttpClientExchangeSummary<i64>> =
              spawn(capture_net_http_client_exchange_summary(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let task: Task<NetSessionSummary<i64>> =
              spawn(capture_net_session_summary(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let summary: NetSessionSummary<i64> = choose_summary(1, task, summary_task);
            return summarize_net_session(summary);
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

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose_summary")
        .unwrap();
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::CpuJoin(_)))]
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::If {
                    then_body: nested_then,
                    else_body: nested_else,
                    ..
                }] if matches!(
                    nested_then.as_slice(),
                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                        if type_name == "SessionSummary"
                            && matches!(
                                type_args.as_slice(),
                                [ty]
                                    if ty.render()
                                        == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                            )
                            && matches!(
                                fields.as_slice(),
                                [
                                    (summary_field, NirExpr::CpuJoin(value)),
                                    (session_value_field, NirExpr::Int(99))
                                ]
                                    if summary_field == "summary"
                                        && session_value_field == "session_value"
                                        && matches!(
                                            value.as_ref(),
                                            NirExpr::Var(name) if name == "summary_task"
                                        )
                            )
                ) && matches!(
                    nested_else.as_slice(),
                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
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
            value: NirExpr::CpuSpawn { .. },
        }) if name == "summary_task"
            && ty.render()
                == "Task<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
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
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "summary"
            && ty.render()
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
            && callee == "choose_summary"
    ));
}

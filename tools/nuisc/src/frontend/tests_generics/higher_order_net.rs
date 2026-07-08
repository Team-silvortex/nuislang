use super::*;

#[test]
fn monomorphizes_higher_order_nested_while_match_std_net_summary_session_flow() {
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

          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return ExchangeSummary {
              session: SessionLane {
                exchange: ExchangeLane {
                  result: ResultEnvelope {
                    response: HttpResponse {
                      body: request.body,
                      status: 200,
                    },
                    recv_ready: true,
                  },
                  attempts: request.retry_budget,
                },
                open: true,
              },
              exchange_value: 41,
            };
          }

          fn nested_loop_summary(
            seed: i64,
            mode: i64,
            summary_task: Task<NetHttpClientExchangeSummary<i64>>
          ) -> NetSessionSummary<i64> {
            while seed > 0 {
              match apply(mode, |x: i64| -> i64 { return x + 1; }) {
                2 => {
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
            return SessionSummary {
              summary: ExchangeSummary {
                session: SessionLane {
                  exchange: ExchangeLane {
                    result: ResultEnvelope {
                      response: HttpResponse { body: Boxed(8), status: 204 },
                      recv_ready: true,
                    },
                    attempts: 0,
                  },
                  open: true,
                },
                exchange_value: 39,
              },
              session_value: 97,
            };
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            return summary.summary.session.exchange.result.response.body.value;
          }

          fn main() -> i64 {
            let summary_task: Task<NetHttpClientExchangeSummary<i64>> =
              spawn(net_http_client_exchange(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let summary: NetSessionSummary<i64> = nested_loop_summary(1, 1, summary_task);
            return summarize_net_session(summary);
          }
        }
        "#,
    )
    .unwrap();

    let nested = module
        .functions
        .iter()
        .find(|function| function.name == "nested_loop_summary")
        .unwrap();
    assert!(matches!(
        nested.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [
                    NirStmt::Let { name, value: NirExpr::Call { .. }, .. },
                    NirStmt::If { then_body, else_body, .. }
                ] if name.starts_with("__match_scrutinee_")
                    && matches!(
                        then_body.as_slice(),
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
                                                NirExpr::Var(task_name) if task_name == "summary_task"
                                            )
                                )
                    ) && matches!(
                        else_body.as_slice(),
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
            value: NirExpr::Call { callee, .. },
        }) if name == "summary"
            && ty.render()
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
            && callee == "nested_loop_summary"
    ));
}

#[test]
fn monomorphizes_higher_order_generic_mapper_with_explicit_helper_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type CellAlias<T> = Cell<T>;
          type PacketAlias<T> = Packet<T>;
          type EnvelopeAlias<T> = Envelope<T>;

          struct Cell<T> {
            value: T,
          }

          struct Packet<T> {
            payload: T,
            tag: i64,
          }

          struct Envelope<T> {
            packet: T,
            ready: bool,
          }

          fn wrap_packet<T>(payload: T, tag: i64) -> PacketAlias<T> {
            return PacketAlias {
              payload: payload,
              tag: tag,
            };
          }

          fn wrap_envelope<T>(packet: T, ready: bool) -> EnvelopeAlias<T> {
            return EnvelopeAlias {
              packet: packet,
              ready: ready,
            };
          }

          fn apply_packetized<T>(
            value: T,
            mapper: Fn1<T, EnvelopeAlias<PacketAlias<CellAlias<T>>>>
          ) -> EnvelopeAlias<PacketAlias<CellAlias<T>>> {
            return mapper(value);
          }

          async fn produce_seed() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(produce_seed());
            let seed: i64 = join(task);
            let selected: EnvelopeAlias<PacketAlias<CellAlias<i64>>> =
              apply_packetized(seed, |x: i64| -> EnvelopeAlias<PacketAlias<CellAlias<i64>>> {
                let cell: CellAlias<i64> = CellAlias { value: x };
                if x > 0 {
                  let packet: PacketAlias<CellAlias<i64>> =
                    wrap_packet<CellAlias<i64>>(cell, 6);
                  return wrap_envelope<PacketAlias<CellAlias<i64>>>(packet, true);
                }
                let packet: PacketAlias<CellAlias<i64>> =
                  wrap_packet<CellAlias<i64>>(cell, 1);
                return wrap_envelope<PacketAlias<CellAlias<i64>>>(packet, false);
              });
            return selected.packet.payload.value + selected.packet.tag;
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
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task" && ty.render() == "Task<i64>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuJoin(_),
        }) if name == "seed" && ty.render() == "i64"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "selected"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee.starts_with("__hof_apply_packetized")
    ));

    let specialized_hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_packetized"))
        .unwrap();
    assert!(specialized_hof.generic_params.is_empty());
    assert!(matches!(
        specialized_hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .unwrap();
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_packet__")
    }));
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));
}

#[test]
fn monomorphizes_higher_order_generic_mapper_from_field_access_arguments_without_typed_locals() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type CellAlias<T> = Cell<T>;
          type PacketAlias<T> = Packet<T>;
          type EnvelopeAlias<T> = Envelope<T>;

          struct Cell<T> {
            value: T,
          }

          struct Packet<T> {
            payload: T,
            tag: i64,
          }

          struct Envelope<T> {
            packet: T,
            ready: bool,
          }

          fn wrap_cell<T>(value: T) -> CellAlias<T> {
            return CellAlias { value: value };
          }

          fn wrap_packet<T>(payload: T, tag: i64) -> PacketAlias<T> {
            return PacketAlias {
              payload: payload,
              tag: tag,
            };
          }

          fn wrap_envelope<T>(packet: T, ready: bool) -> EnvelopeAlias<T> {
            return EnvelopeAlias {
              packet: packet,
              ready: ready,
            };
          }

          fn apply_packetized<T>(
            payload: T,
            tag: i64,
            mapper: Fn2<T, i64, EnvelopeAlias<PacketAlias<T>>>
          ) -> EnvelopeAlias<PacketAlias<T>> {
            return mapper(payload, tag);
          }

          fn main() -> i64 {
            let packet: PacketAlias<CellAlias<i64>> =
              wrap_packet<CellAlias<i64>>(wrap_cell<i64>(7), 9);
            let selected: EnvelopeAlias<PacketAlias<CellAlias<i64>>> =
              apply_packetized(
                packet.payload,
                packet.tag,
                |payload: CellAlias<i64>, tag: i64| -> EnvelopeAlias<PacketAlias<CellAlias<i64>>> {
                  if tag > 0 {
                    return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                      wrap_packet<CellAlias<i64>>(payload, tag),
                      true
                    );
                  }
                  return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                    wrap_packet<CellAlias<i64>>(payload, 0),
                    false
                  );
                }
              );
            return selected.packet.payload.value + selected.packet.tag;
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
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "selected"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee.starts_with("__hof_apply_packetized")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .unwrap();
    assert!(lambda.generic_params.is_empty());
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_packet__")
    }));
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));
}

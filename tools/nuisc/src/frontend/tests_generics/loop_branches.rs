use super::*;

#[test]
fn monomorphizes_continue_branch_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(flag: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              if flag {
                continue;
              } else {
                return Summary {
                  response: join(task),
                  code: 7,
                };
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(true, 1, task);
            return summary.response.payload.value;
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

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(then_body.as_slice(), [NirStmt::Continue])
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
            )
    ));
    assert!(matches!(
        route.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. })))
            if type_name == "SessionSummary"
                && matches!(type_args.as_slice(), [ty] if ty.render() == "Envelope<Boxed<i64>>")
    ));
}

#[test]
fn monomorphizes_break_branch_before_generic_summary_fallback_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(flag: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              if flag {
                break;
              } else {
                return Summary {
                  response: join(task),
                  code: 7,
                };
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(then_body.as_slice(), [NirStmt::Break])
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
            )
    ));
    assert!(matches!(
        route.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields })))
            if type_name == "SessionSummary"
                && matches!(type_args.as_slice(), [ty] if ty.render() == "Envelope<Boxed<i64>>")
                && matches!(
                    fields.as_slice(),
                    [
                        (response_field, NirExpr::StructLiteral { type_name, type_args, .. }),
                        (code_field, NirExpr::Int(8))
                    ]
                        if response_field == "response"
                            && code_field == "code"
                            && type_name == "Envelope"
                            && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
                )
    ));
}

#[test]
fn monomorphizes_guarded_match_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(mode: i64, ready: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              match mode {
                2 if ready => {
                  return Summary {
                    response: join(task),
                    code: 7,
                  };
                }
                _ => {
                  return Summary {
                    response: Response { payload: Boxed(9), ready: false },
                    code: 8,
                  };
                }
              }
            }
            return Summary {
              response: Response { payload: Boxed(10), ready: true },
              code: 9,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(2, true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { condition, then_body, else_body }]
                    if matches!(condition, NirExpr::Binary { .. })
                        && matches!(
                            then_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                        )
            )
    ));
}

#[test]
fn monomorphizes_nested_match_continue_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(mode: i64, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              match mode {
                0 => {
                  continue;
                }
                _ => {
                  return Summary {
                    response: join(task),
                    code: 7,
                  };
                }
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(1, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(then_body.as_slice(), [NirStmt::Continue])
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
            )
    ));
}

#[test]
fn monomorphizes_nested_if_break_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(flag: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              if flag {
                if seed > 1 {
                  break;
                } else {
                  return Summary {
                    response: join(task),
                    code: 7,
                  };
                }
              } else {
                return Summary {
                  response: Response { payload: Boxed(8), ready: false },
                  code: 6,
                };
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(
                        then_body.as_slice(),
                        [NirStmt::If { then_body, else_body, .. }]
                            if matches!(then_body.as_slice(), [NirStmt::Break])
                                && matches!(
                                    else_body.as_slice(),
                                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                        if type_name == "SessionSummary"
                                            && matches!(
                                                type_args.as_slice(),
                                                [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                            )
                                            && matches!(
                                                fields.as_slice(),
                                                [
                                                    (response_field, NirExpr::CpuJoin(value)),
                                                    (code_field, NirExpr::Int(7))
                                                ]
                                                    if response_field == "response"
                                                        && code_field == "code"
                                                        && matches!(
                                                            value.as_ref(),
                                                            NirExpr::Var(task_name) if task_name == "task"
                                                        )
                                            )
                                )
                    ) && matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                            if type_name == "SessionSummary"
                                && matches!(
                                    type_args.as_slice(),
                                    [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                )
                    )
            )
    ));
}

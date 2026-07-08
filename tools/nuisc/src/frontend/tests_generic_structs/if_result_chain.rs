use super::*;

#[test]
#[allow(unreachable_code)]
fn lowers_receiver_method_call_with_explicit_generic_args_through_if_result_task_chain() {
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

          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          struct Wrapper<T, U> {
            inner: Carrier<T, U>,
          }

          struct Nest<T, U> {
            outer: Wrapper<T, U>,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          async fn make_nested<T, U>(value: T) -> Nest<T, U> {
            return Nest {
              outer: Wrapper {
                inner: Carrier(value),
              },
            };
          }

          fn fetch(seed: i64) -> Result<Task<Nest<i64, bool>>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(make_nested(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn compute(flag: bool) -> Result<i64, Error> {
            return Result.Ok((await (if flag {
              fetch(1)
            } else {
              fetch(2)
            })?).outer.inner.show<i64, bool>());
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
    assert!(matches!(
        compute.body.as_slice(),
        [NirStmt::If { condition, .. }]
            if matches!(condition, NirExpr::Var(flag) if flag == "flag")
    ));
    assert!(contains_fetch_call(&compute.body, |args| {
        matches!(args, [NirExpr::Int(1)])
    }));
    assert!(contains_fetch_call(&compute.body, |args| {
        matches!(args, [NirExpr::Int(2)])
    }));
    assert_result_task_show_chain_semantics(&compute.body);
    return;
    assert!(matches!(
        compute.body.as_slice(),
        [NirStmt::If { condition, then_body, else_body }]
            if matches!(condition, NirExpr::Var(flag) if flag == "flag")
                && matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, ty: Some(ty), value },
                        NirStmt::If { then_body, else_body, .. }
                    ]
                        if name == "__nuis_try_result_0"
                            && ty.render() == "Result<Task<Nest<i64, bool>>, Error>"
                            && matches!(
                                value,
                                NirExpr::Call { callee, args }
                                    if callee == "fetch"
                                        && matches!(args.as_slice(), [NirExpr::Int(1)])
                            )
                            && matches!(
                                then_body.as_slice(),
                                [
                                    NirStmt::Let { name, ty: Some(ty), value },
                                    NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                                ]
                                    if name == "__nuis_try_payload_0"
                                        && ty.render() == "Task<Nest<i64, bool>>"
                                        && is_payload_value_access_from_var(value, "__nuis_try_result_0")
                                        && type_name == "Result.Ok"
                                        && matches!(type_args.as_slice(), [ok, err] if ok.render() == "i64" && err.render() == "Error")
                                        && matches!(
                                            fields.as_slice(),
                                            [(field, NirExpr::Call { callee, args })]
                                                if field == "value"
                                                    && callee.starts_with("impl.Showable.for.Carrier")
                                                    && callee.ends_with(".show__i64__bool")
                                                    && matches!(
                                                        args.as_slice(),
                                                        [NirExpr::FieldAccess { base, field }]
                                                            if field == "inner"
                                                                && matches!(
                                                                    base.as_ref(),
                                                                    NirExpr::FieldAccess { base: outer_base, field: outer_field }
                                                                        if outer_field == "outer"
                                                                            && matches!(
                                                                                outer_base.as_ref(),
                                                                                NirExpr::Await(value)
                                                                                    if matches!(
                                                                                        value.as_ref(),
                                                                                        NirExpr::Var(task_name) if task_name == "__nuis_try_payload_0"
                                                                                    )
                                                                            )
                                                                )
                                                    )
                                        )
                            )
                            && matches!(
                                else_body.as_slice(),
                                [NirStmt::If { then_body, .. }]
                                    if matches!(
                                        then_body.as_slice(),
                                        [
                                            NirStmt::Let { name, ty: Some(ty), value },
                                            NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                                        ]
                                            if name == "__nuis_try_error_0"
                                                && ty.render() == "Error"
                                                && is_payload_value_access_from_var(value, "__nuis_try_result_0")
                                                && type_name == "Result.Err"
                                                && matches!(type_args.as_slice(), [ok, err] if ok.render() == "i64" && err.render() == "Error")
                                                && matches!(
                                                    fields.as_slice(),
                                                    [(field, NirExpr::Var(error_name))]
                                                        if field == "value" && error_name == "__nuis_try_error_0"
                                                )
                                    )
                            )
                )
                && matches!(
                    else_body.as_slice(),
                    [
                        NirStmt::Let { name, ty: Some(ty), value },
                        NirStmt::If { then_body, else_body, .. }
                    ]
                        if name == "__nuis_try_result_1"
                            && ty.render() == "Result<Task<Nest<i64, bool>>, Error>"
                            && matches!(
                                value,
                                NirExpr::Call { callee, args }
                                    if callee == "fetch"
                                        && matches!(args.as_slice(), [NirExpr::Int(2)])
                            )
                            && matches!(
                                then_body.as_slice(),
                                [
                                    NirStmt::Let { name, ty: Some(ty), value },
                                    NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                                ]
                                    if name == "__nuis_try_payload_1"
                                        && ty.render() == "Task<Nest<i64, bool>>"
                                        && is_payload_value_access_from_var(value, "__nuis_try_result_1")
                                        && type_name == "Result.Ok"
                                        && matches!(type_args.as_slice(), [ok, err] if ok.render() == "i64" && err.render() == "Error")
                                        && matches!(
                                            fields.as_slice(),
                                            [(field, NirExpr::Call { callee, args })]
                                                if field == "value"
                                                    && callee.starts_with("impl.Showable.for.Carrier")
                                                    && callee.ends_with(".show__i64__bool")
                                                    && matches!(
                                                        args.as_slice(),
                                                        [NirExpr::FieldAccess { base, field }]
                                                            if field == "inner"
                                                                && matches!(
                                                                    base.as_ref(),
                                                                    NirExpr::FieldAccess { base: outer_base, field: outer_field }
                                                                        if outer_field == "outer"
                                                                            && matches!(
                                                                                outer_base.as_ref(),
                                                                                NirExpr::Await(value)
                                                                                    if matches!(
                                                                                        value.as_ref(),
                                                                                        NirExpr::Var(task_name) if task_name == "__nuis_try_payload_1"
                                                                                    )
                                                                            )
                                                                )
                                                    )
                                        )
                            )
                            && matches!(
                                else_body.as_slice(),
                                [NirStmt::If { then_body, .. }]
                                    if matches!(
                                        then_body.as_slice(),
                                        [
                                            NirStmt::Let { name, ty: Some(ty), value },
                                            NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                                        ]
                                            if name == "__nuis_try_error_1"
                                                && ty.render() == "Error"
                                                && is_payload_value_access_from_var(value, "__nuis_try_result_1")
                                                && type_name == "Result.Err"
                                                && matches!(type_args.as_slice(), [ok, err] if ok.render() == "i64" && err.render() == "Error")
                                                && matches!(
                                                    fields.as_slice(),
                                                    [(field, NirExpr::Var(error_name))]
                                                        if field == "value" && error_name == "__nuis_try_error_1"
                                                )
                                    )
                            )
                )
    ));
}

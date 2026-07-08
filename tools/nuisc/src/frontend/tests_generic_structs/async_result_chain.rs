use super::*;

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_awaited_nested_helper_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
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

          async fn main() -> i64 {
            return (await make_nested(7)).outer.inner.show<i64, bool>();
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
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
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
                                                    NirExpr::Call { callee: helper, .. }
                                                        if helper == "make_nested__i64__bool"
                                                )
                                        )
                            )
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_spawn_join_nested_helper_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
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

          fn main() -> i64 {
            let task: Task<Nest<i64, bool>> = spawn(make_nested(7));
            return join(task).outer.inner.show<i64, bool>();
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
        Some(NirStmt::Let { name, ty: Some(ty), value })
            if name == "task"
                && ty.render() == "Task<Nest<i64, bool>>"
                && matches!(
                    value,
                    NirExpr::CpuSpawn { callee, args }
                        if callee == "make_nested__i64__bool"
                            && matches!(args.as_slice(), [NirExpr::Int(7)])
                )
    ));
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
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
                                            NirExpr::CpuJoin(value)
                                                if matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                        )
                            )
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_result_task_chain() {
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

          async fn compute(seed: i64) -> Result<i64, Error> {
            return Result.Ok((await fetch(seed)?).outer.inner.show<i64, bool>());
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
    assert!(contains_fetch_call(&compute.body, |args| {
        matches!(args, [NirExpr::Var(seed)] if seed == "seed")
    }));
    assert!(contains_result_variant(&compute.body, "Result.Ok"));
    assert!(contains_result_variant(&compute.body, "Result.Err"));
    assert!(contains_showable_call_from_awaited_try_payload(
        &compute.body
    ));
}

use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn parses_generic_struct_definition_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }
        }
        "#,
    )
    .unwrap();

    let definition = &ast.structs[0];
    assert_eq!(definition.name, "Boxed");
    assert_eq!(definition.generic_params.len(), 1);
    assert_eq!(definition.generic_params[0].name, "T");
    assert!(definition.generic_params[0].bounds.is_empty());
    assert_eq!(definition.fields[0].ty.name, "T");
}

#[test]
fn lowers_generic_struct_literal_with_expected_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_explicit_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed<i64> { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected explicit generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic struct let, found {other:?}"),
    }
}

#[test]
fn rejects_generic_struct_literal_with_conflicting_inferred_type_args() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair<T> {
            left: T,
            right: T,
          }

          fn main() -> i64 {
            let pair = Pair { left: 7, right: "hi" };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "struct literal `Pair` inferred conflicting types `i64` and `String` for generic parameter `T`"
        ),
        "{error}"
    );
}

#[test]
fn lowers_nested_generic_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          struct Wrapper<T> {
            inner: Boxed<T>,
            tag: i64,
          }

          fn main() -> i64 {
            let wrapped = Wrapper {
              inner: Boxed { value: 7 },
              tag: 1,
            };
            return wrapped.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "wrapped");
            assert_eq!(ty.as_ref().unwrap().render(), "Wrapper<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Wrapper"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
            ));
        }
        other => panic!("expected inferred nested generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_non_transparent_alias_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedStructAlias<T> = Wrapper<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Wrapper<T> {
            inner: T,
            tag: i64,
          }

          fn main() -> i64 {
            let wrapped = WrappedStructAlias {
              inner: Boxed { value: 7 },
              tag: 1,
            };
            return wrapped.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "wrapped");
            assert_eq!(ty.as_ref().unwrap().render(), "Wrapper<Boxed<i64>>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Wrapper"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
            ));
        }
        other => panic!("expected inferred non-transparent alias struct let, found {other:?}"),
    }
}

#[test]
fn lowers_multi_field_generic_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair<T, U> {
            left: T,
            right: U,
          }

          fn main() -> i64 {
            let pair = Pair { left: 7, right: 9 };
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { ty: Some(ty), .. } => {
            assert_eq!(ty.render(), "Pair<i64, i64>");
        }
        other => panic!("expected inferred pair generic struct let, found {other:?}"),
    }
}

#[test]
fn rejects_generic_struct_literal_when_fields_do_not_fully_determine_type_args() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          fn main() -> i64 {
            let phantom = Phantom { value: 7, tag: 1 };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "cannot infer generic arguments for struct literal `Phantom` in the current frontend; add an explicit expected type"
        ),
        "{error}"
    );
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_struct_literal() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          impl<T, U> Showable for Phantom<T, U> {
            fn show(value: Phantom<T, U>) -> i64 {
              return value.tag;
            }
          }

          fn main() -> i64 {
            return Phantom { value: 7, tag: 1 }.show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Phantom")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }]
                        if type_name == "Phantom"
                            && matches!(type_args.as_slice(), [first, second] if first.render() == "i64" && second.render() == "bool")
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_payload_constructor() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn main() -> i64 {
            return Carrier(7).show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }]
                        if type_name == "Carrier"
                            && matches!(type_args.as_slice(), [first, second] if first.render() == "i64" && second.render() == "bool")
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_payload_alias_constructor() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          type CarrierAlias<T, U> = Carrier<T, U>;

          struct Carrier<T, U> {
            value: T,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn main() -> i64 {
            return CarrierAlias(7).show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }]
                        if type_name == "Carrier"
                            && matches!(type_args.as_slice(), [first, second] if first.render() == "i64" && second.render() == "bool")
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_generic_helper_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn make_carrier<T, U>(value: T) -> Carrier<T, U> {
            return Carrier(value);
          }

          fn main() -> i64 {
            return make_carrier(7).show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Call { callee: helper, .. }]
                        if helper == "make_carrier__i64__bool"
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_helper_field_chain() {
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

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn make_carrier<T, U>(value: T) -> Carrier<T, U> {
            return Carrier(value);
          }

          fn wrap<T, U>(inner: Carrier<T, U>) -> Wrapper<T, U> {
            return Wrapper { inner: inner };
          }

          fn main() -> i64 {
            return wrap(make_carrier(7)).inner.show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::FieldAccess { base, field }]
                        if field == "inner"
                            && matches!(
                                base.as_ref(),
                                NirExpr::Call { callee: helper, .. }
                                    if helper == "wrap__i64__bool"
                            )
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_nested_helper_field_chain() {
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

          fn make_carrier<T, U>(value: T) -> Carrier<T, U> {
            return Carrier(value);
          }

          fn wrap<T, U>(inner: Carrier<T, U>) -> Wrapper<T, U> {
            return Wrapper { inner: inner };
          }

          fn nest<T, U>(outer: Wrapper<T, U>) -> Nest<T, U> {
            return Nest { outer: outer };
          }

          fn main() -> i64 {
            return nest(wrap(make_carrier(7))).outer.inner.show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
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
                                            NirExpr::Call { callee: helper, .. }
                                                if helper == "nest__i64__bool"
                                        )
                            )
                )
    ));
}

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
    assert!(matches!(
        compute.body.first(),
        Some(NirStmt::Let { name, ty: Some(ty), value })
            if name == "__nuis_try_result_0"
                && ty.render() == "Result<Task<Nest<i64, bool>>, Error>"
                && matches!(
                    value,
                    NirExpr::Call { callee, args }
                        if callee == "fetch"
                            && matches!(args.as_slice(), [NirExpr::Var(seed)] if seed == "seed")
                )
    ));
    assert!(matches!(
        compute.body.get(1),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let { name, ty: Some(ty), value },
                    NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                ]
                    if name == "__nuis_try_payload_0"
                        && ty.render() == "Task<Nest<i64, bool>>"
                        && matches!(
                            value,
                            NirExpr::FieldAccess { base, field }
                                if field == "value"
                                    && matches!(
                                        base.as_ref(),
                                        NirExpr::Var(result_name) if result_name == "__nuis_try_result_0"
                                    )
                        )
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
                                    && matches!(
                                        value,
                                        NirExpr::FieldAccess { base, field }
                                            if field == "value"
                                                && matches!(
                                                    base.as_ref(),
                                                    NirExpr::Var(result_name) if result_name == "__nuis_try_result_0"
                                                )
                                    )
                                    && type_name == "Result.Err"
                                    && matches!(type_args.as_slice(), [ok, err] if ok.render() == "i64" && err.render() == "Error")
                                    && matches!(
                                        fields.as_slice(),
                                        [(field, NirExpr::Var(error_name))]
                                            if field == "value" && error_name == "__nuis_try_error_0"
                                    )
                        )
                )
    ));
}

#[test]
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
                                        && matches!(
                                            value,
                                            NirExpr::FieldAccess { base, field }
                                                if field == "value"
                                                    && matches!(
                                                        base.as_ref(),
                                                        NirExpr::Var(result_name) if result_name == "__nuis_try_result_0"
                                                    )
                                        )
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
                                                && matches!(
                                                    value,
                                                    NirExpr::FieldAccess { base, field }
                                                        if field == "value"
                                                            && matches!(
                                                                base.as_ref(),
                                                                NirExpr::Var(result_name) if result_name == "__nuis_try_result_0"
                                                            )
                                                )
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
                                        && matches!(
                                            value,
                                            NirExpr::FieldAccess { base, field }
                                                if field == "value"
                                                    && matches!(
                                                        base.as_ref(),
                                                        NirExpr::Var(result_name) if result_name == "__nuis_try_result_1"
                                                    )
                                        )
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
                                                && matches!(
                                                    value,
                                                    NirExpr::FieldAccess { base, field }
                                                        if field == "value"
                                                            && matches!(
                                                                base.as_ref(),
                                                                NirExpr::Var(result_name) if result_name == "__nuis_try_result_1"
                                                            )
                                                )
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

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_match_result_task_chain() {
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
            return Result.Ok((await (match seed {
              1 => { fetch(1) }
              _ => { fetch(seed) }
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
        [NirStmt::If { condition, then_body, else_body }]
            if matches!(
                condition,
                NirExpr::Binary { op, lhs, rhs }
                    if *op == nuis_semantics::model::NirBinaryOp::Eq
                        && matches!(lhs.as_ref(), NirExpr::Var(seed) if seed == "seed")
                        && matches!(rhs.as_ref(), NirExpr::Int(1))
            )
                && matches!(
                    then_body.as_slice(),
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
                                        && matches!(args.as_slice(), [NirExpr::Int(1)])
                            )
                            && matches!(
                                then_body.as_slice(),
                                [
                                    NirStmt::Let { name, ty: Some(ty), value },
                                    NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                                ]
                                    if name == "__nuis_try_payload_1"
                                        && ty.render() == "Task<Nest<i64, bool>>"
                                        && matches!(
                                            value,
                                            NirExpr::FieldAccess { base, field }
                                                if field == "value"
                                                    && matches!(
                                                        base.as_ref(),
                                                        NirExpr::Var(result_name) if result_name == "__nuis_try_result_1"
                                                    )
                                        )
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
                                                && matches!(
                                                    value,
                                                    NirExpr::FieldAccess { base, field }
                                                        if field == "value"
                                                            && matches!(
                                                                base.as_ref(),
                                                                NirExpr::Var(result_name) if result_name == "__nuis_try_result_1"
                                                            )
                                                )
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
                && matches!(
                    else_body.as_slice(),
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
                                        && matches!(args.as_slice(), [NirExpr::Var(seed)] if seed == "seed")
                            )
                            && matches!(
                                then_body.as_slice(),
                                [
                                    NirStmt::Let { name, ty: Some(ty), value },
                                    NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))
                                ]
                                    if name == "__nuis_try_payload_0"
                                        && ty.render() == "Task<Nest<i64, bool>>"
                                        && matches!(
                                            value,
                                            NirExpr::FieldAccess { base, field }
                                                if field == "value"
                                                    && matches!(
                                                        base.as_ref(),
                                                        NirExpr::Var(result_name) if result_name == "__nuis_try_result_0"
                                                    )
                                        )
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
                                                && matches!(
                                                    value,
                                                    NirExpr::FieldAccess { base, field }
                                                        if field == "value"
                                                            && matches!(
                                                                base.as_ref(),
                                                                NirExpr::Var(result_name) if result_name == "__nuis_try_result_0"
                                                            )
                                                )
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
    ));
}

#[test]
fn rejects_generic_alias_struct_literal_when_fields_do_not_fully_determine_type_args() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type PhantomAlias<T, U> = Phantom<T, U>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          fn main() -> i64 {
            let phantom = PhantomAlias { value: 7, tag: 1 };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "generic alias constructor `PhantomAlias` could not infer generic parameter `U` for target `Phantom<T, U>`; add explicit type arguments or a stronger expected type"
        ),
        "{error}"
    );
}

#[test]
fn lowers_outer_generic_struct_literal_when_later_field_completes_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let outer = Outer {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Outer<i64, String>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Outer"
                    && matches!(type_args.as_slice(), [lhs, rhs] if lhs.render() == "i64" && rhs.render() == "String")
            ));
        }
        other => panic!("expected inferred outer generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_outer_generic_struct_literal_when_later_field_completes_inner_payload_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T, U> {
            value: T,
          }

          struct Outer<T, U> {
            inner: Just<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let outer = Outer {
              inner: Just(7),
              meta: "ok",
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Outer<i64, String>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Outer"
                    && matches!(type_args.as_slice(), [lhs, rhs] if lhs.render() == "i64" && rhs.render() == "String")
            ));
        }
        other => panic!("expected inferred outer generic struct let from payload route, found {other:?}"),
    }
}

#[test]
fn lowers_transparent_alias_outer_literal_when_later_field_completes_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Outer<T, U>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let outer = OuterAlias {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Outer<i64, String>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Outer"
                    && matches!(type_args.as_slice(), [lhs, rhs] if lhs.render() == "i64" && rhs.render() == "String")
            ));
        }
        other => panic!("expected inferred transparent alias outer let, found {other:?}"),
    }
}

#[test]
fn lowers_non_transparent_alias_outer_literal_when_later_field_completes_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Wrapper<Outer<T, U>>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          struct Wrapper<T> {
            inner: T,
            mark: i64,
          }

          fn main() -> i64 {
            let outer = OuterAlias {
              inner: Outer {
                inner: Phantom { value: 7, tag: 1 },
                meta: "ok",
              },
              mark: 1,
            };
            return outer.inner.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Wrapper<Outer<i64, String>>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Wrapper"
                    && matches!(type_args.as_slice(), [inner] if inner.render() == "Outer<i64, String>")
            ));
        }
        other => panic!("expected inferred non-transparent alias outer let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: 7 };
            let Boxed<i64> { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let_with_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxI64 = Boxed<i64>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: BoxI64 = Boxed<i64> { value: 7 };
            let BoxI64 { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered aliased generic destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let_with_generic_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: BoxAlias<i64> = Boxed<i64> { value: 7 };
            let BoxAlias<i64> { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered generic-aliased destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_inferred_generic_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = BoxAlias { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic-alias struct let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_expected_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload: Just<i64> = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected lowered generic payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_explicit_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = Just<i64>(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected explicit generic payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_generic_alias_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = JustAlias<i64>(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected lowered generic-alias payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_inferred_generic_alias_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = JustAlias(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic-alias payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_alias_payload_constructor_from_alias_field_access() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type PacketAlias<T> = Packet<T>;

          struct Just<T> {
            value: T,
          }

          struct Packet<T> {
            payload: T,
            tag: i64,
          }

          fn main() -> i64 {
            let packet: PacketAlias<i64> = PacketAlias { payload: 7, tag: 1 };
            let payload = JustAlias(packet.payload);
            return payload.value + packet.tag;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::FieldAccess { field: payload_field, .. })]
                            if field == "value" && payload_field == "payload"
                    )
            ));
        }
        other => panic!(
            "expected alias-field-access generic-alias payload constructor let, found {other:?}"
        ),
    }
}

#[test]
fn rejects_payload_style_constructor_with_wrong_generic_alias_arity() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = JustAlias<i64, bool>(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type alias `JustAlias` expects 1 generic argument(s), found 2"),
        "{error}"
    );
}

#[test]
fn lowers_payload_style_constructor_for_non_transparent_generic_alias_when_inferable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedAlias<T> = Just<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = WrappedAlias(Boxed { value: 7 });
            return payload.value.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<Boxed<i64>>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
            ));
        }
        other => panic!(
            "expected inferred non-transparent generic-alias payload constructor let, found {other:?}"
        ),
    }
}

#[test]
fn reports_non_transparent_alias_payload_constructor_shape_mismatch_clearly() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedAlias<T> = Just<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = WrappedAlias(7);
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "generic alias constructor `WrappedAlias` could not match target field shape `Boxed<T>` with concrete type `i64`"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_constructor_missing_generic_parameter_clearly() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type PhantomAlias<T, U> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = PhantomAlias { value: 7 };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "generic alias constructor `PhantomAlias` could not infer generic parameter `U` for target `Boxed<T>`"
        ),
        "{error}"
    );
}

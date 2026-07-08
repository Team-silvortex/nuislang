use super::*;

#[test]
fn monomorphizes_generic_function_call_into_concrete_nir_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs.add(rhs);
          }

          fn main() -> i64 {
            return sum_two(1, 2);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "sum_two__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "sum_two__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "impl.Addable.for.i64.add"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_call_used_as_method_receiver() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn typed_zero<T: Addable>() -> T {
            return 0;
          }

          fn main() -> i64 {
            return typed_zero().add(1);
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
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Call { callee: receiver_callee, args: receiver_args }, NirExpr::Int(1)]
                        if receiver_callee == "typed_zero__i64" && receiver_args.is_empty()
                )
    ));
}

#[test]
fn monomorphizes_generic_binary_add_with_addable_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return sum_two(1, 2);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "sum_two__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "sum_two__i64")
        .unwrap();
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { op, .. }))) if *op == nuis_semantics::model::NirBinaryOp::Add
    ));
}

#[test]
fn monomorphizes_generic_function_with_parent_enum_parameter_from_variant_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn keep<T>(value: Option<T>) -> Option<T> {
            return value;
          }

          fn main() -> i64 {
            let value: Option<i64> = keep(Option.Some(7));
            match value {
              Option.Some(payload) => {
                return payload;
              }
              _ => {
                return 0;
              }
            }
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
        &main.body[0],
        NirStmt::Let {
            value: NirExpr::Call { callee, args },
            ..
        } if callee == "keep__i64"
            && matches!(
                args.as_slice(),
                [NirExpr::StructLiteral { type_name, type_args, .. }]
                    if type_name == "Option.Some"
                        && type_args.len() == 1
                        && type_args[0].render() == "i64"
            )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.params.as_slice(),
        [param] if param.ty.render() == "Option<i64>"
    ));
}

#[test]
fn monomorphizes_parent_enum_parameter_from_unit_variant_and_sibling_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn fallback<T>(value: Option<T>, fallback: T) -> T {
            match value {
              Option.Some(payload) => {
                return payload;
              }
              _ => {
                return fallback;
              }
            }
          }

          fn main() -> i64 {
            return fallback(Option.None, 7);
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
            if callee == "fallback__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }, NirExpr::Int(7)]
                        if type_name == "Option.None"
                            && type_args.len() == 1
                            && type_args[0].render() == "i64"
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "fallback__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.params.as_slice(),
        [option_param, fallback_param]
            if option_param.ty.render() == "Option<i64>"
                && fallback_param.ty.render() == "i64"
    ));
}

#[test]
fn monomorphizes_parent_enum_parameter_from_unit_variant_and_expected_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn keep<T>(value: Option<T>) -> Option<T> {
            return value;
          }

          fn main() {
            let value: Option<i64> = keep(Option.None);
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
        &main.body[0],
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::Call { callee, args },
            ..
        } if ty.render() == "Option<i64>"
            && callee == "keep__i64"
            && matches!(
                args.as_slice(),
                [NirExpr::StructLiteral { type_name, type_args, .. }]
                    if type_name == "Option.None"
                        && type_args.len() == 1
                        && type_args[0].render() == "i64"
            )
    ));
}

#[test]
fn generic_bound_accepts_enum_variant_argument_via_parent_enum_impl() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Showable for Option<i64> {
            fn show(value: Option<i64>) -> i64 {
              match value {
                Option.Some(payload) => {
                  return payload;
                }
                Option.None => {
                  return 0;
                }
                _ => {
                  return -1;
                }
              }
            }
          }

          fn reveal<T: Showable>(value: T) -> i64 {
            return Showable.show(value);
          }

          fn main() -> i64 {
            return reveal(Option.Some(7));
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
    let specialized_name = match main.body.last() {
        Some(nuis_semantics::model::NirStmt::Return(Some(
            nuis_semantics::model::NirExpr::Call { callee, .. },
        ))) => callee.clone(),
        other => panic!("expected main to return specialized reveal call, found {other:?}"),
    };

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == specialized_name)
        .unwrap();
    assert!(matches!(
        specialized.body.first(),
        Some(nuis_semantics::model::NirStmt::Return(Some(
            nuis_semantics::model::NirExpr::Call { callee, .. }
        ))) if callee == "impl.Showable.for.Option_i64_.show"
    ));
}

#[test]
fn monomorphizes_generic_binary_remainder_with_remainderable_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Remainderable for i64 {
            fn rem(lhs: i64, rhs: i64) -> i64 {
              return lhs % rhs;
            }
          }

          fn reduce<T: Remainderable>(lhs: T, rhs: T) -> T {
            return lhs % rhs;
          }

          fn main() -> i64 {
            return reduce(9, 4);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "reduce__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "reduce__i64")
        .unwrap();
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { op, .. }))) if *op == nuis_semantics::model::NirBinaryOp::Rem
    ));
}

#[test]
fn monomorphizes_branch_local_payload_reconstruction_before_generic_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_payload<T>(value: JustAlias<T>) -> T {
            return value.value;
          }

          fn choose(flag: bool) -> i64 {
            if flag {
              let payload = JustAlias(typed_zero());
              return takes_payload(payload);
            }
            let payload = JustAlias(typed_zero());
            return takes_payload(payload);
          }

          fn main() -> i64 {
            return choose(true);
          }
        }
        "#,
    )
    .unwrap();

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
                [
                    NirStmt::Let {
                        name,
                        ty: Some(ty),
                        value: NirExpr::StructLiteral { type_name, type_args, .. },
                    },
                    NirStmt::Return(Some(NirExpr::Call { callee, .. }))
                ] if name == "payload"
                    && ty.render() == "Just<i64>"
                    && type_name == "Just"
                    && matches!(type_args.as_slice(), [arg] if arg.render() == "i64")
                    && callee == "takes_payload__i64"
            )
                && else_body.is_empty()
    ));
    assert!(matches!(
        choose.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, type_args, .. },
        }) if name == "payload"
            && ty.render() == "Just<i64>"
            && type_name == "Just"
            && matches!(type_args.as_slice(), [arg] if arg.render() == "i64")
    ));
    assert!(matches!(
        choose.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "takes_payload__i64"
    ));
}

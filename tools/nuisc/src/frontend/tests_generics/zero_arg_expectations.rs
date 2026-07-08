use super::*;

#[test]
fn monomorphizes_zero_arg_generic_from_alias_payload_constructor_call_parameter_expectation() {
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

          fn takes_payload(value: Just<i64>) -> i64 {
            return value.value;
          }

          fn main() -> i64 {
            return takes_payload(JustAlias(typed_zero()));
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
            if callee == "takes_payload"
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { fields, .. }]
                        if matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Call { callee, .. })]
                                if field == "value" && callee == "typed_zero__i64"
                        )
                )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_if_branch_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            if 1 == 1 {
              return typed_zero();
            } else {
              return 9;
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
        main.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
                    if callee == "typed_zero__i64"
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_match_arm_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let flag: i64 = 1;
            match flag {
              1 => {
                return typed_zero();
              }
              _ => {
                return 9;
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
        main.body.get(1),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
                    if callee == "typed_zero__i64"
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_if_branch_alias_struct_literal_call_parameter_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_boxed(value: Boxed<i64>) -> i64 {
            return value.value;
          }

          fn main() -> i64 {
            if 1 == 1 {
              return takes_boxed(BoxAlias { value: typed_zero() });
            } else {
              return 9;
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
        main.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "takes_boxed"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { fields, .. }]
                                if matches!(
                                    fields.as_slice(),
                                    [(field, NirExpr::Call { callee, .. })]
                                        if field == "value" && callee == "typed_zero__i64"
                                )
                        )
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_match_arm_alias_payload_constructor_call_parameter_expectation(
) {
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

          fn takes_payload(value: Just<i64>) -> i64 {
            return value.value;
          }

          fn main() -> i64 {
            let flag: i64 = 1;
            match flag {
              1 => {
                return takes_payload(JustAlias(typed_zero()));
              }
              _ => {
                return 9;
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
        main.body.get(1),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "takes_payload"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { fields, .. }]
                                if matches!(
                                    fields.as_slice(),
                                    [(field, NirExpr::Call { callee, .. })]
                                        if field == "value" && callee == "typed_zero__i64"
                                )
                        )
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

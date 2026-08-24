use super::*;

#[test]
fn lowers_loop_syntax_into_existing_unbounded_while_form() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let mut cursor: i64 = 0;
            loop {
              cursor += 1;
              if cursor < 3 {
                continue;
              }
              break;
            }
            return cursor;
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
        Some(NirStmt::While {
            condition: NirExpr::Bool(true),
            body,
        }) if body.iter().any(|stmt| matches!(stmt, NirStmt::Break))
            && body.iter().any(|stmt| matches!(stmt, NirStmt::If { then_body, .. }
                if then_body.iter().any(|stmt| matches!(stmt, NirStmt::Continue))))
    ));
}

#[test]
fn lowers_else_if_statement_into_nested_if_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn classify(value: i64) -> i64 {
            if value < 0 {
              return 1;
            } else if value == 0 {
              return 2;
            } else {
              return 3;
            }
          }
        }
        "#,
    )
    .unwrap();

    let classify = module
        .functions
        .iter()
        .find(|function| function.name == "classify")
        .unwrap();
    assert!(matches!(
        classify.body.first(),
        Some(NirStmt::If {
            else_body,
            ..
        }) if matches!(else_body.as_slice(), [NirStmt::If { .. }])
    ));
}

#[test]
fn lowers_else_if_expression_into_nested_typed_binding_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn classify(value: i64) -> i64 {
            let code: i64 = if value < 0 {
              1
            } else if value == 0 {
              2
            } else {
              3
            };
            return code;
          }
        }
        "#,
    )
    .unwrap();

    let classify = module
        .functions
        .iter()
        .find(|function| function.name == "classify")
        .unwrap();
    assert!(matches!(
        classify.body.first(),
        Some(NirStmt::If {
            then_body,
            else_body,
            ..
        }) if matches!(then_body.as_slice(), [NirStmt::Let { name, .. }] if name == "code")
            && matches!(else_body.as_slice(), [NirStmt::If { .. }])
    ));
}

#[test]
fn lowers_if_let_and_else_if_let_into_nested_match_dispatch() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn unwrap_or(value: Option) -> i64 {
            if let Option.Some(payload) = value {
              return payload;
            } else if let Option.None = value {
              return 0;
            } else {
              return -1;
            }
          }
        }
        "#,
    )
    .unwrap();

    let unwrap_or = module
        .functions
        .iter()
        .find(|function| function.name == "unwrap_or")
        .unwrap();
    assert!(matches!(
        unwrap_or.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::VariantIs { variant, .. },
            then_body,
            else_body,
        }) if variant == "Option.Some"
            && matches!(then_body.as_slice(), [NirStmt::Let { name, .. }, NirStmt::Return(_)]
                if name == "payload")
            && matches!(else_body.as_slice(), [NirStmt::If {
                condition: NirExpr::VariantIs { variant, .. },
                ..
            }] if variant == "Option.None")
    ));
}

#[test]
fn lowers_if_let_expression_binding_into_typed_branch_target() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn increment_or_zero(value: Option) -> i64 {
            let code: i64 = if let Option.Some(payload) = value {
              payload + 1
            } else {
              0
            };
            return code;
          }
        }
        "#,
    )
    .unwrap();

    let increment = module
        .functions
        .iter()
        .find(|function| function.name == "increment_or_zero")
        .unwrap();
    assert!(matches!(
        increment.body.first(),
        Some(NirStmt::If {
            then_body,
            else_body,
            ..
        }) if matches!(then_body.as_slice(), [NirStmt::Let { name, .. }, NirStmt::Let { name: target, .. }]
            if name == "payload" && target == "code")
            && matches!(else_body.as_slice(), [NirStmt::Let { name, .. }] if name == "code")
    ));
}

#[test]
fn rejects_irrefutable_if_let_wildcard() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(value: i64) -> i64 {
            if let _ = value {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("`if let _ = ...` is irrefutable"), "{error}");
}

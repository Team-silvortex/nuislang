use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn lowers_implicit_tail_expr_returns_in_functions() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            7
          }
        }
        "#,
    )
    .unwrap();
    assert!(matches!(
        module.functions[0].body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Int(7)))]
    ));
}

#[test]
fn infers_missing_function_return_types_from_implicit_tail_expr_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn branchy(flag: bool) {
            if flag {
              7
            } else {
              match 1 {
                1 => { 8 }
                _ => { 9 }
              }
            }
          }

          fn main() {
            branchy(true)
          }
        }
        "#,
    )
    .unwrap();
    let branchy = module
        .functions
        .iter()
        .find(|function| function.name == "branchy")
        .unwrap();
    assert_eq!(
        branchy
            .return_type
            .as_ref()
            .map(|ty| ty.render())
            .as_deref(),
        Some("i64")
    );
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(
        main.return_type.as_ref().map(|ty| ty.render()).as_deref(),
        Some("i64")
    );
}

#[test]
fn infers_missing_function_return_types_from_explicit_returns() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn plus_one(x: i64) {
            return x + 1;
          }

          fn main() {
            let value = plus_one(6);
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let plus_one = module
        .functions
        .iter()
        .find(|function| function.name == "plus_one")
        .unwrap();
    assert_eq!(
        plus_one
            .return_type
            .as_ref()
            .map(|ty| ty.render())
            .as_deref(),
        Some("i64")
    );

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(
        main.return_type.as_ref().map(|ty| ty.render()).as_deref(),
        Some("i64")
    );
    match &main.body[0] {
        NirStmt::Let { ty: Some(ty), .. } => assert_eq!(ty.render(), "i64"),
        other => panic!("expected inferred typed let binding, found {other:?}"),
    }
}

#[test]
fn infers_missing_function_return_types_from_total_if_and_match_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn branchy(flag: bool) {
            let state = 1;
            if flag {
              match state {
                1 => { return 7; }
                _ => { return 8; }
              }
            } else {
              return 9;
            }
          }

          fn main() {
            return branchy(true);
          }
        }
        "#,
    )
    .unwrap();

    let branchy = module
        .functions
        .iter()
        .find(|function| function.name == "branchy")
        .unwrap();
    assert_eq!(
        branchy
            .return_type
            .as_ref()
            .map(|ty| ty.render())
            .as_deref(),
        Some("i64")
    );

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert_eq!(
        main.return_type.as_ref().map(|ty| ty.render()).as_deref(),
        Some("i64")
    );
}

#[test]
fn rejects_partial_branch_return_inference_without_explicit_return_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn branchy(flag: bool) {
            if flag {
              return 7;
            }
          }
        }
        "#,
    )
    .unwrap_err();
    assert!(error.contains("explicit return type or total terminal return branches"));
}

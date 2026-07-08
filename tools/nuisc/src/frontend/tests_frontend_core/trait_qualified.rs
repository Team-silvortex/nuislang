use super::*;

#[test]
fn lowers_explicit_trait_qualified_call_to_impl_symbol() {
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

          fn main() -> i64 {
            return Addable.add(7, 8);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Int(7), NirExpr::Int(8)])
    ));
}

#[test]
fn lowers_explicit_trait_qualified_call_with_public_helper_trait() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.add(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Int(7), NirExpr::Int(8)])
    ));
}

#[test]
fn lowers_explicit_trait_qualified_call_to_default_impl_symbol() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn zero(seed: Self) -> Self {
              return Addable.add(seed, 0);
            }
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.zero(7);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.zero"
                && matches!(args.as_slice(), [NirExpr::Int(7)])
    ));
}

#[test]
fn lowers_synthesized_default_impl_body_via_concrete_trait_impl_calls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn zero(seed: Self) -> Self {
              return Addable.add(seed, 0);
            }
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.zero(7);
          }
        }
        "#,
    )
    .unwrap();

    let synthesized = module
        .functions
        .iter()
        .find(|function| function.name == "impl.Addable.for.i64.zero")
        .unwrap();
    assert!(matches!(
        synthesized.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Var(seed), NirExpr::Int(0)] if seed == "seed")
    ));
}

#[test]
fn lowers_zero_arg_explicit_trait_qualified_call_via_explicit_self_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn zero() -> Self {
              return 0;
            }
          }

          impl Addable for i64 {
          }

          fn main() -> i64 {
            return Addable.zero<i64>();
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.zero" && args.is_empty()
    ));
}

#[test]
fn lowers_zero_arg_explicit_trait_qualified_call_via_expected_self_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn zero() -> Self {
              return 0;
            }
          }

          impl Addable for i64 {
          }

          fn main() -> i64 {
            let value: i64 = Addable.zero();
            return value;
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
        Some(NirStmt::Let { name, value, .. })
            if name == "value"
                && matches!(value, NirExpr::Call { callee, args } if callee == "impl.Addable.for.i64.zero" && args.is_empty())
    ));
}

#[test]
fn rejects_zero_arg_explicit_trait_qualified_call_without_self_anchor() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn zero() -> Self {
              return 0;
            }
          }

          impl Addable for i64 {
          }

          fn main() -> i64 {
            Addable.zero();
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("trait method `Addable.zero` without receiver argument cannot infer `Self`"),
        "{error}"
    );
    assert!(error.contains("Addable.zero<Type>()"), "{error}");
    assert!(error.contains("expected return type"), "{error}");
}

#[test]
fn lowers_fully_qualified_helper_trait_call_to_impl_symbol() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Helper.Addable.add(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Helper.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Int(7), NirExpr::Int(8)])
    ));
}

#[test]
fn reports_missing_impl_for_explicit_qualified_trait_call_on_concrete_type() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          fn main() -> i64 {
            return Helper.Addable.add(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("trait `Helper.Addable` has no impl for `i64`"),
        "{error}"
    );
    assert!(error.contains("Helper.Addable.add"), "{error}");
}

#[test]
fn suggests_trait_method_name_for_explicit_qualified_trait_call() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Helper.Addable.ad(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(error.contains("does not define method `ad`"), "{error}");
    assert!(
        error.contains("did you mean `Helper.Addable.add`?"),
        "{error}"
    );
}

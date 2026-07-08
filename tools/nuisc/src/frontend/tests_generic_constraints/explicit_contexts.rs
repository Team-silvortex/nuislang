use super::*;

#[test]
fn reports_explicit_function_generic_arg_bound_failure_inside_if_result_branch() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            let value = if true {
              keep<Text>("hi")
            } else {
              keep<i64>(0)
            };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains(
            "function `main` body local `value` if-then call `keep` generic parameter `U`"
        ),
        "{error}"
    );
}

#[test]
fn reports_explicit_function_generic_arg_bound_failure_inside_match_result_branch() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            let value = match 1 {
              1 => {
                keep<Text>("hi")
              }
              _ => {
                keep<i64>(0)
              }
            };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains(
            "function `main` body local `value` match-arm call `keep` generic parameter `U`"
        ),
        "{error}"
    );
}

#[test]
fn reports_explicit_function_generic_arg_bound_failure_inside_lambda_body() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            let value = apply(0, |x: i64| -> i64 {
              keep<Text>("hi");
              return x;
            });
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains("function `main` body lambda body call `keep` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn accepts_explicit_function_generic_arg_when_where_clause_bounds_are_satisfied() {
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

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            return keep<i64>(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "keep__i64"));
}

#[test]
fn accepts_function_generic_use_site_bound_through_visible_helper_trait_name_variant() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn keep<U: Helper.Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = super::parse_nuis_ast(
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
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "keep__i64"));
}

#[test]
fn accepts_function_generic_use_site_with_helper_and_local_multi_bounds() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn keep<U: Helper.Addable + Printable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = super::parse_nuis_ast(
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
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "keep__i64"));
}

use super::*;

#[test]
fn rejects_type_alias_where_clause_bound_at_use_site() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          type Alias<T> where T: Addable = T;

          fn main() -> i64 {
            let text: String = "hi";
            let value: Alias<String> = text;
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `String` does not satisfy bound `Addable`"));
    assert!(error.contains("type alias `Alias` generic parameter `T`"));
}

#[test]
fn rejects_struct_where_clause_bound_at_field_use_site() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let text: String = "hi";
            let boxed: Boxed<String> = Boxed { value: text };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `String` does not satisfy bound `Addable`"));
    assert!(error.contains("function `main` body local `boxed`"));
    assert!(error.contains("via struct `Boxed` generic parameter `T`"));
}

#[test]
fn reports_explicit_function_generic_arg_bound_failure_at_use_site() {
    let error = parse_nuis_module(
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

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            let text: Text = "hi";
            keep<Text>(text);
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
        error.contains("function `main` body call `keep` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn reports_inferred_function_generic_arg_bound_failure_inside_if_result_branch() {
    let error = parse_nuis_module(
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

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            let value = if true {
              let text: Text = "hi";
              keep(text)
            } else {
              keep(0)
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
fn reports_inferred_function_generic_arg_bound_failure_inside_match_result_branch() {
    let error = parse_nuis_module(
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

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            let value = match 1 {
              1 => {
                let text: Text = "hi";
                keep(text)
              }
              _ => {
                keep(0)
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
fn reports_inferred_function_generic_arg_bound_failure_inside_lambda_body() {
    let error = parse_nuis_module(
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

          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            let value = apply(0, |x: i64| -> i64 {
              let text: Text = "hi";
              keep(text);
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
fn reports_explicit_function_generic_arg_failure_for_second_required_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn keep<U: Addable + Printable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            keep<i64>(7);
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `i64` does not satisfy bound `Printable`"),
        "{error}"
    );
    assert!(
        error.contains("function `main` body call `keep` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn accepts_explicit_function_generic_arg_when_all_required_bounds_are_satisfied() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

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

          fn keep<U: Addable + Printable>(value: U) -> U {
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
fn reports_explicit_function_generic_arg_bound_failure_from_where_clause() {
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
            keep<Text>("hi");
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
        error.contains("function `main` body call `keep` generic parameter `U`"),
        "{error}"
    );
}

use super::*;

#[test]
fn rejects_type_alias_generic_arg_that_does_not_satisfy_bound() {
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

          type Alias<T: Addable> = T;

          fn main() -> i64 {
            let value: Alias<Text> = "hi";
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    println!("{error}");
    assert!(error.contains("type `Text` does not satisfy bound `Addable`"));
    assert!(error.contains("type alias `Alias` generic parameter `T`"));
}

#[test]
fn rejects_unannotated_generic_struct_literal_explicit_type_arg_that_violates_bound() {
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

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed<Text> { value: "hi" };
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
    assert!(error.contains("struct literal `Boxed`"), "{error}");
    assert!(
        error.contains("via struct `Boxed` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn rejects_lambda_body_local_alias_bound_failure() {
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

          type Alias<T: Addable> = T;

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn keep<T>(value: T) -> T {
            return apply(value, |x: T| -> T {
              let local: Alias<Text> = "hi";
              return x;
            });
          }

          fn main() -> i64 {
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
        error.contains("function `keep` body lambda body local `local`"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Alias` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn rejects_if_result_branch_struct_literal_type_arg_that_violates_bound_with_branch_context() {
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

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let boxed = if true {
              Boxed<Text> { value: "hi" }
            } else {
              Boxed<i64> { value: 0 }
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
    assert!(error.contains("function `main` body if-then"), "{error}");
    assert!(error.contains("struct literal `Boxed`"), "{error}");
}

#[test]
fn rejects_match_result_branch_struct_literal_type_arg_that_violates_bound_with_arm_context() {
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

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let boxed = match 1 {
              1 => {
                Boxed<Text> { value: "hi" }
              }
              _ => {
                Boxed<i64> { value: 0 }
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
    assert!(error.contains("function `main` body match-arm"), "{error}");
    assert!(error.contains("struct literal `Boxed`"), "{error}");
}

#[test]
fn rejects_non_trait_shaped_generic_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type Alias<T: Pipe<i64>> = T;

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("generic bounds currently require a bare trait name"));
    assert!(error.contains("Pipe<i64>"));
}

#[test]
fn accepts_type_alias_generic_arg_when_outer_generic_param_satisfies_bound() {
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

          type Alias<T: Addable> = T;

          fn keep<U: Addable>(value: Alias<U>) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
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
fn reports_nested_alias_bound_failure_with_alias_chain_context() {
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

          type Inner<T: Addable> = T;
          type Outer<T> = Inner<T>;

          fn main() -> i64 {
            let value: Outer<Text> = "hi";
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `T` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(error.contains("type alias `Outer` target"), "{error}");
    assert!(
        error.contains("type alias `Inner` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_alias_definition_missing_bound_through_alias_chain() {
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

          type Inner<T: Addable> = T;
          type Outer<T> = Inner<T>;

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `T` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(error.contains("type alias `Outer` target"), "{error}");
    assert!(
        error.contains("type alias `Inner` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_alias_chain_second_missing_bound_through_multi_bound_alias() {
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

          type Inner<T: Addable + Printable> = T;
          type Outer<T: Addable> = Inner<T>;

          fn main() -> i64 {
            let value: Outer<i64> = 7;
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `T` does not satisfy bound `Printable`"),
        "{error}"
    );
    assert!(error.contains("type alias `Outer` target"), "{error}");
    assert!(
        error.contains("type alias `Inner` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_function_generic_call_arg_bound_failure_at_use_site() {
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

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          fn keep<U: Addable>(value: Outer<U>) -> U {
            return value;
          }

          fn main() -> i64 {
            let text: Text = "hi";
            keep(text);
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
fn accepts_alias_generic_arg_when_outer_generic_param_satisfies_all_multi_bounds() {
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

          type Alias<T: Addable + Printable> = T;

          fn keep<U: Addable + Printable>(value: Alias<U>) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
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
fn accepts_repeated_where_predicates_that_merge_into_multi_bounds() {
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

          fn keep<U>(value: U) -> U where U: Addable, U: Printable {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
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

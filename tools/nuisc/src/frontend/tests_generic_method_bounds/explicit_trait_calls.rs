use super::*;

#[test]
fn reports_missing_bound_for_explicit_trait_qualified_call_on_type_param() {
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

          fn bump<T>(value: T) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `bump` body calls trait method `Addable.add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_bound_for_explicit_trait_qualified_call_on_alias_wrapped_type_param() {
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

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T>(value: Outer<T>) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `bump` body via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls trait method `Addable.add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn accepts_explicit_trait_qualified_call_on_bound_type_param() {
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

          fn bump<T: Addable>(value: T) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_explicit_trait_qualified_call_when_trait_is_one_of_multiple_bounds() {
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

          fn bump<T: Addable + Printable>(value: T) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn reports_explicit_trait_qualified_call_when_multiple_bounds_miss_required_trait() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T: Showable + Printable>(value: T) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "calls trait method `Addable.add` on generic parameter `T` but declared bounds `Showable + Printable` do not satisfy required trait `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn accepts_explicit_trait_qualified_call_with_public_helper_trait_bound() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Addable>(value: T) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
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

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_fully_qualified_helper_trait_bound_and_explicit_call() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Helper.Addable>(value: T) -> T {
            return Helper.Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
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

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn reports_mixed_bare_and_qualified_trait_names_consistently() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Addable>(value: T) -> T {
            return Helper.Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
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

    let error = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("bound `Addable` uses a different visible name for the same trait"),
        "{error}"
    );
    assert!(
        error.contains("use `Helper.Addable` consistently"),
        "{error}"
    );
}

#[test]
fn reports_mixed_bare_and_qualified_trait_names_consistently_with_multiple_bounds() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T: Addable + Printable>(value: T) -> T {
            return Helper.Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
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

    let error = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("bound `Addable` uses a different visible name for the same trait"),
        "{error}"
    );
    assert!(
        error.contains("use `Helper.Addable` consistently"),
        "{error}"
    );
}

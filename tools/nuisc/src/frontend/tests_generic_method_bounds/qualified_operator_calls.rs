use super::*;

#[test]
fn accepts_qualified_helper_trait_bound_for_operator_call() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Helper.Addable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
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
fn accepts_qualified_helper_trait_bound_for_operator_call_with_multiple_bounds() {
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

          fn bump<T: Helper.Addable + Printable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
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
fn accepts_qualified_helper_trait_bound_for_operator_call_with_where_clause() {
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

          fn bump<T>(lhs: T, rhs: T) -> T where T: Printable, T: Helper.Addable {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
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
fn accepts_bare_bound_name_for_operator_call_with_multiple_bounds_when_helper_variant_is_visible() {
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

          fn bump<T: Addable + Printable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
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
fn accepts_qualified_helper_trait_bound_for_operator_call_through_alias_chain_and_if() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Helper.Addable>(lhs: Outer<T>, rhs: Outer<T>) -> T {
            if true {
              return lhs + rhs;
            }
            return lhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
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

use super::*;

#[test]
fn accepts_qualified_helper_trait_bound_for_equality_operator_with_multiple_bounds() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn same<T: Helper.Equatable + Printable>(lhs: T, rhs: T) -> bool {
            return lhs == rhs;
          }

          fn main() -> i64 {
            if same(7, 7) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert_eq!(module.unit, "Main");
}

#[test]
fn accepts_bare_bound_name_for_equality_operator_with_multiple_bounds_when_helper_variant_is_visible(
) {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn same<T: Equatable + Printable>(lhs: T, rhs: T) -> bool {
            return lhs == rhs;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert_eq!(module.unit, "Main");
}

#[test]
fn accepts_qualified_helper_trait_bounds_for_mul_div_rem_operators() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Multipliable for i64 {
            fn mul(lhs: i64, rhs: i64) -> i64 {
              return lhs * rhs;
            }
          }

          impl Helper.Dividable for i64 {
            fn div(lhs: i64, rhs: i64) -> i64 {
              return lhs / rhs;
            }
          }

          impl Helper.Remainderable for i64 {
            fn rem(lhs: i64, rhs: i64) -> i64 {
              return lhs % rhs;
            }
          }

          fn mul_it<T: Helper.Multipliable>(lhs: T, rhs: T) -> T {
            return lhs * rhs;
          }

          fn div_it<T: Helper.Dividable>(lhs: T, rhs: T) -> T {
            return lhs / rhs;
          }

          fn rem_it<T: Helper.Remainderable>(lhs: T, rhs: T) -> T {
            return lhs % rhs;
          }

          fn main() -> i64 {
            return mul_it(6, 7) + div_it(8, 2) + rem_it(9, 4);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          pub trait Dividable {
            fn div(lhs: Self, rhs: Self) -> Self;
          }

          pub trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "mul_it__i64"));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "div_it__i64"));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "rem_it__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bounds_for_order_operators() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Orderable for i64 {
            fn lt(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs;
            }
            fn le(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs || lhs == rhs;
            }
            fn gt(lhs: i64, rhs: i64) -> bool {
              return !(lhs <= rhs);
            }
            fn ge(lhs: i64, rhs: i64) -> bool {
              return !(lhs < rhs);
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn ordered<T: Helper.Orderable + Printable>(lhs: T, rhs: T) -> bool {
            return lhs < rhs;
          }

          fn main() -> i64 {
            if ordered(1, 2) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "ordered__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bounds_for_unary_not_and_neg_on_custom_type() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          struct Pair {
            value: i64,
          }

          impl Helper.Notable for Pair {
            fn not(value: Pair) -> bool {
              return value.value == 0;
            }
          }

          impl Helper.Negatable for Pair {
            fn neg(value: Pair) -> Pair {
              return Pair { value: 0 - value.value };
            }
          }

          fn empty<T: Helper.Notable>(value: T) -> bool {
            return !value;
          }

          fn flip<T: Helper.Negatable>(value: T) -> T {
            return -value;
          }

          fn main() -> i64 {
            let zero: Pair = Pair { value: 0 };
            let seven: Pair = Pair { value: 7 };
            let is_empty = empty(zero);
            let flipped = flip(seven);
            if is_empty {
              return flipped.value;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Notable {
            fn not(value: Self) -> bool;
          }

          pub trait Negatable {
            fn neg(value: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "empty__Pair"));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "flip__Pair"));
}

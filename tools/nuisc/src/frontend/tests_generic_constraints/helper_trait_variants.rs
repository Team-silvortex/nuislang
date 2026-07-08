#[test]
fn reports_ambiguous_function_generic_use_site_bound_across_helper_trait_variants() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu HelperA;
        use cpu HelperB;

        mod cpu Main {
          impl HelperA.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl HelperB.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_a_ast = super::parse_nuis_ast(
        r#"
        mod cpu HelperA {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();
    let helper_b_ast = super::parse_nuis_ast(
        r#"
        mod cpu HelperB {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error =
        super::lower_project_ast_to_nir(&main_ast, &[helper_a_ast, helper_b_ast]).unwrap_err();
    assert!(
        error.contains("type `i64` ambiguously satisfies bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains("function `main` body call `keep` generic parameter `U`"),
        "{error}"
    );
    assert!(error.contains("HelperA.Addable"), "{error}");
    assert!(error.contains("HelperB.Addable"), "{error}");
}

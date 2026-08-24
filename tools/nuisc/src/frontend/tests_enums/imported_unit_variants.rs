use super::*;

#[test]
fn imported_helper_match_preserves_unit_enum_parameter_dispatch() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          fn main() -> i64 {
            return code(Kind.Import);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub enum Kind {
            Module,
            Import,
            Item,
          }

          pub fn code(kind: Kind) -> i64 {
            match kind {
              Kind.Module => {
                return 1;
              }
              Kind.Import => {
                return 2;
              }
              Kind.Item => {
                return 3;
              }
              _ => {
                return 0;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "Helper.code")
        .unwrap();
    assert!(matches!(
        helper.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::VariantIs { variant, .. },
            ..
        }) if variant == "Kind.Module"
    ));
}

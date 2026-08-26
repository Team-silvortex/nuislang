use super::*;

#[test]
fn repeated_same_thread_lowering_resets_try_expansion_names() {
    let source = r#"
        mod cpu Main {
          enum Error {
            Invalid,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn take(value: i64) -> Result<i64, Error> {
            return Result.Ok(value);
          }

          fn main() -> Result<i64, Error> {
            let first: i64 = take(1)?;
            let second: i64 = take(2)?;
            return Result.Ok(first + second);
          }
        }
    "#;
    let ast = parse_nuis_ast(source).expect("parse reproducibility fixture");
    let first = lower_ast_to_nir(&ast).expect("lower first module");
    let second = lower_ast_to_nir(&ast).expect("lower second module");
    let first = crate::render::render_nir(&first);
    let second = crate::render::render_nir(&second);
    assert_eq!(first, second);
    assert!(first.contains("__nuis_try_result_0"));
    assert!(first.contains("__nuis_try_result_1"));
    assert!(!first.contains("__nuis_try_result_2"));
}

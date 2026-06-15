use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstExpr, AstStmt, NirStmt};
use std::fs;

fn count_ifs(body: &[NirStmt]) -> usize {
    body.iter().map(count_ifs_in_stmt).sum()
}

fn count_ifs_in_stmt(stmt: &NirStmt) -> usize {
    match stmt {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => 1 + count_ifs(then_body) + count_ifs(else_body),
        NirStmt::While { body, .. } => count_ifs(body),
        _ => 0,
    }
}

#[test]
fn parses_try_postfix_expression_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> Result<i64, Error> {
            let value: i64 = fetch()?;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &ast.functions[0].body[0],
        AstStmt::Let {
            value: AstExpr::Try(inner),
            ..
        } if matches!(inner.as_ref(), AstExpr::Call { callee, .. } if callee == "fetch")
    ));
}

#[test]
fn lowers_try_let_into_result_match_propagation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = fetch(seed)?;
            return Result.Ok(value + 1);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(_)
        ]
    ));
}

#[test]
fn lowers_try_inside_call_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = add_one(fetch(seed)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(_)
        ]
    ));
}

#[test]
fn lowers_try_inside_binary_expression() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = fetch(seed)? + 1;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(_)
        ]
    ));
}

#[test]
fn lowers_try_inside_method_receiver() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          struct Counter {
            value: i64,
          }

          fn fetch(seed: i64) -> Result<Counter, Error> {
            if seed > 0 {
              return Result.Ok(Counter { value: seed });
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = fetch(seed)?.value;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(_)
        ]
    ));
}

#[test]
fn lowers_try_inside_struct_field_value() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          struct Counter {
            value: i64,
          }

          struct Wrap {
            inner: i64,
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let wrapped: Wrap = Wrap { inner: fetch(seed)? };
            return Result.Ok(wrapped.inner);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(_)
        ]
    ));
}

#[test]
fn lowers_try_inside_await_operand() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(work(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          async fn compute(seed: i64) -> Result<i64, Error> {
            let value: i64 = await fetch(seed)?;
            return Result.Ok(value);
          }

          async fn main() -> Result<i64, Error> {
            return await compute(3);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(_)
        ]
    ));
}

#[test]
fn lowers_multiple_try_operands_inside_single_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn add(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn main() -> Result<i64, Error> {
            let value: i64 = add(fetch(2)?, fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_mixed_try_chain_inside_struct_and_binary_expression() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          struct Pair {
            left: i64,
            right: i64,
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main() -> Result<i64, Error> {
            let pair: Pair = Pair {
              left: fetch(4)?,
              right: fetch(5)? + 1,
            };
            return Result.Ok(pair.left + pair.right);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_try_inside_if_expression_branch_used_as_call_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          fn main(flag: bool) -> Result<i64, Error> {
            let value: i64 = add_one(if flag {
              fetch(7)?
            } else {
              0
            });
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_try_inside_match_expression_arm_used_in_binary_expression() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = (match seed {
              0 => { 0 }
              _ => { fetch(seed)? }
            }) + 1;
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_multiple_try_points_inside_nested_if_with_match_branch() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(flag: bool, seed: i64) -> Result<i64, Error> {
            let value: i64 = if flag {
              match seed {
                1 => { fetch(1)? }
                _ => { fetch(seed)? + 1 }
              }
            } else {
              0
            };
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 3, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_nested_match_inside_if_branch_with_multiple_try_operands() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(flag: bool, seed: i64) -> Result<i64, Error> {
            let value: i64 = if flag {
              (match seed {
                1 => { fetch(1)? }
                _ => { fetch(seed)? }
              }) + fetch(3)?
            } else {
              0
            };
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_match_expression_as_call_argument_alongside_try_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn combine(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = combine(match seed {
              1 => { fetch(1)? }
              _ => { fetch(seed)? }
            }, fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_parenthesized_match_expression_as_call_argument_alongside_try_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn combine(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = combine((match seed {
              1 => { fetch(1)? }
              _ => { fetch(seed)? }
            }), fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_if_expression_as_call_argument_alongside_try_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn combine(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn main(flag: bool) -> Result<i64, Error> {
            let value: i64 = combine(if flag {
              fetch(1)?
            } else {
              fetch(2)?
            }, fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_parenthesized_if_expression_as_call_argument_alongside_try_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn combine(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn main(flag: bool) -> Result<i64, Error> {
            let value: i64 = combine((if flag {
              fetch(1)?
            } else {
              fetch(2)?
            }), fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_if_expression_as_method_receiver_with_try_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(flag: bool) -> Result<i64, Error> {
            let value: i64 = (if flag {
              fetch(1)?
            } else {
              fetch(2)?
            }).add(fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_match_expression_as_method_receiver_with_try_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let value: i64 = (match seed {
              1 => { fetch(1)? }
              _ => { fetch(seed)? }
            }).add(fetch(3)?);
            return Result.Ok(value);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_if_expression_as_struct_field_value_with_try_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          struct Wrap {
            inner: i64,
            extra: i64,
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(flag: bool) -> Result<i64, Error> {
            let wrapped: Wrap = Wrap {
              inner: if flag {
                fetch(1)?
              } else {
                fetch(2)?
              },
              extra: fetch(3)?,
            };
            return Result.Ok(wrapped.inner + wrapped.extra);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_match_expression_as_struct_field_value_with_try_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          struct Wrap {
            inner: i64,
            extra: i64,
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.InvalidInput);
          }

          fn main(seed: i64) -> Result<i64, Error> {
            let wrapped: Wrap = Wrap {
              inner: match seed {
                1 => { fetch(1)? }
                _ => { fetch(seed)? }
              },
              extra: fetch(3)?,
            };
            return Result.Ok(wrapped.inner + wrapped.extra);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(count_ifs(&function.body) >= 4, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_await_over_if_expression_returning_result_task() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(work(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn compute(flag: bool) -> Result<i64, Error> {
            let value: i64 = await (if flag {
              fetch(1)
            } else {
              fetch(2)
            })?;
            return Result.Ok(value);
          }

          async fn main() -> Result<i64, Error> {
            return await compute(true);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_await_over_match_expression_returning_result_task() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(work(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn compute(seed: i64) -> Result<i64, Error> {
            let value: i64 = await (match seed {
              1 => { fetch(1) }
              _ => { fetch(seed) }
            })?;
            return Result.Ok(value);
          }

          async fn main() -> Result<i64, Error> {
            return await compute(2);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_if_expression_inside_try_before_await() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(work(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn compute(flag: bool) -> Result<i64, Error> {
            let task: Task<i64> = (if flag {
              fetch(1)
            } else {
              fetch(2)
            })?;
            let value: i64 = await task;
            return Result.Ok(value);
          }

          async fn main() -> Result<i64, Error> {
            return await compute(true);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn lowers_match_expression_inside_try_before_await() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          async fn work(seed: i64) -> i64 {
            return seed + 1;
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            if seed > 0 {
              return Result.Ok(spawn(work(seed)));
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn compute(seed: i64) -> Result<i64, Error> {
            let task: Task<i64> = (match seed {
              1 => { fetch(1) }
              _ => { fetch(seed) }
            })?;
            let value: i64 = await task;
            return Result.Ok(value);
          }

          async fn main() -> Result<i64, Error> {
            return await compute(2);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    assert!(count_ifs(&function.body) >= 2, "{:?}", function.body);
    assert!(matches!(function.body.last(), Some(NirStmt::Return(_))));
}

#[test]
fn parses_memory_task_result_control_flow_example_into_nir() {
    let source = fs::read_to_string(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_result_control_flow.ns",
    )
    .expect("example source should be readable");

    let module = parse_nuis_module(&source).expect("example source should lower to NIR");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("expected choose function in example source");

    assert!(choose.is_async);
    assert!(count_ifs(&choose.body) >= 2, "{:?}", choose.body);
    assert!(matches!(choose.body.last(), Some(NirStmt::Return(_))));
}

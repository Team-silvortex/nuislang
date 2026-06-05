use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn combines_higher_order_specialization_with_trait_generic_monomorphization() {
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

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs.add(rhs);
          }

          fn apply_and_sum(x: i64, y: i64, f: Fn1<i64, i64>) -> i64 {
            return sum_two(f(x), y);
          }

          fn main() -> i64 {
            return apply_and_sum(6, 1, |x: i64| -> i64 { return x; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda function");
    let higher_order = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_and_sum_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        higher_order.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "sum_two__i64"
                && matches!(args.as_slice(), [NirExpr::Call { callee: inner, .. }, NirExpr::Var(y)] if inner == &lambda.name && y == "y")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order.name
    ));
}

#[test]
fn lowers_generic_fn1_higher_order_lambda_family() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee == "impl.Addable.for.i64.add"
                || callee.starts_with("__lambda_main_")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_generic_fn1_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: Mapper<T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_nested_generic_fn1_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;
          type NestedMapper<T> = Mapper<T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: NestedMapper<T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized nested higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_higher_order_call_scrutinee_match_inside_while_via_hoisted_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match apply(state, |x: i64| -> i64 { return x + 1; }) {
                3 => { return 7; },
                _ => { return 9; }
              }
            }
            return 0;
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
    match &function.body[1] {
        NirStmt::While { body, .. } => {
            assert!(matches!(
                body.as_slice(),
                [
                    NirStmt::Let { name, value: NirExpr::Call { .. }, .. },
                    NirStmt::If { .. }
                ] if name.starts_with("__match_scrutinee_")
            ));
        }
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_generic_fn3_higher_order_lambda_family() {
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

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(5, 1, 1, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_generic_fn3_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Reducer<T>) -> T {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(5, 1, 1, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 alias higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

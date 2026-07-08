use super::*;

#[test]
fn lowers_non_numeric_binary_add_via_addable_impl() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for Pair {
            fn add(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value + rhs.value };
            }
          }

          fn main() -> i64 {
            let sum: Pair = Pair { value: 1 } + Pair { value: 2 };
            return sum.value;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "sum" && callee == "impl.Addable.for.Pair.add"
    ));
}

#[test]
fn lowers_non_numeric_binary_sub_mul_div_via_trait_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Subtractable {
            fn sub(lhs: Self, rhs: Self) -> Self;
          }

          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          trait Dividable {
            fn div(lhs: Self, rhs: Self) -> Self;
          }

          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Subtractable for Pair {
            fn sub(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value - rhs.value };
            }
          }

          impl Multipliable for Pair {
            fn mul(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value * rhs.value };
            }
          }

          impl Dividable for Pair {
            fn div(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value / rhs.value };
            }
          }

          impl Remainderable for Pair {
            fn rem(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value % rhs.value };
            }
          }

          fn main() -> i64 {
            let diff: Pair = Pair { value: 6 } - Pair { value: 2 };
            let prod: Pair = Pair { value: 3 } * Pair { value: 4 };
            let quot: Pair = Pair { value: 8 } / Pair { value: 2 };
            let rest: Pair = Pair { value: 9 } % Pair { value: 4 };
            return diff.value + prod.value + quot.value + rest.value;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "diff" && callee == "impl.Subtractable.for.Pair.sub"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "prod" && callee == "impl.Multipliable.for.Pair.mul"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "quot" && callee == "impl.Dividable.for.Pair.div"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "rest" && callee == "impl.Remainderable.for.Pair.rem"
    ));
}

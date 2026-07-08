use super::*;

#[test]
fn parses_binary_operator_precedence_in_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return 1 + 2 * 3;
          }
        }
        "#,
    )
    .unwrap();

    let main = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(AstStmt::Return(Some(AstExpr::Binary { op, lhs, rhs })))
            if *op == AstBinaryOp::Add
                && matches!(lhs.as_ref(), AstExpr::Int(1))
                && matches!(
                    rhs.as_ref(),
                    AstExpr::Binary {
                        op: inner_op,
                        lhs: inner_lhs,
                        rhs: inner_rhs,
                    } if *inner_op == AstBinaryOp::Mul
                        && matches!(inner_lhs.as_ref(), AstExpr::Int(2))
                        && matches!(inner_rhs.as_ref(), AstExpr::Int(3))
                )
    ));
}

#[test]
fn parses_logical_and_comparison_precedence_in_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> bool {
            return 1 == 1 || 2 == 3 && 4 < 5;
          }
        }
        "#,
    )
    .unwrap();

    let main = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(AstStmt::Return(Some(AstExpr::Binary { op, lhs, rhs })))
            if *op == AstBinaryOp::Or
                && matches!(
                    lhs.as_ref(),
                    AstExpr::Binary {
                        op: lhs_op,
                        lhs: lhs_lhs,
                        rhs: lhs_rhs,
                    } if *lhs_op == AstBinaryOp::Eq
                        && matches!(lhs_lhs.as_ref(), AstExpr::Int(1))
                        && matches!(lhs_rhs.as_ref(), AstExpr::Int(1))
                )
                && matches!(
                    rhs.as_ref(),
                    AstExpr::Binary {
                        op: rhs_op,
                        lhs: rhs_lhs,
                        rhs: rhs_rhs,
                    } if *rhs_op == AstBinaryOp::And
                        && matches!(
                            rhs_lhs.as_ref(),
                            AstExpr::Binary {
                                op: inner_eq_op,
                                lhs: inner_eq_lhs,
                                rhs: inner_eq_rhs,
                            } if *inner_eq_op == AstBinaryOp::Eq
                                && matches!(inner_eq_lhs.as_ref(), AstExpr::Int(2))
                                && matches!(inner_eq_rhs.as_ref(), AstExpr::Int(3))
                        )
                        && matches!(
                            rhs_rhs.as_ref(),
                            AstExpr::Binary {
                                op: inner_lt_op,
                                lhs: inner_lt_lhs,
                                rhs: inner_lt_rhs,
                            } if *inner_lt_op == AstBinaryOp::Lt
                                && matches!(inner_lt_lhs.as_ref(), AstExpr::Int(4))
                                && matches!(inner_lt_rhs.as_ref(), AstExpr::Int(5))
                        )
                )
    ));
}

#[test]
fn lowers_logical_and_comparison_precedence_in_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let ready: bool = 1 == 1 || 2 == 3 && 4 < 5;
            if ready {
              return 1;
            }
            return 0;
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
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "ready"
            && *op == NirBinaryOp::Or
            && matches!(
                lhs.as_ref(),
                NirExpr::Binary {
                    op: lhs_op,
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                } if *lhs_op == NirBinaryOp::Eq
                    && matches!(lhs_lhs.as_ref(), NirExpr::Int(1))
                    && matches!(lhs_rhs.as_ref(), NirExpr::Int(1))
            )
            && matches!(
                rhs.as_ref(),
                NirExpr::Binary {
                    op: rhs_op,
                    lhs: rhs_lhs,
                    rhs: rhs_rhs,
                } if *rhs_op == NirBinaryOp::And
                    && matches!(
                        rhs_lhs.as_ref(),
                        NirExpr::Binary {
                            op: inner_eq_op,
                            lhs: inner_eq_lhs,
                            rhs: inner_eq_rhs,
                        } if *inner_eq_op == NirBinaryOp::Eq
                            && matches!(inner_eq_lhs.as_ref(), NirExpr::Int(2))
                            && matches!(inner_eq_rhs.as_ref(), NirExpr::Int(3))
                    )
                    && matches!(
                        rhs_rhs.as_ref(),
                        NirExpr::Binary {
                            op: inner_lt_op,
                            lhs: inner_lt_lhs,
                            rhs: inner_lt_rhs,
                        } if *inner_lt_op == NirBinaryOp::Lt
                            && matches!(inner_lt_lhs.as_ref(), NirExpr::Int(4))
                            && matches!(inner_lt_rhs.as_ref(), NirExpr::Int(5))
                    )
            )
    ));
}

#[test]
fn lowers_parenthesized_logical_precedence_in_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let grouped: bool = (1 == 1 || 2 == 3) && 4 < 5;
            if grouped {
              return 1;
            }
            return 0;
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
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "grouped"
            && *op == NirBinaryOp::And
            && matches!(
                lhs.as_ref(),
                NirExpr::Binary {
                    op: lhs_op,
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                } if *lhs_op == NirBinaryOp::Or
                    && matches!(
                        lhs_lhs.as_ref(),
                        NirExpr::Binary { op: inner_op, .. } if *inner_op == NirBinaryOp::Eq
                    )
                    && matches!(
                        lhs_rhs.as_ref(),
                        NirExpr::Binary { op: inner_op, .. } if *inner_op == NirBinaryOp::Eq
                    )
            )
            && matches!(
                rhs.as_ref(),
                NirExpr::Binary { op: rhs_op, .. } if *rhs_op == NirBinaryOp::Lt
            )
    ));
}

#[test]
fn lowers_overloaded_binary_precedence_without_parentheses() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for Pair {
            fn add(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value + rhs.value };
            }
          }

          impl Multipliable for Pair {
            fn mul(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value * rhs.value };
            }
          }

          fn main() -> i64 {
            let mixed: Pair = Pair { value: 1 } + Pair { value: 2 } * Pair { value: 3 };
            return mixed.value;
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
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "mixed"
            && callee == "impl.Addable.for.Pair.add"
            && matches!(
                args.as_slice(),
                [
                    NirExpr::StructLiteral { fields: lhs_fields, .. },
                    NirExpr::Call { callee: rhs_callee, args: rhs_args }
                ] if matches!(lhs_fields.as_slice(), [(field, NirExpr::Int(1))] if field == "value")
                    && rhs_callee == "impl.Multipliable.for.Pair.mul"
                    && matches!(
                        rhs_args.as_slice(),
                        [
                            NirExpr::StructLiteral { fields: mul_lhs_fields, .. },
                            NirExpr::StructLiteral { fields: mul_rhs_fields, .. }
                        ] if matches!(mul_lhs_fields.as_slice(), [(field, NirExpr::Int(2))] if field == "value")
                            && matches!(mul_rhs_fields.as_slice(), [(field, NirExpr::Int(3))] if field == "value")
                    )
            )
    ));
}

#[test]
fn lowers_builtin_remainder_with_multiplicative_precedence() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 9 % 4 * 2;
            return value;
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
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "value"
            && *op == NirBinaryOp::Mul
            && matches!(
                lhs.as_ref(),
                NirExpr::Binary {
                    op: lhs_op,
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                } if *lhs_op == NirBinaryOp::Rem
                    && matches!(lhs_lhs.as_ref(), NirExpr::Int(9))
                    && matches!(lhs_rhs.as_ref(), NirExpr::Int(4))
            )
            && matches!(rhs.as_ref(), NirExpr::Int(2))
    ));
}

#[test]
fn lowers_overloaded_binary_remainder_via_trait_impl() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Remainderable for Pair {
            fn rem(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value % rhs.value };
            }
          }

          fn main() -> i64 {
            let rest: Pair = Pair { value: 9 } % Pair { value: 4 };
            return rest.value;
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
        }) if name == "rest" && callee == "impl.Remainderable.for.Pair.rem"
    ));
}

#[test]
fn lowers_overloaded_binary_parentheses_and_left_associativity() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Subtractable {
            fn sub(lhs: Self, rhs: Self) -> Self;
          }

          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for Pair {
            fn add(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value + rhs.value };
            }
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

          fn main() -> i64 {
            let grouped: Pair = (Pair { value: 1 } + Pair { value: 2 }) * Pair { value: 3 };
            let folded: Pair = Pair { value: 9 } - Pair { value: 3 } - Pair { value: 1 };
            return grouped.value + folded.value;
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
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "grouped"
            && callee == "impl.Multipliable.for.Pair.mul"
            && matches!(
                args.as_slice(),
                [
                    NirExpr::Call { callee: lhs_callee, .. },
                    NirExpr::StructLiteral { fields: rhs_fields, .. }
                ] if lhs_callee == "impl.Addable.for.Pair.add"
                    && matches!(rhs_fields.as_slice(), [(field, NirExpr::Int(3))] if field == "value")
            )
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "folded"
            && callee == "impl.Subtractable.for.Pair.sub"
            && matches!(
                args.as_slice(),
                [
                    NirExpr::Call { callee: lhs_callee, args: lhs_args },
                    NirExpr::StructLiteral { fields: rhs_fields, .. }
                ] if lhs_callee == "impl.Subtractable.for.Pair.sub"
                    && matches!(
                        lhs_args.as_slice(),
                        [
                            NirExpr::StructLiteral { fields: first_fields, .. },
                            NirExpr::StructLiteral { fields: second_fields, .. }
                        ] if matches!(first_fields.as_slice(), [(field, NirExpr::Int(9))] if field == "value")
                            && matches!(second_fields.as_slice(), [(field, NirExpr::Int(3))] if field == "value")
                    )
                    && matches!(rhs_fields.as_slice(), [(field, NirExpr::Int(1))] if field == "value")
            )
    ));
}

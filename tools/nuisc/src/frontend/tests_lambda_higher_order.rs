use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstExpr, AstStmt, NirExpr, NirStmt, NirVisibility};

#[test]
fn parses_lambda_expr_in_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let inc = |x: i64| -> i64 { return x + 1; };
            return inc(6);
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        AstStmt::Let {
            value:
                AstExpr::Lambda {
                    params,
                    return_type,
                    body,
                },
            ..
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "x");
            assert_eq!(params[0].ty.name, "i64");
            assert_eq!(return_type.as_ref().unwrap().name, "i64");
            assert!(matches!(
                body.as_slice(),
                [AstStmt::Return(Some(AstExpr::Binary { .. }))]
            ));
        }
        other => panic!("expected lambda in let binding, found {other:?}"),
    }
}

#[test]
fn lowers_no_capture_lambda_binding_into_private_synth_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let inc = |x: i64| -> i64 { return x + 1; };
            return inc(6);
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.functions.len(), 2);
    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda function");
    assert!(matches!(lambda.visibility, NirVisibility::Private));
    assert_eq!(lambda.params.len(), 1);
    assert_eq!(lambda.params[0].name, "x");

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name && matches!(args.as_slice(), [NirExpr::Int(6)])
    ));
}

#[test]
fn lowers_immediate_no_capture_lambda_invocation_into_private_synth_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return (|x: i64| -> i64 { return x + 1; })(6);
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.functions.len(), 2);
    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda function");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name && matches!(args.as_slice(), [NirExpr::Int(6)])
    ));
}

#[test]
fn parses_lambda_tail_expression_body_as_implicit_return() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let inc = |x: i64| -> i64 { x + 1 };
            return inc(6);
          }
        }
        "#,
    )
    .unwrap();

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        AstStmt::Let {
            value: AstExpr::Lambda { body, .. },
            ..
        } => {
            assert!(matches!(
                body.as_slice(),
                [AstStmt::Return(Some(AstExpr::Binary { .. }))]
            ));
        }
        other => panic!("expected lambda in let binding, found {other:?}"),
    }
}

#[test]
fn lowers_no_capture_lambda_without_explicit_return_type_with_tail_if_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| {
              if x == 6 {
                x + 1
              } else {
                x
              }
            });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "x")
    ));
}

#[test]
fn lowers_lambda_capture_of_outer_local_into_extra_helper_arg() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let seed: i64 = 6;
            let inc = |x: i64| -> i64 { return x + seed; };
            return inc(1);
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured lambda");
    assert_eq!(lambda.params.len(), 2);
    assert_eq!(lambda.params[0].name, "x");
    assert_eq!(lambda.params[1].name, "seed");

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [
            NirStmt::Let { name, value, .. },
            NirStmt::Return(Some(NirExpr::Call { callee, args }))
        ] if name == "seed"
            && matches!(value, NirExpr::Int(6))
            && callee == &lambda.name
            && matches!(args.as_slice(), [NirExpr::Int(1), NirExpr::Var(seed_name)] if seed_name == "seed")
    ));
}

#[test]
fn lowers_lambda_capture_inside_nested_while_match_higher_order_scrutinee() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            while seed > 0 {
              match apply(seed, |x: i64| -> i64 { return x + seed; }) {
                7 => { return 1; }
                _ => { return 0; }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured lambda");
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected specialized higher-order helper");
    assert_eq!(specialized.params.len(), 2);
    assert_eq!(specialized.params[0].name, "x");
    assert_eq!(specialized.params[1].name, "__capture_f_seed_0");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(seed)] if x == "x" && seed == "__capture_f_seed_0")
    ));
}

#[test]
fn rejects_calling_non_lambda_expression_value_in_invoke_form() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return (1 + 2)(3);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "only immediate lambda invocation and named function or lambda binding invocation are supported in the current MVP"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn lowers_no_capture_lambda_passed_to_named_fn1_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 1);
    assert_eq!(specialized.params[0].name, "x");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "x")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &specialized.name && matches!(args.as_slice(), [NirExpr::Int(6)])
    ));
}

#[test]
fn lowers_no_capture_lambda_without_explicit_return_type_passed_to_named_fn1_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| { return x + 1; });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 1);
    assert_eq!(specialized.params[0].name, "x");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "x")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &specialized.name && matches!(args.as_slice(), [NirExpr::Int(6)])
    ));
}

#[test]
fn lowers_named_function_passed_to_named_fn1_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn plus_one(x: i64) -> i64 {
            return x + 1;
          }

          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, plus_one);
          }
        }
        "#,
    )
    .unwrap();

    let plus_one = module
        .functions
        .iter()
        .find(|function| function.name == "plus_one")
        .expect("expected source function");
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 1);
    assert_eq!(specialized.params[0].name, "x");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &plus_one.name && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "x")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &specialized.name && matches!(args.as_slice(), [NirExpr::Int(6)])
    ));
}

#[test]
fn lowers_no_capture_lambda_passed_to_named_fn2_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return f(x, y);
          }

          fn main() -> i64 {
            return apply2(6, 1, |x: i64, y: i64| -> i64 { return x + y; });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply2_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 2);
    assert_eq!(specialized.params[0].name, "x");
    assert_eq!(specialized.params[1].name, "y");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(y)] if x == "x" && y == "y")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &specialized.name
                && matches!(args.as_slice(), [NirExpr::Int(6), NirExpr::Int(1)])
    ));
}

#[test]
fn lowers_no_capture_lambda_without_explicit_return_type_passed_to_named_fn2_with_tail_if_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return f(x, y);
          }

          fn main() -> i64 {
            return apply2(6, 1, |x: i64, y: i64| {
              if x > y {
                x - y
              } else {
                y - x
              }
            });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply2_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(y)] if x == "x" && y == "y")
    ));
}

#[test]
fn lowers_named_function_passed_to_named_fn2_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn plus(x: i64, y: i64) -> i64 {
            return x + y;
          }

          fn apply2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return f(x, y);
          }

          fn main() -> i64 {
            return apply2(6, 1, plus);
          }
        }
        "#,
    )
    .unwrap();

    let plus = module
        .functions
        .iter()
        .find(|function| function.name == "plus")
        .expect("expected source function");
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply2_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 2);
    assert_eq!(specialized.params[0].name, "x");
    assert_eq!(specialized.params[1].name, "y");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &plus.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(y)] if x == "x" && y == "y")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &specialized.name
                && matches!(args.as_slice(), [NirExpr::Int(6), NirExpr::Int(1)])
    ));
}

#[test]
fn lowers_no_capture_lambda_passed_to_named_fn3_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(6, 1, 2, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply3_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 3);
    assert_eq!(specialized.params[0].name, "x");
    assert_eq!(specialized.params[1].name, "y");
    assert_eq!(specialized.params[2].name, "z");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(y), NirExpr::Var(z)] if x == "x" && y == "y" && z == "z")
    ));
}

#[test]
fn lowers_no_capture_lambda_without_explicit_return_type_passed_to_named_fn3_with_tail_match_expr()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(6, 1, 2, |x: i64, y: i64, z: i64| {
              match z {
                2 => {
                  x + y + z
                },
                _ => {
                  x
                }
              }
            });
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply3_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(y), NirExpr::Var(z)] if x == "x" && y == "y" && z == "z")
    ));
}

#[test]
fn lowers_named_function_passed_to_named_fn3_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn plus3(x: i64, y: i64, z: i64) -> i64 {
            return x + y + z;
          }

          fn apply3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(6, 1, 2, plus3);
          }
        }
        "#,
    )
    .unwrap();

    let plus3 = module
        .functions
        .iter()
        .find(|function| function.name == "plus3")
        .expect("expected source function");
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply3_"))
        .expect("expected synthesized higher-order specialization");
    assert_eq!(specialized.params.len(), 3);
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &plus3.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(y), NirExpr::Var(z)] if x == "x" && y == "y" && z == "z")
    ));
}

#[test]
fn lowers_typed_lambda_binding_without_explicit_lambda_return_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let inc: Fn1<i64, i64> = |x: i64| { return x + 1; };
            return apply(6, inc);
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
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "x")
    ));
}

#[test]
fn lowers_typed_named_function_binding_as_callable_alias() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn plus_one(x: i64) -> i64 {
            return x + 1;
          }

          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let inc: Fn1<i64, i64> = plus_one;
            return apply(6, inc) + inc(2);
          }
        }
        "#,
    )
    .unwrap();

    let plus_one = module
        .functions
        .iter()
        .find(|function| function.name == "plus_one")
        .expect("expected source function");
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &plus_one.name && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "x")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. })))
            if matches!(lhs.as_ref(), NirExpr::Call { callee, .. } if callee == &specialized.name)
                && matches!(rhs.as_ref(), NirExpr::Call { callee, args } if callee == &plus_one.name && matches!(args.as_slice(), [NirExpr::Int(2)]))
    ));
}

#[test]
fn rejects_typed_lambda_binding_with_mismatched_callable_return_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let inc: Fn1<i64, bool> = |x: i64| -> i64 { return x + 1; };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("lambda binding `inc` return type `i64` does not match declared callable return type `bool`"),
        "unexpected error: {error}"
    );
}

#[test]
fn lowers_untyped_named_function_binding_as_callable_alias() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn plus_one(x: i64) -> i64 {
            return x + 1;
          }

          fn main() -> i64 {
            let inc = plus_one;
            return inc(2);
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
        main.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "plus_one" && matches!(args.as_slice(), [NirExpr::Int(2)])
    ));
}

#[test]
fn lowers_immediate_capturing_lambda_invocation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let seed: i64 = 6;
            return (|x: i64| -> i64 { return x + seed; })(2);
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured lambda");
    assert_eq!(lambda.params.len(), 2);

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [
            NirStmt::Let { name, value, .. },
            NirStmt::Return(Some(NirExpr::Call { callee, args }))
        ] if name == "seed"
            && matches!(value, NirExpr::Int(6))
            && callee == &lambda.name
            && matches!(args.as_slice(), [NirExpr::Int(2), NirExpr::Var(seed_name)] if seed_name == "seed")
    ));
}

#[test]
fn lowers_capturing_lambda_binding_passed_to_higher_order_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            let inc = |x: i64| -> i64 { return x + seed; };
            return apply(1, inc);
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured lambda");
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected specialized helper");
    assert_eq!(specialized.params.len(), 2);
    assert!(matches!(
        specialized.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(seed)] if x == "x" && seed == "__capture_f_seed_0")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.as_slice(),
        [
            NirStmt::Let { name, value, .. },
            NirStmt::Return(Some(NirExpr::Call { callee, args }))
        ] if name == "seed"
            && matches!(value, NirExpr::Int(6))
            && callee == &specialized.name
            && matches!(args.as_slice(), [NirExpr::Int(1), NirExpr::Var(seed_name)] if seed_name == "seed")
    ));
}

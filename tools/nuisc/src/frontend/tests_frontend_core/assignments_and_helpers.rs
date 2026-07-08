use super::*;

#[test]
fn rejects_reassignment_of_immutable_local() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 1;
            value = 2;
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("cannot assign to immutable local `value`"),
        "{error}"
    );
    assert!(error.contains("let mut"), "{error}");
}

#[test]
fn rejects_reassignment_of_unknown_local() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            value = 2;
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("cannot assign to unknown local `value`"),
        "{error}"
    );
}

#[test]
fn lowers_float_literals_with_expected_scalar_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add32() -> f32 {
            let sum: f32 = 1.5 + 2.25;
            return sum;
          }

          fn add64() -> f64 {
            return 1.5 + 2.25;
          }
        }
        "#,
    )
    .unwrap();

    let add32 = module
        .functions
        .iter()
        .find(|function| function.name == "add32")
        .unwrap();
    let sum_ty = add32
        .body
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::Let { name, ty, value } if name == "sum" => {
                assert!(matches!(
                    value,
                    NirExpr::Binary {
                        lhs,
                        rhs,
                        ..
                    } if matches!(lhs.as_ref(), NirExpr::F32(value) if value == "1.5")
                        && matches!(rhs.as_ref(), NirExpr::F32(value) if value == "2.25")
                ));
                ty.as_ref()
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(sum_ty.render(), "f32");

    let add64 = module
        .functions
        .iter()
        .find(|function| function.name == "add64")
        .unwrap();
    assert!(matches!(
        add64.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. })))
            if matches!(lhs.as_ref(), NirExpr::F64(value) if value == "1.5")
                && matches!(rhs.as_ref(), NirExpr::F64(value) if value == "2.25")
    ));
}

#[test]
fn lowers_project_local_cpu_helper_calls_with_qualified_callees() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_completed(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          pub fn encode_completed(value: i64) -> i64 {
            return value + 1;
          }

          pub fn task_policy_completed(value: i64) -> i64 {
            return encode_completed(value);
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
    let helper_function = module
        .functions
        .iter()
        .find(|function| function.name == "TaskHelpers.task_policy_completed")
        .unwrap();
    assert!(matches!(
        helper_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "TaskHelpers.encode_completed"
    ));

    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "TaskHelpers.task_policy_completed"
    ));
}

#[test]
fn lowers_payload_style_single_field_struct_constructor_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just {
            value: i64,
          }

          fn main() -> i64 {
            let payload: Just = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "payload");
            assert!(matches!(
                value,
                NirExpr::StructLiteral { type_name, fields, .. }
                    if type_name == "Just"
                        && matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Int(7))] if field == "value"
                        )
            ));
        }
        other => panic!("expected lowered payload constructor let, found {other:?}"),
    }
}

#[test]
fn parses_compound_buffer_assignment_into_store_at_sugar() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main(scratch: ref Buffer, slot: i64, step: i64) -> i64 {
            scratch[slot] += step;
            return 0;
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
        Some(AstStmt::Expr(AstExpr::Call { callee, generic_args, args }))
            if callee == "store_at"
                && generic_args.is_empty()
                && matches!(
                    args.as_slice(),
                    [
                        AstExpr::Var(buffer),
                        AstExpr::Var(slot),
                        AstExpr::Binary {
                            op,
                            lhs,
                            rhs,
                        }
                    ] if buffer == "scratch"
                        && slot == "slot"
                        && *op == AstBinaryOp::Add
                        && matches!(
                            lhs.as_ref(),
                            AstExpr::Call { callee: load_callee, generic_args: load_generics, args: load_args }
                                if load_callee == "load_at"
                                    && load_generics.is_empty()
                                    && matches!(
                                        load_args.as_slice(),
                                        [AstExpr::Var(load_buffer), AstExpr::Var(load_slot)]
                                            if load_buffer == "scratch" && load_slot == "slot"
                                    )
                        )
                        && matches!(rhs.as_ref(), AstExpr::Var(step) if step == "step")
                )
    ));
}

#[test]
fn lowers_compound_pointer_value_assignment_into_store_value_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(head: ref Node) -> i64 {
            head.value %= 2;
            return head.value;
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
        Some(NirStmt::Expr(NirExpr::StoreValue { target, value }))
            if matches!(target.as_ref(), NirExpr::Var(name) if name == "head")
                && matches!(
                    value.as_ref(),
                    NirExpr::Binary {
                        op,
                        lhs,
                        rhs,
                    } if *op == NirBinaryOp::Rem
                        && matches!(lhs.as_ref(), NirExpr::LoadValue(inner) if matches!(inner.as_ref(), NirExpr::Var(name) if name == "head"))
                        && matches!(rhs.as_ref(), NirExpr::Int(2))
                )
    ));
}

#[test]
fn rejects_compound_pointer_next_assignment() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(head: ref Node, next: ref Node) -> i64 {
            head.next += next;
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("compound assignment target `.next` is not supported yet"),
        "{error}"
    );
}

#[test]
fn rejects_private_local_cpu_helper_calls_across_modules() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_completed(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("unknown function `task_policy_completed`"),
        "unexpected error: {error}"
    );
}

#[test]
fn suggests_similar_local_function_name_for_unknown_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }

          fn main() -> i64 {
            return task_policy_complted(7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("unknown function `task_policy_complted`"),
        "{error}"
    );
    assert!(
        error.contains("did you mean `task_policy_completed`?"),
        "{error}"
    );
}

#[test]
fn suggests_similar_imported_helper_function_name_for_unknown_call() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_complted(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          pub fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("unknown function `task_policy_complted`"),
        "{error}"
    );
    assert!(
        error.contains("did you mean `task_policy_completed`?"),
        "{error}"
    );
}

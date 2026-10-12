use super::*;

#[test]
fn float_comparison_inference_keeps_six_exact_bool_results_and_nested_boolean_uses() {
    for ty in [f32_type(), f64_type()] {
        let bindings = BTreeMap::from([("left".into(), ty.clone()), ("right".into(), ty)]);
        for op in [
            NirBinaryOp::Eq,
            NirBinaryOp::Ne,
            NirBinaryOp::Lt,
            NirBinaryOp::Le,
            NirBinaryOp::Gt,
            NirBinaryOp::Ge,
        ] {
            let comparison = NirExpr::Binary {
                op,
                lhs: Box::new(NirExpr::Var("left".into())),
                rhs: Box::new(NirExpr::Var("right".into())),
            };
            let inferred =
                |e: &NirExpr| infer_nir_expr_type(e, &bindings, &BTreeMap::new(), &BTreeMap::new());
            assert_eq!(inferred(&comparison), Some(bool_type()));
            assert_eq!(
                inferred(&NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(comparison),
                    rhs: Box::new(NirExpr::Bool(true))
                }),
                Some(bool_type())
            );
        }
    }
}

#[test]
fn float_comparison_inference_rejects_cross_width_resources_and_modified_operands() {
    for ty in [f32_type(), f64_type()] {
        let mut invalid = vec![
            i64_type(),
            bool_type(),
            if ty == f32_type() {
                f64_type()
            } else {
                f32_type()
            },
        ];
        for kind in 0..3 {
            let mut modified = ty.clone();
            match kind {
                0 => modified.is_ref = true,
                1 => modified.is_optional = true,
                _ => modified.generic_args.push(i64_type()),
            }
            invalid.push(modified);
        }
        for other in invalid {
            let bindings = BTreeMap::from([("left".into(), ty.clone()), ("right".into(), other)]);
            for op in [
                NirBinaryOp::Eq,
                NirBinaryOp::Ne,
                NirBinaryOp::Lt,
                NirBinaryOp::Le,
                NirBinaryOp::Gt,
                NirBinaryOp::Ge,
            ] {
                let e = NirExpr::Binary {
                    op,
                    lhs: Box::new(NirExpr::Var("left".into())),
                    rhs: Box::new(NirExpr::Var("right".into())),
                };
                assert_eq!(
                    infer_nir_expr_type(&e, &bindings, &BTreeMap::new(), &BTreeMap::new()),
                    None
                );
            }
        }
    }
}

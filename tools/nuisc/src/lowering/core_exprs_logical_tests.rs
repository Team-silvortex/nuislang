use super::*;

#[test]
fn eager_logical_fallback_only_admits_total_ready_expression_trees() {
    let variable = || NirExpr::Var("ready".into());
    let read = || NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("pixels".into())),
        index: Box::new(NirExpr::Int(99)),
    };
    let binary = |op, rhs| NirExpr::Binary {
        op,
        lhs: Box::new(variable()),
        rhs: Box::new(rhs),
    };
    for value in [
        variable(),
        NirExpr::Bool(false),
        binary(NirBinaryOp::And, variable()),
        binary(NirBinaryOp::Eq, variable()),
        binary(NirBinaryOp::Xor, variable()),
        NirExpr::FieldAccess {
            base: Box::new(variable()),
            field: "flag".into(),
        },
    ] {
        assert!(eager_logical_rhs(&value));
    }
    for value in [
        read(),
        binary(NirBinaryOp::Eq, read()),
        binary(NirBinaryOp::Div, NirExpr::Int(0)),
        binary(NirBinaryOp::Rem, NirExpr::Int(0)),
        NirExpr::BytesLen(Box::new(variable())),
        NirExpr::Call {
            callee: "unknown".into(),
            args: vec![],
        },
        NirExpr::Move(Box::new(variable())),
        NirExpr::StructLiteral {
            type_name: "Envelope".into(),
            type_args: vec![],
            fields: vec![("payload".into(), variable())],
        },
        NirExpr::VariantFieldAccess {
            base: Box::new(variable()),
            variant: "Ok".into(),
            field: "value".into(),
        },
    ] {
        assert!(!eager_logical_rhs(&value));
        assert!(!eager_logical_rhs(&binary(NirBinaryOp::Or, value)));
    }
}

#[test]
fn eager_logical_fallback_has_a_nonrecursive_work_bound() {
    let mut value = NirExpr::Bool(true);
    for _ in 0..2047 {
        value = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Bool(true)),
            rhs: Box::new(value),
        };
    }
    assert!(eager_logical_rhs(&value));
    value = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(NirExpr::Bool(true)),
        rhs: Box::new(value),
    };
    assert!(!eager_logical_rhs(&value));
}

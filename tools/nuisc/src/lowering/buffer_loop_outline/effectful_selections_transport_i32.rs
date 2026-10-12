use super::*;

const BIAS: i64 = 2_147_483_648;
const FLAG: i64 = 4_294_967_296;

pub(super) fn pack(owner: &str, child: &str) -> NirExpr {
    // A sign-extended i32 plus BIAS is in 0..=2^32-1. The bool occupies bit 32;
    // the complete carrier is in 0..=2^33-1, with no overflow or lost sign bit.
    binary(
        NirBinaryOp::Add,
        binary(
            NirBinaryOp::Add,
            NirExpr::CastI32ToI64(Box::new(NirExpr::Var(child.into()))),
            NirExpr::Int(BIAS),
        ),
        binary(
            NirBinaryOp::Mul,
            NirExpr::CastBoolToI64(Box::new(NirExpr::Var(owner.into()))),
            NirExpr::Int(FLAG),
        ),
    )
}

pub(super) fn restore(carrier: String, owner: String, child: String) -> [NirStmt; 2] {
    // Subtract the restored flag and bias before narrowing. Every result is
    // already an exact i32; no division, remainder or fallible source work.
    let value = binary(
        NirBinaryOp::Sub,
        binary(
            NirBinaryOp::Sub,
            NirExpr::Var(carrier.clone()),
            binary(
                NirBinaryOp::Mul,
                NirExpr::CastBoolToI64(Box::new(NirExpr::Var(owner.clone()))),
                NirExpr::Int(FLAG),
            ),
        ),
        NirExpr::Int(BIAS),
    );
    [
        NirStmt::Let {
            name: owner,
            ty: Some(scalar_type("bool")),
            value: binary(NirBinaryOp::Ge, NirExpr::Var(carrier), NirExpr::Int(FLAG)),
        },
        NirStmt::Let {
            name: child,
            ty: Some(scalar_type("i32")),
            value: NirExpr::CastI64ToI32(Box::new(value)),
        },
    ]
}

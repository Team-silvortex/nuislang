use super::*;

pub(super) fn operand<'a>(value: &'a NirExpr, expected: &NirTypeRef) -> Option<&'a NirExpr> {
    let (word, mask) = match (expected.name.as_str(), value) {
        ("f32", NirExpr::UnpackF32Word(word)) => (word, 1_i64 << 31),
        ("f64", NirExpr::UnpackF64Word(word)) => (word, i64::MIN),
        _ => return None,
    };
    let NirExpr::Binary {
        op: NirBinaryOp::Xor,
        lhs,
        rhs,
    } = word.as_ref()
    else {
        return None;
    };
    if rhs.as_ref() != &NirExpr::Int(mask) {
        return None;
    }
    match (expected.name.as_str(), lhs.as_ref()) {
        ("f32", NirExpr::PackF32Word(leaf)) | ("f64", NirExpr::PackF64Word(leaf)) => Some(leaf),
        _ => None,
    }
}

pub(super) fn field_operand<'a>(
    value: &'a NirExpr,
    expected: &NirTypeRef,
    mut depth: usize,
) -> Option<&'a NirExpr> {
    if expected != &scalar_type(&expected.name) {
        return None;
    }
    let mut leaf = value;
    let mut consumed = false;
    while !matches!(leaf, NirExpr::FieldAccess { .. }) {
        leaf = operand(leaf, expected)?;
        depth = depth.checked_add(3)?;
        if depth > 64 {
            return None;
        }
        consumed = true;
    }
    if !consumed {
        return None;
    }
    // Every operation and original projection consumes depth. Even an even
    // chain is validated without folding or substituting a ready record.
    let mut base = leaf;
    while let NirExpr::FieldAccess { base: parent, .. } = base {
        depth = depth.checked_add(1)?;
        if depth > 64 {
            return None;
        }
        base = parent;
    }
    (depth <= 64).then_some(leaf)
}

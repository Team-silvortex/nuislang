use super::*;

struct Proof {
    ty: NirTypeRef,
    total: bool,
    ready_atom: bool,
    ready_predicate: bool,
    comparison_only: bool,
    captures: BTreeSet<Vec<String>>,
}

enum Step<'a> {
    Value(&'a NirExpr, usize, bool),
    Binary(NirBinaryOp, &'a NirExpr, &'a NirExpr),
    Exact(&'a NirTypeRef),
    Call(usize, &'a NirTypeRef),
    Sign(NirTypeRef),
}

pub(super) fn validate(
    value: &NirExpr,
    expected: &NirTypeRef,
    depth: usize,
    catalog: &ScalarHelpers,
    mut leaf: impl FnMut(&NirExpr, usize) -> Option<NirTypeRef>,
) -> Option<()> {
    if expected != &scalar_type(&expected.name) || !matches!(expected.name.as_str(), "f32" | "f64")
    {
        return None;
    }
    let mut pending = vec![Step::Value(value, depth, false)];
    let mut proofs: Vec<Proof> = Vec::new();
    let mut work = 4096usize;
    let mut computed = false;
    while let Some(step) = pending.pop() {
        match step {
            Step::Value(value, depth, lazy_rhs) => {
                if depth > 64 || work == 0 {
                    return None;
                }
                work -= 1;
                match value {
                    NirExpr::Binary { op, lhs, rhs } => {
                        let child = depth.checked_add(1)?;
                        pending.push(Step::Binary(*op, lhs, rhs));
                        // LIFO traversal preserves left-before-right checking.
                        let right = lazy_rhs || matches!(op, NirBinaryOp::And | NirBinaryOp::Or);
                        pending.push(Step::Value(rhs, child, right));
                        pending.push(Step::Value(lhs, child, lazy_rhs));
                        computed = true;
                    }
                    NirExpr::Call { callee, args } => {
                        let (params, result) =
                            scalar_helpers::pure_scalar_signature(callee, catalog)?;
                        if !matches!(result.name.as_str(), "f32" | "f64")
                            && !(lazy_rhs && matches!(result.name.as_str(), "bool" | "i32" | "i64"))
                        {
                            return None;
                        }
                        if args.len() != params.len()
                            || args.len() > work.saturating_sub(pending.len())
                        {
                            return None;
                        }
                        let child = depth.checked_add(1)?;
                        pending.push(Step::Call(args.len(), result));
                        for (arg, ty) in args.iter().zip(params).rev() {
                            pending.push(Step::Exact(ty));
                            pending.push(Step::Value(arg, child, lazy_rhs));
                        }
                        computed = true;
                    }
                    NirExpr::UnpackF32Word(_) | NirExpr::UnpackF64Word(_) => {
                        let ty = scalar_type(if matches!(value, NirExpr::UnpackF32Word(_)) {
                            "f32"
                        } else {
                            "f64"
                        });
                        let operand = unary_float::operand(value, &ty)?;
                        work = work.checked_sub(3)?;
                        pending.push(Step::Sign(ty));
                        pending.push(Step::Value(operand, depth.checked_add(3)?, lazy_rhs));
                    }
                    NirExpr::Int(_)
                    | NirExpr::Bool(_)
                    | NirExpr::F32(_)
                    | NirExpr::F64(_)
                    | NirExpr::Var(_)
                    | NirExpr::FieldAccess { .. } => {
                        let mut base = value;
                        let mut end = depth;
                        let mut path = Vec::new();
                        while let NirExpr::FieldAccess {
                            base: parent,
                            field,
                        } = base
                        {
                            path.push(field.clone());
                            end = end.checked_add(1)?;
                            if end > 64 {
                                return None;
                            }
                            work = work.checked_sub(1)?;
                            base = parent;
                        }
                        if matches!(value, NirExpr::FieldAccess { .. })
                            && !matches!(base, NirExpr::Var(_))
                        {
                            return None;
                        }
                        let ty = leaf(value, depth)?;
                        if ty != scalar_type(&ty.name)
                            || !matches!(ty.name.as_str(), "bool" | "i32" | "i64" | "f32" | "f64")
                        {
                            return None;
                        }
                        let mut captures = BTreeSet::new();
                        if let NirExpr::Var(name) = base {
                            path.push(name.clone());
                            path.reverse();
                            captures.insert(path);
                        }
                        proofs.push(Proof {
                            ready_atom: !matches!(value, NirExpr::FieldAccess { .. }),
                            ready_predicate: !matches!(value, NirExpr::FieldAccess { .. })
                                && ty == scalar_type("bool"),
                            comparison_only: false,
                            ty,
                            total: true,
                            captures,
                        });
                    }
                    _ => return None,
                }
            }
            Step::Exact(expected) => {
                if &proofs.last()?.ty != expected {
                    return None;
                }
            }
            Step::Call(count, result) => {
                let mut captures = BTreeSet::new();
                let mut comparison_only = false;
                for mut proof in proofs.drain(proofs.len().checked_sub(count)?..) {
                    comparison_only |= proof.comparison_only;
                    captures.append(&mut proof.captures);
                }
                // A new i32 call result may flow through i32 calls, not escape as argument readiness.
                if comparison_only && result != &scalar_type("i32") {
                    return None;
                }
                // A pure call can fail even though capturing its scalar inputs is total.
                proofs.push(Proof {
                    ty: result.clone(),
                    total: false,
                    ready_atom: false,
                    ready_predicate: false,
                    comparison_only: result == &scalar_type("i32"),
                    captures,
                });
            }
            Step::Sign(expected) => {
                if proofs.last()?.ty != expected {
                    return None;
                }
                proofs.last_mut()?.ready_atom = false;
                proofs.last_mut()?.ready_predicate = false;
            }
            Step::Binary(op, lhs, rhs) => {
                let mut right = proofs.pop()?;
                let mut left = proofs.pop()?;
                if (left.comparison_only || right.comparison_only)
                    && !matches!(
                        op,
                        NirBinaryOp::Eq
                            | NirBinaryOp::Ne
                            | NirBinaryOp::Lt
                            | NirBinaryOp::Le
                            | NirBinaryOp::Gt
                            | NirBinaryOp::Ge
                    )
                {
                    return None;
                }
                match op {
                    NirBinaryOp::And | NirBinaryOp::Or
                        if (!right.total || !right.ready_predicate)
                            && right.captures.len() > 31 =>
                    {
                        return None
                    }
                    NirBinaryOp::Xor if left.ty == scalar_type("bool") => return None,
                    NirBinaryOp::Eq | NirBinaryOp::Ne
                        if left.ty == scalar_type("bool")
                            && (!bool_atom(lhs) || !bool_atom(rhs)) =>
                    {
                        return None
                    }
                    _ => {}
                }
                let total =
                    left.total && right.total && !matches!(op, NirBinaryOp::Div | NirBinaryOp::Rem);
                // Non-trapping fields/arithmetic can still require a private RHS helper.
                let ready_predicate = match op {
                    NirBinaryOp::And | NirBinaryOp::Or => {
                        left.ready_predicate && right.ready_predicate
                    }
                    NirBinaryOp::Eq
                    | NirBinaryOp::Ne
                    | NirBinaryOp::Lt
                    | NirBinaryOp::Le
                    | NirBinaryOp::Gt
                    | NirBinaryOp::Ge => left.ready_atom && right.ready_atom,
                    _ => false,
                };
                left.captures.append(&mut right.captures);
                let ty = control_values::binary_type(op, left.ty, right.ty)?;
                proofs.push(Proof {
                    ty,
                    total,
                    ready_atom: false,
                    ready_predicate,
                    comparison_only: false,
                    captures: left.captures,
                });
            }
        }
    }
    (computed && proofs.len() == 1 && proofs[0].ty == *expected && !proofs[0].comparison_only)
        .then_some(())
}

use super::*;

pub(super) fn canonicalize_tail_recursive_loop_arg(
    expr: &NirExpr,
    current_name: &str,
    non_current_param_names: &[String],
    invariant_param_names: &BTreeSet<String>,
    target_carry_name: Option<&str>,
    next_current_expr: &NirExpr,
) -> NirExpr {
    if expr == next_current_expr {
        return NirExpr::Var(current_name.to_owned());
    }
    match expr {
        NirExpr::Var(name) if name == current_name => {
            NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned())
        }
        NirExpr::Var(name) => {
            if target_carry_name.is_some() && Some(name.as_str()) == target_carry_name {
                expr.clone()
            } else if invariant_param_names.contains(name) {
                expr.clone()
            } else {
                non_current_param_names
                    .iter()
                    .position(|param_name| param_name == name)
                    .map(tail_recursive_prev_carry_binding)
                    .map(NirExpr::Var)
                    .unwrap_or_else(|| expr.clone())
            }
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => expr.clone(),
        NirExpr::CastI64ToI32(inner) => {
            NirExpr::CastI64ToI32(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI32ToI64(inner) => {
            NirExpr::CastI32ToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI64ToBool(inner) => {
            NirExpr::CastI64ToBool(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastBoolToI64(inner) => {
            NirExpr::CastBoolToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI64ToF32(inner) => {
            NirExpr::CastI64ToF32(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastF32ToI64(inner) => {
            NirExpr::CastF32ToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastI64ToF64(inner) => {
            NirExpr::CastI64ToF64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::CastF64ToI64(inner) => {
            NirExpr::CastF64ToI64(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::Await(inner) => NirExpr::Await(Box::new(canonicalize_tail_recursive_loop_arg(
            inner,
            current_name,
            non_current_param_names,
            invariant_param_names,
            target_carry_name,
            next_current_expr,
        ))),
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_loop_arg(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                        target_carry_name,
                        next_current_expr,
                    )
                })
                .collect(),
        },
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(canonicalize_tail_recursive_loop_arg(
                receiver,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_loop_arg(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                        target_carry_name,
                        next_current_expr,
                    )
                })
                .collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    (
                        field.clone(),
                        canonicalize_tail_recursive_loop_arg(
                            value,
                            current_name,
                            non_current_param_names,
                            invariant_param_names,
                            target_carry_name,
                            next_current_expr,
                        ),
                    )
                })
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(canonicalize_tail_recursive_loop_arg(
                base,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            field: field.clone(),
        },
        NirExpr::IsNull(inner) => NirExpr::IsNull(Box::new(canonicalize_tail_recursive_loop_arg(
            inner,
            current_name,
            non_current_param_names,
            invariant_param_names,
            target_carry_name,
            next_current_expr,
        ))),
        NirExpr::LoadValue(inner) => {
            NirExpr::LoadValue(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::LoadNext(inner) => {
            NirExpr::LoadNext(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::BufferLen(inner) => {
            NirExpr::BufferLen(Box::new(canonicalize_tail_recursive_loop_arg(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )))
        }
        NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
            buffer: Box::new(canonicalize_tail_recursive_loop_arg(
                buffer,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            index: Box::new(canonicalize_tail_recursive_loop_arg(
                index,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(canonicalize_tail_recursive_loop_arg(
                lhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
            rhs: Box::new(canonicalize_tail_recursive_loop_arg(
                rhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
                target_carry_name,
                next_current_expr,
            )),
        },
        _ => expr.clone(),
    }
}

pub(super) fn canonicalize_tail_recursive_condition_expr(
    expr: &NirExpr,
    current_name: &str,
    non_current_param_names: &[String],
    invariant_param_names: &BTreeSet<String>,
) -> NirExpr {
    match expr {
        NirExpr::Var(name) if name == current_name => {
            NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned())
        }
        NirExpr::Var(name) => {
            if invariant_param_names.contains(name) {
                expr.clone()
            } else {
                non_current_param_names
                    .iter()
                    .position(|param_name| param_name == name)
                    .map(tail_recursive_prev_carry_binding)
                    .map(NirExpr::Var)
                    .unwrap_or_else(|| expr.clone())
            }
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => expr.clone(),
        NirExpr::CastI64ToI32(inner) => {
            NirExpr::CastI64ToI32(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI32ToI64(inner) => {
            NirExpr::CastI32ToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI64ToBool(inner) => {
            NirExpr::CastI64ToBool(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastBoolToI64(inner) => {
            NirExpr::CastBoolToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI64ToF32(inner) => {
            NirExpr::CastI64ToF32(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastF32ToI64(inner) => {
            NirExpr::CastF32ToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastI64ToF64(inner) => {
            NirExpr::CastI64ToF64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::CastF64ToI64(inner) => {
            NirExpr::CastF64ToI64(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::Await(inner) => {
            NirExpr::Await(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_condition_expr(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                    )
                })
                .collect(),
        },
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(canonicalize_tail_recursive_condition_expr(
                receiver,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    canonicalize_tail_recursive_condition_expr(
                        arg,
                        current_name,
                        non_current_param_names,
                        invariant_param_names,
                    )
                })
                .collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    (
                        field.clone(),
                        canonicalize_tail_recursive_condition_expr(
                            value,
                            current_name,
                            non_current_param_names,
                            invariant_param_names,
                        ),
                    )
                })
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(canonicalize_tail_recursive_condition_expr(
                base,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            field: field.clone(),
        },
        NirExpr::IsNull(inner) => {
            NirExpr::IsNull(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::LoadValue(inner) => {
            NirExpr::LoadValue(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::LoadNext(inner) => {
            NirExpr::LoadNext(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::BufferLen(inner) => {
            NirExpr::BufferLen(Box::new(canonicalize_tail_recursive_condition_expr(
                inner,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )))
        }
        NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
            buffer: Box::new(canonicalize_tail_recursive_condition_expr(
                buffer,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            index: Box::new(canonicalize_tail_recursive_condition_expr(
                index,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(canonicalize_tail_recursive_condition_expr(
                lhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
            rhs: Box::new(canonicalize_tail_recursive_condition_expr(
                rhs,
                current_name,
                non_current_param_names,
                invariant_param_names,
            )),
        },
        _ => expr.clone(),
    }
}

use super::*;

pub(in crate::lowering) fn collect_pure_helper_functions(module: &NirModule) -> BTreeSet<String> {
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::<String, bool>::new();
    let mut visiting = BTreeSet::<String>::new();
    let mut helpers = module
        .functions
        .iter()
        .filter(|function| function.name != "main")
        .filter(|function| {
            is_pure_helper_function(function, &function_map, &mut memo, &mut visiting)
        })
        .map(|function| function.name.clone())
        .collect::<BTreeSet<_>>();
    helpers.extend(enum_variant_constructor_names(module));
    helpers
}

fn enum_variant_constructor_names(module: &NirModule) -> impl Iterator<Item = String> + '_ {
    module.enums.iter().flat_map(|definition| {
        definition
            .variants
            .iter()
            .map(|variant| format!("{}.{}", definition.name, variant.name))
    })
}

pub(in crate::lowering) fn collect_inlineable_pure_helper_exprs(
    module: &NirModule,
) -> BTreeMap<String, InlineablePureHelper> {
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::<String, Option<InlineablePureHelper>>::new();
    let mut visiting = BTreeSet::<String>::new();
    module
        .functions
        .iter()
        .filter_map(|function| {
            extract_inlineable_pure_helper(function, &function_map, &mut memo, &mut visiting)
                .map(|helper| (function.name.clone(), helper))
        })
        .collect()
}

pub(in crate::lowering) fn collect_pure_helper_blocks(
    module: &NirModule,
) -> BTreeMap<String, PureHelperBlock> {
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::<String, bool>::new();
    let mut visiting = BTreeSet::<String>::new();
    module
        .functions
        .iter()
        .filter(|function| {
            is_pure_helper_function(function, &function_map, &mut memo, &mut visiting)
        })
        .map(|function| {
            (
                function.name.clone(),
                PureHelperBlock {
                    params: function
                        .params
                        .iter()
                        .map(|param| param.name.clone())
                        .collect(),
                    body: function.body.clone(),
                },
            )
        })
        .collect()
}

fn extract_inlineable_pure_helper(
    function: &NirFunction,
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, Option<InlineablePureHelper>>,
    visiting: &mut BTreeSet<String>,
) -> Option<InlineablePureHelper> {
    if let Some(cached) = memo.get(&function.name) {
        return cached.clone();
    }
    if !visiting.insert(function.name.clone()) {
        return None;
    }
    let result = if function.is_async {
        None
    } else {
        extract_inlineable_pure_expr_from_block(&function.body, function_map, memo, visiting).map(
            |expr| InlineablePureHelper {
                params: function
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
                expr,
            },
        )
    };
    visiting.remove(&function.name);
    memo.insert(function.name.clone(), result.clone());
    result
}

fn extract_inlineable_pure_expr_from_block(
    body: &[NirStmt],
    function_map: &BTreeMap<&str, &NirFunction>,
    _memo: &mut BTreeMap<String, Option<InlineablePureHelper>>,
    visiting: &mut BTreeSet<String>,
) -> Option<NirExpr> {
    let (NirStmt::Return(Some(expr)), prefix) = body.split_last()? else {
        return None;
    };
    let mut substituted = expr.clone();
    let mut pure_memo = BTreeMap::<String, bool>::new();
    for stmt in prefix.iter().rev() {
        let (binding_name, binding_value) = match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                (name.clone(), value.clone())
            }
            _ => return None,
        };
        if !is_pure_helper_expr(&binding_value, function_map, &mut pure_memo, visiting) {
            return None;
        }
        substituted = substitute_branch_binding(&substituted, &binding_name, &binding_value);
    }
    Some(substituted)
}

pub(in crate::lowering) fn inline_pure_helper_calls(
    expr: &NirExpr,
    inlineable_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> NirExpr {
    fn inline_expr(
        expr: &NirExpr,
        inlineable_helpers: &BTreeMap<String, InlineablePureHelper>,
        visiting: &mut BTreeSet<String>,
    ) -> NirExpr {
        match expr {
            NirExpr::Call { callee, args } => {
                let rewritten_args = args
                    .iter()
                    .map(|arg| inline_expr(arg, inlineable_helpers, visiting))
                    .collect::<Vec<_>>();
                if let Some(helper) = inlineable_helpers.get(callee) {
                    if helper.params.len() == rewritten_args.len()
                        && visiting.insert(callee.clone())
                    {
                        let mut expanded = helper.expr.clone();
                        for (param, arg) in helper.params.iter().zip(rewritten_args.iter()) {
                            expanded = substitute_branch_binding(&expanded, param, arg);
                        }
                        let rewritten = inline_expr(&expanded, inlineable_helpers, visiting);
                        visiting.remove(callee);
                        return rewritten;
                    }
                }
                NirExpr::Call {
                    callee: callee.clone(),
                    args: rewritten_args,
                }
            }
            NirExpr::Await(inner) => {
                NirExpr::Await(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToI32(inner) => {
                NirExpr::CastI64ToI32(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI32ToI64(inner) => {
                NirExpr::CastI32ToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToBool(inner) => {
                NirExpr::CastI64ToBool(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastBoolToI64(inner) => {
                NirExpr::CastBoolToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToF32(inner) => {
                NirExpr::CastI64ToF32(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastF32ToI64(inner) => {
                NirExpr::CastF32ToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToF64(inner) => {
                NirExpr::CastI64ToF64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastF64ToI64(inner) => {
                NirExpr::CastF64ToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::MethodCall {
                receiver,
                method,
                args,
            } => NirExpr::MethodCall {
                receiver: Box::new(inline_expr(receiver, inlineable_helpers, visiting)),
                method: method.clone(),
                args: args
                    .iter()
                    .map(|arg| inline_expr(arg, inlineable_helpers, visiting))
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
                            inline_expr(value, inlineable_helpers, visiting),
                        )
                    })
                    .collect(),
            },
            NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
                base: Box::new(inline_expr(base, inlineable_helpers, visiting)),
                field: field.clone(),
            },
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op: *op,
                lhs: Box::new(inline_expr(lhs, inlineable_helpers, visiting)),
                rhs: Box::new(inline_expr(rhs, inlineable_helpers, visiting)),
            },
            _ => expr.clone(),
        }
    }

    inline_expr(expr, inlineable_helpers, &mut BTreeSet::new())
}

fn is_pure_helper_function(
    function: &NirFunction,
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if let Some(&cached) = memo.get(&function.name) {
        return cached;
    }
    if !visiting.insert(function.name.clone()) {
        return false;
    }
    let result =
        !function.is_async && is_pure_helper_block(&function.body, function_map, memo, visiting);
    visiting.remove(&function.name);
    memo.insert(function.name.clone(), result);
    result
}

fn is_pure_helper_block(
    body: &[NirStmt],
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let Some((first, tail)) = body.split_first() else {
        return false;
    };
    match first {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
            is_pure_helper_expr(value, function_map, memo, visiting)
                && is_pure_helper_block(tail, function_map, memo, visiting)
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            is_pure_helper_expr(condition, function_map, memo, visiting)
                && is_pure_helper_block(then_body, function_map, memo, visiting)
                && if else_body.is_empty() {
                    is_pure_helper_block(tail, function_map, memo, visiting)
                } else {
                    tail.is_empty() && is_pure_helper_block(else_body, function_map, memo, visiting)
                }
        }
        NirStmt::Return(Some(expr)) => {
            tail.is_empty() && is_pure_helper_expr(expr, function_map, memo, visiting)
        }
        _ => false,
    }
}

fn is_pure_helper_expr(
    expr: &NirExpr,
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match expr {
        NirExpr::Call { callee, args } => {
            let Some(function) = function_map.get(callee.as_str()).copied() else {
                return false;
            };
            is_pure_helper_function(function, function_map, memo, visiting)
                && args
                    .iter()
                    .all(|arg| is_pure_helper_expr(arg, function_map, memo, visiting))
        }
        NirExpr::MethodCall { .. } => false,
        NirExpr::Await(_) | NirExpr::Instantiate { .. } => false,
        NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner) => is_pure_helper_expr(inner, function_map, memo, visiting),
        NirExpr::CastI64ToF32(inner) | NirExpr::CastF32ToI64(inner) => {
            is_pure_helper_expr(inner, function_map, memo, visiting)
        }
        NirExpr::CastI64ToF64(inner) | NirExpr::CastF64ToI64(inner) => {
            is_pure_helper_expr(inner, function_map, memo, visiting)
        }
        NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskFailed(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexValue(inner) => is_pure_helper_expr(inner, function_map, memo, visiting),
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| is_pure_helper_expr(value, function_map, memo, visiting)),
        NirExpr::FieldAccess { base, .. } => {
            is_pure_helper_expr(base, function_map, memo, visiting)
        }
        NirExpr::VariantIs { base, .. } | NirExpr::VariantFieldAccess { base, .. } => {
            is_pure_helper_expr(base, function_map, memo, visiting)
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            is_pure_helper_expr(lhs, function_map, memo, visiting)
                && is_pure_helper_expr(rhs, function_map, memo, visiting)
        }
        _ => matches!(
            nir_expr_effect_class(expr),
            NirExprEffectClass::Pure
                | NirExprEffectClass::LocalReadOnly
                | NirExprEffectClass::HostReadOnly
                | NirExprEffectClass::DomainReadOnly
        ),
    }
}

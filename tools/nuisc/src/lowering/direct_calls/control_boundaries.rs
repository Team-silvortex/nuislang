use super::*;

pub(super) fn lower_guarded_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some((
        NirStmt::If {
            condition,
            then_body,
            else_body,
        },
        tail,
    )) = function.body.split_first()
    else {
        return Err(format!(
            "outlined helper `{}` is missing its leading guard",
            function.name
        ));
    };
    let [NirStmt::Return(Some(default))] = then_body.as_slice() else {
        return Err(format!(
            "outlined helper `{}` has an invalid leading guard",
            function.name
        ));
    };
    let neutral = match function.return_type.as_ref().map(|ty| ty.name.as_str()) {
        Some("bool") => default == &NirExpr::Bool(false),
        Some("i64") => default == &NirExpr::Int(0),
        _ => false,
    };
    if !else_body.is_empty() || (!neutral && !is_pass_through_guard_seed(function, default, state))
    {
        return Err(format!(
            "outlined helper `{}` has an invalid leading guard",
            function.name
        ));
    }
    // A speculative select is not equivalent: even unused branch arithmetic can trap.
    let condition = lower_expr(condition, state, bindings)?;
    let returned = lower_expr(default, state, bindings)?;
    lower_guard_return(condition, returned, state);
    crate::lowering::body_lowering::lower_inline_stmts(tail, state, bindings, &mut BTreeMap::new())
}

fn is_pass_through_guard_seed(
    function: &NirFunction,
    value: &NirExpr,
    state: &LoweringState<'_>,
) -> bool {
    let i64_parameter = |value: &NirExpr| {
        matches!(value, NirExpr::Var(name)
        if function.params.iter().any(|param| &param.name == name
            && direct_call_scalar_kind(&param.ty) == Some(DirectCallScalarKind::I64)))
    };
    let Some(ty) = &function.return_type else {
        return false;
    };
    if direct_call_scalar_kind(ty) == Some(DirectCallScalarKind::I64) {
        return i64_parameter(value);
    }
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    else {
        return false;
    };
    let Some(definition) = state.struct_defs.get(type_name.as_str()) else {
        return false;
    };
    !ty.is_ref
        && !ty.is_optional
        && ty.generic_args.is_empty()
        && type_name == &ty.name
        && type_args.is_empty()
        && definition.generic_params.is_empty()
        && !fields.is_empty()
        && fields.len() == definition.fields.len()
        && fields
            .iter()
            .zip(&definition.fields)
            .all(|((name, value), field)| {
                name == &field.name
                    && direct_call_scalar_kind(&field.ty) == Some(DirectCallScalarKind::I64)
                    && i64_parameter(value)
            })
}

pub(in crate::lowering) fn collect_guarded_loop_direct_call_functions(
    module: &NirModule,
) -> BTreeSet<String> {
    let pure_helpers = collect_pure_helper_functions(module);
    let function_names = module
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::from(["main".to_owned()]);
    let mut frontier = vec!["main".to_owned()];
    while let Some(name) = frontier.pop() {
        let Some(function) = function_map.get(name.as_str()) else {
            continue;
        };
        for called in function_called_functions(function, &function.body, &function_names) {
            if reachable.insert(called.clone()) {
                frontier.push(called);
            }
        }
    }

    module
        .functions
        .iter()
        .filter(|function| function.name != "main")
        .filter(|function| !function.is_async)
        .filter(|function| reachable.contains(&function.name))
        .filter(|function| supports_direct_call_signature(function))
        // A guarded borrowed-buffer helper must return to its caller, not from it.
        .filter(|function| {
            stmts_contain_guarded_loop_boundary(&function.body, &pure_helpers)
                || (function.params.iter().any(|param| param.ty.is_ref && param.ty.name == "Buffer")
                    && !pure_helpers.contains(&function.name)
                    && function.body.iter().any(|stmt| matches!(stmt,
                        NirStmt::If { then_body, else_body, .. }
                            if else_body.is_empty() && matches!(then_body.as_slice(), [NirStmt::Return(Some(_))])
                    )))
        })
        .map(|function| function.name.clone())
        .collect()
}

/// A guard is a control boundary even for pure operations that can trap.
pub(super) fn order_guarded_function_nodes(
    state: &mut LoweringState<'_>,
    start: usize,
    preserve_source_order: bool,
) {
    let mut guard: Option<String> = None;
    let mut edges = Vec::new();
    if preserve_source_order {
        // Iterations, guarded arms and admitted scalar callees retain source evaluation order.
        edges.extend(
            state.yir.nodes[start..]
                .windows(2)
                .map(|pair| (pair[0].name.clone(), pair[1].name.clone())),
        );
    }
    for node in &state.yir.nodes[start..] {
        if let Some(previous) = &guard {
            edges.push((previous.clone(), node.name.clone()));
        }
        if node.op.module == "cpu" && node.op.instruction == "guard_return" {
            guard = Some(node.name.clone());
        }
    }
    for (from, to) in edges {
        crate::lowering::edge_helpers::push_effect_edge(state, &from, &to);
    }
}

fn stmts_contain_guarded_loop_boundary(stmts: &[NirStmt], pure_helpers: &BTreeSet<String>) -> bool {
    stmts.iter().any(|stmt| match stmt {
        NirStmt::While { body, .. } => {
            prepare_guarded_loop_body(body, pure_helpers).is_some()
                || stmts_contain_guarded_loop_boundary(body, pure_helpers)
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            stmts_contain_guarded_loop_boundary(then_body, pure_helpers)
                || stmts_contain_guarded_loop_boundary(else_body, pure_helpers)
        }
        _ => false,
    })
}

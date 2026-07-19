use super::*;

pub(super) fn collect_conditional_owned_return_helpers(module: &NirModule) -> BTreeSet<String> {
    let functions = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut helpers = BTreeSet::new();
    for function in &module.functions {
        collect_from_stmts(&function.body, &functions, &mut helpers);
    }
    helpers
}

fn collect_from_stmts<'a>(
    stmts: &[NirStmt],
    functions: &BTreeMap<&'a str, &'a NirFunction>,
    helpers: &mut BTreeSet<String>,
) {
    for stmt in stmts {
        match stmt {
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if let (Some((then_callee, _, _)), Some((else_callee, _, _))) = (
                    owned_return_call(then_body, functions),
                    owned_return_call(else_body, functions),
                ) {
                    helpers.insert(then_callee.to_owned());
                    helpers.insert(else_callee.to_owned());
                }
                collect_from_stmts(then_body, functions, helpers);
                collect_from_stmts(else_body, functions, helpers);
                if owned_return_tree(then_body, functions)
                    && owned_return_tree(else_body, functions)
                {
                    collect_owned_return_tree_helpers(then_body, functions, helpers);
                    collect_owned_return_tree_helpers(else_body, functions, helpers);
                }
            }
            NirStmt::While { body, .. } => collect_from_stmts(body, functions, helpers),
            _ => {}
        }
    }
}

fn owned_return_tree(stmts: &[NirStmt], functions: &BTreeMap<&str, &NirFunction>) -> bool {
    match stmts {
        [NirStmt::Return(Some(NirExpr::Move(_)))] => true,
        [NirStmt::Return(Some(NirExpr::Call { .. }))] => {
            owned_return_call(stmts, functions).is_some()
        }
        [NirStmt::If {
            then_body,
            else_body,
            ..
        }] => owned_return_tree(then_body, functions) && owned_return_tree(else_body, functions),
        _ => false,
    }
}

fn collect_owned_return_tree_helpers<'a>(
    stmts: &'a [NirStmt],
    functions: &BTreeMap<&str, &'a NirFunction>,
    helpers: &mut BTreeSet<String>,
) {
    if let Some((callee, _, _)) = owned_return_call(stmts, functions) {
        helpers.insert(callee.to_owned());
    } else if let [NirStmt::If {
        then_body,
        else_body,
        ..
    }] = stmts
    {
        collect_owned_return_tree_helpers(then_body, functions, helpers);
        collect_owned_return_tree_helpers(else_body, functions, helpers);
    }
}

pub(super) fn owned_return_call<'a>(
    stmts: &'a [NirStmt],
    functions: &BTreeMap<&str, &NirFunction>,
) -> Option<(&'a str, &'a NirExpr, &'a [NirExpr])> {
    let [NirStmt::Return(Some(NirExpr::Call { callee, args }))] = stmts else {
        return None;
    };
    let (owner @ NirExpr::Move(_), scalar_args) = args.split_first()? else {
        return None;
    };
    let function = functions.get(callee.as_str())?;
    owned_bytes_scalar_signature(function, args.len()).then_some((callee, owner, scalar_args))
}

fn owned_bytes_scalar_signature(function: &NirFunction, arg_count: usize) -> bool {
    function.params.len() == arg_count
        && !function.params.is_empty()
        && is_owned_bytes_type(&function.params[0].ty)
        && function.params[1..]
            .iter()
            .all(|param| is_plain_scalar_type(&param.ty))
        && function
            .return_type
            .as_ref()
            .is_some_and(is_owned_bytes_type)
}

fn is_plain_scalar_type(ty: &nuis_semantics::model::NirTypeRef) -> bool {
    !ty.is_ref
        && !ty.is_optional
        && ty.generic_args.is_empty()
        && matches!(ty.name.as_str(), "bool" | "i32" | "i64" | "f32" | "f64")
}

fn is_owned_bytes_type(ty: &nuis_semantics::model::NirTypeRef) -> bool {
    !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty() && ty.name == "Bytes"
}

pub(super) fn moved_owned_source(
    expr: &NirExpr,
    state: &LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<String> {
    let NirExpr::Move(value) = expr else {
        return None;
    };
    let NirExpr::Var(binding) = value.as_ref() else {
        return None;
    };
    let source = bindings.get(binding)?;
    state
        .yir
        .nodes
        .iter()
        .find(|node| node.name == *source)
        .filter(|node| {
            matches!(
                node.op.instruction.as_str(),
                "copy_buffer_owned"
                    | "move_owned_bytes"
                    | "param_owned_bytes"
                    | "call_owned_bytes"
                    | "branch_call_owned_bytes"
                    | "loop_owned_result"
                    | "select_owned_bytes"
                    | "select_owned_bytes_drop_unselected"
            )
        })
        .map(|_| source.clone())
}

pub(super) fn lower_conditional_owned_return_call(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some((then_callee, then_arg, then_scalar_args)) =
        owned_return_call(then_body, &state.function_map)
    else {
        return Ok(None);
    };
    let Some((else_callee, else_arg, else_scalar_args)) =
        owned_return_call(else_body, &state.function_map)
    else {
        return Ok(None);
    };
    if !state.direct_call_functions.contains(then_callee)
        || !state.direct_call_functions.contains(else_callee)
    {
        return Err(
            "conditional owned return helpers must be registered for direct static lowering"
                .to_owned(),
        );
    }
    let then_owner = moved_owned_source(then_arg, state, bindings).ok_or_else(|| {
        "conditional owned helper call requires `move(<Bytes binding>)` in the then branch"
            .to_owned()
    })?;
    let else_owner = moved_owned_source(else_arg, state, bindings).ok_or_else(|| {
        "conditional owned helper call requires `move(<Bytes binding>)` in the else branch"
            .to_owned()
    })?;
    if then_owner != else_owner {
        return Err("conditional owned helper calls currently require both branches to consume the same owner; distinct owners require conditional cleanup"
            .to_owned());
    }
    let then_scalar_args = lower_pure_scalar_args(then_scalar_args, state, bindings, "then")?;
    let else_scalar_args = lower_pure_scalar_args(else_scalar_args, state, bindings, "else")?;
    Ok(Some(lower_branch_call_owned_bytes(
        condition_name,
        then_callee.to_owned(),
        else_callee.to_owned(),
        then_owner,
        then_scalar_args,
        else_scalar_args,
        state,
    )))
}

pub(super) fn lower_pure_scalar_args(
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    branch: &str,
) -> Result<Vec<String>, String> {
    args.iter()
        .map(|arg| {
            if nir_expr_effect_class(arg) != NirExprEffectClass::Pure {
                return Err(format!(
                    "conditional owned helper {branch} scalar arguments must be pure"
                ));
            }
            lower_expr(arg, state, bindings)
        })
        .collect()
}

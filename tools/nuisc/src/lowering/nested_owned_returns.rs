use super::conditional_owned_calls::{
    lower_pure_scalar_args, moved_owned_source, owned_return_call_with_non_null_proofs,
};
use super::*;

enum OwnedReturnBranch<'a> {
    Owner(&'a NirExpr),
    Call {
        callee: &'a str,
        owner: &'a NirExpr,
        scalar_args: Vec<OwnedReturnScalarArg<'a>>,
    },
    If {
        condition: &'a NirExpr,
        then_branch: Box<OwnedReturnBranch<'a>>,
        else_branch: Box<OwnedReturnBranch<'a>>,
    },
}

enum OwnedReturnScalarArg<'a> {
    Value(&'a NirExpr),
    VariantField {
        base: &'a NirExpr,
        variant: &'a str,
        field: &'a str,
    },
    StructField {
        field: &'a str,
        base: Box<OwnedReturnScalarArg<'a>>,
    },
    Cast {
        kind: yir_core::OwnedSelectScalarCast,
        value: Box<OwnedReturnScalarArg<'a>>,
    },
    NonNull {
        value: Box<OwnedReturnScalarArg<'a>>,
    },
    TraversalBorrow {
        value: Box<OwnedReturnScalarArg<'a>>,
    },
}

pub(super) fn collect_owned_return_tree_helpers(
    stmts: &[NirStmt],
    functions: &BTreeMap<&str, &NirFunction>,
) -> Option<BTreeSet<String>> {
    let branch = parse_owned_return_branch(stmts, functions, &[])?;
    let mut helpers = BTreeSet::new();
    collect_branch_helpers(&branch, &mut helpers);
    Some(helpers)
}

pub(super) fn collect_owned_return_if_helpers(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    functions: &BTreeMap<&str, &NirFunction>,
) -> Option<BTreeSet<String>> {
    let then_proofs = branch_non_null_proofs(&[], condition, true);
    let else_proofs = branch_non_null_proofs(&[], condition, false);
    let then_branch = parse_owned_return_branch(then_body, functions, &then_proofs)?;
    let else_branch = parse_owned_return_branch(else_body, functions, &else_proofs)?;
    let mut helpers = BTreeSet::new();
    collect_branch_helpers(&then_branch, &mut helpers);
    collect_branch_helpers(&else_branch, &mut helpers);
    Some(helpers)
}

fn collect_branch_helpers(branch: &OwnedReturnBranch<'_>, helpers: &mut BTreeSet<String>) {
    match branch {
        OwnedReturnBranch::Owner(_) => {}
        OwnedReturnBranch::Call { callee, .. } => {
            helpers.insert((*callee).to_owned());
        }
        OwnedReturnBranch::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_branch_helpers(then_branch, helpers);
            collect_branch_helpers(else_branch, helpers);
        }
    }
}

fn parse_owned_return_branch<'a>(
    stmts: &'a [NirStmt],
    functions: &BTreeMap<&str, &'a NirFunction>,
    non_null_proofs: &[&'a NirExpr],
) -> Option<OwnedReturnBranch<'a>> {
    let (projections, tail) = split_variant_field_prelude(stmts);
    if let [NirStmt::Return(Some(NirExpr::Call { .. }))] = tail {
        let (callee, owner, scalar_args) =
            owned_return_call_with_non_null_proofs(tail, functions, non_null_proofs)?;
        let scalar_args = scalar_args
            .iter()
            .map(|arg| selected_leaf_scalar_arg(arg, &projections, 0))
            .collect::<Option<Vec<_>>>()?;
        return Some(OwnedReturnBranch::Call {
            callee,
            owner,
            scalar_args,
        });
    }
    let names = projections.keys().copied().collect::<BTreeSet<_>>();
    if !names.is_empty() && stmts_reference_any_binding(tail, &names) {
        return None;
    }
    let stmts = tail;
    match stmts {
        [NirStmt::Return(Some(expr @ NirExpr::Move(_)))] => Some(OwnedReturnBranch::Owner(expr)),
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let then_proofs = branch_non_null_proofs(non_null_proofs, condition, true);
            let else_proofs = branch_non_null_proofs(non_null_proofs, condition, false);
            Some(OwnedReturnBranch::If {
                condition,
                then_branch: Box::new(parse_owned_return_branch(
                    then_body,
                    functions,
                    &then_proofs,
                )?),
                else_branch: Box::new(parse_owned_return_branch(
                    else_body,
                    functions,
                    &else_proofs,
                )?),
            })
        }
        _ => None,
    }
}

fn branch_non_null_proofs<'a>(
    inherited: &[&'a NirExpr],
    condition: &'a NirExpr,
    branch_value: bool,
) -> Vec<&'a NirExpr> {
    let mut proofs = inherited.to_vec();
    let Some((source, condition_means_null)) = null_test_source(condition) else {
        return proofs;
    };
    if branch_value != condition_means_null && !proofs.contains(&source) {
        proofs.push(source);
    }
    proofs
}

fn null_test_source(expr: &NirExpr) -> Option<(&NirExpr, bool)> {
    match expr {
        NirExpr::IsNull(source) => Some((source, true)),
        NirExpr::Binary { op, lhs, rhs } => {
            let (source, expected) = match (lhs.as_ref(), rhs.as_ref()) {
                (NirExpr::IsNull(source), NirExpr::Bool(expected))
                | (NirExpr::Bool(expected), NirExpr::IsNull(source)) => {
                    (source.as_ref(), *expected)
                }
                _ => return None,
            };
            match op {
                NirBinaryOp::Eq => Some((source, expected)),
                NirBinaryOp::Ne => Some((source, !expected)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn selected_leaf_scalar_arg<'a>(
    expr: &'a NirExpr,
    projections: &BTreeMap<&'a str, &'a NirExpr>,
    depth: usize,
) -> Option<OwnedReturnScalarArg<'a>> {
    if depth >= 64 {
        return None;
    }
    match expr {
        NirExpr::Var(name) if projections.contains_key(name.as_str()) => {
            selected_leaf_scalar_arg(projections[name.as_str()], projections, depth + 1)
        }
        NirExpr::VariantFieldAccess {
            base,
            variant,
            field,
        } => Some(OwnedReturnScalarArg::VariantField {
            base,
            variant,
            field,
        }),
        NirExpr::FieldAccess { base, field } => Some(OwnedReturnScalarArg::StructField {
            field,
            base: Box::new(selected_leaf_scalar_arg(base, projections, depth + 1)?),
        }),
        NirExpr::Call { callee, args }
            if callee == "__nuis_require_non_null_buffer" && args.len() == 1 =>
        {
            Some(OwnedReturnScalarArg::NonNull {
                value: Box::new(selected_leaf_scalar_arg(&args[0], projections, depth + 1)?),
            })
        }
        NirExpr::Borrow(value) => Some(OwnedReturnScalarArg::TraversalBorrow {
            value: Box::new(selected_leaf_scalar_arg(value, projections, depth + 1)?),
        }),
        NirExpr::CastI64ToI32(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::I64ToI32,
            value,
            projections,
            depth,
        ),
        NirExpr::CastI32ToI64(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::I32ToI64,
            value,
            projections,
            depth,
        ),
        NirExpr::CastI64ToBool(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::I64ToBool,
            value,
            projections,
            depth,
        ),
        NirExpr::CastBoolToI64(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::BoolToI64,
            value,
            projections,
            depth,
        ),
        NirExpr::CastI64ToF32(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::I64ToF32,
            value,
            projections,
            depth,
        ),
        NirExpr::CastF32ToI64(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::F32ToI64,
            value,
            projections,
            depth,
        ),
        NirExpr::CastI64ToF64(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::I64ToF64,
            value,
            projections,
            depth,
        ),
        NirExpr::CastF64ToI64(value) => selected_leaf_cast(
            yir_core::OwnedSelectScalarCast::F64ToI64,
            value,
            projections,
            depth,
        ),
        _ if expr_references_any_binding(expr, &projections.keys().copied().collect()) => None,
        _ => Some(OwnedReturnScalarArg::Value(expr)),
    }
}

fn selected_leaf_cast<'a>(
    kind: yir_core::OwnedSelectScalarCast,
    value: &'a NirExpr,
    projections: &BTreeMap<&'a str, &'a NirExpr>,
    depth: usize,
) -> Option<OwnedReturnScalarArg<'a>> {
    Some(OwnedReturnScalarArg::Cast {
        kind,
        value: Box::new(selected_leaf_scalar_arg(value, projections, depth + 1)?),
    })
}

fn split_variant_field_prelude<'a>(
    stmts: &'a [NirStmt],
) -> (BTreeMap<&'a str, &'a NirExpr>, &'a [NirStmt]) {
    let mut projections = BTreeMap::new();
    let mut prefix_len = 0;
    for stmt in stmts {
        let NirStmt::Let { name, value, .. } = stmt else {
            break;
        };
        if !matches!(value, NirExpr::VariantFieldAccess { .. }) {
            break;
        }
        projections.insert(name.as_str(), value);
        prefix_len += 1;
    }
    (projections, &stmts[prefix_len..])
}

pub(super) fn strip_unused_pure_leaf_prelude(stmts: &[NirStmt]) -> Option<&[NirStmt]> {
    let prefix_len = stmts
        .iter()
        .take_while(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { value, .. }
                    if nir_expr_effect_class(value) == NirExprEffectClass::Pure
            )
        })
        .count();
    if prefix_len == 0 {
        return Some(stmts);
    }
    let names = stmts[..prefix_len]
        .iter()
        .filter_map(|stmt| match stmt {
            NirStmt::Let { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let tail = &stmts[prefix_len..];
    (!stmts_reference_any_binding(tail, &names)).then_some(tail)
}

fn stmts_reference_any_binding(stmts: &[NirStmt], names: &BTreeSet<&str>) -> bool {
    stmts.iter().any(|stmt| match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value)
        | NirStmt::Return(Some(value)) => expr_references_any_binding(value, names),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_references_any_binding(condition, names)
                || stmts_reference_any_binding(then_body, names)
                || stmts_reference_any_binding(else_body, names)
        }
        NirStmt::While { condition, body } => {
            expr_references_any_binding(condition, names)
                || stmts_reference_any_binding(body, names)
        }
        NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => false,
    })
}

fn expr_references_any_binding(expr: &NirExpr, names: &BTreeSet<&str>) -> bool {
    if matches!(expr, NirExpr::Var(name) if names.contains(name.as_str())) {
        return true;
    }
    let mut found = false;
    crate::nir_walk::walk_child_exprs(expr, &mut |child| {
        found |= expr_references_any_binding(child, names);
    });
    found
}

fn encode_owned_return_branch(
    branch: &OwnedReturnBranch<'_>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    owners: &mut Vec<String>,
    conditions: &mut Vec<String>,
    tokens: &mut Vec<String>,
) -> Result<(), String> {
    match branch {
        OwnedReturnBranch::Owner(expr) => {
            let owner = moved_owned_source(expr, state, bindings).ok_or_else(|| {
                "nested conditional owned return requires `move(<Bytes binding>)` leaves".to_owned()
            })?;
            let index = owners
                .iter()
                .position(|known| known == &owner)
                .unwrap_or_else(|| {
                    owners.push(owner);
                    owners.len() - 1
                });
            tokens.extend(["owner".to_owned(), index.to_string()]);
        }
        OwnedReturnBranch::Call {
            callee,
            owner,
            scalar_args,
        } => {
            if !state.direct_call_functions.contains(*callee) {
                return Err(format!(
                    "nested conditional owned helper `{callee}` is not registered for direct static lowering"
                ));
            }
            let owner = moved_owned_source(owner, state, bindings).ok_or_else(|| {
                "nested conditional owned helper requires `move(<Bytes binding>)` as its first argument"
                    .to_owned()
            })?;
            let index = owners
                .iter()
                .position(|known| known == &owner)
                .unwrap_or_else(|| {
                    owners.push(owner);
                    owners.len() - 1
                });
            tokens.extend([
                "call".to_owned(),
                (*callee).to_owned(),
                index.to_string(),
                scalar_args.len().to_string(),
            ]);
            for arg in scalar_args {
                encode_owned_scalar_arg(arg, state, bindings, tokens)?;
            }
        }
        OwnedReturnBranch::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if nir_expr_effect_class(condition) != NirExprEffectClass::Pure {
                return Err(
                    "nested conditional owned return conditions must be pure before survivor-state lowering"
                        .to_owned(),
                );
            }
            let condition = lower_expr(condition, state, bindings)?;
            conditions.push(condition.clone());
            tokens.extend(["if".to_owned(), condition]);
            encode_owned_return_branch(then_branch, state, bindings, owners, conditions, tokens)?;
            encode_owned_return_branch(else_branch, state, bindings, owners, conditions, tokens)?;
        }
    }
    Ok(())
}

fn encode_owned_scalar_arg(
    arg: &OwnedReturnScalarArg<'_>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    tokens: &mut Vec<String>,
) -> Result<(), String> {
    match arg {
        OwnedReturnScalarArg::Value(arg) => {
            let [arg] =
                lower_pure_scalar_args(std::slice::from_ref(arg), state, bindings, "tree leaf")?
                    .try_into()
                    .expect("one scalar argument");
            tokens.extend(["value".to_owned(), arg]);
        }
        OwnedReturnScalarArg::VariantField {
            base,
            variant,
            field,
        } => {
            let base = lower_expr(base, state, bindings)?;
            tokens.extend([
                "variant_field".to_owned(),
                base,
                (*variant).to_owned(),
                (*field).to_owned(),
            ]);
        }
        OwnedReturnScalarArg::StructField { field, base } => {
            tokens.extend(["struct_field".to_owned(), (*field).to_owned()]);
            encode_owned_scalar_arg(base, state, bindings, tokens)?;
        }
        OwnedReturnScalarArg::Cast { kind, value } => {
            tokens.extend(["cast".to_owned(), kind.as_str().to_owned()]);
            encode_owned_scalar_arg(value, state, bindings, tokens)?;
        }
        OwnedReturnScalarArg::NonNull { value } => {
            tokens.push("non_null".to_owned());
            encode_owned_scalar_arg(value, state, bindings, tokens)?;
        }
        OwnedReturnScalarArg::TraversalBorrow { value } => {
            tokens.push("traversal_borrow".to_owned());
            encode_owned_scalar_arg(value, state, bindings, tokens)?;
        }
    }
    Ok(())
}

pub(super) fn lower_nested_owned_return_tree(
    root_condition: String,
    root_condition_expr: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let then_proofs = branch_non_null_proofs(&[], root_condition_expr, true);
    let else_proofs = branch_non_null_proofs(&[], root_condition_expr, false);
    let Some(then_branch) = parse_owned_return_branch(then_body, &state.function_map, &then_proofs)
    else {
        return Ok(None);
    };
    let Some(else_branch) = parse_owned_return_branch(else_body, &state.function_map, &else_proofs)
    else {
        return Ok(None);
    };
    if matches!(&then_branch, OwnedReturnBranch::Owner(_))
        && matches!(&else_branch, OwnedReturnBranch::Owner(_))
    {
        return Ok(None);
    }
    let mut owners = Vec::new();
    let mut conditions = vec![root_condition.clone()];
    let mut tokens = vec!["if".to_owned(), root_condition];
    encode_owned_return_branch(
        &then_branch,
        state,
        bindings,
        &mut owners,
        &mut conditions,
        &mut tokens,
    )?;
    encode_owned_return_branch(
        &else_branch,
        state,
        bindings,
        &mut owners,
        &mut conditions,
        &mut tokens,
    )?;
    Ok(Some(lower_select_owned_bytes_tree(
        owners,
        tokens,
        &conditions,
        state,
    )))
}

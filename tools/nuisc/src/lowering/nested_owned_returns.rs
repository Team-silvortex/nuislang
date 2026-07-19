use super::conditional_owned_calls::{
    lower_pure_scalar_args, moved_owned_source, owned_return_call,
};
use super::*;

enum OwnedReturnBranch<'a> {
    Owner(&'a NirExpr),
    Call {
        callee: &'a str,
        owner: &'a NirExpr,
        scalar_args: &'a [NirExpr],
    },
    If {
        condition: &'a NirExpr,
        then_branch: Box<OwnedReturnBranch<'a>>,
        else_branch: Box<OwnedReturnBranch<'a>>,
    },
}

fn parse_owned_return_branch<'a>(
    stmts: &'a [NirStmt],
    functions: &BTreeMap<&str, &'a NirFunction>,
) -> Option<OwnedReturnBranch<'a>> {
    match stmts {
        [NirStmt::Return(Some(expr @ NirExpr::Move(_)))] => Some(OwnedReturnBranch::Owner(expr)),
        [NirStmt::Return(Some(NirExpr::Call { .. }))] => {
            let (callee, owner, scalar_args) = owned_return_call(stmts, functions)?;
            Some(OwnedReturnBranch::Call {
                callee,
                owner,
                scalar_args,
            })
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => Some(OwnedReturnBranch::If {
            condition,
            then_branch: Box::new(parse_owned_return_branch(then_body, functions)?),
            else_branch: Box::new(parse_owned_return_branch(else_body, functions)?),
        }),
        _ => None,
    }
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
            let scalar_args = lower_pure_scalar_args(scalar_args, state, bindings, "tree leaf")?;
            tokens.extend([
                "call".to_owned(),
                (*callee).to_owned(),
                index.to_string(),
                scalar_args.len().to_string(),
            ]);
            tokens.extend(scalar_args);
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

pub(super) fn lower_nested_owned_return_tree(
    root_condition: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some(then_branch) = parse_owned_return_branch(then_body, &state.function_map) else {
        return Ok(None);
    };
    let Some(else_branch) = parse_owned_return_branch(else_body, &state.function_map) else {
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

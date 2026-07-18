use nuis_semantics::model::NirStmt;

use super::{finish_loop_edge, CleanupContext, CleanupState};

pub(super) fn rewrite_direct_loop_control_if(
    then_body: Vec<NirStmt>,
    else_body: Vec<NirStmt>,
    state: &CleanupState,
    entry: &CleanupState,
    scope_start: usize,
    context: &mut CleanupContext<'_>,
) -> Result<Option<(Vec<NirStmt>, Vec<NirStmt>)>, ()> {
    let then_is_empty = then_body.is_empty();
    let else_is_empty = else_body.is_empty();
    let then_body = if then_is_empty {
        Vec::new()
    } else {
        let Some(rewritten) =
            rewrite_direct_loop_control_branch(then_body, state, entry, scope_start, context)?
        else {
            return Ok(None);
        };
        rewritten
    };
    let else_body = if else_is_empty {
        Vec::new()
    } else {
        let Some(rewritten) =
            rewrite_direct_loop_control_branch(else_body, state, entry, scope_start, context)?
        else {
            return Ok(None);
        };
        rewritten
    };
    if then_is_empty && else_is_empty {
        Ok(None)
    } else {
        Ok(Some((then_body, else_body)))
    }
}

fn rewrite_direct_loop_control_branch(
    body: Vec<NirStmt>,
    state: &CleanupState,
    entry: &CleanupState,
    scope_start: usize,
    context: &mut CleanupContext<'_>,
) -> Result<Option<Vec<NirStmt>>, ()> {
    match body.as_slice() {
        [control @ (NirStmt::Break | NirStmt::Continue)] => {
            let mut branch_state = state.clone();
            let mut rewritten = Vec::new();
            finish_loop_edge(
                &mut rewritten,
                &mut branch_state,
                entry,
                scope_start,
                context,
            )?;
            rewritten.push(control.clone());
            Ok(Some(rewritten))
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let Some((then_body, else_body)) = rewrite_direct_loop_control_if(
                then_body.clone(),
                else_body.clone(),
                state,
                entry,
                scope_start,
                context,
            )?
            else {
                return Ok(None);
            };
            Ok(Some(vec![NirStmt::If {
                condition: condition.clone(),
                then_body,
                else_body,
            }]))
        }
        _ => Ok(None),
    }
}

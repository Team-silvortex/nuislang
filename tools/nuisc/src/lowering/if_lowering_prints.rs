use super::*;

pub(super) fn lower(
    condition: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let ([NirStmt::Print(value)], []) = (then_body, else_body) else {
        return Ok(None);
    };
    // Only ready atoms cross this shortcut. Fallible/effectful print arguments
    // need a separate selected-evaluation proof, not eager materialization.
    if !matches!(value, NirExpr::Int(_) | NirExpr::Var(_)) {
        return Ok(None);
    }
    let value = lower_expr(value, state, bindings)?;
    lower_guard_print(condition, value, state);
    Ok(Some(LoweredIfOutcome::Printed))
}

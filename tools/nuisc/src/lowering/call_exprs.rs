use super::*;

pub(super) fn lower_call_family_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::Await(value) => Some(lower_await_expr(value, state, bindings)),
        NirExpr::Call { callee, args } => Some(lower_call_expr(callee, args, state, bindings)),
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => Some(lower_method_call_expr(
            receiver, method, args, state, bindings,
        )),
        _ => None,
    }
}

fn lower_await_expr(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let awaited = match value {
        NirExpr::Call { callee, args } => lower_async_call_boundary(callee, args, state, bindings)?,
        _ => lower_expr(value, state, bindings)?,
    };
    let await_name = push_await_node(state, &awaited);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: awaited,
        to: await_name.clone(),
    });
    Ok(await_name)
}

fn lower_method_call_expr(
    receiver: &NirExpr,
    method: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut call_args = Vec::with_capacity(args.len() + 1);
    call_args.push(receiver.clone());
    call_args.extend(args.iter().cloned());
    lower_call_expr(method, &call_args, state, bindings)
}

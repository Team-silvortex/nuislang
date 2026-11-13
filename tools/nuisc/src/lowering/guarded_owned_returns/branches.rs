use super::*;

pub(super) fn pair(
    template: &NirFunction,
    then_body: &mut Vec<NirStmt>,
    else_body: &mut Vec<NirStmt>,
    remaining: &mut VecDeque<NirStmt>,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    depth: usize,
    context: &mut OutlineContext,
) -> Result<NirStmt, String> {
    if !regions::block(then_body, depth + 1)? || !regions::block(else_body, depth + 1)? {
        return Err("unsupported owned branch-pair region".into());
    }
    let then_returns = regions::ends_in_return(then_body);
    let else_returns = regions::ends_in_return(else_body);
    if !then_returns || !else_returns {
        for branch in [&*then_body, &*else_body] {
            if !regions::ends_in_return(branch) {
                continuations::check_captures(branch, scope, live)?;
            }
        }
        let tail = continuations::share(
            template,
            std::mem::take(remaining).into_iter().collect(),
            scope,
            live,
            depth + 2,
            context,
        )?;
        if !then_returns {
            then_body.push(tail.clone());
        }
        if !else_returns {
            else_body.push(tail);
        }
    }
    let selected = arm(
        template,
        std::mem::take(then_body),
        scope,
        live,
        depth + 1,
        context,
    )?;
    let fallback = arm(
        template,
        std::mem::take(else_body),
        scope,
        live,
        depth + 1,
        context,
    )?;
    // A taken guard ends the caller. Only its untaken edge reaches the fallback;
    // do not reevaluate or negate a source condition to dispatch the second arm.
    *then_body = vec![selected];
    Ok(fallback)
}

pub(super) fn arm(
    template: &NirFunction,
    body: Vec<NirStmt>,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    depth: usize,
    context: &mut OutlineContext,
) -> Result<NirStmt, String> {
    if !regions::terminal_return_arm(&body, depth)? {
        return Err("unsupported owned return region".into());
    }
    if context.roots + context.outlines.owners.len() >= MAX_RETURN_HELPER_FUNCTIONS {
        return Err("resource return helpers exceed the source closure bound".into());
    }
    let mut name = format!("__nuis_owned_arm_{}", context.outlines.owners.len());
    while !context.reserved.insert(name.clone()) {
        name.push('_');
    }
    let mut helper = template.clone();
    helper.name = name.clone();
    helper.annotations.clear();
    helper.params = scope.to_vec();
    helper.body = body;
    let edges =
        regions::partition(&mut helper.body, live).ok_or("invalid owned return captures")?;
    helper
        .params
        .extend(edges.transfers.iter().map(|name| NirParam {
            name: name.clone(),
            ty: NirTypeRef {
                name: "Bytes".into(),
                generic_args: vec![],
                is_ref: false,
                is_optional: false,
            },
        }));
    let args = helper
        .params
        .iter()
        .map(|p| NirExpr::Var(p.name.clone()))
        .collect();
    context.outlines.owners.insert(name.clone(), edges);
    context.pending.push_back((helper, depth));
    Ok(NirStmt::Return(Some(NirExpr::Call { callee: name, args })))
}

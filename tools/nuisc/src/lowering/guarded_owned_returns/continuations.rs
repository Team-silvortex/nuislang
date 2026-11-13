use super::*;

pub(super) fn share(
    template: &NirFunction,
    body: Vec<NirStmt>,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    depth: usize,
    context: &mut OutlineContext,
) -> Result<NirStmt, String> {
    if context.roots + context.outlines.owners.len() >= MAX_RETURN_HELPER_FUNCTIONS {
        return Err("resource return helpers exceed the source closure bound".into());
    }
    if !regions::terminal_return_arm(&body, depth)? {
        return Err("owned continuation requires a bounded terminal suffix".into());
    }
    let mut name = format!("__nuis_owned_continue_{}", context.outlines.owners.len());
    while !context.reserved.insert(name.clone()) {
        name.push('_');
    }
    let mut helper = template.clone();
    helper.name = name.clone();
    helper.annotations.clear();
    helper.body = body;
    helper.params = scope.to_vec();
    helper.params.extend(live.iter().map(|name| NirParam {
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
    context.outlines.owners.insert(
        name.clone(),
        OwnerEdges {
            cleanup: vec![],
            transfers: live.iter().cloned().collect(),
        },
    );
    context.pending.push_back((helper, depth));
    Ok(NirStmt::Return(Some(NirExpr::Call { callee: name, args })))
}

// Suffix captures refer to the entry scope, not a redefinition inside the arm.
// Resource-root scalar declarations have lexical identities before NIR. Actual
// captured writes and resource rebinding still need a carry/phi contract.
pub(super) fn check_captures(
    body: &[NirStmt],
    scope: &[NirParam],
    live: &BTreeSet<String>,
) -> Result<(), String> {
    for stmt in body {
        if let NirStmt::Let { name, .. } | NirStmt::Const { name, .. } = stmt {
            if live.contains(name) || scope.iter().any(|p| p.name == *name) {
                return Err(format!(
                    "owned continuation cannot rebind captured `{name}`"
                ));
            }
        }
    }
    Ok(())
}

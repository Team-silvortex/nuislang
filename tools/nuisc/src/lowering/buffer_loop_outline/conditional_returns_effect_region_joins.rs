use super::*;

pub(super) struct Arm<'a> {
    pub body: &'a [NirStmt],
    pub scope: &'a Scope,
    pub values: &'a BTreeMap<String, NirExpr>,
    pub continues: bool,
}

pub(super) struct Join {
    pub name: String,
    pub ty: NirTypeRef,
    condition: String,
    yes: Option<NirExpr>,
    no: Option<NirExpr>,
    captures: Vec<NirParam>,
}

pub(super) fn prepare(
    yes: Arm<'_>,
    no: Arm<'_>,
    scope: &Scope,
    ready: &mut Scope,
    condition: &str,
    bindings: &mut BTreeSet<String>,
) -> Option<(String, Join)> {
    // Preparation reaches the final binding only on continuing source paths;
    // installation uses their merged live mask, not a scalar seed, for presence.
    if !yes.continues && !no.continues {
        return None;
    }
    let continuing = if yes.continues { &yes } else { &no };
    let (target, constant) = destination(continuing.body)?;
    if scope.contains_key(target) {
        return None;
    }
    let ty = continuing.scope.get(target)?;
    if !data_scalars::admitted(ty) {
        return None;
    }
    // All computation was already staged at its source position. Joining may
    // read only selected snapshots or total atoms, never replay an expression.
    let value = |arm: &Arm<'_>| -> Option<Option<NirExpr>> {
        if !arm.continues {
            return Some(None);
        }
        if destination(arm.body)? != (target, constant) || arm.scope.get(target)? != ty {
            return None;
        }
        let value = arm.values.get(target)?;
        atom(value, ty, ready).then(|| Some(value.clone()))
    };
    let yes = value(&yes)?;
    let no = value(&no)?;
    if ready.get(condition) != Some(&scalar_type("bool")) {
        return None;
    }
    let mut inputs = BTreeSet::new();
    for value in yes.iter().chain(no.iter()) {
        control_values::collect_inputs(value, &mut inputs);
    }
    // Only distinct validated atoms need a selector. Even equal results may
    // read the saved condition as data, which the atom inputs above retain.
    if matches!((&yes, &no), (Some(yes), Some(no)) if yes != no) {
        inputs.insert(condition.to_string());
    }
    if inputs
        .iter()
        .any(|name| !ready.get(name).is_some_and(data_scalars::admitted))
    {
        return None;
    }
    let name = branches::fresh_name("__nuis_effect_join", bindings);
    let joined = Join {
        name: name.clone(),
        ty: ty.clone(),
        condition: condition.into(),
        yes,
        no,
        captures: captured_params(inputs, ready),
    };
    ready.insert(name, ty.clone());
    Some((target.into(), joined))
}

#[cfg(test)]
#[path = "conditional_returns_effect_join_capture_tests.rs"]
mod tests;

fn destination(body: &[NirStmt]) -> Option<(&str, bool)> {
    match body.last()? {
        NirStmt::Let { name, .. } => Some((name, false)),
        NirStmt::Const { name, .. } => Some((name, true)),
        _ => None,
    }
}

fn atom(value: &NirExpr, ty: &NirTypeRef, ready: &Scope) -> bool {
    match value {
        NirExpr::Var(name) => ready.get(name) == Some(ty),
        NirExpr::Int(_) => ty == &scalar_type("i64"),
        NirExpr::Bool(_) => ty == &scalar_type("bool"),
        NirExpr::F32(_) => ty == &scalar_type("f32"),
        NirExpr::F64(_) => ty == &scalar_type("f64"),
        _ => false,
    }
}

pub(super) fn install(
    joined: Join,
    live: &NirExpr,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    output: &mut Vec<NirStmt>,
) {
    let mut parameter_names = joined.captures.iter().map(|p| p.name.clone()).collect();
    let gate = branches::fresh_name("__nuis_join_live", &mut parameter_names);
    let mut params = vec![NirParam {
        name: gate.clone(),
        ty: scalar_type("bool"),
    }];
    params.extend(joined.captures);
    let mut args = vec![live.clone()];
    args.extend(params[1..].iter().map(|p| NirExpr::Var(p.name.clone())));
    let name = branches::fresh_name("__nuis_effect_join_value", names);
    let seed = data_scalars::seed(&joined.ty);
    // Source-arm presence stays separate from atom equality. Only the merged
    // source-live mask authorizes a value after its continuing arm has executed.
    let selected = match (joined.yes, joined.no) {
        (Some(yes), Some(no)) if yes == no => NirStmt::Return(Some(yes)),
        (Some(yes), Some(no)) => NirStmt::If {
            condition: NirExpr::Var(joined.condition),
            then_body: vec![NirStmt::Return(Some(yes))],
            else_body: vec![NirStmt::Return(Some(no))],
        },
        (Some(value), None) | (None, Some(value)) => NirStmt::Return(Some(value)),
        (None, None) => unreachable!("join preparation requires a continuing source arm"),
    };
    let mut function = helper(
        name.clone(),
        params,
        vec![NirStmt::If {
            condition: NirExpr::Var(gate),
            then_body: vec![selected],
            else_body: vec![NirStmt::Return(Some(seed))],
        }],
    );
    function.return_type = Some(joined.ty.clone());
    helpers.push(function);
    output.push(NirStmt::Let {
        name: joined.name,
        ty: Some(joined.ty),
        value: NirExpr::Call { callee: name, args },
    });
}

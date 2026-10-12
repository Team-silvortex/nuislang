use super::*;

pub(in super::super) struct Selection {
    plan: Plan,
    fields: Vec<(String, String, NirTypeRef)>,
}

pub(in super::super) fn arity(count: usize) -> bool {
    // Keep the owner plus certified children within the 64-field scalar bound.
    (2..64).contains(&count)
}

pub(in super::super) fn child_scalar(ty: &NirTypeRef) -> bool {
    ty == &scalar_type("bool") || numeric(ty)
}

pub(in super::super) fn prepare(
    condition: predicates::Predicate,
    yes: regions::Staging,
    no: regions::Staging,
    scope: &Scope,
    owner: &str,
    children: &[String],
) -> Option<(Selection, BTreeSet<String>, BTreeSet<String>)> {
    // Arity is representation capacity, never lexical write authority.
    if !arity(children.len()) || !numeric(scope.get(owner)?) {
        return None;
    }
    let mut seen = BTreeSet::from([owner.to_owned()]);
    let mut fields = vec![("owner".into(), owner.into(), scope.get(owner)?.clone())];
    for (index, name) in children.iter().enumerate() {
        let ty = scope.get(name)?;
        if !seen.insert(name.clone()) || !child_scalar(ty) {
            return None;
        }
        fields.push((format!("child_{index}"), name.clone(), ty.clone()));
    }
    let mut captures = BTreeSet::new();
    let mut calls = BTreeSet::new();
    for arm in [&yes, &no] {
        captures.extend(arm.inputs.iter().cloned());
        captures.extend(
            fields
                .iter()
                .filter(|(_, name, _)| !arm.defined.contains(name))
                .map(|(_, name, _)| name.clone()),
        );
        calls.extend(arm.calls.iter().cloned());
    }
    if captures.len() > 31 {
        return None;
    }
    let params = captures
        .iter()
        .map(|name| {
            let ty = scope.get(name)?;
            scalar(ty).then(|| NirParam {
                name: name.clone(),
                ty: ty.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some((
        Selection {
            plan: Plan {
                condition,
                yes: SelectedValue::Region(yes.statements, NirExpr::Int(0)),
                no: SelectedValue::Region(no.statements, NirExpr::Int(0)),
                destination: None,
                ty: scalar_type("i64"),
                params,
            },
            fields,
        },
        captures,
        calls,
    ))
}

pub(in super::super) fn install(
    selection: Selection,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    structs: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    record::install(
        selection.plan,
        selection.fields,
        "__nuis_effect_fields_value",
        "__nuis_effect_call_fields",
        names,
        bindings,
        helpers,
        structs,
    )
}

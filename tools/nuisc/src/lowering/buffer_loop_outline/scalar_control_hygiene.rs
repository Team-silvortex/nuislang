use super::*;

pub(super) fn prepare_arms(
    then_body: &mut [NirStmt],
    else_body: &mut [NirStmt],
    tail: &[NirStmt],
    scope: &Scope,
    reserved: &mut BTreeSet<String>,
) {
    let mut arms = BTreeSet::new();
    collect_bindings(then_body, &mut arms);
    collect_bindings(else_body, &mut arms);
    let mut suffix = BTreeSet::new();
    collect_bindings(tail, &mut suffix);
    let collisions = arms
        .intersection(&suffix)
        .filter(|name| !scope.contains_key(*name))
        .cloned()
        .collect::<BTreeSet<_>>();
    if collisions.is_empty() {
        return;
    }
    let visible = scope
        .keys()
        .map(|name| (name.clone(), name.clone()))
        .collect();
    rename_block(then_body, &visible, &collisions, reserved);
    rename_block(else_body, &visible, &collisions, reserved);
}

fn rename_block(
    body: &mut [NirStmt],
    visible: &BTreeMap<String, String>,
    collisions: &BTreeSet<String>,
    reserved: &mut BTreeSet<String>,
) {
    let mut pending = vec![(body, visible.clone())];
    while let Some((body, mut visible)) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    rename_binding(name, value, &mut visible, collisions, reserved);
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    rename_expr(condition, &visible);
                    // An admitted paired declaration creates one identity for
                    // both arms and the suffix, not two arm-private identities.
                    if rename_join(then_body, else_body, &mut visible, collisions, reserved) {
                        continue;
                    }
                    pending.push((else_body, visible.clone()));
                    pending.push((then_body, visible.clone()));
                }
                NirStmt::While { condition, body } => {
                    rename_expr(condition, &visible);
                    pending.push((body, visible.clone()));
                }
                NirStmt::Return(Some(value)) | NirStmt::Expr(value) | NirStmt::Print(value) => {
                    rename_expr(value, &visible)
                }
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
                _ => unreachable!("admitted normalized scalar body"),
            }
        }
    }
}

fn rename_join(
    then_body: &mut [NirStmt],
    else_body: &mut [NirStmt],
    visible: &mut BTreeMap<String, String>,
    collisions: &BTreeSet<String>,
    reserved: &mut BTreeSet<String>,
) -> bool {
    let [yes_prefix @ .., NirStmt::Let {
        name,
        ty: Some(ty),
        value: yes,
    }] = then_body
    else {
        return false;
    };
    let [no_prefix @ .., NirStmt::Let {
        name: other,
        ty: Some(other_ty),
        value: no,
    }] = else_body
    else {
        return false;
    };
    if name != other
        || ty != other_ty
        || visible.contains_key(name)
        || yes_prefix
            .iter()
            .chain(no_prefix.iter())
            .any(|s| !matches!(s, NirStmt::Let { .. } | NirStmt::Const { .. }))
    {
        return false;
    }
    for (prefix, result) in [(yes_prefix, yes), (no_prefix, no)] {
        let mut inner = visible.clone();
        for stmt in prefix {
            let (name, value) = match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    (name, value)
                }
                _ => unreachable!("validated straight prefix"),
            };
            rename_binding(name, value, &mut inner, collisions, reserved);
        }
        rename_expr(result, &inner);
    }
    let replacement = if collisions.contains(name) {
        fresh_name("__nuis_scalar_local", reserved)
    } else {
        name.clone()
    };
    visible.insert(name.clone(), replacement.clone());
    *name = replacement.clone();
    *other = replacement;
    true
}

fn rename_binding(
    name: &mut String,
    value: &mut NirExpr,
    visible: &mut BTreeMap<String, String>,
    collisions: &BTreeSet<String>,
    reserved: &mut BTreeSet<String>,
) {
    rename_expr(value, visible);
    // Loop writes retain identity; a fresh initializer sees the preceding scope.
    let replacement = visible.entry(name.clone()).or_insert_with(|| {
        if collisions.contains(name) {
            fresh_name("__nuis_scalar_local", reserved)
        } else {
            name.clone()
        }
    });
    *name = replacement.clone();
}

fn rename_expr(expr: &mut NirExpr, visible: &BTreeMap<String, String>) {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        match expr {
            NirExpr::Var(name) => {
                if let Some(replacement) = visible.get(name) {
                    *name = replacement.clone();
                }
            }
            NirExpr::Binary { lhs, rhs, .. } => {
                pending.push(rhs);
                pending.push(lhs);
            }
            // Function symbols, nominal types and field labels are not locals.
            NirExpr::Call { args, .. } => pending.extend(args),
            NirExpr::StructLiteral { fields, .. } => {
                pending.extend(fields.iter_mut().map(|(_, value)| value));
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::CastBoolToI64(base)
            | NirExpr::CastI64ToBool(base) => pending.push(base),
            NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::F32(_) | NirExpr::F64(_) => {}
            _ => unreachable!("admitted normalized scalar expression"),
        }
    }
}

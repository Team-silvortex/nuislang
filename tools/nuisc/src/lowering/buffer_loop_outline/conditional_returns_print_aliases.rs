use super::*;

struct Arm<'a> {
    body: &'a [NirStmt],
    end: usize,
    aliases: BTreeMap<String, &'a NirExpr>,
}

pub(in super::super) fn prepare(
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<(Vec<NirStmt>, Vec<NirStmt>)> {
    let yes = prepare_arm(then_body, result, scope, catalog, layouts)?;
    let no = prepare_arm(else_body, result, scope, catalog, layouts)?;
    if yes.aliases.is_empty() && no.aliases.is_empty() {
        return None;
    }
    // Both original lexical scopes and expanded budgets are proven before
    // removing copies. A later veto still leaves the source module untouched.
    Some((materialize(yes), materialize(no)))
}

fn prepare_arm<'a>(
    body: &'a [NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Arm<'a>> {
    let end = body
        .iter()
        .rposition(|stmt| matches!(stmt, NirStmt::Print(_)))
        .map_or(0, |index| index + 1);
    if !preflight(body, end) {
        return None;
    }
    let roots = body[..end]
        .iter()
        .map(|stmt| match stmt {
            NirStmt::Print(value) | NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
                Some(value)
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    // Every removed alias still consumes its original statement and node.
    if !suffix::reserve_prefix(&body[end..], &roots) {
        return None;
    }
    let mut scope = scope.clone();
    let mut aliases = BTreeMap::<String, &NirExpr>::new();
    for stmt in &body[..end] {
        match stmt {
            NirStmt::Print(value) => {
                print_values::prepare(value, &scope, catalog, layouts)?;
            }
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if scope.contains_key(name)
                    || !matches!(value, NirExpr::Var(_) | NirExpr::Int(_) | NirExpr::Bool(_))
                {
                    return None;
                }
                let ty = control_values::value_type(value, &scope, catalog, layouts)?;
                let declared = match stmt {
                    NirStmt::Let { ty, .. } => ty.as_ref(),
                    NirStmt::Const { ty, .. } => Some(ty),
                    _ => unreachable!(),
                };
                if !scalar(&ty) || declared.is_some_and(|declared| declared != &ty) {
                    return None;
                }
                let atom = match value {
                    NirExpr::Var(name) => aliases.get(name).copied().unwrap_or(value),
                    _ => value,
                };
                aliases.insert(name.clone(), atom);
                scope.insert(name.clone(), ty);
            }
            _ => unreachable!(),
        }
    }
    // Validate before substitution: erasing a name must not conceal a later
    // shadow, use-before-definition, branch-local leak or unreachable suffix.
    suffix::validate(&body[end..], scope, result, catalog, layouts)?;
    Some(Arm { body, end, aliases })
}

fn materialize(arm: Arm<'_>) -> Vec<NirStmt> {
    let mut body = arm.body[..arm.end]
        .iter()
        .filter(|stmt| matches!(stmt, NirStmt::Print(_)))
        .chain(arm.body[arm.end..].iter())
        .cloned()
        .collect::<Vec<_>>();
    rewrite(&mut body, &arm.aliases);
    body
}

pub(super) fn rewrite(body: &mut [NirStmt], aliases: &BTreeMap<String, &NirExpr>) {
    let mut blocks = vec![body];
    let mut expressions = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Print(value)
                | NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Return(Some(value)) => expressions.push(value),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push(condition);
                    blocks.extend([then_body.as_mut_slice(), else_body.as_mut_slice()]);
                }
                _ => unreachable!("print alias statement was preflighted"),
            }
        }
    }
    rewrite_expressions(expressions, aliases);
}

pub(super) fn rewrite_expr(expr: &mut NirExpr, aliases: &BTreeMap<String, &NirExpr>) {
    rewrite_expressions(vec![expr], aliases);
}

fn rewrite_expressions(mut expressions: Vec<&mut NirExpr>, aliases: &BTreeMap<String, &NirExpr>) {
    while let Some(expr) = expressions.pop() {
        if let NirExpr::Var(name) = expr {
            if let Some(atom) = aliases.get(name) {
                *expr = (*atom).clone();
            }
        }
        match expr {
            NirExpr::Binary { lhs, rhs, .. } => expressions.extend([lhs.as_mut(), rhs.as_mut()]),
            NirExpr::Call { args, .. } => expressions.extend(args),
            NirExpr::StructLiteral { fields, .. } => {
                expressions.extend(fields.iter_mut().map(|(_, value)| value));
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::PackF32Word(base)
            | NirExpr::UnpackF32Word(base)
            | NirExpr::PackF64Word(base)
            | NirExpr::UnpackF64Word(base) => expressions.push(base),
            NirExpr::Var(_)
            | NirExpr::Int(_)
            | NirExpr::Bool(_)
            | NirExpr::F32(_)
            | NirExpr::F64(_) => {}
            _ => unreachable!("print alias expression was preflighted"),
        }
    }
}

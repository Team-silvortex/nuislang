use super::*;

pub(super) const MAX_EFFECT_STORAGE_CANDIDATES: usize = 8;

// A selected return evacuates every enclosing loop before any source suffix
// executes. Its complete payload may replace an existing mutable record, but
// only after evaluation succeeds. Scalars can also be induction/header state;
// keep those and all parameter/constant/branch-local storage independent.
pub(super) fn candidate(
    function: &NirFunction,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
) -> Option<String> {
    candidate_with_prints(
        function,
        layouts,
        catalog,
        validation::LiteralPrints::Reject,
    )?
    .into_iter()
    .next()
}

pub(super) fn candidate_effects(
    function: &NirFunction,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
) -> Option<Vec<String>> {
    let mut candidates =
        candidate_with_prints(function, layouts, catalog, validation::LiteralPrints::Allow)?;
    candidates.truncate(MAX_EFFECT_STORAGE_CANDIDATES);
    Some(candidates)
}

fn candidate_with_prints(
    function: &NirFunction,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    literal_prints: validation::LiteralPrints,
) -> Option<Vec<String>> {
    let result = function.return_type.as_ref()?;
    layouts.get(&result.name)?;
    let mut scope = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<Scope>();
    let mut candidates = Vec::new();
    for stmt in &function.body {
        let (name, value, mutable) = match stmt {
            NirStmt::Let { name, value, .. } => (name, value, true),
            NirStmt::Const { name, value, .. } => (name, value, false),
            // Scan only; the rewrite must retain each print in its source slot.
            NirStmt::Print(NirExpr::Int(_))
                if literal_prints == validation::LiteralPrints::Allow =>
            {
                continue
            }
            _ => break,
        };
        let ty = control_values::value_type(value, &scope, catalog, layouts)?;
        if mutable && ty == *result && !scope.contains_key(name) {
            candidates.push(name.clone());
        }
        scope.insert(name.clone(), ty);
    }
    // Reuse an actual loop carry, not an unrelated record which would merely
    // move the extra storage. Lexical order makes the bounded choice stable.
    let mut writes = BTreeSet::new();
    let mut blocks = vec![function.body.as_slice()];
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::While { body, .. } => writes.extend(sequences::carry_names(body)),
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => blocks.extend([then_body.as_slice(), else_body.as_slice()]),
                _ => {}
            }
        }
    }
    candidates.retain(|name| writes.contains(name));
    (!candidates.is_empty()).then_some(candidates)
}

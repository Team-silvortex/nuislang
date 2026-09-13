use super::*;

pub(super) fn contains_loop(body: &[NirStmt]) -> bool {
    body.iter().any(|stmt| match stmt {
        NirStmt::While { .. } => true,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => contains_loop(then_body) || contains_loop(else_body),
        _ => false,
    })
}

// This catalog checks source shape, not termination. The existing native loop
// preflight still proves the selected invocation's finite, non-wrapping bound.
pub(super) fn validate(
    condition: &NirExpr,
    body: &[NirStmt],
    scope: &Scope,
    loop_bindings: &BTreeSet<String>,
) -> Option<()> {
    let (first @ NirStmt::Let { name, .. }, tail) = body.split_first()? else {
        return None;
    };
    let prepared = prepare_counted_while(
        condition,
        std::slice::from_ref(first),
        &BTreeSet::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )?;
    if name != &prepared.binding_name {
        return None;
    }
    let mut updates = BTreeSet::new();
    for stmt in body {
        let NirStmt::Let { name, ty, value } = stmt else {
            return None;
        };
        if !loop_bindings.contains(name)
            || scope.get(name)? != &scalar_type("i64")
            || ty.as_ref().is_some_and(|ty| ty != &scalar_type("i64"))
            || !updates.insert(name.clone())
            || !nonfallible_i64(value, scope)
        {
            return None;
        }
    }
    // Captured atoms are invariant. No body-only calls or fallible stride
    // expressions may be hoisted into the preflight, including on zero trips.
    for input in [&prepared.limit, &prepared.step] {
        match input {
            NirExpr::Int(_) => {}
            NirExpr::Var(input)
                if !updates.contains(input) && scope.get(input)? == &scalar_type("i64") => {}
            _ => return None,
        }
    }
    if !tail.is_empty() {
        // Share ordered carry interpretation with ordinary lowering. No prefix
        // or update can silently disappear into the counter-only fallback.
        let chained = prepare_chained_while(
            condition,
            body,
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )?;
        if chained.carries.len() != tail.len()
            || !chained.carries.iter().zip(tail).all(|(carry, stmt)| {
                matches!(stmt, NirStmt::Let { name, .. }
                    if name == &carry.binding_name
                        && matches!(carry.kind, PreparedCarryUpdateKind::Linear { .. }))
            })
        {
            return None;
        }
    }
    Some(())
}

fn nonfallible_i64(value: &NirExpr, scope: &Scope) -> bool {
    match value {
        NirExpr::Int(_) => true,
        NirExpr::Var(name) => scope.get(name) == Some(&scalar_type("i64")),
        NirExpr::Binary {
            op: NirBinaryOp::Add | NirBinaryOp::Sub | NirBinaryOp::Mul,
            lhs,
            rhs,
        } => nonfallible_i64(lhs, scope) && nonfallible_i64(rhs, scope),
        _ => false,
    }
}

#[cfg(test)]
#[path = "control_loops/carries_tests.rs"]
mod carries_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parse_nuis_module;

    const SOURCE: &str = "mod cpu Main {
      struct Pair { value: i64, seed: i64 }
      fn walk(initial: i64, limit: i64, stride: i64) -> i64 {
        let index: i64 = initial;
        while index < limit { let index: i64 = index + stride; }
        return index;
      }
      fn wrap(a: i64, b: i64, step: i64) -> i64 { return walk(a, b, step); }
      fn choose(flag: bool, a: i64, b: i64, step: i64) -> Pair {
        if flag { return Pair { value: wrap(a, b, step), seed: a }; }
        return Pair { value: a, seed: b };
      }
      fn main() -> i64 { return 0; }
    }";

    #[test]
    fn counted_control_catalog_is_transitive_without_widening_buffer_admission() {
        let mut module = parse_nuis_module(SOURCE).unwrap();
        let layouts = control_values::layouts(&module);
        let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
        for name in ["walk", "wrap", "choose"] {
            assert!(catalog[name].may_loop);
        }
        assert!(!catalog["main"].may_loop);
        assert_eq!(
            scalar_helpers::collect(&module).keys().collect::<Vec<_>>(),
            ["main"]
        );
        module.functions.reverse();
        let reversed = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert_eq!(
            catalog.keys().collect::<Vec<_>>(),
            reversed.keys().collect::<Vec<_>>()
        );
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(outlined.functions.contains("walk"));
        assert!(outlined.functions.contains("choose"));
        assert_eq!(outlined.guarded_functions.len(), 2);
    }

    #[test]
    fn counted_control_catalog_rejects_mutation_type_effect_and_invariant_drift() {
        for (from, to) in [
            ("index < limit", "index < index"),
            ("index + stride", "index + index"),
            ("index + stride", "index + (stride / limit)"),
            ("let index: i64 = initial", "const index: i64 = initial"),
            (
                "let index: i64 = index + stride;",
                "let index: i64 = index + stride; print(index);",
            ),
            (
                "let index: i64 = index + stride;",
                "let index: i64 = index + stride; let limit: i64 = limit + 1;",
            ),
            (
                "let index: i64 = index + stride;",
                "let index: i64 = wrap(index, limit, stride);",
            ),
            ("return index;", "let index: i64 = initial; return index;"),
            (
                "let index: i64 = index + stride;",
                "if index > 0 { break; } let index: i64 = index + stride;",
            ),
        ] {
            let module = parse_nuis_module(&SOURCE.replace(from, to)).unwrap();
            let catalog =
                scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["walk", "wrap", "choose"] {
                assert!(!catalog.contains_key(name), "{name}: {to}");
            }
        }
        let mut module = parse_nuis_module(SOURCE).unwrap();
        module
            .functions
            .iter_mut()
            .find(|f| f.name == "walk")
            .unwrap()
            .params[2]
            .ty = scalar_type("bool");
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"));
        assert!(!catalog.contains_key("choose"));
    }
}

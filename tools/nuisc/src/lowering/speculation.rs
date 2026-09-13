use super::*;

// Checked arithmetic is distinct from side-effect purity. Other runtime
// families retain their existing branch contracts rather than being reclassified.
pub(super) fn collect_checked_arithmetic(module: &NirModule) -> BTreeSet<String> {
    let mut callers = BTreeMap::<String, Vec<String>>::new();
    let mut pending = BTreeSet::new();
    for function in &module.functions {
        let (checked, calls) = scan(&function.body);
        if checked {
            pending.insert(function.name.clone());
        }
        for call in calls {
            callers.entry(call).or_default().push(function.name.clone());
        }
    }
    let mut checked = BTreeSet::new();
    while let Some(name) = pending.pop_first() {
        if checked.insert(name.clone()) {
            pending.extend(callers.get(&name).into_iter().flatten().cloned());
        }
    }
    checked
}

pub(super) fn block_has_checked_arithmetic(body: &[NirStmt], helpers: &BTreeSet<String>) -> bool {
    let (checked, calls) = scan(body);
    checked || calls.iter().any(|name| helpers.contains(name))
}

fn scan(body: &[NirStmt]) -> (bool, BTreeSet<String>) {
    enum Item<'a> {
        Stmt(&'a NirStmt),
        Expr(&'a NirExpr),
    }
    let mut pending = body.iter().map(Item::Stmt).collect::<Vec<_>>();
    let mut calls = BTreeSet::new();
    let mut checked = false;
    while let Some(item) = pending.pop() {
        match item {
            Item::Stmt(
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Expr(value)
                | NirStmt::Return(Some(value))
                | NirStmt::Print(value)
                | NirStmt::Await(value),
            ) => pending.push(Item::Expr(value)),
            Item::Stmt(NirStmt::If {
                condition,
                then_body,
                else_body,
            }) => {
                pending.push(Item::Expr(condition));
                pending.extend(then_body.iter().chain(else_body).map(Item::Stmt));
            }
            Item::Stmt(NirStmt::While { condition, body }) => {
                pending.push(Item::Expr(condition));
                pending.extend(body.iter().map(Item::Stmt));
            }
            Item::Expr(NirExpr::Binary { op, lhs, rhs }) => {
                checked |= matches!(op, NirBinaryOp::Div | NirBinaryOp::Rem);
                pending.extend([Item::Expr(lhs), Item::Expr(rhs)]);
            }
            Item::Expr(NirExpr::Call { callee, args }) => {
                calls.insert(callee.clone());
                pending.extend(args.iter().map(Item::Expr));
            }
            Item::Expr(NirExpr::StructLiteral { fields, .. }) => {
                pending.extend(fields.iter().map(|(_, value)| Item::Expr(value)));
            }
            Item::Expr(
                NirExpr::FieldAccess { base, .. }
                | NirExpr::VariantIs { base, .. }
                | NirExpr::VariantFieldAccess { base, .. }
                | NirExpr::Await(base)
                | NirExpr::CastI64ToI32(base)
                | NirExpr::CastI32ToI64(base)
                | NirExpr::CastI64ToBool(base)
                | NirExpr::CastBoolToI64(base)
                | NirExpr::CastI64ToF32(base)
                | NirExpr::CastI64ToF64(base)
                | NirExpr::CastF32ToI64(base)
                | NirExpr::CastF64ToI64(base),
            ) => pending.push(Item::Expr(base)),
            _ => {}
        }
    }
    (checked, calls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parse_nuis_module;

    #[test]
    fn purity_does_not_make_transitive_checked_arithmetic_speculatable() {
        let module = parse_nuis_module(
            "mod cpu Main {
          struct Pair { value: i64 }
          fn pure(a: i64) -> i64 { return a + 1; }
          fn divide(a: i64, b: i64) -> i64 { return a / b; }
          fn unused(a: i64, b: i64) -> i64 { let x: i64 = divide(a, b); return 0; }
          fn record(a: i64, b: i64) -> Pair { return Pair { value: unused(a, b) }; }
          fn remainder(a: i64, b: i64) -> i64 { return a % b; }
          fn a(v: i64) -> i64 { return b(v); }
          fn b(v: i64) -> i64 { let x: i64 = divide(v, v); return a(v); }
          fn main() -> i64 { return 0; }
        }",
        )
        .unwrap();
        let pure = collect_pure_helper_functions(&module);
        assert!(pure.contains("divide"));
        let checked = collect_checked_arithmetic(&module);
        assert!(!checked.contains("pure"));
        for name in ["divide", "unused", "record", "remainder", "a", "b"] {
            assert!(checked.contains(name), "{name}");
        }
        let mut reordered = module.clone();
        reordered.functions.reverse();
        assert_eq!(checked, collect_checked_arithmetic(&reordered));
    }
}

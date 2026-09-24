use super::*;

#[cfg(test)]
#[path = "capture_scoped_tests.rs"]
mod tests;

// Projection precedes scoped lowering. Keep every loop-written argument whole
// so the driver can still identify induction, carry and break seeds by identity.
pub(super) fn protected_inputs(
    module: &NirModule,
    targets: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<usize>> {
    let mut protected = BTreeMap::<String, BTreeSet<usize>>::new();
    let mut pending = module
        .functions
        .iter()
        .map(|f| f.body.as_slice())
        .collect::<Vec<_>>();
    while let Some(body) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { body, .. } => {
                    let call = match body.first() {
                        Some(NirStmt::Expr(call)) | Some(NirStmt::Let { value: call, .. }) => {
                            Some(call)
                        }
                        _ => None,
                    };
                    if let Some(NirExpr::Call { callee, args }) = call.filter(|expr| {
                        matches!(expr, NirExpr::Call { callee, .. } if targets.contains(callee))
                    }) {
                        let mut written = BTreeSet::new();
                        branches::collect_bindings(body, &mut written);
                        let inputs = protected.entry(callee.clone()).or_default();
                        for (index, arg) in args.iter().enumerate() {
                            if access(arg).is_none_or(|path| written.contains(&path[0])) {
                                inputs.insert(index);
                            }
                        }
                    }
                    pending.push(body);
                }
                _ => {}
            }
        }
    }
    protected
}

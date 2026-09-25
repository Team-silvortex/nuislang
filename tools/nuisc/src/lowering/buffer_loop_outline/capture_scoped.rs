use super::*;

#[cfg(test)]
#[path = "capture_scoped_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "capture_scoped_carries_tests.rs"]
mod carry_tests;

#[derive(Default)]
pub(super) struct Inputs {
    pub(super) protected: BTreeSet<usize>,
    pub(super) carried: BTreeSet<usize>,
}

// Only an exact flat record reconstruction grants field-mapped backedge inputs.
// Other loop-written arguments retain induction/carry/break seed identities.
pub(super) fn protected_inputs(
    module: &NirModule,
    targets: &BTreeSet<String>,
) -> BTreeMap<String, Inputs> {
    let mut protected = BTreeMap::<String, Inputs>::new();
    let functions = module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect::<BTreeMap<_, _>>();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
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
                        let carried = functions
                            .get(callee.as_str())
                            .map(|function| {
                                scoped_loop_lowering::projectable_record_seed_inputs(
                                    &body[0], &body[1..], function, &definitions,
                                )
                            })
                            .unwrap_or_default();
                        for (index, arg) in args.iter().enumerate() {
                            if access(arg).is_none_or(|path| {
                                written.contains(&path[0]) && !carried.contains(&index)
                            }) {
                                inputs.protected.insert(index);
                            }
                        }
                        inputs.carried.extend(carried);
                    }
                    pending.push(body);
                }
                _ => {}
            }
        }
    }
    protected
}

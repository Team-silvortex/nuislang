use super::*;

#[cfg(test)]
#[path = "capture_scoped_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "capture_scoped_carries_tests.rs"]
pub(super) mod carry_tests;

#[cfg(test)]
#[path = "capture_scoped_elision_tests.rs"]
mod elision_tests;

#[derive(Default)]
pub(super) struct Inputs {
    pub(super) protected: BTreeSet<usize>,
    pub(super) carried: BTreeSet<usize>,
    pub(super) elidable: BTreeMap<usize, scoped_loop_lowering::RecordSeed>,
}

// Only an exact typed record reconstruction grants field-mapped backedge inputs.
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
                        let carried = functions
                            .get(callee.as_str())
                            .map(|function| {
                                scoped_loop_lowering::projectable_record_seed_inputs(
                                    &body[0], &body[1..], function, &definitions,
                                )
                            })
                            .unwrap_or_default();
                        let seen = protected.contains_key(callee);
                        let inputs = protected.entry(callee.clone()).or_default();
                        if seen {
                            // Every scoped call must agree on the nominal record
                            // and its complete output slot range before elision.
                            inputs.elidable.retain(|index, seed| carried.get(index) == Some(seed));
                        } else {
                            inputs.elidable = carried.clone();
                        }
                        for (index, arg) in args.iter().enumerate() {
                            if !carried.contains_key(&index)
                                && access(arg).is_none_or(|path| written.contains(&path[0]))
                            {
                                inputs.protected.insert(index);
                            }
                        }
                        inputs.carried.extend(carried.keys());
                    }
                    pending.push(body);
                }
                _ => {}
            }
        }
    }
    protected
}

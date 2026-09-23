use super::*;
use branches::{collect_bindings, fresh_name};
use control_values::collect_inputs as collect_expr_inputs;

#[path = "scalar_control_hygiene.rs"]
mod hygiene;

#[cfg(test)]
#[path = "scalar_control_tests.rs"]
mod tests;

pub(super) fn outline(
    module: &mut NirModule,
    retained: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
) {
    for function in &mut module.functions {
        if !retained.contains(&function.name)
            || !catalog.contains_key(&function.name)
            || !function
                .body
                .iter()
                .any(|stmt| matches!(stmt, NirStmt::If { .. }))
        {
            continue;
        }
        let scope = function
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect::<Scope>();
        let mut bindings = scope.keys().cloned().collect();
        collect_bindings(&function.body, &mut bindings);
        let mut builder = Builder {
            names,
            helpers,
            guarded,
            catalog,
            layouts,
            bindings,
            result: function
                .return_type
                .clone()
                .expect("admitted scalar result"),
        };
        function.body = builder.block(std::mem::take(&mut function.body), scope, None);
    }
}

struct Builder<'a> {
    names: &'a mut BTreeSet<String>,
    helpers: &'a mut Vec<NirFunction>,
    guarded: &'a mut BTreeSet<String>,
    catalog: &'a ScalarHelpers,
    layouts: &'a control_values::FlatLayouts,
    bindings: BTreeSet<String>,
    result: NirTypeRef,
}

impl Builder<'_> {
    fn block(
        &mut self,
        body: Vec<NirStmt>,
        mut scope: Scope,
        continuation: Option<NirExpr>,
    ) -> Vec<NirStmt> {
        let mut output = Vec::new();
        let mut stmts = body.into_iter();
        while let Some(stmt) = stmts.next() {
            match stmt {
                NirStmt::Let {
                    ref name,
                    ref value,
                    ..
                }
                | NirStmt::Const {
                    ref name,
                    ref value,
                    ..
                } => {
                    let ty = control_values::value_type(value, &scope, self.catalog, self.layouts)
                        .expect("admitted scalar binding");
                    scope.insert(name.clone(), ty);
                    output.push(stmt);
                }
                NirStmt::Return(_) => {
                    output.push(stmt);
                    return output;
                }
                // Preserve the loop as a control boundary. Its admitted update
                // changes existing value bindings, not the lexical type scope.
                NirStmt::While { .. } => output.push(stmt),
                NirStmt::If {
                    condition,
                    mut then_body,
                    mut else_body,
                } => {
                    let tail = stmts.collect::<Vec<_>>();
                    let single_use = has_single_continuation_use(&then_body, &else_body);
                    // Move a suffix only into its sole generated use after making
                    // arm-local bindings hygienic. Multi-use work stays shared.
                    let next = match tail.as_slice() {
                        [] => continuation,
                        [NirStmt::Return(Some(value))] if is_atom(value) => Some(value.clone()),
                        [NirStmt::Return(Some(value))] if single_use => Some(value.clone()),
                        _ if single_use => {
                            hygiene::prepare_arms(
                                &mut then_body,
                                &mut else_body,
                                &tail,
                                &scope,
                                &mut self.bindings,
                            );
                            append_single_use_tail(&mut then_body, &mut else_body, tail);
                            continuation
                        }
                        _ => {
                            let body = self.block(tail, scope.clone(), continuation);
                            Some(self.function("__nuis_scalar_continue", body, &scope, false))
                        }
                    };
                    let predicate = fresh_name("__nuis_scalar_condition", &mut self.bindings);
                    output.push(NirStmt::Let {
                        name: predicate.clone(),
                        ty: Some(scalar_type("bool")),
                        value: condition,
                    });
                    scope.insert(predicate.clone(), scalar_type("bool"));
                    let mut values = Vec::new();
                    for (selected, arm) in [(true, then_body), (false, else_body)] {
                        // Existing values need no speculative work or allocation. Keep
                        // calls, projections and constructors behind their original guard.
                        let atom = match arm.as_slice() {
                            [NirStmt::Return(Some(value))] if is_atom(value) => Some(value),
                            [] => next.as_ref().filter(|value| is_atom(value)),
                            _ => None,
                        };
                        if let Some(value) = atom {
                            values.push(value.clone());
                            continue;
                        }
                        let default = control_values::zero_value(&self.result, self.layouts);
                        let mut body = vec![NirStmt::If {
                            condition: NirExpr::Binary {
                                op: NirBinaryOp::Ne,
                                lhs: Box::new(NirExpr::Var(predicate.clone())),
                                rhs: Box::new(NirExpr::Bool(selected)),
                            },
                            then_body: vec![NirStmt::Return(Some(default))],
                            else_body: vec![],
                        }];
                        body.extend(self.block(arm, scope.clone(), next.clone()));
                        let call = self.function("__nuis_scalar_branch", body, &scope, true);
                        let name = fresh_name("__nuis_scalar_result", &mut self.bindings);
                        output.push(NirStmt::Let {
                            name: name.clone(),
                            ty: Some(self.result.clone()),
                            value: call,
                        });
                        values.push(NirExpr::Var(name));
                    }
                    // Remaining calls receive only captured values and return before
                    // unselected work. The select chooses ready atoms or guarded results.
                    output.push(NirStmt::If {
                        condition: NirExpr::Var(predicate),
                        then_body: vec![NirStmt::Return(Some(values.remove(0)))],
                        else_body: vec![NirStmt::Return(Some(values.remove(0)))],
                    });
                    return output;
                }
                _ => unreachable!("only admitted scalar control flow is outlined"),
            }
        }
        output.push(NirStmt::Return(Some(
            continuation.expect("validated fallthrough continuation"),
        )));
        output
    }

    fn function(
        &mut self,
        prefix: &str,
        body: Vec<NirStmt>,
        scope: &Scope,
        guarded: bool,
    ) -> NirExpr {
        let mut inputs = BTreeSet::new();
        collect_inputs(&body, &mut inputs);
        let params = captured_params(inputs, scope);
        let args = params
            .iter()
            .map(|param| NirExpr::Var(param.name.clone()))
            .collect();
        let name = fresh_name(prefix, self.names);
        let mut function = helper(name.clone(), params, body);
        function.return_type = Some(self.result.clone());
        if guarded {
            self.guarded.insert(name.clone());
        }
        self.helpers.push(function);
        NirExpr::Call { callee: name, args }
    }
}

fn is_atom(value: &NirExpr) -> bool {
    matches!(value, NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::Var(_))
}

fn has_single_continuation_use(then_body: &[NirStmt], else_body: &[NirStmt]) -> bool {
    // Count generated uses in block(), not runtime paths. A nonempty suffix
    // is moved once or shared once; terminal fallthroughs can duplicate a value.
    let mut pending = vec![then_body, else_body];
    let mut uses = 0;
    while let Some(body) = pending.pop() {
        let boundary = body
            .iter()
            .enumerate()
            .find(|(_, stmt)| matches!(stmt, NirStmt::If { .. } | NirStmt::Return(_)));
        match boundary {
            Some((
                index,
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                },
            )) => {
                let tail = &body[index + 1..];
                if tail.is_empty() {
                    pending.extend([then_body.as_slice(), else_body.as_slice()]);
                } else {
                    pending.push(tail);
                }
            }
            Some((_, NirStmt::Return(_))) => {}
            None => {
                uses += 1;
                if uses > 1 {
                    return false;
                }
            }
            _ => unreachable!("only branch/return boundaries are selected"),
        }
    }
    uses == 1
}

fn append_single_use_tail(
    then_body: &mut Vec<NirStmt>,
    else_body: &mut Vec<NirStmt>,
    tail: Vec<NirStmt>,
) {
    // Follow the same sharing boundaries as has_single_continuation_use. Moving
    // the owned tail avoids cloning it into returning or mutually exclusive arms.
    let mut pending = vec![(then_body, 0), (else_body, 0)];
    while let Some((body, start)) = pending.pop() {
        let boundary = (start..body.len())
            .find(|&index| matches!(body[index], NirStmt::If { .. } | NirStmt::Return(_)));
        match boundary {
            None => {
                body.extend(tail);
                return;
            }
            Some(index) if matches!(body[index], NirStmt::Return(_)) => {}
            Some(index) if index + 1 < body.len() => pending.push((body, index + 1)),
            Some(index) => {
                let NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } = &mut body[index]
                else {
                    unreachable!("only branch/return boundaries are selected")
                };
                pending.extend([(then_body, 0), (else_body, 0)]);
            }
        }
    }
    unreachable!("validated single continuation use")
}

fn collect_inputs(body: &[NirStmt], inputs: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Return(Some(value)) => collect_expr_inputs(value, inputs),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_expr_inputs(condition, inputs);
                collect_inputs(then_body, inputs);
                collect_inputs(else_body, inputs);
            }
            NirStmt::While { condition, body } => {
                collect_expr_inputs(condition, inputs);
                collect_inputs(body, inputs);
            }
            _ => unreachable!("normalized scalar body"),
        }
    }
}

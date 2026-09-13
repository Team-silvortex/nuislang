use super::*;
use branches::{collect_bindings, fresh_name};

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
                // changes existing i64 bindings, not the lexical type scope.
                NirStmt::While { .. } => output.push(stmt),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let tail = stmts.collect::<Vec<_>>();
                    // Share the suffix instead of copying it into both arms. Nested
                    // fallthroughs therefore generate a linear-size function DAG.
                    let next = if tail.is_empty() {
                        continuation
                    } else {
                        let body = self.block(tail, scope.clone(), continuation);
                        Some(self.function("__nuis_scalar_continue", body, &scope, false))
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
                    // Both calls receive only captured values. The unselected arm
                    // returns before its arguments, arithmetic or nested callees run.
                    // This final select only chooses already-guarded results.
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

fn collect_expr_inputs(expr: &NirExpr, inputs: &mut BTreeSet<String>) {
    match expr {
        NirExpr::Var(name) => {
            inputs.insert(name.clone());
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_expr_inputs(lhs, inputs);
            collect_expr_inputs(rhs, inputs);
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                collect_expr_inputs(arg, inputs);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_expr_inputs(value, inputs);
            }
        }
        NirExpr::FieldAccess { base, .. } => collect_expr_inputs(base, inputs),
        NirExpr::Int(_) | NirExpr::Bool(_) => {}
        _ => unreachable!("normalized scalar expression"),
    }
}

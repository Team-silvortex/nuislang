use super::*;

// Reuse typed carries and scoped break rather than teaching native loop opcodes
// about function returns. Admission and outlining must consume the same rewrite.
pub(in crate::lowering::buffer_loop_outline) fn normalize(
    function: &NirFunction,
    layouts: &control_values::FlatLayouts,
) -> Option<Option<Vec<NirStmt>>> {
    if !contains_loop_return(&function.body) {
        return Some(None);
    }
    let result = function.return_type.as_ref()?;
    if !control_values::supported_type(result, layouts) {
        return None;
    }
    let mut names = function.params.iter().map(|p| p.name.clone()).collect();
    branches::collect_bindings(&function.body, &mut names);
    reserve_references(&function.body, &mut names)?;
    let state = State {
        flag: branches::fresh_name("__nuis_return_pending", &mut names),
        value: branches::fresh_name("__nuis_return_value", &mut names),
        result,
    };
    let (body, _) = state.block(&function.body, false, 0)?;
    let mut output = vec![
        binding(&state.flag, &scalar_type("i64"), NirExpr::Int(0)),
        binding(
            &state.value,
            result,
            control_values::zero_value(result, layouts),
        ),
    ];
    output.extend(body);
    Some(Some(output))
}

fn contains_loop_return(body: &[NirStmt]) -> bool {
    let mut pending = vec![(body, false)];
    while let Some((body, in_loop)) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Return(_) if in_loop => return true,
                NirStmt::While { body, .. } => pending.push((body, true)),
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => pending.extend([
                    (then_body.as_slice(), in_loop),
                    (else_body.as_slice(), in_loop),
                ]),
                _ => {}
            }
        }
    }
    false
}

fn reserve_references(body: &[NirStmt], names: &mut BTreeSet<String>) -> Option<()> {
    // Never turn a malformed NIR free reference into an initialized private local.
    // This runs before value admission, so unsupported expressions fail closed.
    let mut blocks = vec![body];
    let mut values = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Return(Some(value)) => values.push(value),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    values.push(condition);
                    blocks.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { condition, body } => {
                    values.push(condition);
                    blocks.push(body);
                }
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
                _ => return None,
            }
        }
    }
    while let Some(value) = values.pop() {
        match value {
            NirExpr::Var(name) => {
                names.insert(name.clone());
            }
            NirExpr::Binary { lhs, rhs, .. } => values.extend([lhs.as_ref(), rhs.as_ref()]),
            NirExpr::Call { args, .. } => values.extend(args),
            NirExpr::StructLiteral { fields, .. } => {
                values.extend(fields.iter().map(|(_, value)| value))
            }
            NirExpr::FieldAccess { base, .. } => values.push(base),
            NirExpr::Int(_) | NirExpr::Bool(_) => {}
            _ => return None,
        }
    }
    Some(())
}

struct State<'a> {
    flag: String,
    value: String,
    result: &'a NirTypeRef,
}

impl State<'_> {
    fn block(&self, body: &[NirStmt], in_loop: bool, depth: usize) -> Option<(Vec<NirStmt>, bool)> {
        if depth > 32 {
            return None;
        }
        let mut output = Vec::new();
        let mut returns = false;
        for (index, stmt) in body.iter().enumerate() {
            match stmt {
                NirStmt::Return(value) if in_loop => {
                    if index + 1 != body.len() {
                        return None;
                    }
                    // Publish the pending bit only after the selected return
                    // expression has completed, including its fallible callees.
                    output.push(binding(&self.value, self.result, value.clone()?));
                    output.push(binding(&self.flag, &scalar_type("i64"), NirExpr::Int(1)));
                    output.push(NirStmt::Break);
                    returns = true;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let (then_body, left) = self.block(then_body, in_loop, depth + 1)?;
                    let (else_body, right) = self.block(else_body, in_loop, depth + 1)?;
                    returns |= left || right;
                    output.push(NirStmt::If {
                        condition: condition.clone(),
                        then_body,
                        else_body,
                    });
                }
                NirStmt::While { condition, body } => {
                    let iteration = induction::parse(condition, body)?;
                    let (mut effects, child_returns) =
                        self.block(iteration.effects, true, depth + 1)?;
                    if iteration.leading {
                        effects.insert(0, iteration.step.clone());
                    } else {
                        effects.push(iteration.step.clone());
                    }
                    output.push(NirStmt::While {
                        condition: condition.clone(),
                        body: effects,
                    });
                    if child_returns {
                        output.push(self.propagate(in_loop));
                        returns = true;
                    }
                }
                _ => output.push(stmt.clone()),
            }
        }
        Some((output, returns))
    }

    fn propagate(&self, in_loop: bool) -> NirStmt {
        NirStmt::If {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(NirExpr::Var(self.flag.clone())),
                rhs: Box::new(NirExpr::Int(1)),
            },
            then_body: vec![if in_loop {
                NirStmt::Break
            } else {
                NirStmt::Return(Some(NirExpr::Var(self.value.clone())))
            }],
            else_body: vec![],
        }
    }
}

fn binding(name: &str, ty: &NirTypeRef, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.to_owned(),
        ty: Some(ty.clone()),
        value,
    }
}

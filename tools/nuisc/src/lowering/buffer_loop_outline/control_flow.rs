use super::*;

pub(super) struct Normalized {
    pub running: String,
    pub breaking: Option<String>,
    pub effects: Vec<NirStmt>,
}

// Continue returns to the driver's unit step; break suppresses that step entirely.
// A continue is admissible only when its source path already performs that step.
pub(super) fn normalize(
    effects: &[NirStmt],
    scope: &Scope,
    step: &NirStmt,
) -> Option<Option<Normalized>> {
    if !contains_exit(effects, false) {
        return Some(None);
    }
    let mut names = scope.keys().cloned().collect();
    branches::collect_bindings(effects, &mut names);
    let flag = branches::fresh_name("__nuis_buffer_continue", &mut names);
    let breaking = contains_exit(effects, true)
        .then(|| branches::fresh_name("__nuis_buffer_break", &mut names));
    let (mut body, _) = rewrite(effects, step, &flag, breaking.as_deref())?;
    body.insert(0, set_flag(&flag, 1));
    if let Some(breaking) = &breaking {
        body.insert(0, set_flag(breaking, 0));
    }
    Some(Some(Normalized {
        running: flag,
        breaking,
        effects: body,
    }))
}

fn contains_exit(body: &[NirStmt], break_only: bool) -> bool {
    body.iter().any(|stmt| match stmt {
        NirStmt::Break => true,
        NirStmt::Continue => !break_only,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => contains_exit(then_body, break_only) || contains_exit(else_body, break_only),
        // A child loop owns its control scope and is normalized independently.
        _ => false,
    })
}

fn rewrite(
    body: &[NirStmt],
    step: &NirStmt,
    flag: &str,
    breaking: Option<&str>,
) -> Option<(Vec<NirStmt>, bool)> {
    let mut rewritten = Vec::with_capacity(body.len());
    for (index, stmt) in body.iter().enumerate() {
        match stmt {
            NirStmt::Break => {
                if index + 1 != body.len() {
                    return None;
                }
                rewritten.push(set_flag(breaking?, 1));
                rewritten.push(set_flag(flag, 0));
                return Some((rewritten, true));
            }
            NirStmt::Continue => {
                if index + 1 != body.len() || !same_step(rewritten.last()?, step) {
                    return None;
                }
                rewritten.pop();
                rewritten.push(set_flag(flag, 0));
                return Some((rewritten, true));
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let (then_body, then_continues) = rewrite(then_body, step, flag, breaking)?;
                let (else_body, else_continues) = rewrite(else_body, step, flag, breaking)?;
                rewritten.push(NirStmt::If {
                    condition: condition.clone(),
                    then_body,
                    else_body,
                });
                if then_continues || else_continues {
                    // Outline the suffix once. A skipped suffix never evaluates its inputs.
                    let (tail, _) = rewrite(&body[index + 1..], step, flag, breaking)?;
                    if !tail.is_empty() {
                        rewritten.push(NirStmt::If {
                            condition: NirExpr::Binary {
                                op: NirBinaryOp::Eq,
                                lhs: Box::new(NirExpr::Var(flag.to_owned())),
                                rhs: Box::new(NirExpr::Int(1)),
                            },
                            then_body: tail,
                            else_body: vec![],
                        });
                    }
                    return Some((rewritten, true));
                }
            }
            _ => rewritten.push(stmt.clone()),
        }
    }
    Some((rewritten, false))
}

fn same_step(candidate: &NirStmt, step: &NirStmt) -> bool {
    match (candidate, step) {
        (
            NirStmt::Let { name, ty, value },
            NirStmt::Let {
                name: expected,
                value: step,
                ..
            },
        ) => {
            name == expected
                && ty.as_ref().is_none_or(|ty| ty == &scalar_type("i64"))
                && value == step
        }
        _ => false,
    }
}

fn set_flag(name: &str, value: i64) -> NirStmt {
    NirStmt::Let {
        name: name.to_owned(),
        ty: Some(scalar_type("i64")),
        value: NirExpr::Int(value),
    }
}

use super::*;

// Called only after the complete expression has passed its source budget.
pub(super) fn contains_logic(value: &NirExpr) -> bool {
    contains_nested_logic(value, false)
}

pub(super) fn contains_argument_logic(value: &NirExpr) -> bool {
    contains_nested_logic(value, true)
}

fn contains_nested_logic(value: &NirExpr, require_argument: bool) -> bool {
    let mut pending = vec![(value, false)];
    while let Some((value, argument)) = pending.pop() {
        match value {
            NirExpr::Binary { op, lhs, rhs } => {
                if (!require_argument || argument)
                    && matches!(op, NirBinaryOp::And | NirBinaryOp::Or)
                {
                    return true;
                }
                pending.extend([(lhs.as_ref(), argument), (rhs.as_ref(), argument)]);
            }
            NirExpr::Call { args, .. } => pending.extend(args.iter().map(|arg| (arg, true))),
            NirExpr::FieldAccess { base, .. }
            | NirExpr::BytesLen(base)
            | NirExpr::BufferLen(base) => pending.push((base, argument)),
            NirExpr::LoadAt { buffer, index } => {
                pending.extend([(buffer.as_ref(), argument), (index.as_ref(), argument)])
            }
            _ => {}
        }
    }
    false
}

pub(super) fn extract(
    value: &NirExpr,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
) -> (Vec<NirStmt>, NirExpr) {
    let mut builder = Builder {
        template,
        scope,
        live,
        context,
        prefix: vec![],
    };
    let value = builder.eager(value);
    (builder.prefix, value)
}

struct Builder<'a> {
    template: &'a NirFunction,
    scope: &'a [NirParam],
    live: &'a BTreeSet<String>,
    context: &'a mut OutlineContext,
    prefix: Vec<NirStmt>,
}

impl Builder<'_> {
    fn operand(&mut self, value: &NirExpr) -> NirExpr {
        if matches!(value, NirExpr::Var(_) | NirExpr::Int(_) | NirExpr::Bool(_)) {
            return value.clone();
        }
        let kind = predicate_kind(value, self.scope, self.live, &self.context.calls)
            .expect("complete source proof precedes eager operand extraction");
        let value = if logical_parts(value).is_some() {
            // Never walk either child here: requeued declarations let the owned
            // branch outliner select the RHS and preserve the live owner set.
            value.clone()
        } else {
            self.eager(value)
        };
        let name = fresh_left(self.template, self.scope, self.live, self.context);
        self.prefix.push(NirStmt::Let {
            name: name.clone(),
            ty: Some(NirTypeRef {
                name: match kind {
                    Kind::Bool => "bool",
                    Kind::I64 => "i64",
                }
                .into(),
                is_ref: false,
                is_optional: false,
                generic_args: vec![],
            }),
            value,
        });
        NirExpr::Var(name)
    }

    fn eager(&mut self, value: &NirExpr) -> NirExpr {
        match value {
            NirExpr::Call { callee, args } => NirExpr::Call {
                callee: callee.clone(),
                // Freeze preceding reads and trapping values before any later
                // operand branches. Only scalar snapshots cross the join.
                args: args.iter().map(|arg| self.operand(arg)).collect(),
            },
            NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
                base: Box::new(self.eager(base)),
                field: field.clone(),
            },
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op: *op,
                lhs: Box::new(self.operand(lhs)),
                rhs: Box::new(self.operand(rhs)),
            },
            NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
                buffer: buffer.clone(),
                index: Box::new(self.operand(index)),
            },
            _ => value.clone(),
        }
    }
}

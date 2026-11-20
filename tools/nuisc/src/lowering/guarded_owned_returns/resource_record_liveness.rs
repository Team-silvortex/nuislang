use super::*;

#[cfg(test)]
#[path = "resource_record_liveness_tests.rs"]
mod tests;

pub(super) fn prepare(
    value: &NirExpr,
    ty: &NirTypeRef,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> (NirExpr, Vec<NirStmt>, BTreeSet<String>) {
    let mut planner = Planner {
        template,
        scope,
        live,
        context,
        locals,
        branched: false,
        prefix: vec![],
        leaves: BTreeMap::new(),
    };
    let value = planner.visit(value, ty);
    let names = planner
        .leaves
        .values()
        .map(|value| {
            let NirExpr::Var(name) = value else {
                unreachable!()
            };
            name.clone()
        })
        .collect();
    (value, planner.prefix, names)
}

struct Planner<'a> {
    template: &'a NirFunction,
    scope: &'a [NirParam],
    live: &'a BTreeSet<String>,
    context: &'a mut OutlineContext,
    locals: &'a LocalRecords,
    branched: bool,
    prefix: Vec<NirStmt>,
    leaves: BTreeMap<Vec<String>, NirExpr>,
}

impl Planner<'_> {
    // The caller proves the original expression and any expanded constructor shape.
    fn visit(&mut self, value: &NirExpr, ty: &NirTypeRef) -> NirExpr {
        if let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = value
        {
            return NirExpr::StructLiteral {
                type_name: type_name.clone(),
                type_args: type_args.clone(),
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        let ty = self
                            .context
                            .calls
                            .field_type(ty, name)
                            .expect("proved field type");
                        (name.clone(), self.visit(value, &ty))
                    })
                    .collect(),
            };
        }
        if self.locals.value_type(value, &self.context.calls).as_ref() == Some(ty) {
            if self.branched {
                return self.freeze(value, ty, path(value));
            }
        } else if capture_scalar(ty) {
            let kind = if ty.name == "bool" {
                Kind::Bool
            } else {
                Kind::I64
            };
            let proof = operands::for_proof(value, self.locals, &self.context.calls);
            self.branched |= arguments::contains_logic(&proof)
                && admitted(&proof, self.scope, self.live, &self.context.calls, kind);
        } else {
            self.branched |= arguments::contains_argument_logic(value);
        }
        // Calls, resource reads and checked arithmetic stay at their source position.
        self.operands(value)
    }

    fn operands(&mut self, value: &NirExpr) -> NirExpr {
        if let Some(ty) = self
            .locals
            .value_type(value, &self.context.calls)
            .filter(capture_scalar)
        {
            return self.freeze(value, &ty, path(value));
        }
        operands::children(value, |child| self.operands(child))
    }

    fn freeze(&mut self, value: &NirExpr, ty: &NirTypeRef, key: Vec<String>) -> NirExpr {
        if capture_scalar(ty) {
            if let Some(saved) = self.leaves.get(&key) {
                return saved.clone();
            }
            // Only projections of already completed, exact i64/bool records move.
            let saved = snapshot(
                value,
                ty,
                self.template,
                self.scope,
                self.live,
                self.context,
                &mut self.prefix,
            );
            self.leaves.insert(key, saved.clone());
            return saved;
        }
        let fields = self
            .context
            .calls
            .record_fields(ty)
            .expect("proved local layout")
            .to_vec();
        NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: vec![],
            fields: fields
                .into_iter()
                .map(|(name, ty)| {
                    let projected = NirExpr::FieldAccess {
                        base: Box::new(value.clone()),
                        field: name.clone(),
                    };
                    let mut path = key.clone();
                    path.push(name.clone());
                    (name, self.freeze(&projected, &ty, path))
                })
                .collect(),
        }
    }
}

fn path(value: &NirExpr) -> Vec<String> {
    match value {
        NirExpr::Var(name) => vec![name.clone()],
        NirExpr::FieldAccess { base, field } => {
            let mut result = path(base);
            result.push(field.clone());
            result
        }
        _ => unreachable!("local availability only proves variable-rooted projections"),
    }
}

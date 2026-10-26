use super::*;

pub(super) struct Captures {
    pub(super) value: NirExpr,
    pub(super) params: Vec<NirParam>,
    pub(super) args: Vec<NirExpr>,
}

pub(super) fn collect(
    value: &NirExpr,
    scope: &Scope,
    layouts: &control_values::CarryLayouts,
) -> Option<Captures> {
    let mut builder = Builder {
        scope,
        layouts,
        reserved: scope.keys().cloned().collect(),
        paths: BTreeMap::new(),
        params: Vec::new(),
        args: Vec::new(),
        work: 4096,
    };
    let value = builder.value(value, 0)?;
    Some(Captures {
        value,
        params: builder.params,
        args: builder.args,
    })
}

struct Builder<'a> {
    scope: &'a Scope,
    layouts: &'a control_values::CarryLayouts,
    reserved: BTreeSet<String>,
    paths: BTreeMap<Vec<String>, String>,
    params: Vec<NirParam>,
    args: Vec<NirExpr>,
    work: usize,
}

impl Builder<'_> {
    fn value(&mut self, value: &NirExpr, depth: usize) -> Option<NirExpr> {
        if depth > 64 {
            return None;
        }
        self.work = self.work.checked_sub(1)?;
        Some(match value {
            NirExpr::Bool(_) | NirExpr::Int(_) | NirExpr::F32(_) | NirExpr::F64(_) => value.clone(),
            NirExpr::Var(_) | NirExpr::FieldAccess { .. } => {
                let mut base = value;
                let mut path = Vec::new();
                let mut end = depth;
                while let NirExpr::FieldAccess {
                    base: parent,
                    field,
                } = base
                {
                    end = end.checked_add(1)?;
                    if end > 64 {
                        return None;
                    }
                    self.work = self.work.checked_sub(1)?;
                    path.push(field.clone());
                    base = parent;
                }
                let NirExpr::Var(root) = base else {
                    return None;
                };
                path.push(root.clone());
                path.reverse();
                let mut ty = self.scope.get(root)?.clone();
                for field in &path[1..] {
                    if ty != scalar_type(&ty.name) {
                        return None;
                    }
                    let mut fields = self
                        .layouts
                        .get(&ty.name)?
                        .iter()
                        .filter(|(name, _)| name == field);
                    ty = fields.next()?.1.clone();
                    if fields.next().is_some() {
                        return None;
                    }
                }
                if ty != scalar_type(&ty.name)
                    || !matches!(ty.name.as_str(), "bool" | "i32" | "i64" | "f32" | "f64")
                {
                    return None;
                }
                if let Some(name) = self.paths.get(&path) {
                    return Some(NirExpr::Var(name.clone()));
                }
                // The existing private predicate convention reserves one of 32 slots for its gate.
                if self.params.len() >= 31 {
                    return None;
                }
                let name = if path.len() == 1 {
                    root.clone()
                } else {
                    branches::fresh_name("__nuis_predicate_field", &mut self.reserved)
                };
                self.paths.insert(path, name.clone());
                self.params.push(NirParam {
                    name: name.clone(),
                    ty,
                });
                self.args.push(value.clone());
                NirExpr::Var(name)
            }
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op: *op,
                lhs: Box::new(self.value(lhs, depth.checked_add(1)?)?),
                rhs: Box::new(self.value(rhs, depth.checked_add(1)?)?),
            },
            NirExpr::Call { callee, args } => {
                if args.len() > self.work {
                    return None;
                }
                NirExpr::Call {
                    callee: callee.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.value(arg, depth.checked_add(1)?))
                        .collect::<Option<_>>()?,
                }
            }
            NirExpr::PackF32Word(v) => {
                NirExpr::PackF32Word(Box::new(self.value(v, depth.checked_add(1)?)?))
            }
            NirExpr::UnpackF32Word(v) => {
                NirExpr::UnpackF32Word(Box::new(self.value(v, depth.checked_add(1)?)?))
            }
            NirExpr::PackF64Word(v) => {
                NirExpr::PackF64Word(Box::new(self.value(v, depth.checked_add(1)?)?))
            }
            NirExpr::UnpackF64Word(v) => {
                NirExpr::UnpackF64Word(Box::new(self.value(v, depth.checked_add(1)?)?))
            }
            NirExpr::CastI64ToI32(v) => {
                NirExpr::CastI64ToI32(Box::new(self.value(v, depth.checked_add(1)?)?))
            }
            NirExpr::CastI32ToI64(v) => {
                NirExpr::CastI32ToI64(Box::new(self.value(v, depth.checked_add(1)?)?))
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
#[path = "logical_field_captures_tests.rs"]
mod tests;

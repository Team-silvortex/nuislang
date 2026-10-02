use super::*;
use std::ops::Range;
use std::rc::Rc;

#[path = "return_invariant_loops.rs"]
mod loops;

#[path = "return_invariant_snapshots.rs"]
mod snapshots;

#[path = "return_invariant_parent_entries.rs"]
mod parent_entries;

pub(super) use snapshots::Snapshots;

#[cfg(test)]
#[path = "return_invariant_facts_tests.rs"]
mod tests;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Origin {
    Entry(String, Vec<String>),
    Snapshot(usize, usize),
}

#[derive(PartialEq, Eq)]
struct Value {
    ty: NirTypeRef,
    words: Vec<Option<Origin>>,
}

type Env = BTreeMap<String, Rc<Value>>;

struct RecordShape {
    tree: Shape,
    leaves: Vec<(Vec<String>, NirTypeRef)>,
    fields: BTreeMap<String, (NirTypeRef, Range<usize>)>,
}

#[cfg(test)]
pub(super) fn analyze(
    body: &[NirStmt],
    scope: &Scope,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    budget: &mut Budget,
) -> Option<BTreeMap<String, (Shape, Vec<bool>)>> {
    analyze_at(body, scope, layouts, catalog, budget, None)
}

pub(super) fn analyze_at(
    body: &[NirStmt],
    scope: &Scope,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    budget: &mut Budget,
    snapshots: Option<&Snapshots>,
) -> Option<BTreeMap<String, (Shape, Vec<bool>)>> {
    let mut proof = Proof::new(layouts, catalog, budget);
    let writes = sequences::carry_names(body)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut env = Env::new();
    for (name, ty) in scope {
        let shape = proof.shape(ty)?;
        let paths = &shape.leaves;
        proof.budget.charge(paths.len())?;
        if !shape.tree.fields.is_empty() && writes.contains(name) {
            proof
                .stable
                .insert(name.clone(), (shape.tree.clone(), vec![true; paths.len()]));
        }
        let value = if let Some(value) = snapshots.and_then(|s| s.env.get(name)) {
            if value.ty != *ty || value.words.len() != paths.len() {
                return None;
            }
            Rc::clone(value)
        } else {
            Rc::new(Value {
                ty: ty.clone(),
                words: paths
                    .iter()
                    .map(|(path, _)| Some(Origin::Entry(name.clone(), path.clone())))
                    .collect(),
            })
        };
        env.insert(name.clone(), value);
    }
    proof.budget.charge(env.len())?;
    proof.entries = env.clone();
    if snapshots.is_some() {
        // Entry aliases can agree on the first trip and diverge on a later one.
        // Check their fixed point as well as each intermediate publication.
        proof.loop_summary(&NirExpr::Bool(true), body, &mut env, 0)?;
    } else {
        proof.block(body, &mut env, 0)?;
    }
    proof
        .stable
        .retain(|_, (_, fields)| fields.iter().any(|stable| *stable));
    Some(proof.stable)
}

struct Proof<'a> {
    layouts: &'a control_values::CarryLayouts,
    catalog: &'a ScalarHelpers,
    budget: &'a mut Budget,
    shapes: BTreeMap<String, Rc<RecordShape>>,
    stable: BTreeMap<String, (Shape, Vec<bool>)>,
    entries: Env,
    observations: Vec<Env>,
}

impl Proof<'_> {
    fn new<'a>(
        layouts: &'a control_values::CarryLayouts,
        catalog: &'a ScalarHelpers,
        budget: &'a mut Budget,
    ) -> Proof<'a> {
        Proof {
            layouts,
            catalog,
            budget,
            shapes: BTreeMap::new(),
            stable: BTreeMap::new(),
            entries: Env::new(),
            observations: Vec::new(),
        }
    }

    fn shape(&mut self, ty: &NirTypeRef) -> Option<Rc<RecordShape>> {
        self.budget.charge(1)?;
        if !self.shapes.contains_key(&ty.name) {
            if self
                .layouts
                .get(&ty.name)
                .is_some_and(|fields| fields.len() > 64)
            {
                return None;
            }
            let shape = Shape::from_layouts(ty, self.layouts)?;
            let leaves = shape.leaves();
            self.budget
                .charge(leaves.iter().map(|(path, _)| path.len() + 1).sum())?;
            if leaves.len() > 64 {
                return None;
            }
            self.budget.charge(shape.fields.len())?;
            let mut start = 0;
            let fields = shape
                .fields
                .iter()
                .map(|(name, child)| {
                    let end = start + child.leaves().len();
                    let field = (name.clone(), (child.ty.clone(), start..end));
                    start = end;
                    field
                })
                .collect();
            self.shapes.insert(
                ty.name.clone(),
                Rc::new(RecordShape {
                    tree: shape,
                    leaves,
                    fields,
                }),
            );
        }
        let shape = self.shapes.get(&ty.name)?;
        if shape.tree.ty != *ty {
            return None;
        }
        Some(Rc::clone(shape))
    }

    fn unknown(&mut self, ty: NirTypeRef) -> Option<Rc<Value>> {
        let width = self.shape(&ty)?.leaves.len();
        self.budget.charge(width)?;
        Some(Rc::new(Value {
            ty,
            words: vec![None; width],
        }))
    }

    fn block(&mut self, body: &[NirStmt], env: &mut Env, depth: usize) -> Option<()> {
        for (index, stmt) in body.iter().enumerate() {
            self.budget.tick(depth)?;
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    let value = self.expression(value, env, depth + 1)?;
                    if ty.as_ref().is_some_and(|ty| ty != &value.ty)
                        || env.get(name).is_some_and(|old| old.ty != value.ty)
                    {
                        return None;
                    }
                    if let Some((shape, stable)) = self.stable.get_mut(name) {
                        let paths = shape.leaves();
                        if value.words.len() != paths.len() {
                            return None;
                        }
                        self.budget.charge(paths.len())?;
                        let entry = self.entries.get(name)?;
                        for (index, _) in paths.into_iter().enumerate() {
                            stable[index] &= entry.words[index].is_some()
                                && value.words[index] == entry.words[index];
                        }
                    }
                    self.observe(name, &value)?;
                    env.insert(name.clone(), value);
                }
                NirStmt::Const { name, ty, value } if !env.contains_key(name) => {
                    let value = self.expression(value, env, depth + 1)?;
                    if ty != &value.ty {
                        return None;
                    }
                    env.insert(name.clone(), value);
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    if self.expression(condition, env, depth + 1)?.ty != scalar_type("bool") {
                        return None;
                    }
                    self.budget.charge(env.len().checked_mul(2)?)?;
                    let mut left = env.clone();
                    let mut right = env.clone();
                    self.block(then_body, &mut left, depth + 1)?;
                    self.block(else_body, &mut right, depth + 1)?;
                    for (name, current) in env.iter_mut() {
                        let (left, right) = (left.get(name)?, right.get(name)?);
                        if left.ty != right.ty || left.words.len() != right.words.len() {
                            return None;
                        }
                        self.budget.charge(left.words.len())?;
                        *current = Rc::new(Value {
                            ty: left.ty.clone(),
                            words: left
                                .words
                                .iter()
                                .zip(&right.words)
                                .map(
                                    |(left, right)| {
                                        if left == right {
                                            left.clone()
                                        } else {
                                            None
                                        }
                                    },
                                )
                                .collect(),
                        });
                    }
                }
                NirStmt::While { condition, body } => {
                    self.loop_summary(condition, body, env, depth + 1)?;
                }
                NirStmt::Expr(value) => {
                    self.expression(value, env, depth + 1)?;
                }
                NirStmt::Break | NirStmt::Continue if index + 1 == body.len() => {}
                _ => return None,
            }
        }
        Some(())
    }

    fn expression(&mut self, expr: &NirExpr, env: &Env, depth: usize) -> Option<Rc<Value>> {
        self.budget.tick(depth)?;
        match expr {
            NirExpr::Var(name) => env.get(name).cloned(),
            NirExpr::Int(_) => self.unknown(scalar_type("i64")),
            NirExpr::Bool(_) => self.unknown(scalar_type("bool")),
            NirExpr::F32(_) => self.unknown(scalar_type("f32")),
            NirExpr::F64(_) => self.unknown(scalar_type("f64")),
            NirExpr::FieldAccess { base, field } => {
                let base = self.expression(base, env, depth + 1)?;
                // Reusing a validated layout avoids scanning every sibling for
                // each selected field of a wide constructor. Origins stay fresh.
                let shape = self.shape(&base.ty)?;
                let (ty, range) = shape.fields.get(field)?;
                if base.words.len() != shape.leaves.len() {
                    return None;
                }
                self.budget.charge(range.len())?;
                Some(Rc::new(Value {
                    ty: ty.clone(),
                    words: base.words.get(range.clone())?.to_vec(),
                }))
            }
            NirExpr::StructLiteral {
                type_name,
                type_args,
                fields,
            } if type_args.is_empty() => {
                let declared = self.layouts.get(type_name)?;
                if declared.len() > 64 {
                    return None;
                }
                self.budget.charge(declared.len())?;
                let declared = declared.clone();
                if fields.len() != declared.len() {
                    return None;
                }
                let mut values = BTreeMap::new();
                for (name, expr) in fields {
                    let value = self.expression(expr, env, depth + 1)?;
                    if values.insert(name, value).is_some() {
                        return None;
                    }
                }
                let mut words = Vec::new();
                for (name, ty) in declared {
                    let value = values.remove(&name)?;
                    if value.ty != ty {
                        return None;
                    }
                    self.budget.charge(value.words.len())?;
                    words.extend(value.words.iter().cloned());
                }
                if !values.is_empty() {
                    return None;
                }
                Some(Rc::new(Value {
                    ty: scalar_type(type_name),
                    words,
                }))
            }
            NirExpr::Call { callee, args } => {
                let types = args
                    .iter()
                    .map(|arg| Some(self.expression(arg, env, depth + 1)?.ty.clone()))
                    .collect::<Option<Vec<_>>>()?;
                self.unknown(scalar_helpers::typed_call_type(
                    callee,
                    &types,
                    self.catalog,
                )?)
            }
            NirExpr::Binary { op, lhs, rhs } => {
                let left = self.expression(lhs, env, depth + 1)?;
                let right = self.expression(rhs, env, depth + 1)?;
                if left.ty != right.ty {
                    return None;
                }
                let ty = match op {
                    NirBinaryOp::Add
                    | NirBinaryOp::Sub
                    | NirBinaryOp::Mul
                    | NirBinaryOp::Div
                    | NirBinaryOp::Rem => left.ty.clone(),
                    NirBinaryOp::Eq
                    | NirBinaryOp::Ne
                    | NirBinaryOp::Lt
                    | NirBinaryOp::Le
                    | NirBinaryOp::Gt
                    | NirBinaryOp::Ge
                    | NirBinaryOp::And
                    | NirBinaryOp::Or => scalar_type("bool"),
                    _ => return None,
                };
                self.unknown(ty)
            }
            NirExpr::CastI64ToI32(value)
            | NirExpr::CastI32ToI64(value)
            | NirExpr::PackF32Word(value)
            | NirExpr::UnpackF32Word(value)
            | NirExpr::PackF64Word(value)
            | NirExpr::UnpackF64Word(value) => {
                self.expression(value, env, depth + 1)?;
                let name = match expr {
                    NirExpr::CastI64ToI32(_) => "i32",
                    NirExpr::UnpackF32Word(_) => "f32",
                    NirExpr::UnpackF64Word(_) => "f64",
                    _ => "i64",
                };
                self.unknown(scalar_type(name))
            }
            _ => None,
        }
    }
}

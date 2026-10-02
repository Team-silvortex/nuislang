use super::*;

#[cfg(test)]
#[path = "return_invariant_snapshots_tests.rs"]
mod tests;

#[derive(Clone, Default)]
pub(in super::super) struct Snapshots {
    pub(super) env: Env,
}

impl Snapshots {
    pub(in super::super) fn new(
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
        catalog: &ScalarHelpers,
        budget: &mut Budget,
    ) -> Option<Self> {
        let mut proof = Proof::new(layouts, catalog, budget);
        let mut env = Env::new();
        for (name, ty) in scope {
            let shape = proof.shape(ty)?;
            proof.budget.charge(shape.leaves.len())?;
            env.insert(
                name.clone(),
                Rc::new(Value {
                    ty: ty.clone(),
                    words: shape
                        .leaves
                        .iter()
                        .map(|(path, _)| Some(Origin::Entry(name.clone(), path.clone())))
                        .collect(),
                }),
            );
        }
        Some(Self { env })
    }

    pub(in super::super) fn bind(
        &mut self,
        stmt: &NirStmt,
        layouts: &control_values::CarryLayouts,
        catalog: &ScalarHelpers,
        budget: &mut Budget,
        next: &mut usize,
        depth: usize,
    ) -> Option<()> {
        let (name, declared, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } if !self.env.contains_key(name) => {
                (name, Some(ty), value)
            }
            _ => return None,
        };
        let value = Proof::new(layouts, catalog, budget).expression(value, &self.env, depth)?;
        if declared.is_some_and(|ty| ty != &value.ty)
            || self.env.get(name).is_some_and(|old| old.ty != value.ty)
        {
            return None;
        }
        budget.charge(value.words.len() + 1)?;
        let version = *next;
        *next = next.checked_add(1)?;
        // Unknown computation becomes identifiable only after evaluation into
        // this snapshot. Each assignment gets a distinct version, never a lens.
        let words = value
            .words
            .iter()
            .enumerate()
            .map(|(index, origin)| origin.clone().or(Some(Origin::Snapshot(version, index))))
            .collect();
        self.env.insert(
            name.clone(),
            Rc::new(Value {
                ty: value.ty.clone(),
                words,
            }),
        );
        Some(())
    }

    pub(in super::super) fn forget_writes(
        &mut self,
        body: &[NirStmt],
        budget: &mut Budget,
        next: &mut usize,
        depth: usize,
    ) -> Option<()> {
        let mut writes = BTreeSet::new();
        Self::writes(body, &mut writes, budget, depth)?;
        for name in writes {
            if let Some(old) = self.env.get_mut(&name) {
                budget.charge(old.words.len() + 1)?;
                let version = *next;
                *next = next.checked_add(1)?;
                *old = Rc::new(Value {
                    ty: old.ty.clone(),
                    words: (0..old.words.len())
                        .map(|index| Some(Origin::Snapshot(version, index)))
                        .collect(),
                });
            }
        }
        Some(())
    }

    pub(in super::super) fn join(
        &mut self,
        left: &Self,
        right: &Self,
        budget: &mut Budget,
        next: &mut usize,
    ) -> Option<()> {
        budget.charge(self.env.len() + 1)?;
        let version = *next;
        *next = next.checked_add(1)?;
        let mut pairs = BTreeMap::new();
        let mut slots = 0;
        let mut joined = Env::new();
        for (name, entry) in &self.env {
            let (left, right) = (left.env.get(name)?, right.env.get(name)?);
            if left.ty != entry.ty
                || right.ty != entry.ty
                || left.words.len() != entry.words.len()
                || right.words.len() != entry.words.len()
            {
                return None;
            }
            budget.charge(entry.words.len())?;
            let mut words = Vec::new();
            for (left, right) in left.words.iter().zip(&right.words) {
                let origin = match (left, right) {
                    (Some(left), Some(right)) if left == right => left.clone(),
                    (Some(left), Some(right)) => {
                        // The same ordered pair means equality on either arm,
                        // not equality between the arms or separate evaluations.
                        let slot =
                            *pairs
                                .entry((left.clone(), right.clone()))
                                .or_insert_with(|| {
                                    let slot = slots;
                                    slots += 1;
                                    slot
                                });
                        Origin::Snapshot(version, slot)
                    }
                    _ => {
                        let slot = slots;
                        slots += 1;
                        Origin::Snapshot(version, slot)
                    }
                };
                words.push(Some(origin));
            }
            joined.insert(
                name.clone(),
                Rc::new(Value {
                    ty: entry.ty.clone(),
                    words,
                }),
            );
        }
        self.env = joined;
        Some(())
    }

    fn writes(
        body: &[NirStmt],
        writes: &mut BTreeSet<String>,
        budget: &mut Budget,
        depth: usize,
    ) -> Option<()> {
        for stmt in body {
            budget.tick(depth)?;
            match stmt {
                NirStmt::Let { name, .. } => {
                    writes.insert(name.clone());
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    Self::writes(then_body, writes, budget, depth + 1)?;
                    Self::writes(else_body, writes, budget, depth + 1)?;
                }
                NirStmt::While { body, .. } => Self::writes(body, writes, budget, depth + 1)?,
                _ => {}
            }
        }
        Some(())
    }
}

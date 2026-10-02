use super::*;

#[cfg(test)]
#[path = "return_invariant_parent_entries_tests.rs"]
mod tests;

impl Snapshots {
    pub(in super::super) fn parent_entries(
        &self,
        body: &[NirStmt],
        layouts: &control_values::CarryLayouts,
        catalog: &ScalarHelpers,
        budget: &mut Budget,
        next: &mut usize,
        depth: usize,
    ) -> Option<Self> {
        let mut proof = Proof::new(layouts, catalog, budget);
        proof.budget.charge(self.env.len())?;
        for value in self.env.values() {
            if proof.shape(&value.ty)?.leaves.len() != value.words.len() {
                return None;
            }
        }
        let mut env = self.env.clone();
        // A child may run after any intermediate publication on any parent
        // trip. The summary includes zero trips and every write, not just exits.
        proof.loop_summary(&NirExpr::Bool(true), body, &mut env, depth)?;
        let budget = proof.budget;
        for value in env.values_mut() {
            budget.charge(value.words.len() + 1)?;
            if value.words.iter().all(Option::is_some) {
                continue;
            }
            let version = *next;
            *next = next.checked_add(1)?;
            // Varying fields get independent entry identities. Two unknown
            // summaries never establish equality, even if both bindings vary.
            *value = Rc::new(Value {
                ty: value.ty.clone(),
                words: value
                    .words
                    .iter()
                    .enumerate()
                    .map(|(index, origin)| {
                        origin.clone().or(Some(Origin::Snapshot(version, index)))
                    })
                    .collect(),
            });
        }
        Some(Self { env })
    }
}

use super::*;

#[cfg(test)]
#[path = "return_invariant_parent_entries_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "return_invariant_post_loop_snapshots_tests.rs"]
mod post_loop_tests;

pub(in super::super) struct LoopFacts {
    env: Env,
}

impl Snapshots {
    pub(in super::super) fn loop_facts(
        &self,
        body: &[NirStmt],
        layouts: &control_values::CarryLayouts,
        catalog: &ScalarHelpers,
        budget: &mut Budget,
        depth: usize,
    ) -> Option<LoopFacts> {
        let mut proof = Proof::new(layouts, catalog, budget);
        proof.budget.charge(self.env.len())?;
        for value in self.env.values() {
            if proof.shape(&value.ty)?.leaves.len() != value.words.len() {
                return None;
            }
        }
        let mut env = self.env.clone();
        // Both child entries and post-loop users must include zero trips and
        // every intermediate publication, not just the final backedge.
        proof.loop_summary(&NirExpr::Bool(true), body, &mut env, depth)?;
        Some(LoopFacts { env })
    }
}

impl LoopFacts {
    pub(in super::super) fn at_boundary(
        &self,
        budget: &mut Budget,
        next: &mut usize,
    ) -> Option<Snapshots> {
        budget.charge(self.env.len())?;
        let mut env = self.env.clone();
        let mut clock = *next;
        for value in env.values_mut() {
            budget.charge(value.words.len() + 1)?;
            if value.words.iter().all(Option::is_some) {
                continue;
            }
            let version = clock;
            clock = clock.checked_add(1)?;
            // Varying fields are independent across bindings AND boundaries.
            // Reusing entry identities at exit would equate old live copies.
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
        *next = clock;
        Some(Snapshots { env })
    }
}

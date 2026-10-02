use super::*;

#[cfg(test)]
#[path = "return_invariant_loops_tests.rs"]
mod tests;

impl Proof<'_> {
    pub(super) fn observe(&mut self, name: &str, value: &Rc<Value>) -> Option<()> {
        for observation in &mut self.observations {
            if let Some(previous) = observation.get_mut(name) {
                if previous.ty != value.ty || previous.words.len() != value.words.len() {
                    return None;
                }
                self.budget.charge(previous.words.len())?;
                if previous != value {
                    *previous = Rc::new(Value {
                        ty: previous.ty.clone(),
                        words: previous
                            .words
                            .iter()
                            .zip(&value.words)
                            .map(|(old, new)| if old == new { old.clone() } else { None })
                            .collect(),
                    });
                }
            }
        }
        Some(())
    }

    pub(super) fn loop_summary(
        &mut self,
        condition: &NirExpr,
        body: &[NirStmt],
        env: &mut Env,
        depth: usize,
    ) -> Option<()> {
        // Include zero trips and every intermediate write, not just the final
        // backedge: break/continue may expose a value that a suffix restores.
        // Each entry origin can only become unknown, so this bounded fixed point
        // covers arbitrarily many trips without interpreting the trip count.
        loop {
            self.budget.tick(depth)?;
            self.budget.charge(env.len().checked_mul(2)?)?;
            if self.expression(condition, env, depth + 1)?.ty != scalar_type("bool") {
                return None;
            }
            self.observations.push(env.clone());
            let mut iteration = env.clone();
            self.block(body, &mut iteration, depth + 1)?;
            let summary = self.observations.pop()?;
            self.budget
                .charge(env.values().map(|value| value.words.len()).sum())?;
            if &summary == env {
                return Some(());
            }
            // Only bindings that existed at entry survive. Child-local aliases
            // are recreated on each iteration and cannot escape the loop.
            *env = summary;
        }
    }
}

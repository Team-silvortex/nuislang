use super::*;

// This certifies initialized entry names, not expression purity or write authority.
// Its read extension lives only in one print validation and never updates Locals.
#[derive(Default)]
pub(super) struct Epoch {
    seeds: BTreeSet<String>,
}

impl Epoch {
    pub(super) fn enter(
        scope: &Scope,
        writable: &BTreeSet<String>,
        updates: &BTreeSet<String>,
    ) -> Self {
        Self {
            seeds: updates
                .intersection(writable)
                .filter(|name| scope.get(*name) == Some(&scalar_type("i64")))
                .cloned()
                .collect(),
        }
    }

    pub(super) fn reads(
        &self,
        value: &NirExpr,
        scope: &Scope,
        available: &BTreeSet<String>,
        updates: &BTreeSet<String>,
    ) -> Option<BTreeSet<String>> {
        let mut inputs = BTreeSet::new();
        control_values::collect_inputs(value, &mut inputs);
        let missing = inputs
            .intersection(updates)
            .filter(|name| !available.contains(*name))
            .cloned()
            .collect::<BTreeSet<_>>();
        missing
            .iter()
            .all(|name| self.seeds.contains(name) && scope.get(name) == Some(&scalar_type("i64")))
            .then_some(missing)
    }
}

#[cfg(test)]
#[path = "print_epoch_tests.rs"]
mod tests;

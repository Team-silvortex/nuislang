use super::*;

// This owns a separate capability-specific signature table. Pure discovery and
// backend grants remain independent; real print statements are never erased.
pub(super) struct Plan {
    catalog: ScalarHelpers,
    roots: BTreeSet<String>,
}

impl Plan {
    pub(super) fn collect(
        module: &NirModule,
        layouts: &control_values::CarryLayouts,
        pure: &ScalarHelpers,
    ) -> Self {
        let candidates = module
            .functions
            .iter()
            .filter(|f| {
                !pure.contains_key(&f.name)
                    && control_loops::contains_loop(&f.body)
                    && !control_loops::returns::contains_loop_return(&f.body)
            })
            .map(|f| f.name.clone())
            .collect::<BTreeSet<_>>();
        if candidates.is_empty() {
            return Self {
                catalog: ScalarHelpers::new(),
                roots: BTreeSet::new(),
            };
        }
        let catalog = scalar_helpers::collect_iteration_calls(module, layouts);
        // Return-in-loop normalization is not part of this effect capability.
        // Also exclude callers of such roots, not just the roots themselves.
        let mut excluded = module
            .functions
            .iter()
            .filter(|f| {
                !pure.contains_key(&f.name) && control_loops::returns::contains_loop_return(&f.body)
            })
            .map(|f| f.name.clone())
            .collect::<BTreeSet<_>>();
        if !excluded.is_empty() {
            let mut callers = BTreeMap::<&str, Vec<&str>>::new();
            for (name, helper) in &catalog {
                for dependency in &helper.dependencies {
                    callers.entry(dependency).or_default().push(name);
                }
            }
            let mut pending = excluded.iter().cloned().collect::<Vec<_>>();
            while let Some(name) = pending.pop() {
                for caller in callers.get(name.as_str()).into_iter().flatten() {
                    if excluded.insert((*caller).to_owned()) {
                        pending.push((*caller).to_owned());
                    }
                }
            }
        }
        let roots = candidates
            .into_iter()
            .filter(|name| catalog.contains_key(name) && !excluded.contains(name))
            .collect();
        Self { catalog, roots }
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.roots.contains(name)
    }

    pub(super) fn outline(
        self,
        module: &mut NirModule,
        layouts: &control_values::CarryLayouts,
        names: &mut BTreeSet<String>,
        helpers: &mut Vec<NirFunction>,
        outlined: &mut BufferLoopOutlines,
    ) {
        if self.roots.is_empty() {
            return;
        }
        let mut retained = self.roots.clone();
        for function in module
            .functions
            .iter()
            .filter(|f| self.roots.contains(&f.name))
        {
            scalar_helpers::retain_reachable(&function.body, &self.catalog, &mut retained);
        }
        // Already-normalized pure/guarded dependencies must not be outlined twice
        // using a signature snapshot taken before their private helpers existed.
        let fresh = retained.difference(&outlined.functions).cloned().collect();
        outlined.functions.extend(retained);
        control_loops::outline(
            module,
            names,
            helpers,
            &mut outlined.guarded_functions,
            &self.catalog,
            layouts,
            validation::LiteralPrints::Allow,
            &mut outlined.break_controls,
            false,
            &BTreeMap::new(),
            Some(&self.roots),
        );
        let layouts = control_values::TypedLayouts::collect(module);
        scalar_control::outline(
            module,
            &fresh,
            names,
            helpers,
            &mut outlined.guarded_functions,
            &self.catalog,
            &layouts,
        );
    }
}

#[cfg(test)]
#[path = "iteration_effects_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "iteration_effects_exits_tests.rs"]
mod exits_tests;

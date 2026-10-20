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
            .filter(|f| !pure.contains_key(&f.name) && control_loops::contains_loop(&f.body))
            .map(|f| f.name.clone())
            .collect::<BTreeSet<_>>();
        if candidates.is_empty() {
            return Self {
                catalog: ScalarHelpers::new(),
                roots: BTreeSet::new(),
            };
        }
        let catalog = scalar_helpers::collect_iteration_calls(module, layouts);
        // Completed dependency admission, including typed return normalization,
        // releases callers. No provisional or pure-value authority is granted.
        let roots = candidates
            .into_iter()
            .filter(|name| catalog.contains_key(name))
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
        let mut return_signals = BTreeMap::new();
        for function in &mut module.functions {
            if self.roots.contains(&function.name) {
                if let Some(plan) =
                    control_loops::returns::prepare_effects(function, layouts, &self.catalog)
                        .expect("admitted effectful counted return flow")
                {
                    function.body = plan.body;
                    return_signals.insert(function.name.clone(), plan.signal);
                }
            }
        }
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
            &return_signals,
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

#[cfg(test)]
#[path = "iteration_effects_children_tests.rs"]
mod children_tests;

#[cfg(test)]
#[path = "iteration_effects_selected_tests.rs"]
mod selected_tests;

#[cfg(test)]
#[path = "iteration_effects_parents_tests.rs"]
mod parents_tests;

#[cfg(test)]
#[path = "iteration_effects_returns_tests.rs"]
mod returns_tests;

#[cfg(test)]
#[path = "iteration_effects_return_values_tests.rs"]
mod return_values_tests;

#[cfg(test)]
#[path = "iteration_effects_return_selected_tests.rs"]
mod return_selected_tests;

#[cfg(test)]
#[path = "iteration_effects_return_parent_steps_tests.rs"]
mod return_parent_steps_tests;

#[cfg(test)]
#[path = "iteration_effects_return_parent_continue_tests.rs"]
mod return_parent_continue_tests;

#[cfg(test)]
#[path = "iteration_effects_return_parent_break_tests.rs"]
mod return_parent_break_tests;

#[cfg(test)]
#[path = "iteration_effects_return_parent_mixed_tests.rs"]
mod return_parent_mixed_tests;

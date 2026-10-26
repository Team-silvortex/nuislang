use super::*;

#[path = "print_field_reads.rs"]
mod field_reads;

#[path = "print_unary_float.rs"]
mod unary_float;

#[path = "print_binary_float.rs"]
mod binary_float;

// This certifies initialized entry names, not expression purity or write authority.
// Its read extension lives only in one print validation and never updates Locals.
#[derive(Default)]
pub(super) struct Epoch {
    seeds: BTreeSet<String>,
    fields: field_reads::Fields,
}

impl Epoch {
    pub(super) fn enter_child(
        scope: &Scope,
        writable: &BTreeSet<String>,
        changed: &BTreeSet<String>,
        available: &BTreeSet<String>,
        layouts: &control_values::CarryLayouts,
    ) -> Self {
        let ready = changed.intersection(available).cloned().collect();
        Self::enter(scope, writable, &ready, layouts)
    }

    pub(super) fn enter(
        scope: &Scope,
        writable: &BTreeSet<String>,
        updates: &BTreeSet<String>,
        layouts: &control_values::CarryLayouts,
    ) -> Self {
        Self {
            seeds: updates
                .intersection(writable)
                .filter(|name| scope.get(*name) == Some(&scalar_type("i64")))
                .cloned()
                .collect(),
            fields: field_reads::Fields::enter(scope, writable, updates, layouts),
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
            .all(|name| {
                (self.seeds.contains(name) && scope.get(name) == Some(&scalar_type("i64")))
                    || self.fields.contains(name, scope)
            })
            .then(|| missing.intersection(&self.seeds).cloned().collect())
    }

    pub(super) fn field(
        &self,
        value: &NirExpr,
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
    ) -> Option<NirTypeRef> {
        self.fields
            .project(value, scope, layouts, &scalar_type("i64"))
    }

    pub(super) fn argument(
        &self,
        value: &NirExpr,
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
        expected: &NirTypeRef,
    ) -> Option<NirTypeRef> {
        if expected != &scalar_type(&expected.name)
            || !matches!(expected.name.as_str(), "bool" | "i32" | "f32" | "f64")
        {
            return None;
        }
        self.fields.project(value, scope, layouts, expected)
    }

    pub(super) fn unary_argument(
        &self,
        value: &NirExpr,
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
        expected: &NirTypeRef,
        depth: usize,
    ) -> Option<NirTypeRef> {
        let leaf = unary_float::field_operand(value, expected, depth)?;
        self.fields.project(leaf, scope, layouts, expected)
    }

    pub(super) fn float_argument(
        &self,
        value: &NirExpr,
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
        expected: &NirTypeRef,
        depth: usize,
        catalog: &ScalarHelpers,
        mut ready: impl FnMut(&NirExpr, usize) -> Option<NirTypeRef>,
    ) -> Option<NirTypeRef> {
        binary_float::validate(value, expected, depth, catalog, |value, depth| {
            self.fields
                .project_type(value, scope, layouts)
                .or_else(|| ready(value, depth))
        })?;
        Some(expected.clone())
    }
}

#[cfg(test)]
#[path = "print_epoch_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "bool_field_print_epoch_tests.rs"]
mod bool_field_tests;
#[cfg(test)]
#[path = "child_print_epoch_tests.rs"]
mod child_tests;
#[cfg(test)]
#[path = "f32_field_print_epoch_tests.rs"]
mod f32_field_tests;
#[cfg(test)]
#[path = "f64_field_print_epoch_tests.rs"]
mod f64_field_tests;
#[cfg(test)]
#[path = "field_print_epoch_tests.rs"]
mod field_tests;
#[cfg(test)]
#[path = "i32_field_print_epoch_tests.rs"]
mod i32_field_tests;

#[cfg(test)]
#[path = "unary_float_print_epoch_tests.rs"]
mod unary_float_tests;

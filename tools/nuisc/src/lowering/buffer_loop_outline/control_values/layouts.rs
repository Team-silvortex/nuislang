use super::*;

// Value selection and loop carry admission share expression rules, not authority.
// FlatLayouts remains the independent, i64-word-only scoped loop profile.
pub(crate) trait ValueLayouts {
    fn scalar(&self, name: &str) -> bool;
    fn fields(&self, name: &str) -> Option<impl ExactSizeIterator<Item = (&str, NirTypeRef)>>;
}

impl ValueLayouts for FlatLayouts {
    fn scalar(&self, name: &str) -> bool {
        matches!(name, "i64" | "bool")
    }

    fn fields(&self, name: &str) -> Option<impl ExactSizeIterator<Item = (&str, NirTypeRef)>> {
        self.get(name).map(|fields| {
            fields
                .iter()
                .map(|name| (name.as_str(), scalar_type("i64")))
        })
    }
}

#[derive(Default)]
pub(crate) struct TypedLayouts(BTreeMap<String, Vec<(String, NirTypeRef)>>);

impl ValueLayouts for TypedLayouts {
    fn scalar(&self, name: &str) -> bool {
        matches!(name, "bool" | "i32" | "i64" | "f32" | "f64")
    }

    fn fields(&self, name: &str) -> Option<impl ExactSizeIterator<Item = (&str, NirTypeRef)>> {
        self.0
            .get(name)
            .map(|fields| fields.iter().map(|(name, ty)| (name.as_str(), ty.clone())))
    }
}

impl TypedLayouts {
    pub(in crate::lowering::buffer_loop_outline) fn collect(module: &NirModule) -> Self {
        let mut layouts = Self::default();
        let mut definitions = BTreeMap::new();
        let mut remaining = BTreeMap::new();
        let mut parents = BTreeMap::<String, Vec<String>>::new();
        let mut ready = BTreeSet::new();
        let mut shapes = BTreeMap::<String, (usize, usize)>::new();
        for definition in &module.structs {
            let mut names = BTreeSet::new();
            if !definition.generic_params.is_empty()
                || !definition.where_bounds.is_empty()
                || definition.fields.is_empty()
                || definition.fields.iter().any(|field| {
                    !names.insert(&field.name) || field.ty != scalar_type(&field.ty.name)
                })
            {
                continue;
            }
            let dependencies = definition
                .fields
                .iter()
                .filter(|field| !layouts.scalar(&field.ty.name))
                .map(|field| &field.ty.name)
                .collect::<BTreeSet<_>>();
            if dependencies.is_empty() {
                ready.insert(definition.name.clone());
            }
            remaining.insert(definition.name.clone(), dependencies.len());
            for dependency in dependencies {
                parents
                    .entry(dependency.clone())
                    .or_default()
                    .push(definition.name.clone());
            }
            definitions.insert(definition.name.clone(), definition);
        }
        // Resolve leaves first. Resources, unknown names and recursive cycles
        // never release a parent; declaration order does not affect admission.
        while let Some(name) = ready.pop_first() {
            let definition = definitions[&name];
            let mut nodes = 1usize;
            let mut depth = 1usize;
            let mut nested = false;
            for field in &definition.fields {
                if layouts.scalar(&field.ty.name) {
                    nodes = nodes.saturating_add(1);
                } else {
                    let (child_nodes, child_depth) = shapes[&field.ty.name];
                    nodes = nodes.saturating_add(child_nodes);
                    depth = depth.max(child_depth + 1);
                    nested = true;
                }
            }
            // A small type DAG can expand into an enormous neutral initializer.
            // Bound new nested expansion without narrowing the existing flat profile.
            if depth > 64 || (nested && nodes > 4096) {
                continue;
            }
            shapes.insert(name.clone(), (nodes, depth));
            layouts.0.insert(
                name.clone(),
                definition
                    .fields
                    .iter()
                    .map(|field| (field.name.clone(), field.ty.clone()))
                    .collect(),
            );
            for parent in parents.get(&name).into_iter().flatten() {
                let count = remaining.get_mut(parent).expect("known value layout");
                *count -= 1;
                if *count == 0 {
                    ready.insert(parent.clone());
                }
            }
        }
        layouts
    }
}

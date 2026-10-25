use super::*;
use crate::lowering::scalar_record_shape::Shape;

#[derive(Default)]
pub(super) struct Fields(BTreeMap<String, Shape>);

impl Fields {
    pub(super) fn enter(
        scope: &Scope,
        writable: &BTreeSet<String>,
        updates: &BTreeSet<String>,
        layouts: &control_values::CarryLayouts,
    ) -> Self {
        Self(
            updates
                .intersection(writable)
                .filter_map(|name| {
                    let shape = Shape::from_layouts(scope.get(name)?, layouts)?;
                    (exact(&shape) && !shape.fields.is_empty()).then(|| (name.clone(), shape))
                })
                .collect(),
        )
    }

    pub(super) fn contains(&self, name: &str, scope: &Scope) -> bool {
        self.0
            .get(name)
            .is_some_and(|shape| scope.get(name) == Some(&shape.ty))
    }

    pub(super) fn project(
        &self,
        value: &NirExpr,
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
        expected: &NirTypeRef,
    ) -> Option<NirTypeRef> {
        let mut value = value;
        let mut path = Vec::new();
        while let NirExpr::FieldAccess { base, field } = value {
            if path.len() >= 64 {
                return None;
            }
            path.push(field.as_str());
            value = base;
        }
        let NirExpr::Var(name) = value else {
            return None;
        };
        let entry = self.0.get(name)?;
        if path.is_empty()
            || !self.contains(name, scope)
            || Shape::from_layouts(&entry.ty, layouts).as_ref() != Some(entry)
        {
            return None;
        }
        // Resolve the original projection, never substitute a ready whole-record Var.
        let mut leaf = entry;
        for field in path.into_iter().rev() {
            leaf = &leaf.fields.iter().find(|(name, _)| name == field)?.1;
        }
        (leaf.fields.is_empty() && leaf.ty == *expected).then(|| leaf.ty.clone())
    }
}

fn exact(shape: &Shape) -> bool {
    shape.ty == scalar_type(&shape.ty.name) && shape.fields.iter().all(|(_, child)| exact(child))
}

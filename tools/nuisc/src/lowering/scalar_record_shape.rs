use super::*;

#[cfg(test)]
#[path = "scalar_record_shape_tests.rs"]
mod tests;

#[derive(Clone)]
pub(super) struct Shape {
    pub ty: NirTypeRef,
    pub fields: Vec<(String, Shape)>,
}

impl Shape {
    pub fn from_definitions(
        ty: &NirTypeRef,
        definitions: &BTreeMap<&str, &NirStructDef>,
    ) -> Option<Self> {
        Self::collect(ty, &|name| {
            let definition = definitions.get(name)?;
            if !definition.generic_params.is_empty() || !definition.where_bounds.is_empty() {
                return None;
            }
            Some(
                definition
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect(),
            )
        })
    }

    pub fn from_layouts(
        ty: &NirTypeRef,
        layouts: &BTreeMap<String, Vec<(String, NirTypeRef)>>,
    ) -> Option<Self> {
        Self::collect(ty, &|name| layouts.get(name).cloned())
    }

    fn collect(
        ty: &NirTypeRef,
        fields: &impl Fn(&str) -> Option<Vec<(String, NirTypeRef)>>,
    ) -> Option<Self> {
        // Flat input size is explicit; only nested type DAGs can expand it.
        let flat_width = fields(&ty.name)
            .filter(|fields| fields.iter().all(|(_, ty)| scalar(ty)))
            .map_or(0, |fields| fields.len());
        Self::visit(
            ty,
            fields,
            &mut BTreeSet::new(),
            0,
            &mut 4096.max(flat_width + 1),
        )
    }

    fn visit(
        ty: &NirTypeRef,
        fields: &impl Fn(&str) -> Option<Vec<(String, NirTypeRef)>>,
        active: &mut BTreeSet<String>,
        depth: usize,
        remaining: &mut usize,
    ) -> Option<Self> {
        if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() {
            return None;
        }
        *remaining = remaining.checked_sub(1)?;
        if scalar(ty) {
            return Some(Self {
                ty: ty.clone(),
                fields: Vec::new(),
            });
        }
        if depth >= 64 || !active.insert(ty.name.clone()) {
            return None;
        }
        let declared = fields(&ty.name)?;
        let mut names = BTreeSet::new();
        if declared.is_empty() || declared.iter().any(|(name, _)| !names.insert(name)) {
            return None;
        }
        let children = declared
            .iter()
            .map(|(name, ty)| {
                Some((
                    name.clone(),
                    Self::visit(ty, fields, active, depth + 1, remaining)?,
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        active.remove(&ty.name);
        Some(Self {
            ty: ty.clone(),
            fields: children,
        })
    }

    pub fn leaves(&self) -> Vec<(Vec<String>, NirTypeRef)> {
        fn walk(
            shape: &Shape,
            path: &mut Vec<String>,
            result: &mut Vec<(Vec<String>, NirTypeRef)>,
        ) {
            if shape.fields.is_empty() {
                result.push((path.clone(), shape.ty.clone()));
            } else {
                for (name, child) in &shape.fields {
                    path.push(name.clone());
                    walk(child, path, result);
                    path.pop();
                }
            }
        }
        let mut result = Vec::new();
        walk(self, &mut Vec::new(), &mut result);
        result
    }

    pub fn reconstruct(&self, leaf: &mut impl FnMut(&NirTypeRef) -> NirExpr) -> NirExpr {
        if self.fields.is_empty() {
            leaf(&self.ty)
        } else {
            NirExpr::StructLiteral {
                type_name: self.ty.name.clone(),
                type_args: vec![],
                fields: self
                    .fields
                    .iter()
                    .map(|(name, child)| (name.clone(), child.reconstruct(leaf)))
                    .collect(),
            }
        }
    }

    pub fn values<'a>(&self, value: &'a NirExpr) -> Option<Vec<&'a NirExpr>> {
        if self.fields.is_empty() {
            return Some(vec![value]);
        }
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = value
        else {
            return None;
        };
        if type_name != &self.ty.name || !type_args.is_empty() || fields.len() != self.fields.len()
        {
            return None;
        }
        let mut result = Vec::new();
        for ((name, child), (actual, value)) in self.fields.iter().zip(fields) {
            if name != actual {
                return None;
            }
            result.extend(child.values(value)?);
        }
        Some(result)
    }
}

pub(super) fn scalar(ty: &NirTypeRef) -> bool {
    !ty.is_ref
        && !ty.is_optional
        && ty.generic_args.is_empty()
        && matches!(ty.name.as_str(), "bool" | "i32" | "i64" | "f32" | "f64")
}

pub(super) fn source_value(binding: &str, path: &[String]) -> NirExpr {
    path.iter()
        .fold(NirExpr::Var(binding.to_owned()), |base, field| {
            NirExpr::FieldAccess {
                base: Box::new(base),
                field: field.clone(),
            }
        })
}

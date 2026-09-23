use std::collections::BTreeMap;

use yir_core::{
    native_scalar_session::ScalarKind, OwnedStructFieldLayout, OwnedStructLayout,
    OwnedStructScalarLayout, StructValue, Value, YirFunctionParameter,
};

// Match the owned-layout nesting bound without imposing the native 64-slot limit
// on reference sessions whose state comes from flattened function signatures.
const MAX_STATE_DEPTH: usize = 64;

pub(super) struct StateShape {
    nominal: Option<String>,
    fields: Vec<(String, Field)>,
}

enum Field {
    Scalar(ScalarKind),
    Struct(StateShape),
}

fn mismatch() -> String {
    "application session state fields do not match the registered flattened signature".to_owned()
}

impl StateShape {
    pub fn bind(
        state_type: &str,
        prefix: &str,
        parameters: &[YirFunctionParameter],
    ) -> Result<Self, String> {
        let mut shape = Self {
            nominal: Some(state_type.to_owned()),
            fields: Vec::new(),
        };
        for parameter in parameters {
            let path = parameter.name.strip_prefix(prefix).ok_or_else(mismatch)?;
            let parts = path.split('.').collect::<Vec<_>>();
            if parts.len() > MAX_STATE_DEPTH || parts.iter().any(|part| part.is_empty()) {
                return Err(mismatch());
            }
            shape.insert(&parts, ScalarKind::parse(&parameter.ty)?)?;
        }
        let mut ordered = Vec::new();
        shape.paths(prefix.trim_end_matches('.'), &mut ordered);
        if ordered
            .iter()
            .map(String::as_str)
            .ne(parameters.iter().map(|p| p.name.as_str()))
        {
            return Err(mismatch());
        }
        Ok(shape)
    }

    fn insert(&mut self, path: &[&str], kind: ScalarKind) -> Result<(), String> {
        let name = path[0];
        let existing = self.fields.iter().position(|(field, _)| field == name);
        if path.len() == 1 {
            if existing.is_some() {
                return Err(mismatch());
            }
            self.fields.push((name.to_owned(), Field::Scalar(kind)));
            return Ok(());
        }
        let index = existing.unwrap_or_else(|| {
            self.fields.push((
                name.to_owned(),
                Field::Struct(Self {
                    nominal: None,
                    fields: Vec::new(),
                }),
            ));
            self.fields.len() - 1
        });
        let Field::Struct(child) = &mut self.fields[index].1 else {
            return Err(mismatch());
        };
        child.insert(&path[1..], kind)
    }

    fn paths(&self, prefix: &str, out: &mut Vec<String>) {
        for (name, field) in &self.fields {
            let path = format!("{prefix}.{name}");
            match field {
                Field::Scalar(_) => out.push(path),
                Field::Struct(child) => child.paths(&path, out),
            }
        }
    }

    // Layout metadata supplies nested nominal identities, not constructor order.
    // Metadata-free hand-authored YIR retains its signature-only boundary.
    pub fn bind_layout(&mut self, layout: &OwnedStructLayout) -> Result<(), String> {
        if self
            .nominal
            .as_ref()
            .is_some_and(|name| name != &layout.type_name)
            || self.fields.len() != layout.fields.len()
        {
            return Err("application session state layout disagrees with its signature".to_owned());
        }
        self.nominal = Some(layout.type_name.clone());
        for ((name, field), (declared, layout)) in self.fields.iter_mut().zip(&layout.fields) {
            if name != declared {
                return Err(mismatch());
            }
            match (field, layout) {
                (Field::Struct(shape), OwnedStructFieldLayout::Struct(layout)) => {
                    shape.bind_layout(layout)?
                }
                (Field::Scalar(kind), OwnedStructFieldLayout::Scalar(layout))
                    if matches!(
                        (*kind, *layout),
                        (ScalarKind::Bool, OwnedStructScalarLayout::Bool)
                            | (ScalarKind::I32, OwnedStructScalarLayout::I32)
                            | (ScalarKind::I64, OwnedStructScalarLayout::I64)
                            | (ScalarKind::F32, OwnedStructScalarLayout::F32)
                            | (ScalarKind::F64, OwnedStructScalarLayout::F64)
                    ) => {}
                _ => return Err(mismatch()),
            }
        }
        Ok(())
    }

    pub fn normalize(&self, value: Value) -> Result<Value, String> {
        let Value::Struct(value) = value else {
            return Err(mismatch());
        };
        if self
            .nominal
            .as_ref()
            .is_some_and(|name| name != &value.type_name)
        {
            return Err("application session returned the wrong state type".to_owned());
        }
        if value.fields.len() != self.fields.len() {
            return Err(mismatch());
        }
        let mut supplied = BTreeMap::new();
        for (name, field) in value.fields {
            if supplied.insert(name, field).is_some() {
                return Err(mismatch());
            }
        }
        let mut fields = Vec::with_capacity(self.fields.len());
        for (name, field) in &self.fields {
            let value = supplied.remove(name).ok_or_else(mismatch)?;
            let value = match field {
                Field::Struct(child) => child.normalize(value)?,
                Field::Scalar(kind) => {
                    kind.pack(&value).map_err(|_| {
                        format!(
                            "application session state field `{name}` has the wrong scalar kind"
                        )
                    })?;
                    value
                }
            };
            fields.push((name.clone(), value));
        }
        Ok(Value::Struct(StructValue {
            type_name: value.type_name,
            fields,
        }))
    }
}

#[cfg(test)]
mod tests;

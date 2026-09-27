use yir_core::{
    native_scalar_session::ScalarStateLayout, Node, OwnedStructFieldLayout, OwnedStructLayout,
    OwnedStructScalarLayout, Value,
};

#[cfg(test)]
#[path = "value_parameters_tests.rs"]
mod tests;

/// Synchronous, bounded, resource-free value transport, not an external ABI.
pub struct ValueParameter {
    pub index: usize,
    pub layout: OwnedStructLayout,
}

pub fn parse(node: &Node) -> Result<Option<ValueParameter>, String> {
    if node.op.module != "cpu" || node.op.instruction != "param_value_struct" {
        return Ok(None);
    }
    let [index, encoded] = node.op.args.as_slice() else {
        return Err(format!(
            "CPU value parameter `{}` requires index and layout",
            node.name
        ));
    };
    let index = index
        .parse::<usize>()
        .map_err(|_| format!("CPU value parameter `{}` has invalid index", node.name))?;
    ScalarStateLayout::parse(encoded)?;
    Ok(Some(ValueParameter {
        index,
        layout: yir_core::parse_owned_struct_layout(encoded)?,
    }))
}

impl ValueParameter {
    pub fn validate(&self, value: &Value) -> Result<(), String> {
        if matches_record(&self.layout, value) {
            Ok(())
        } else {
            Err(format!(
                "CPU value parameter {} does not match nominal layout `{}`",
                self.index, self.layout.type_name
            ))
        }
    }
}

fn matches_record(layout: &OwnedStructLayout, value: &Value) -> bool {
    let Value::Struct(value) = value else {
        return false;
    };
    value.type_name == layout.type_name
        && value.fields.len() == layout.fields.len()
        && layout.fields.iter().all(|(name, field)| {
            let Some((_, value)) = value.fields.iter().find(|(candidate, _)| candidate == name)
            else {
                return false;
            };
            match field {
                OwnedStructFieldLayout::Struct(layout) => matches_record(layout, value),
                OwnedStructFieldLayout::Scalar(kind) => matches!(
                    (kind, value),
                    (OwnedStructScalarLayout::Bool, Value::Bool(_))
                        | (OwnedStructScalarLayout::I32, Value::I32(_))
                        | (OwnedStructScalarLayout::I64, Value::Int(_))
                        | (OwnedStructScalarLayout::F32, Value::F32(_))
                        | (OwnedStructScalarLayout::F64, Value::F64(_))
                ),
            }
        })
}

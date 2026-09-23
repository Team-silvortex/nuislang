use yir_core::{Value, YirFunction, YirFunctionParameter, YirModule};
use yir_exec::FunctionSession;

use super::state_shape::StateShape;
use super::ApplicationSessionEntries;

pub(super) struct SessionBoundary<'a> {
    pub open: &'a YirFunction,
    pub event: &'a YirFunction,
    pub close: &'a YirFunction,
    state_parameters: &'a [YirFunctionParameter],
    state_prefix: String,
    state_type: &'a str,
    state_shape: StateShape,
}

impl<'a> SessionBoundary<'a> {
    pub fn bind(
        module: &'a YirModule,
        entries: ApplicationSessionEntries<'_>,
    ) -> Result<Self, String> {
        let yir_core::ApplicationSessionSignature {
            open,
            event,
            close,
            state_parameters,
            state_prefix,
            state_type,
        } = yir_core::ApplicationSessionSignature::bind(module, entries)?;
        let mut state_shape = StateShape::bind(state_type, &state_prefix, state_parameters)?;
        for function in [open, event, close] {
            let result = function.result.as_ref().unwrap();
            if let Some(node) = module.nodes.iter().find(|node| node.name == result.node) {
                if node.op.instruction == "return_owned_struct" {
                    if let Some(encoded) = node.op.args.get(1) {
                        state_shape.bind_layout(&yir_core::parse_owned_struct_layout(encoded)?)?;
                    }
                }
            }
        }
        Ok(Self {
            open,
            event,
            close,
            state_parameters,
            state_prefix,
            state_type,
            state_shape,
        })
    }

    pub fn normalize_state(&self, value: Value) -> Result<Value, String> {
        self.state_shape.normalize(value)
    }

    pub fn state_arguments(&self, state: &Value) -> Result<Vec<Value>, String> {
        if !matches!(state, Value::Struct(value) if value.type_name == self.state_type) {
            return Err("application session returned the wrong state type".to_owned());
        }
        let mut leaves = Vec::new();
        flatten_fields(state, self.state_prefix.trim_end_matches('.'), &mut leaves)?;
        if leaves.len() != self.state_parameters.len()
            || leaves
                .iter()
                .zip(self.state_parameters)
                .any(|((path, _), parameter)| path != &parameter.name)
        {
            return Err(
                "application session state fields do not match the registered flattened signature"
                    .to_owned(),
            );
        }
        let arguments = leaves
            .into_iter()
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>();
        FunctionSession::validate_arguments(self.state_parameters, &arguments)?;
        Ok(arguments)
    }

    pub fn call_arguments(
        &self,
        state: &Value,
        function: &YirFunction,
        extra: Vec<Value>,
    ) -> Result<Vec<Value>, String> {
        let mut arguments = self.state_arguments(state)?;
        arguments.extend(extra);
        FunctionSession::validate_arguments(&function.parameters, &arguments)?;
        Ok(arguments)
    }
}

fn flatten_fields<'a>(
    value: &'a Value,
    prefix: &str,
    leaves: &mut Vec<(String, &'a Value)>,
) -> Result<(), String> {
    match value {
        Value::Struct(value) => {
            for (name, value) in &value.fields {
                flatten_fields(value, &format!("{prefix}.{name}"), leaves)?;
            }
        }
        Value::Bool(_) | Value::I32(_) | Value::Int(_) | Value::F32(_) | Value::F64(_) => {
            leaves.push((prefix.to_owned(), value))
        }
        _ => {
            return Err(format!(
                "application session state `{prefix}` is not a scalar aggregate"
            ))
        }
    }
    Ok(())
}

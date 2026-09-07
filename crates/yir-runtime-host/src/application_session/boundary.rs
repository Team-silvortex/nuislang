use yir_core::{Value, YirFunction, YirFunctionParameter, YirModule, YirValueOwnership};
use yir_exec::FunctionSession;

use super::ApplicationSessionEntries;

pub(super) struct SessionBoundary<'a> {
    pub open: &'a YirFunction,
    pub event: &'a YirFunction,
    pub close: &'a YirFunction,
    state_parameters: &'a [YirFunctionParameter],
    state_prefix: String,
    state_type: &'a str,
}

impl<'a> SessionBoundary<'a> {
    pub fn bind(
        module: &'a YirModule,
        entries: ApplicationSessionEntries<'_>,
    ) -> Result<Self, String> {
        if entries.open == entries.event
            || entries.open == entries.close
            || entries.event == entries.close
        {
            return Err(
                "application session requires distinct open, event and close functions".to_owned(),
            );
        }
        if entries.state_parameter.is_empty()
            || !entries
                .state_parameter
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("application session requires a simple state parameter name".to_owned());
        }
        let find = |name: &str| -> Result<&'a YirFunction, String> {
            let function = module
                .functions
                .iter()
                .find(|function| function.name == name)
                .ok_or_else(|| {
                    format!("application session references unknown function `{name}`")
                })?;
            FunctionSession::validate_function(function)?;
            Ok(function)
        };
        let open = find(entries.open)?;
        let event = find(entries.event)?;
        let close = find(entries.close)?;
        let result = open.result.as_ref().unwrap();
        if result.ownership != YirValueOwnership::Owned {
            return Err("application session requires an owned scalar aggregate result".to_owned());
        }
        for function in [event, close] {
            let other = function.result.as_ref().unwrap();
            if other.ty != result.ty || other.ownership != result.ownership {
                return Err(format!(
                    "application session `{}` must return the same owned state type",
                    function.name
                ));
            }
        }
        let state_prefix = format!("{}.", entries.state_parameter);
        let state_count = event
            .parameters
            .iter()
            .take_while(|parameter| parameter.name.starts_with(&state_prefix))
            .count();
        if state_count == 0 {
            return Err(
                "application session event requires a leading flattened state parameter".to_owned(),
            );
        }
        let state_parameters = &event.parameters[..state_count];
        for function in [event, close] {
            let Some(parameters) = function.parameters.get(..state_count) else {
                return Err("application session close is missing state fields".to_owned());
            };
            if parameters
                .iter()
                .zip(state_parameters)
                .any(|(left, right)| left.name != right.name || left.ty != right.ty)
                || function.parameters[state_count..]
                    .iter()
                    .any(|parameter| parameter.name.starts_with(&state_prefix))
            {
                return Err("application session event/close state signatures disagree".to_owned());
            }
        }
        Ok(Self {
            open,
            event,
            close,
            state_parameters,
            state_prefix,
            state_type: &result.ty,
        })
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

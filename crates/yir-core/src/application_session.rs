use std::collections::BTreeSet;

use crate::{YirFunction, YirFunctionParameter, YirFunctionRole, YirModule, YirValueOwnership};

pub const APPLICATION_SESSION_CONTRACT: &str = "nuis-yir-application-session-v1";

/// Explicit helper bindings. No application or provider names are implicit.
#[derive(Clone, Copy)]
pub struct ApplicationSessionEntries<'a> {
    pub open: &'a str,
    pub event: &'a str,
    pub close: &'a str,
    pub state_parameter: &'a str,
}

/// Static registration carried by YIR, including the embedded artifact copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirApplicationSession {
    pub id: String,
    pub open: String,
    pub event: String,
    pub close: String,
    pub state_parameter: String,
}

impl YirApplicationSession {
    pub fn from_fields(fields: &[&str]) -> Result<Self, String> {
        let [id, contract, open, event, close, state_parameter] = fields else {
            return Err(
                "expected application session id, contract, open, event, close and state parameter"
                    .to_owned(),
            );
        };
        if *contract != APPLICATION_SESSION_CONTRACT {
            return Err(format!(
                "unsupported application session contract `{contract}`"
            ));
        }
        let registration = Self {
            id: (*id).to_owned(),
            open: (*open).to_owned(),
            event: (*event).to_owned(),
            close: (*close).to_owned(),
            state_parameter: (*state_parameter).to_owned(),
        };
        registration.validate_declaration()?;
        Ok(registration)
    }

    pub fn entries(&self) -> ApplicationSessionEntries<'_> {
        ApplicationSessionEntries {
            open: &self.open,
            event: &self.event,
            close: &self.close,
            state_parameter: &self.state_parameter,
        }
    }

    pub fn validate_declaration(&self) -> Result<(), String> {
        for value in [&self.id, &self.open, &self.event, &self.close] {
            if value.is_empty()
                || value.len() > 256
                || value.chars().any(|ch| {
                    ch.is_whitespace() || ch.is_control() || matches!(ch, '"' | '\\' | '#')
                })
            {
                return Err("application session requires nonempty bounded identifiers".to_owned());
            }
        }
        validate_entries(self.entries())
    }
}

/// Shared static signature contract, used by compilation, verification and hosts.
pub struct ApplicationSessionSignature<'a> {
    pub open: &'a YirFunction,
    pub event: &'a YirFunction,
    pub close: &'a YirFunction,
    pub state_parameters: &'a [YirFunctionParameter],
    pub state_prefix: String,
    pub state_type: &'a str,
}

impl<'a> ApplicationSessionSignature<'a> {
    pub fn bind(
        module: &'a YirModule,
        entries: ApplicationSessionEntries<'_>,
    ) -> Result<Self, String> {
        validate_entries(entries)?;
        let find = |name: &str| -> Result<&'a YirFunction, String> {
            let mut functions = module
                .functions
                .iter()
                .filter(|function| function.name == name);
            let function = functions.next().ok_or_else(|| {
                format!("application session references unknown function `{name}`")
            })?;
            if functions.next().is_some() {
                return Err(format!(
                    "application session references ambiguous function `{name}`"
                ));
            }
            validate_session_function(function)?;
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
}

fn validate_entries(entries: ApplicationSessionEntries<'_>) -> Result<(), String> {
    if entries.open == entries.event
        || entries.open == entries.close
        || entries.event == entries.close
    {
        return Err(
            "application session requires distinct open, event and close functions".to_owned(),
        );
    }
    if entries.state_parameter.is_empty()
        || entries.state_parameter.len() > 256
        || !entries
            .state_parameter
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("application session requires a simple state parameter name".to_owned());
    }
    Ok(())
}

pub fn is_session_scalar_type(ty: &str) -> bool {
    matches!(ty, "bool" | "i32" | "i64" | "f32" | "f64")
}

pub fn validate_session_function(function: &YirFunction) -> Result<(), String> {
    if function.role != YirFunctionRole::Helper {
        return Err(format!(
            "session function `{}` must be a helper",
            function.name
        ));
    }
    let mut nodes = BTreeSet::new();
    let mut names = BTreeSet::new();
    for parameter in &function.parameters {
        if !nodes.insert(&parameter.node) {
            return Err(format!(
                "session function `{}` aliases parameter nodes",
                function.name
            ));
        }
        if !names.insert(&parameter.name) {
            return Err(format!(
                "session function `{}` duplicates parameter names",
                function.name
            ));
        }
        if parameter.ownership != YirValueOwnership::Value || !is_session_scalar_type(&parameter.ty)
        {
            return Err(format!(
                "session parameter `{}` must be a value-owned scalar",
                parameter.name
            ));
        }
    }
    let result = function
        .result
        .as_ref()
        .ok_or_else(|| format!("session function `{}` requires a result", function.name))?;
    let valid = if is_session_scalar_type(&result.ty) {
        result.ownership == YirValueOwnership::Value
    } else {
        !result.ty.is_empty() && result.ownership == YirValueOwnership::Owned
    };
    if !valid {
        return Err(format!(
            "session function `{}` has an unsupported result ownership",
            function.name
        ));
    }
    Ok(())
}

pub fn validate_application_sessions(module: &YirModule) -> Result<(), String> {
    if module.application_sessions.len() > 64 {
        return Err("more than 64 application session registrations".to_owned());
    }
    let mut ids = BTreeSet::new();
    for registration in &module.application_sessions {
        registration.validate_declaration()?;
        if !ids.insert(&registration.id) {
            return Err(format!(
                "duplicate application session `{}`",
                registration.id
            ));
        }
        ApplicationSessionSignature::bind(module, registration.entries())?;
    }
    Ok(())
}

pub fn registered_application_session<'a>(
    module: &'a YirModule,
    id: &str,
) -> Result<&'a YirApplicationSession, String> {
    validate_application_sessions(module)?;
    module
        .application_sessions
        .iter()
        .find(|session| session.id == id)
        .ok_or_else(|| format!("unknown application session registration `{id}`"))
}

#[cfg(test)]
#[path = "application_session_tests.rs"]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use yir_core::{YirApplicationSession, YirModule, APPLICATION_SESSION_CONTRACT};

pub(super) fn parse_application_sessions(
    source: &str,
) -> Result<Vec<YirApplicationSession>, String> {
    let mut registrations = None;
    let mut offset = 0;
    for raw in source.split_inclusive('\n') {
        if let Some((key, _)) = raw.split_once('=') {
            if key.trim() == "application_sessions" {
                if registrations.is_some() {
                    return Err("duplicate application_sessions field".to_owned());
                }
                let start = offset + raw.find('=').unwrap() + 1;
                let values = parse_array(&source[start..])?;
                let mut sessions = Vec::new();
                let mut ids = BTreeSet::new();
                for value in values {
                    let session = parse_declaration(&value)?;
                    if !ids.insert(session.id.clone()) {
                        return Err(format!("duplicate application session `{}`", session.id));
                    }
                    sessions.push(session);
                }
                registrations = Some(sessions);
            }
        }
        offset += raw.len();
    }
    Ok(registrations.unwrap_or_default())
}

fn parse_declaration(source: &str) -> Result<YirApplicationSession, String> {
    let mut tokens = source.split_whitespace();
    let id = tokens
        .next()
        .ok_or("empty application session declaration")?;
    let mut fields = BTreeMap::new();
    for token in tokens {
        let (key, value) = token
            .split_once('=')
            .ok_or("malformed application session field")?;
        if !matches!(key, "open" | "event" | "close" | "state")
            || fields.insert(key, value).is_some()
        {
            return Err(format!(
                "unknown or duplicate application session field `{key}`"
            ));
        }
    }
    let required = |key| {
        fields
            .get(key)
            .copied()
            .ok_or_else(|| format!("application session is missing `{key}`"))
    };
    YirApplicationSession::from_fields(&[
        id,
        APPLICATION_SESSION_CONTRACT,
        required("open")?,
        required("event")?,
        required("close")?,
        required("state")?,
    ])
}

// This field is strict even though older optional manifest arrays are permissive.
// Only unescaped basic strings are needed for bounded whitespace-free YIR names.
fn parse_array(mut input: &str) -> Result<Vec<String>, String> {
    let error = || "malformed application_sessions string array".to_owned();
    input = input.trim_start();
    input = input.strip_prefix('[').ok_or_else(error)?;
    let mut values = Vec::new();
    loop {
        input = skip_space_and_comments(input);
        if let Some(rest) = input.strip_prefix(']') {
            let tail = rest.lines().next().unwrap_or("").trim();
            if !tail.is_empty() && !tail.starts_with('#') {
                return Err(error());
            }
            return Ok(values);
        }
        input = input.strip_prefix('"').ok_or_else(error)?;
        let end = input.find('"').ok_or_else(error)?;
        let value = &input[..end];
        if value.len() > 2048 || value.contains(['\\', '\n', '\r']) || values.len() >= 64 {
            return Err(error());
        }
        values.push(value.to_owned());
        input = skip_space_and_comments(&input[end + 1..]);
        if input.starts_with(']') {
            continue;
        }
        input = input.strip_prefix(',').ok_or_else(error)?;
    }
}

fn skip_space_and_comments(mut input: &str) -> &str {
    loop {
        input = input.trim_start();
        if !input.starts_with('#') {
            return input;
        }
        input = input.split_once('\n').map_or("", |(_, rest)| rest);
    }
}

pub(crate) fn register_project_application_sessions(
    project: &super::LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    if !module.application_sessions.is_empty() {
        return Err(
            "project application session registration would replace existing YIR bindings"
                .to_owned(),
        );
    }
    module
        .application_sessions
        .clone_from(&project.manifest.application_sessions);
    yir_core::validate_application_sessions(module)
}

#[cfg(test)]
#[path = "application_sessions_tests.rs"]
mod tests;

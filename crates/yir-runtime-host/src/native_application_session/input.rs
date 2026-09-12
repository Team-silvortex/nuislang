use yir_core::{native_scalar_session::ScalarKind, Value};

pub(super) const MAX_EVENTS: usize = 64;
pub(super) const MAX_ARGC: usize = 2 * MAX_EVENTS + 7;
pub(super) const MAX_ARGUMENT_BYTES: usize = 4096;

pub(super) struct Script {
    pub open: Vec<Value>,
    pub events: Vec<Vec<Value>>,
    pub close: Vec<Value>,
}

pub(super) fn parse(
    arguments: &[&str],
    id: &str,
    kinds: &[Vec<ScalarKind>; 3],
    state_slots: usize,
) -> Result<Script, String> {
    if arguments.len() >= MAX_ARGC || arguments.iter().any(|arg| arg.len() > MAX_ARGUMENT_BYTES) {
        return Err("native application script exceeds its input bounds".to_owned());
    }
    let mut selected = None;
    let mut open = None;
    let mut close = None;
    let mut events = Vec::new();
    let mut arguments = arguments.iter();
    while let Some(&option) = arguments.next() {
        let value = arguments
            .next()
            .ok_or("native application script option requires a value")?;
        match option {
            "--application-session" if selected.is_none() => selected = Some(*value),
            "--open-args" if open.is_none() => open = Some(values(value, &kinds[0])?),
            "--event-args" if events.len() < MAX_EVENTS => {
                events.push(values(value, &kinds[1][state_slots..])?)
            }
            "--close-args" if close.is_none() => {
                close = Some(values(value, &kinds[2][state_slots..])?)
            }
            _ => {
                return Err(format!(
                    "unsupported, duplicate or excess native application option `{option}`"
                ))
            }
        }
    }
    if selected != Some(id) {
        return Err("native application session registration identity mismatch".to_owned());
    }
    Ok(Script {
        open: open.ok_or("--open-args is required (use an empty value for no arguments)")?,
        events,
        close: close.ok_or("--close-args is required (use an empty value for no arguments)")?,
    })
}

fn values(raw: &str, kinds: &[ScalarKind]) -> Result<Vec<Value>, String> {
    let tokens = if raw.is_empty() {
        Vec::new()
    } else {
        raw.split(',').collect::<Vec<_>>()
    };
    if tokens.len() != kinds.len() {
        return Err("native application script scalar argument count mismatch".to_owned());
    }
    tokens
        .iter()
        .zip(kinds)
        .map(|(token, kind)| {
            let value = match kind {
                ScalarKind::Bool => token.parse().ok().map(Value::Bool),
                ScalarKind::I32 => token.parse().ok().map(Value::I32),
                ScalarKind::I64 => token.parse().ok().map(Value::Int),
                ScalarKind::F32 => token.parse().ok().map(Value::F32),
                ScalarKind::F64 => token.parse().ok().map(Value::F64),
            };
            value.ok_or_else(|| format!("invalid native scalar argument for {kind:?}"))
        })
        .collect()
}

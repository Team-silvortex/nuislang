use yir_core::{
    native_resource_session::{registered, ResourceSessionRegistration},
    YirModule,
};

pub const MAX_ARGC: usize = 528;

#[derive(Debug)]
pub struct ResourceScript {
    pub initial: Vec<i64>,
    pub open: i64,
    pub events: Vec<i64>,
    pub close: i64,
    pub frame: Option<super::ResourceFrameExport>,
}

impl ResourceScript {
    pub fn parse(
        arguments: &[&str],
        registration: &ResourceSessionRegistration,
    ) -> Result<Self, String> {
        registration.validate()?;
        if arguments.len() + 1 > MAX_ARGC
            || !arguments.len().is_multiple_of(2)
            || arguments.iter().any(|s| s.len() > 128 * 1024)
        {
            return Err("invalid resource script size".to_owned());
        }
        let (mut id, mut initial, mut open, mut close) = (None, None, None, None);
        let mut events = Vec::new();
        let (mut dimensions, mut output, mut fields) = (None, None, None);
        for pair in arguments.chunks_exact(2) {
            match pair[0] {
                "--resource-session" if id.is_none() => id = Some(pair[1]),
                "--buffer-words" if initial.is_none() => {
                    let words = if pair[1].is_empty() {
                        Vec::new()
                    } else {
                        pair[1]
                            .split(',')
                            .take(registration.max_words + 1)
                            .map(integer)
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    registration.admit_extent(words.len())?;
                    initial = Some(words);
                }
                "--open-arg" if open.is_none() => open = Some(integer(pair[1])?),
                "--close-arg" if close.is_none() => close = Some(integer(pair[1])?),
                "--event-arg" if events.len() < 256 => events.push(integer(pair[1])?),
                "--resource-frame" if dimensions.is_none() => dimensions = Some(pair[1]),
                "--resource-frame-fields" if fields.is_none() => fields = Some(pair[1]),
                "--resource-frame-output" if output.is_none() => output = Some(pair[1]),
                _ => {
                    return Err("unknown, duplicate or excessive resource script option".to_owned())
                }
            }
        }
        if id != Some(registration.id.as_str()) {
            return Err("resource script registration identity mismatch".to_owned());
        }
        Ok(Self {
            initial: initial.ok_or("resource script requires --buffer-words")?,
            open: open.ok_or("resource script requires --open-arg")?,
            events,
            close: close.ok_or("resource script requires --close-arg")?,
            frame: match (
                dimensions,
                fields,
                output,
                registration.state_type.is_some(),
            ) {
                (None, None, None, _) => None,
                (Some(dimensions), None, Some(output), false) => {
                    Some(super::ResourceFrameExport::parse(dimensions, output)?)
                }
                (None, Some(fields), Some(output), true) => Some(
                    super::ResourceFrameExport::from_state_fields(fields, output)?,
                ),
                _ => return Err("resource frame requires both dimensions and output".to_owned()),
            },
        })
    }
}

fn integer(value: &str) -> Result<i64, String> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| "resource argument must be i64")?;
    if parsed.to_string() != value {
        return Err("resource argument must be canonical i64".to_owned());
    }
    Ok(parsed)
}

pub fn validate_resource_script(
    module: &YirModule,
    id: &str,
    arguments: &[&str],
) -> Result<(), String> {
    yir_verify::verify_module(module)?;
    let script = ResourceScript::parse(arguments, registered(module, id)?)?;
    if let Some(frame) = &script.frame {
        frame.preflight_state(
            &script.initial,
            registered(module, id)?.mixed_layout(module)?.as_ref(),
        )?;
    }
    Ok(())
}

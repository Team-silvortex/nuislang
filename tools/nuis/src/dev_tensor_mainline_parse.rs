use super::{MainlinePlan, MAINLINE_PROTOCOL};
use std::collections::{BTreeMap, BTreeSet};

// This manifest is a strict TOML subset: identifier strings and string arrays.
// Reject unknown fields rather than silently changing scheduling intent.
pub(super) fn parse_plan(source: &str) -> Result<MainlinePlan, String> {
    if source.len() > 65536 {
        return Err("mainline plan exceeds 64 KiB".to_owned());
    }
    let mut sections = vec![BTreeMap::<String, String>::new()];
    let mut pending = None::<(String, String)>;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, mut value)) = pending.take() {
            value.push_str(line);
            if line.ends_with(']') {
                insert(sections.last_mut().unwrap(), key, value)?;
            } else {
                pending = Some((key, value));
            }
            continue;
        }
        if line == "[[nodes]]" {
            if sections.len() > 256 {
                return Err("mainline plan exceeds 256 nodes".to_owned());
            }
            sections.push(BTreeMap::new());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("invalid mainline assignment `{line}`"));
        };
        let key = key.trim().to_owned();
        let value = value.trim().to_owned();
        if value.starts_with('[') && !value.ends_with(']') {
            pending = Some((key, value));
        } else {
            insert(sections.last_mut().unwrap(), key, value)?;
        }
    }
    if pending.is_some() {
        return Err("unterminated mainline array".to_owned());
    }
    let mut header = sections.remove(0);
    let schema = identifier(&take(&mut header, "schema")?)?;
    if schema != MAINLINE_PROTOCOL {
        return Err(format!("unsupported mainline schema `{schema}`"));
    }
    let id = identifier(&take(&mut header, "id")?)?;
    let goals = identifiers(&take(&mut header, "goals")?)?;
    let interrupts = identifiers(&take(&mut header, "interrupts")?)?;
    reject_unknown(&header)?;
    if goals.is_empty() {
        return Err("mainline goals must not be empty".to_owned());
    }
    let mut nodes = BTreeMap::new();
    for mut section in sections {
        let coordinate = identifier(&take(&mut section, "coordinate")?)?;
        let dependencies = identifiers(&take(&mut section, "depends_on")?)?;
        reject_unknown(&section)?;
        if nodes.insert(coordinate.clone(), dependencies).is_some() {
            return Err(format!("duplicate mainline node `{coordinate}`"));
        }
    }
    Ok(MainlinePlan {
        id,
        goals,
        interrupts,
        nodes,
    })
}

fn insert(map: &mut BTreeMap<String, String>, key: String, value: String) -> Result<(), String> {
    if map.insert(key.clone(), value).is_some() {
        return Err(format!("duplicate mainline field `{key}`"));
    }
    Ok(())
}

fn take(map: &mut BTreeMap<String, String>, key: &str) -> Result<String, String> {
    map.remove(key)
        .ok_or_else(|| format!("missing mainline field `{key}`"))
}

fn reject_unknown(map: &BTreeMap<String, String>) -> Result<(), String> {
    match map.keys().next() {
        Some(key) => Err(format!("unknown mainline field `{key}`")),
        None => Ok(()),
    }
}

fn identifier(value: &str) -> Result<String, String> {
    let value = value.trim();
    let Some(value) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) else {
        return Err("expected quoted mainline identifier".to_owned());
    };
    if value.is_empty()
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-./".contains(&b))
    {
        return Err("invalid mainline identifier".to_owned());
    }
    Ok(value.to_owned())
}

fn identifiers(value: &str) -> Result<Vec<String>, String> {
    let Some(body) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) else {
        return Err("expected mainline identifier array".to_owned());
    };
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    let body = body.trim().strip_suffix(',').unwrap_or(body.trim());
    let values = body
        .split(',')
        .map(identifier)
        .collect::<Result<Vec<_>, _>>()?;
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        return Err("duplicate mainline array entry".to_owned());
    }
    Ok(values)
}

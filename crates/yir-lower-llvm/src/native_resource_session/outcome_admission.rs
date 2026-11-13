use super::{admission::Kind, mixed_admission::require_layout};
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{native_resource_session::MixedResourceLayout, Node};

pub(super) fn node(
    node: &Node,
    layout: &MixedResourceLayout,
    values: &mut BTreeMap<String, Kind>,
    owners: &mut BTreeSet<String>,
) -> Result<bool, String> {
    let Some(outcome) = &layout.outcome else {
        return Ok(false);
    };
    let args = &node.op.args;
    match node.op.instruction.as_str() {
        "struct"
            if args.first().is_some_and(|name| {
                name == &outcome.ok_variant || name == &outcome.err_variant
            }) =>
        {
            if args.len() != 2 {
                return Err("checked Result variant requires one payload".to_owned());
            }
            let source = args[1]
                .strip_prefix("value=")
                .ok_or("checked Result payload field drift")?;
            let value = values
                .get(source)
                .ok_or("unavailable checked Result payload")?
                .clone();
            if args[0] == outcome.ok_variant {
                let Kind::Record(name, fields) = &value else {
                    return Err("checked Ok requires its complete State".to_owned());
                };
                require_layout(name, fields, layout)?;
                if !owners.remove(source) {
                    return Err("checked Ok has no live State owner".to_owned());
                }
                owners.insert(node.name.clone());
            } else if value != Kind::I64 {
                return Err("checked Err requires an i64 application code".to_owned());
            }
            values.insert(
                node.name.clone(),
                Kind::Record(args[0].clone(), vec![("value".to_owned(), value)]),
            );
        }
        "guard_return" => {
            if args.len() != 3
                || args[2] != layout.encoded
                || values.get(&args[0]) != Some(&Kind::Bool)
            {
                return Err(
                    "checked guard requires bool, Err and the exact Result layout".to_owned(),
                );
            }
            // A rejected branch owns no resource. Requiring no live owners here
            // proves both early exit and continuation without branch-local leaks.
            if !owners.is_empty() || !is_error(values.get(&args[1]), &outcome.err_variant) {
                return Err(
                    "checked early return requires Err without live resource owners".to_owned(),
                );
            }
            values.insert(node.name.clone(), Kind::Void);
        }
        "return_owned_struct" => {
            if args.len() != 2 || args[1] != layout.encoded {
                return Err("checked return layout drift".to_owned());
            }
            if !is_error(values.get(&args[0]), &outcome.err_variant) {
                let Some(Kind::Record(name, fields)) = values.get(&args[0]) else {
                    return Err("checked return requires Ok or Err".to_owned());
                };
                if name != &outcome.ok_variant || fields.len() != 1 || !owners.remove(&args[0]) {
                    return Err("checked return has no live Ok State".to_owned());
                }
            }
            if !owners.is_empty() {
                return Err("checked return leaks resource owners".to_owned());
            }
            values.insert(node.name.clone(), Kind::Void);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn is_error(value: Option<&Kind>, name: &str) -> bool {
    matches!(value, Some(Kind::Record(variant, fields)) if variant == name
        && fields == &[("value".to_owned(), Kind::I64)])
}

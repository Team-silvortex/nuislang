use super::admission::Kind;
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{native_resource_session::MixedResourceLayout, Node};

pub(super) fn node(
    node: &Node,
    layout: &MixedResourceLayout,
    values: &mut BTreeMap<String, Kind>,
    owners: &mut BTreeSet<String>,
) -> Result<bool, String> {
    if super::outcome_admission::node(node, layout, values, owners)? {
        return Ok(true);
    }
    let args = &node.op.args;
    let result = match node.op.instruction.as_str() {
        "struct" => {
            let (name, args) = args.split_first().ok_or("missing mixed record name")?;
            if args.is_empty() || args.len() > 17 {
                return Err("mixed record exceeds field bound".to_owned());
            }
            let mut fields = Vec::new();
            let mut names = BTreeSet::new();
            let mut bytes = None;
            for arg in args {
                let (field, source) = arg.split_once('=').ok_or("malformed mixed record field")?;
                if !names.insert(field) {
                    return Err("duplicate mixed record field".to_owned());
                }
                let kind = values.get(source).ok_or("unavailable mixed record field")?;
                match kind {
                    Kind::I64 => {}
                    Kind::Bytes if bytes.is_none() && owners.contains(source) => {
                        bytes = Some(source.to_owned())
                    }
                    _ => {
                        return Err(
                            "mixed record requires ready i64 values and one live Bytes owner"
                                .to_owned(),
                        )
                    }
                }
                fields.push((field.to_owned(), kind.clone()));
            }
            if let Some(bytes) = bytes {
                require_layout(name, &fields, layout)?;
                owners.remove(&bytes);
                owners.insert(node.name.clone());
            }
            Kind::Record(name.clone(), fields)
        }
        "field" => {
            if args.len() != 2 {
                return Err("malformed mixed scalar projection".to_owned());
            }
            let Some(Kind::Record(_, fields)) = values.get(&args[0]) else {
                return Err("unavailable mixed record projection".to_owned());
            };
            // Check the complete record, including unselected siblings. Projecting
            // a scalar must never bypass an owner hidden in another subtree.
            if fields.iter().any(|(_, kind)| !kind.is_record_value()) {
                return Err("mixed resource projection must not alias an owned record".to_owned());
            }
            fields
                .iter()
                .find(|(name, _)| name == &args[1])
                .ok_or("unknown mixed scalar field")?
                .1
                .clone()
        }
        "return_owned_struct" => {
            if args.len() != 2 || args[1] != layout.encoded {
                return Err("mixed return layout drift".to_owned());
            }
            let Some(Kind::Record(name, fields)) = values.get(&args[0]) else {
                return Err("missing mixed return record".to_owned());
            };
            require_layout(name, fields, layout)?;
            if !owners.remove(&args[0]) {
                return Err("mixed return has no live record owner".to_owned());
            }
            Kind::Void
        }
        _ => return Ok(false),
    };
    values.insert(node.name.clone(), result);
    Ok(true)
}

pub(super) fn require_layout(
    name: &str,
    fields: &[(String, Kind)],
    layout: &MixedResourceLayout,
) -> Result<(), String> {
    if name != layout.type_name
        || fields.len() != layout.scalar_fields.len() + 1
        || !fields.iter().all(|(name, kind)| match kind {
            Kind::Bytes => name == &layout.resource_field,
            Kind::I64 => layout.scalar_fields.contains(name),
            _ => false,
        })
    {
        return Err("mixed record does not match its declared State layout".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_projection_rejects_resources_even_in_an_unselected_sibling() {
        let layout = MixedResourceLayout::parse("Canvas{width:i64;pixels:Bytes}").unwrap();
        let node = Node {
            name: "projected".into(),
            resource: "cpu".into(),
            op: yir_core::Operation::parse("cpu.field", vec!["record".into(), "value".into()])
                .unwrap(),
        };
        for forbidden in [Kind::Bytes, Kind::Buffer, Kind::Void] {
            let mut values = BTreeMap::from([(
                "record".into(),
                Kind::Record(
                    "Envelope".into(),
                    vec![
                        ("value".into(), Kind::I64),
                        (
                            "hidden".into(),
                            Kind::Record("Nested".into(), vec![("payload".into(), forbidden)]),
                        ),
                    ],
                ),
            )]);
            let before = values.clone();
            let mut owners = BTreeSet::from(["snapshot".into()]);
            assert!(super::node(&node, &layout, &mut values, &mut owners)
                .unwrap_err()
                .contains("must not alias"));
            assert!(values == before);
            assert_eq!(owners, BTreeSet::from(["snapshot".into()]));
        }
    }

    #[test]
    fn nested_projection_requires_ready_exact_paths_and_preserves_owners() {
        let layout = MixedResourceLayout::parse("Canvas{width:i64;pixels:Bytes}").unwrap();
        let nested = Kind::Record("Nested".into(), vec![("count".into(), Kind::I64)]);
        let mut values = BTreeMap::from([(
            "record".into(),
            Kind::Record("Envelope".into(), vec![("value".into(), nested.clone())]),
        )]);
        let mut owners = BTreeSet::from(["snapshot".into()]);
        let mut node = Node {
            name: "projected".into(),
            resource: "cpu".into(),
            op: yir_core::Operation::parse("cpu.field", vec!["record".into(), "value".into()])
                .unwrap(),
        };
        assert!(super::node(&node, &layout, &mut values, &mut owners).unwrap());
        assert!(values.get("projected") == Some(&nested));
        for args in [
            vec!["future".into(), "value".into()],
            vec!["record".into(), "value.count".into()],
            vec!["record".into()],
        ] {
            node.op.args = args;
            assert!(super::node(&node, &layout, &mut values, &mut owners).is_err());
        }
        assert_eq!(owners, BTreeSet::from(["snapshot".into()]));
    }

    #[test]
    fn boolean_value_projection_does_not_grant_boolean_owned_state_fields() {
        let layout = MixedResourceLayout::parse("Canvas{width:i64;pixels:Bytes}").unwrap();
        let mut values = BTreeMap::from([
            (
                "record".into(),
                Kind::Record(
                    "Flags".into(),
                    vec![("ready".into(), Kind::Bool), ("count".into(), Kind::I64)],
                ),
            ),
            ("snapshot".into(), Kind::Bytes),
        ]);
        let mut owners = BTreeSet::from(["snapshot".into()]);
        let projection = Node {
            name: "flag".into(),
            resource: "cpu".into(),
            op: yir_core::Operation::parse("cpu.field", vec!["record".into(), "ready".into()])
                .unwrap(),
        };
        assert!(node(&projection, &layout, &mut values, &mut owners).unwrap());
        assert!(values.get("flag") == Some(&Kind::Bool));
        let state = Node {
            name: "state".into(),
            resource: "cpu".into(),
            op: yir_core::Operation::parse(
                "cpu.struct",
                vec![
                    "Canvas".into(),
                    "width=flag".into(),
                    "pixels=snapshot".into(),
                ],
            )
            .unwrap(),
        };
        assert!(node(&state, &layout, &mut values, &mut owners).is_err());
        assert!(!values.contains_key("state"));
        assert_eq!(owners, BTreeSet::from(["snapshot".into()]));
    }
}

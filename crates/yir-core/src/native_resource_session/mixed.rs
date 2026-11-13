use super::*;
use crate::{OwnedStructFieldLayout as Field, OwnedStructScalarLayout as Scalar};
use std::collections::BTreeSet;

/// Bounded value cells plus exactly one owned Bytes resource, never pointer words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedResourceLayout {
    pub encoded: String,
    pub type_name: String,
    pub scalar_fields: Vec<String>,
    pub scalar_slots: Vec<usize>,
    pub resource_field: String,
    pub resource_slot: usize,
    pub outcome: Option<ResourceOutcomeLayout>,
}

impl MixedResourceLayout {
    pub fn parse(encoded: &str) -> Result<Self, String> {
        if encoded.len() > 2048 {
            return Err("mixed resource layout exceeds its bound".to_owned());
        }
        let layout = crate::parse_owned_struct_layout(encoded)?;
        Self::from_layout(encoded, layout)
    }

    pub(super) fn from_layout(
        encoded: &str,
        layout: crate::OwnedStructLayout,
    ) -> Result<Self, String> {
        YirApplicationSession::validate_identifier(&layout.type_name)?;
        if !(2..=17).contains(&layout.fields.len()) {
            return Err(
                "mixed resource State requires 1..=16 i64 fields and one Bytes field".to_owned(),
            );
        }
        let mut scalar_fields = Vec::new();
        let mut scalar_slots = Vec::new();
        let mut names = BTreeSet::new();
        let mut resource = None;
        for (slot, (name, kind)) in layout.fields.iter().enumerate() {
            YirApplicationSession::validate_identifier(name)?;
            if !names.insert(name) {
                return Err("duplicate mixed State field".to_owned());
            }
            match kind {
                Field::Scalar(Scalar::I64) => {
                    scalar_fields.push(name.clone());
                    scalar_slots.push(slot);
                }
                Field::Scalar(Scalar::Bytes) if resource.is_none() => {
                    resource = Some((name.clone(), slot))
                }
                _ => {
                    return Err(
                        "mixed State admits flat i64 fields and exactly one owned Bytes resource"
                            .to_owned(),
                    )
                }
            }
        }
        let (resource_field, resource_slot) =
            resource.ok_or("mixed State requires an owned Bytes resource")?;
        Ok(Self {
            encoded: encoded.to_owned(),
            type_name: layout.type_name,
            scalar_fields,
            scalar_slots,
            resource_field,
            resource_slot,
            outcome: None,
        })
    }

    pub fn scalar_index(&self, name: &str) -> Result<usize, String> {
        self.scalar_fields
            .iter()
            .position(|field| field == name)
            .ok_or_else(|| format!("unknown mixed State scalar field `{name}`"))
    }
}

pub(super) fn validate_roots(
    registration: &ResourceSessionRegistration,
    module: &YirModule,
    roots: [&YirFunction; 3],
) -> Result<MixedResourceLayout, String> {
    let mut selected = None;
    for (role, function) in roots.into_iter().enumerate() {
        let result = function
            .result
            .as_ref()
            .ok_or("mixed callback requires a result")?;
        let node = module
            .nodes
            .iter()
            .find(|n| n.name == result.node)
            .ok_or("missing mixed return node")?;
        let expected_type = match (&registration.result_type, &registration.state_type) {
            (Some(outcome), Some(state)) => format!("{outcome}<{state}, i64>"),
            _ => registration.state_type.clone().unwrap_or_default(),
        };
        if result.ty != expected_type
            || result.ownership != YirValueOwnership::Owned
            || node.op.module != "cpu"
            || !match node.op.instruction.as_str() {
                "return_owned_struct" => node.op.args.len() == 2,
                "tail_call_owned_struct_return" => {
                    crate::guarded_owned_call::parse_tail(&node.op.args).is_ok()
                }
                _ => false,
            }
        {
            return Err("mixed callback requires its declared owned State return".to_owned());
        }
        let layout = if let Some(outcome) = &registration.result_type {
            MixedResourceLayout::parse_outcome(
                &node.op.args[1],
                &registration.state_type.clone().unwrap(),
                outcome,
            )?
        } else {
            MixedResourceLayout::parse(&node.op.args[1])?
        };
        if Some(&layout.type_name) != registration.state_type.as_ref()
            || selected.as_ref().is_some_and(|old| old != &layout)
        {
            return Err("mixed callback State layout drift".to_owned());
        }
        let count = if role == 0 {
            0
        } else {
            layout.scalar_fields.len()
        };
        let p = &function.parameters;
        if p.len() != count + 2
            || p[0].ty != "ref Buffer"
            || p[0].ownership != YirValueOwnership::Borrowed
            || p[1..]
                .iter()
                .any(|p| p.ty != "i64" || p.ownership != YirValueOwnership::Value)
            || p.iter().map(|p| &p.node).collect::<BTreeSet<_>>().len() != p.len()
            || p.iter().map(|p| &p.name).collect::<BTreeSet<_>>().len() != p.len()
        {
            return Err("mixed callback requires borrowed Buffer, ordered i64 State fields (except open), then one i64 argument".to_owned());
        }
        if p[1..1 + count]
            .iter()
            .zip(&layout.scalar_fields)
            .any(|(p, name)| p.name.rsplit('.').next() != Some(name.as_str()))
        {
            return Err("mixed callback State parameter field order drift".to_owned());
        }
        selected = Some(layout);
    }
    Ok(selected.expect("three validated callbacks"))
}

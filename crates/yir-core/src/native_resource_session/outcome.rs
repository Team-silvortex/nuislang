use super::MixedResourceLayout;
use crate::{OwnedStructFieldLayout as Field, OwnedStructScalarLayout as Scalar};

/// Slots of a checked Result<State, i64>, derived from the owned union layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOutcomeLayout {
    pub ok_variant: String,
    pub err_variant: String,
    pub tag_slot: usize,
    pub error_slot: usize,
}

impl MixedResourceLayout {
    pub fn parse_outcome(encoded: &str, state: &str, result: &str) -> Result<Self, String> {
        if encoded.len() > 4096 {
            return Err("checked resource layout exceeds its bound".to_owned());
        }
        let union = crate::parse_owned_struct_layout(encoded)?;
        let ok_variant = format!("{result}.Ok");
        let err_variant = format!("{result}.Err");
        if union.type_name != format!("{}{result}", crate::OWNED_VARIANT_UNION_LAYOUT_PREFIX)
            || union.fields.len() != 3
        {
            return Err(
                "checked resource requires exactly Result.Ok(State) and Result.Err(i64)".to_owned(),
            );
        }
        let mut tag = None;
        let mut error = None;
        let mut success = None;
        let mut slot = 0;
        for (name, field) in union.fields {
            match (name.as_str(), field) {
                ("tag", Field::Scalar(Scalar::I64)) if tag.is_none() => {
                    tag = Some(slot);
                    slot += 1;
                }
                (_, Field::Struct(variant))
                    if name == ok_variant
                        && success.is_none()
                        && variant.type_name == ok_variant
                        && variant.fields.len() == 1 =>
                {
                    let (field, payload) = &variant.fields[0];
                    let Field::Struct(payload) = payload else {
                        return Err("checked Ok requires its mixed State payload".to_owned());
                    };
                    if field != "value" || payload.type_name != state {
                        return Err("checked Ok State identity drift".to_owned());
                    }
                    let mut layout = Self::from_layout(encoded, payload.clone())?;
                    layout.resource_slot += slot;
                    for scalar in &mut layout.scalar_slots {
                        *scalar += slot;
                    }
                    slot += payload.fields.len();
                    success = Some(layout);
                }
                (_, Field::Struct(variant))
                    if name == err_variant
                        && error.is_none()
                        && variant.type_name == err_variant
                        && variant.fields == [("value".to_owned(), Field::Scalar(Scalar::I64))] =>
                {
                    error = Some(slot);
                    slot += 1;
                }
                _ => return Err("checked resource Result layout drift".to_owned()),
            }
        }
        let mut layout = success.ok_or("missing checked Ok State")?;
        layout.outcome = Some(ResourceOutcomeLayout {
            ok_variant,
            err_variant,
            tag_slot: tag.ok_or("missing checked Result tag")?,
            error_slot: error.ok_or("missing checked Err code")?,
        });
        Ok(layout)
    }
}

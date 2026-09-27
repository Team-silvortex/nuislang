use crate::native_scalar_session::{ScalarKind, ScalarStateLayout};

const PREFIX: &str = "$value_record:";

/// One by-value argument assembled from explicitly ordered per-trip i64 leaves.
/// Only scoped multi-carry actions admit this descriptor; it is not a node name.
pub struct ScopedRecordInput<'a> {
    pub layout: ScalarStateLayout,
    pub operands: Vec<&'a str>,
}

impl<'a> ScopedRecordInput<'a> {
    pub fn parse(input: &'a str) -> Result<Option<Self>, String> {
        let Some(payload) = input.strip_prefix(PREFIX) else {
            return Ok(None);
        };
        let invalid = || {
            "invalid scoped value record: expected bounded i64 layout and exact leaf mappings"
                .to_owned()
        };
        let (encoded, inputs) = payload.split_once('|').ok_or_else(invalid)?;
        let layout = ScalarStateLayout::parse(encoded)?;
        let operands = inputs.split('|').collect::<Vec<_>>();
        if operands.len() != layout.fields().len()
            || layout
                .fields()
                .iter()
                .any(|(_, kind)| *kind != ScalarKind::I64)
        {
            return Err(invalid());
        }
        for operand in &operands {
            let name = crate::parse_loop_owned_struct_carry(operand)?
                .map(|(_, seed)| seed)
                .unwrap_or(operand);
            if *operand != "$current"
                && (name.is_empty()
                    || name.contains(['$', ':', '|'])
                    || name.chars().any(char::is_whitespace))
            {
                return Err(invalid());
            }
        }
        Ok(Some(Self { layout, operands }))
    }

    pub fn encode(layout: &str, operands: &[String]) -> Result<String, String> {
        if layout.contains('|') || operands.iter().any(|input| input.contains('|')) {
            return Err("scoped value record contains a reserved separator".into());
        }
        let encoded = format!("{PREFIX}{layout}|{}", operands.join("|"));
        ScopedRecordInput::parse(&encoded)?;
        Ok(encoded)
    }
}

pub fn scoped_input_leaves(input: &str) -> Result<Vec<&str>, String> {
    Ok(ScopedRecordInput::parse(input)?.map_or_else(|| vec![input], |record| record.operands))
}

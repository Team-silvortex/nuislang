use yir_core::{
    loop_carry_contract::ScopedRecordInput, native_scalar_session::ScalarStateLayout,
    ExecutionState, StructValue, Value,
};

enum Leaf {
    Current,
    Carry(usize),
    Capture(i64),
}

pub(super) struct RecordInput {
    layout: ScalarStateLayout,
    leaves: Vec<Leaf>,
}

impl RecordInput {
    pub(super) fn parse(input: &str, state: &ExecutionState) -> Result<Option<Self>, String> {
        let Some(record) = ScopedRecordInput::parse(input)? else {
            return Ok(None);
        };
        let leaves = record
            .operands
            .iter()
            .map(|input| {
                if *input == "$current" {
                    Ok(Leaf::Current)
                } else if let Some((index, _)) = yir_core::parse_loop_owned_struct_carry(input)? {
                    Ok(Leaf::Carry(index))
                } else if let Value::Int(value) = state.expect_value(input)? {
                    Ok(Leaf::Capture(*value))
                } else {
                    Err(format!("scoped record leaf `{input}` requires exact i64"))
                }
            })
            .collect::<Result<_, String>>()?;
        Ok(Some(Self {
            layout: record.layout,
            leaves,
        }))
    }

    pub(super) fn value(
        &self,
        current: i64,
        carries: Option<&StructValue>,
    ) -> Result<Value, String> {
        let words = self
            .leaves
            .iter()
            .map(|leaf| match leaf {
                Leaf::Current => Ok(current as u64),
                Leaf::Capture(value) => Ok(*value as u64),
                Leaf::Carry(index) => {
                    match carries.and_then(|carries| carries.fields.get(*index)) {
                        Some((_, Value::Int(value))) => Ok(*value as u64),
                        _ => Err("scoped record refers to an invalid i64 carry slot".into()),
                    }
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.layout.unpack(&words)
    }
}

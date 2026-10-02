use yir_core::{
    loop_carry_contract::ScopedRecordInput, native_scalar_session::ScalarStateLayout,
    ExecutionState, StructValue, Value,
};

enum Leaf {
    Current,
    Carry(usize),
    Capture(u64),
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
            .zip(record.layout.fields())
            .map(|(input, (_, kind))| {
                if *input == "$current" {
                    Ok(Leaf::Current)
                } else if let Some((index, _)) = yir_core::parse_loop_owned_struct_carry(input)? {
                    Ok(Leaf::Carry(index))
                } else {
                    kind.pack(state.expect_value(input)?).map(Leaf::Capture)
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
                Leaf::Capture(value) => Ok(*value),
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

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::native_scalar_session::ScalarKind;

    #[test]
    fn scoped_record_mixed_captures_snapshot_exact_bits_and_dynamic_i64_slots() {
        let layout = "S{b:bool;n:i32;f:f32;d:f64;i:i64;c:i64}";
        let descriptor = format!(
            "$value_record:{layout}|flag|tag|gain|value|$current|$owned_struct_carry:0:seed"
        );
        for (f32_bits, f64_bits) in [
            (0x8000_0000, 0x8000_0000_0000_0000),
            (0x7fc0_1234, 0x7ff8_0000_0000_4321),
        ] {
            let mut state = ExecutionState::default();
            state.values.extend([
                ("flag".into(), Value::Bool(true)),
                ("tag".into(), Value::I32(-17)),
                ("gain".into(), Value::F32(f32::from_bits(f32_bits))),
                ("value".into(), Value::F64(f64::from_bits(f64_bits))),
            ]);
            let record = RecordInput::parse(&descriptor, &state).unwrap().unwrap();
            state.values.clear();
            for (current, carry) in [(0, -9), (3, 17)] {
                let carries = StructValue {
                    type_name: "Words".into(),
                    fields: vec![("carry0".into(), Value::Int(carry))],
                };
                let Value::Struct(value) = record.value(current, Some(&carries)).unwrap() else {
                    panic!()
                };
                let kinds = [
                    ScalarKind::Bool,
                    ScalarKind::I32,
                    ScalarKind::F32,
                    ScalarKind::F64,
                    ScalarKind::I64,
                    ScalarKind::I64,
                ];
                let words = kinds
                    .into_iter()
                    .zip(value.fields)
                    .map(|(kind, (_, value))| kind.pack(&value).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(
                    words,
                    [
                        1,
                        -17_i64 as u64,
                        f32_bits as u64,
                        f64_bits,
                        current as u64,
                        carry as u64
                    ]
                );
            }
            assert!(record.value(0, None).is_err());
            assert!(RecordInput::parse(&descriptor, &state).is_err());
        }
    }

    #[test]
    fn scoped_record_capture_rejects_wrong_kinds_without_scalar_coercions() {
        let values = [
            Value::Bool(true),
            Value::I32(1),
            Value::Int(1),
            Value::F32(1.0),
            Value::F64(1.0),
        ];
        for (expected, kind) in ["bool", "i32", "i64", "f32", "f64"].into_iter().enumerate() {
            for (actual, value) in values.iter().enumerate() {
                let mut state = ExecutionState::default();
                state.values.insert("input".into(), value.clone());
                assert_eq!(
                    RecordInput::parse(&format!("$value_record:S{{x:{kind}}}|input"), &state)
                        .is_ok(),
                    expected == actual
                );
            }
        }
    }
}

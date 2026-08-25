pub const DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2: &str = "dynamic-pattern-payload-carry-v2";
pub const DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER: &str = "@dynamic-pattern-payload-carry";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicPatternPayloadCodec {
    I64,
    BoolAsI64,
}

impl DynamicPatternPayloadCodec {
    pub fn render(self) -> &'static str {
        match self {
            Self::I64 => "i64",
            Self::BoolAsI64 => "bool-as-i64",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "i64" => Ok(Self::I64),
            "bool-as-i64" => Ok(Self::BoolAsI64),
            other => Err(format!(
                "unsupported dynamic pattern payload codec `{other}`"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicPatternPayloadCarrySlot {
    pub carry_index: usize,
    pub codec: DynamicPatternPayloadCodec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicPatternPayloadCarryContract {
    pub slots: Vec<DynamicPatternPayloadCarrySlot>,
}

pub fn encode_dynamic_pattern_payload_carry_trailer(
    contract: &DynamicPatternPayloadCarryContract,
) -> Result<Vec<String>, String> {
    validate_slots(&contract.slots)?;
    let mut args = Vec::with_capacity(3 + contract.slots.len() * 2);
    args.push(DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER.to_owned());
    args.push(DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2.to_owned());
    args.push(contract.slots.len().to_string());
    for slot in &contract.slots {
        args.push(slot.carry_index.to_string());
        args.push(slot.codec.render().to_owned());
    }
    Ok(args)
}

pub fn split_dynamic_pattern_payload_carry_trailer(
    args: &[String],
) -> Result<(&[String], Option<DynamicPatternPayloadCarryContract>), String> {
    let mut markers = args
        .iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_str() == DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER);
    let Some((marker_index, _)) = markers.next() else {
        return Ok((args, None));
    };
    if markers.next().is_some() {
        return Err(
            "dynamic pattern payload carry trailer marker appears more than once".to_owned(),
        );
    }

    let trailer = &args[marker_index..];
    if trailer.len() < 3 {
        return Err("dynamic pattern payload carry trailer is truncated".to_owned());
    }
    if trailer[1] != DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2 {
        return Err(format!(
            "unsupported dynamic pattern payload carry protocol `{}`",
            trailer[1]
        ));
    }
    let slot_count = trailer[2].parse::<usize>().map_err(|_| {
        format!(
            "invalid dynamic pattern payload slot count `{}`",
            trailer[2]
        )
    })?;
    let expected_len = slot_count
        .checked_mul(2)
        .and_then(|len| len.checked_add(3))
        .ok_or_else(|| "dynamic pattern payload slot count overflows the trailer".to_owned())?;
    if trailer.len() != expected_len {
        return Err(format!(
            "dynamic pattern payload carry trailer declares {slot_count} slots but contains {} slot fields",
            trailer.len().saturating_sub(3)
        ));
    }

    let mut slots = Vec::with_capacity(slot_count);
    for fields in trailer[3..].chunks_exact(2) {
        let carry_index = fields[0].parse::<usize>().map_err(|_| {
            format!(
                "invalid dynamic pattern payload carry index `{}`",
                fields[0]
            )
        })?;
        slots.push(DynamicPatternPayloadCarrySlot {
            carry_index,
            codec: DynamicPatternPayloadCodec::parse(&fields[1])?,
        });
    }
    validate_slots(&slots)?;
    Ok((
        &args[..marker_index],
        Some(DynamicPatternPayloadCarryContract { slots }),
    ))
}

pub fn validate_dynamic_pattern_payload_carry_context(
    contract: &DynamicPatternPayloadCarryContract,
    entry_gate: &str,
    carry_count: usize,
) -> Result<(), String> {
    validate_slots(&contract.slots)?;
    if entry_gate != "pattern_carry0" {
        return Err(format!(
            "dynamic pattern payload carries require the `pattern_carry0` entry gate, found `{entry_gate}`"
        ));
    }
    for slot in &contract.slots {
        if slot.carry_index >= carry_count {
            return Err(format!(
                "dynamic pattern payload carry{} is unavailable because the loop has {carry_count} carries",
                slot.carry_index
            ));
        }
    }
    Ok(())
}

fn validate_slots(slots: &[DynamicPatternPayloadCarrySlot]) -> Result<(), String> {
    let mut previous = None;
    for slot in slots {
        if slot.carry_index == 0 {
            return Err(
                "dynamic pattern payload carry index 0 is reserved for the active-state carry"
                    .to_owned(),
            );
        }
        if previous.is_some_and(|index| slot.carry_index <= index) {
            return Err(
                "dynamic pattern payload carry indices must be unique and strictly ascending"
                    .to_owned(),
            );
        }
        previous = Some(slot.carry_index);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn round_trips_a_versioned_payload_carry_trailer() {
        let contract = DynamicPatternPayloadCarryContract {
            slots: vec![
                DynamicPatternPayloadCarrySlot {
                    carry_index: 1,
                    codec: DynamicPatternPayloadCodec::I64,
                },
                DynamicPatternPayloadCarrySlot {
                    carry_index: 2,
                    codec: DynamicPatternPayloadCodec::BoolAsI64,
                },
            ],
        };
        let mut encoded = args(&["loop-prefix", "carry-fields"]);
        encoded.extend(encode_dynamic_pattern_payload_carry_trailer(&contract).unwrap());

        let (prefix, decoded) = split_dynamic_pattern_payload_carry_trailer(&encoded).unwrap();

        assert_eq!(prefix, args(&["loop-prefix", "carry-fields"]));
        assert_eq!(decoded, Some(contract));
    }

    #[test]
    fn preserves_legacy_args_without_a_trailer() {
        let encoded = args(&["loop-prefix", "carry-fields"]);

        let (prefix, decoded) = split_dynamic_pattern_payload_carry_trailer(&encoded).unwrap();

        assert_eq!(prefix, encoded);
        assert_eq!(decoded, None);
    }

    #[test]
    fn rejects_duplicate_or_reserved_payload_slots() {
        let duplicate = args(&[
            "prefix",
            DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER,
            DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2,
            "2",
            "1",
            "i64",
            "1",
            "bool-as-i64",
        ]);
        assert!(split_dynamic_pattern_payload_carry_trailer(&duplicate)
            .unwrap_err()
            .contains("strictly ascending"));

        let reserved = DynamicPatternPayloadCarryContract {
            slots: vec![DynamicPatternPayloadCarrySlot {
                carry_index: 0,
                codec: DynamicPatternPayloadCodec::I64,
            }],
        };
        assert!(encode_dynamic_pattern_payload_carry_trailer(&reserved)
            .unwrap_err()
            .contains("reserved"));
    }

    #[test]
    fn rejects_unknown_or_truncated_wire_values() {
        for (encoded, expected) in [
            (
                args(&[
                    "prefix",
                    DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER,
                    "dynamic-pattern-payload-carry-v3",
                    "0",
                ]),
                "unsupported dynamic pattern payload carry protocol",
            ),
            (
                args(&[
                    "prefix",
                    DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER,
                    DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2,
                    "1",
                    "1",
                    "opaque",
                ]),
                "unsupported dynamic pattern payload codec",
            ),
            (
                args(&[
                    "prefix",
                    DYNAMIC_PATTERN_PAYLOAD_CARRY_TRAILER_MARKER,
                    DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2,
                    "1",
                    "1",
                ]),
                "declares 1 slots",
            ),
        ] {
            let error = split_dynamic_pattern_payload_carry_trailer(&encoded).unwrap_err();
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn validates_the_active_gate_and_available_physical_slots() {
        let contract = DynamicPatternPayloadCarryContract {
            slots: vec![DynamicPatternPayloadCarrySlot {
                carry_index: 1,
                codec: DynamicPatternPayloadCodec::BoolAsI64,
            }],
        };

        validate_dynamic_pattern_payload_carry_context(&contract, "pattern_carry0", 2).unwrap();
        assert!(
            validate_dynamic_pattern_payload_carry_context(&contract, "always", 2)
                .unwrap_err()
                .contains("entry gate")
        );
        assert!(
            validate_dynamic_pattern_payload_carry_context(&contract, "pattern_carry0", 1)
                .unwrap_err()
                .contains("unavailable")
        );
    }
}

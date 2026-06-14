use nuis_semantics::model::NirTypeRef;

use super::builtin_fields_packets::builtin_packet_struct_field_type;
use super::builtin_fields_states::builtin_state_struct_field_type;
use super::{bool_type, generic_named_type, i64_type, ref_type};

pub(crate) fn builtin_struct_field_type(type_name: &str, field: &str) -> Option<NirTypeRef> {
    if type_name == "Slice" {
        return match field {
            "buffer" => Some(ref_type("Buffer")),
            "start" | "len" => Some(i64_type()),
            _ => None,
        };
    }
    if type_name == "ByteSplit" {
        return match field {
            "before" | "after" => Some(generic_named_type("Slice", vec![i64_type()])),
            "index" => Some(i64_type()),
            "found" => Some(bool_type()),
            _ => None,
        };
    }
    builtin_packet_struct_field_type(type_name, field)
        .or_else(|| builtin_state_struct_field_type(type_name, field))
}

use nuis_semantics::model::NirTypeRef;

use super::builtin_fields_packets::builtin_packet_struct_field_type;
use super::builtin_fields_states::builtin_state_struct_field_type;
use super::{i64_type, ref_type};

pub(crate) fn builtin_struct_field_type(type_name: &str, field: &str) -> Option<NirTypeRef> {
    if type_name == "Slice" {
        return match field {
            "buffer" => Some(ref_type("Buffer")),
            "start" | "len" => Some(i64_type()),
            _ => None,
        };
    }
    builtin_packet_struct_field_type(type_name, field)
        .or_else(|| builtin_state_struct_field_type(type_name, field))
}

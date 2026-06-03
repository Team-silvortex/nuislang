use nuis_semantics::model::NirTypeRef;

use super::builtin_fields_packets::builtin_packet_struct_field_type;
use super::builtin_fields_states::builtin_state_struct_field_type;

pub(crate) fn builtin_struct_field_type(type_name: &str, field: &str) -> Option<NirTypeRef> {
    builtin_packet_struct_field_type(type_name, field)
        .or_else(|| builtin_state_struct_field_type(type_name, field))
}

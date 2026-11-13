use super::*;

#[test]
fn native_session_resource_boundary_is_not_implied_by_scalar_record_admission() {
    for instruction in [
        "param_value_struct",
        "return_owned_struct",
        "call_owned_struct",
    ] {
        assert!(admission::admitted(instruction));
    }
    // This is the current rejection contract, not an implemented resource ABI.
    for instruction in [
        "alloc_buffer",
        "load_at",
        "store_at",
        "copy_buffer_owned",
        "move_owned_bytes",
        "drop_owned_bytes",
        "free",
        "dispatch",
    ] {
        assert!(!admission::admitted(instruction), "{instruction}");
    }
    for layout in ["State{payload:Bytes}", "State{payload:Buffer}"] {
        assert!(ScalarStateLayout::parse(layout).is_err());
    }
}

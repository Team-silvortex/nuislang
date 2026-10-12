#[test]
fn native_return_numeric_field_vector_exact_layout_guarded_order_and_policy_vetoes() {
    super::return_private_fields::check_layout(
        include_str!("../tests/native_application_bridge/return_numeric_field_vector.ns"),
        "{owner:i64;child_0:i32;child_1:FLOAT;child_2:i64}",
    );
}

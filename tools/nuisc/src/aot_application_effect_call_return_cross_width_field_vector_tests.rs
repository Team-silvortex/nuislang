#[allow(dead_code)]
#[path = "../tests/native_application_bridge/return_cross_width_field_vector_oracle.rs"]
mod oracle;

#[test]
fn native_return_cross_width_field_vector_exact_layout_guarded_order_and_policy_vetoes() {
    for owner in ["f32", "f64"] {
        let child = oracle::child_type(owner);
        super::return_private_fields::check_typed_layout(
            &oracle::source(owner),
            &format!("{{owner:{owner};child_0:i32;child_1:{child};child_2:bool;child_3:bool}}"),
            owner,
        );
    }
}

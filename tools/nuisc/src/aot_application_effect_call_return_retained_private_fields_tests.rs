#[test]
fn native_return_retained_private_fields_exact_layout_guarded_order_and_policy_vetoes() {
    super::return_private_fields::check_source(include_str!(
        "../tests/native_application_bridge/return_retained_private_fields.ns"
    ));
}

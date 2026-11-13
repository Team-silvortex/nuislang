use super::state_return_logical_exit_argument_print::check_delivery;

#[test]
fn state_return_eager_exit_fresh_cache_and_source_free_delivery_preserve_both_state_words() {
    check_delivery(
        include_str!("../../../nuisc/tests/native_application_bridge/state_return_eager_exit_argument_print.ns"),
        "state_return_eager_exit",
    );
}

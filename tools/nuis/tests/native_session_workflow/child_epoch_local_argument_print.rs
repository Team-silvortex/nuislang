#[test]
fn child_epoch_local_argument_print_fresh_cache_and_source_free_delivery() {
    super::epoch_local_argument_print::delivery(
        include_str!(
            "../../../nuisc/tests/native_application_bridge/child_epoch_local_argument_print.ns"
        ),
        4,
        64,
        "child_epoch_local_argument_print_delivery",
    );
}

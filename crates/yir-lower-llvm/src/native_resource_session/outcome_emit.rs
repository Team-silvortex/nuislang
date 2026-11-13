use yir_core::native_resource_session::ResourceOutcomeLayout;

/// The error channel is separate from staged State, including Err(0).
pub(super) fn dispatch(layout: &ResourceOutcomeLayout) -> String {
    let tag_slot = layout.tag_slot;
    let error_slot = layout.error_slot;
    let ok = crate::variant_select::variant_tag_value(&layout.ok_variant);
    let err = crate::variant_select::variant_tag_value(&layout.err_variant);
    format!(
        r#"  %variant = call i64 @nuis_scheduler_owned_aggregate_get_v1(ptr %aggregate, i64 {tag_slot})
  %is_ok = icmp eq i64 %variant, {ok}
  br i1 %is_ok, label %take, label %check_error
check_error:
  %is_err = icmp eq i64 %variant, {err}
  br i1 %is_err, label %application_error, label %bad_variant
application_error:
  %error = call i64 @nuis_scheduler_owned_aggregate_get_v1(ptr %aggregate, i64 {error_slot})
  store i64 %error, ptr %out_error, align 8
  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr %aggregate)
  ret i32 4
bad_variant:
  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr %aggregate)
  ret i32 3
"#
    )
}

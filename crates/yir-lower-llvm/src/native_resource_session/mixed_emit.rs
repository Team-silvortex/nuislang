use yir_core::native_resource_session::{
    MixedResourceLayout, ResourceSessionRegistration, CHECKED_CAPABILITY_TAG, MIXED_CAPABILITY_TAG,
};

pub(super) fn callback(
    function: &str,
    symbol: &str,
    registration: &ResourceSessionRegistration,
    layout: &MixedResourceLayout,
    role: usize,
) -> String {
    let min = registration.min_words;
    let max = registration.max_words;
    let count = layout.scalar_fields.len();
    let input_count = if role == 0 { 0 } else { count };
    let resource_slot = layout.resource_slot;
    let tag = if layout.outcome.is_some() {
        CHECKED_CAPABILITY_TAG
    } else {
        MIXED_CAPABILITY_TAG
    };
    let extra_parameter = if layout.outcome.is_some() {
        ", ptr %out_error"
    } else {
        ""
    };
    let extra_validation = if layout.outcome.is_some() {
        "  %errorptr = icmp ne ptr %out_error, null\n  %errorbits = ptrtoint ptr %out_error to i64\n  %erroralign = and i64 %errorbits, 7\n  %erroraligned = icmp eq i64 %erroralign, 0\n  %errorvalid = and i1 %errorptr, %erroraligned\n  %ready = and i1 %base_ready, %errorvalid\n"
    } else {
        "  %ready = and i1 %base_ready, true\n"
    };
    let dispatch = layout
        .outcome
        .as_ref()
        .map(super::outcome_emit::dispatch)
        .unwrap_or_else(|| "  br label %take\n".to_owned());
    let mut loads = String::new();
    let mut arguments = String::new();
    for i in 0..input_count {
        loads.push_str(&format!("  %in{i} = getelementptr i64, ptr %scalars, i64 {i}\n  %scalar{i} = load i64, ptr %in{i}, align 8\n"));
        arguments.push_str(&format!(", i64 %scalar{i}"));
    }
    let mut publish = String::new();
    for (i, slot) in layout.scalar_slots.iter().enumerate() {
        publish.push_str(&format!("  %value{i} = call i64 @nuis_scheduler_owned_aggregate_get_v1(ptr %aggregate, i64 {slot})\n  %scalar_out{i} = getelementptr i64, ptr %out_scalars, i64 {i}\n  store i64 %value{i}, ptr %scalar_out{i}, align 8\n"));
    }
    // The admitted return owns one complete flat record. Its layout and slot kinds
    // are proven before emission; inspect it only through the allocation ABI.
    format!(
        r#"
define i32 @{symbol}(i64 %capability, ptr %buffer, i64 %words, ptr %scalars, i64 %scalarc, i64 %argument, ptr %out, i64 %outc, ptr %out_scalars, i64 %out_scalarc{extra_parameter}) {{
entry:
  %tag = icmp eq i64 %capability, {tag}
  %lower = icmp uge i64 %words, {min}
  %upper = icmp ule i64 %words, {max}
  %shape = icmp eq i64 %words, %outc
  %inc = icmp eq i64 %scalarc, {input_count}
  %outcount = icmp eq i64 %out_scalarc, {count}
  %inptr = icmp ne ptr %buffer, null
  %outptr = icmp ne ptr %out, null
  %scalarptr = icmp ne ptr %scalars, null
  %outscalarptr = icmp ne ptr %out_scalars, null
  %inbits = ptrtoint ptr %buffer to i64
  %outbits = ptrtoint ptr %out to i64
  %scalarbits = ptrtoint ptr %scalars to i64
  %outscalarbits = ptrtoint ptr %out_scalars to i64
  %bits1 = or i64 %inbits, %outbits
  %bits2 = or i64 %scalarbits, %outscalarbits
  %bits = or i64 %bits1, %bits2
  %alignment = and i64 %bits, 7
  %aligned = icmp eq i64 %alignment, 0
  %a = and i1 %tag, %lower
  %b = and i1 %upper, %shape
  %c = and i1 %inptr, %outptr
  %d = and i1 %inc, %outcount
  %e = and i1 %scalarptr, %outscalarptr
  %f = and i1 %a, %b
  %g = and i1 %c, %d
  %h = and i1 %e, %aligned
  %i = and i1 %f, %g
  %base_ready = and i1 %i, %h
{extra_validation}
  br i1 %ready, label %invoke, label %reject
reject:
  ret i32 1
invoke:
{loads}  %aggregate_bits = call i64 @nuis_fn_{function}(ptr %buffer, i64 %words{arguments}, i64 %argument)
  %aggregate = inttoptr i64 %aggregate_bits to ptr
  %aggregate_present = icmp ne ptr %aggregate, null
  br i1 %aggregate_present, label %dispatch, label %allocation_failed
allocation_failed:
  ret i32 2
dispatch:
{dispatch}
take:
  %blob = call ptr @nuis_scheduler_owned_aggregate_take_blob_v1(ptr %aggregate, i64 {resource_slot})
  %nonnull = icmp ne ptr %blob, null
  br i1 %nonnull, label %validate, label %malformed
validate:
  %bytes = mul i64 %words, 8
  %length = call i64 @nuis_scheduler_owned_blob_len_v1(ptr %blob)
  %data = call ptr @nuis_scheduler_owned_blob_data_v1(ptr %blob)
  %same = icmp eq i64 %length, %bytes
  %present = icmp ne ptr %data, null
  %valid = and i1 %same, %present
  br i1 %valid, label %copy, label %malformed
malformed:
  call void @nuis_scheduler_owned_blob_drop_v1(ptr %blob)
  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr %aggregate)
  ret i32 3
copy:
  br label %loop
loop:
  %index = phi i64 [ 0, %copy ], [ %next, %body ]
  %more = icmp ult i64 %index, %words
  br i1 %more, label %body, label %release
body:
  %src = getelementptr i64, ptr %data, i64 %index
  %value = load i64, ptr %src, align 8
  %dst = getelementptr i64, ptr %out, i64 %index
  store i64 %value, ptr %dst, align 8
  %next = add i64 %index, 1
  br label %loop
release:
{publish}  call void @nuis_scheduler_owned_blob_drop_v1(ptr %blob)
  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr %aggregate)
  ret i32 0
}}
"#
    )
}

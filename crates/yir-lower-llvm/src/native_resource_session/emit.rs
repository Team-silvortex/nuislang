use yir_core::native_resource_session::{ResourceSessionRegistration, CAPABILITY_TAG};

pub(super) fn callback(
    function: &str,
    symbol: &str,
    registration: &ResourceSessionRegistration,
) -> String {
    let min = registration.min_words;
    let max = registration.max_words;
    // The tag is an ABI discriminator, not a secret. Static binding/provenance
    // supplies authority. Nonnull extents are the trusted host's responsibility.
    format!(
        r#"
define i32 @{symbol}(i64 %capability, ptr %buffer, i64 %words, i64 %argument, ptr %out, i64 %outc) {{
entry:
  %tag = icmp eq i64 %capability, {CAPABILITY_TAG}
  %lower = icmp uge i64 %words, {min}
  %upper = icmp ule i64 %words, {max}
  %shape = icmp eq i64 %words, %outc
  %inptr = icmp ne ptr %buffer, null
  %outptr = icmp ne ptr %out, null
  %inbits = ptrtoint ptr %buffer to i64
  %outbits = ptrtoint ptr %out to i64
  %bits = or i64 %inbits, %outbits
  %alignment = and i64 %bits, 7
  %aligned = icmp eq i64 %alignment, 0
  %a = and i1 %tag, %lower
  %b = and i1 %upper, %shape
  %c = and i1 %inptr, %outptr
  %d = and i1 %a, %b
  %e = and i1 %c, %aligned
  %ready = and i1 %d, %e
  br i1 %ready, label %invoke, label %reject
reject:
  ret i32 1
invoke:
  %blob = call ptr @nuis_fn_{function}(ptr %buffer, i64 %words, i64 %argument)
  %nonnull = icmp ne ptr %blob, null
  br i1 %nonnull, label %validate, label %allocation_failed
allocation_failed:
  ret i32 2
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
  call void @nuis_scheduler_owned_blob_drop_v1(ptr %blob)
  ret i32 0
}}
"#
    )
}

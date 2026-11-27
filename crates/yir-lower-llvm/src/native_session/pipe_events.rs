use crate::emit_utils::{fresh_block, fresh_global, fresh_reg, llvm_c_string_bytes};
use crate::types::{LlvmLoweringState, LlvmValueRef, StructLlvmValueRef};
use yir_core::Node;

/// The call is an observable effect, not a print or a reconstructed post-run
/// receipt. The synchronous observer copies the SSA words; no address escapes.
pub(super) fn emit(
    n: &Node,
    packet: &StructLlvmValueRef,
    s: &mut LlvmLoweringState,
) -> Result<(), String> {
    if n.name.len() > yir_core::native_scalar_session::MAX_PIPE_SITE_BYTES {
        return Err("native Data pipe site exceeds its name bound".into());
    }
    let site = fresh_global(&mut s.next_global);
    let (bytes, len) = llvm_c_string_bytes(&n.name);
    s.globals.push(format!(
        "{site} = private unnamed_addr constant [{len} x i8] c\"{bytes}\""
    ));
    let words = fresh_reg(&mut s.next_reg);
    let count = packet.fields.len();
    s.body
        .push(format!("  {words} = alloca i64, i64 {count}, align 8"));
    for (i, (_, value)) in packet.fields.iter().enumerate() {
        let LlvmValueRef::I64(value) = value else {
            return Err("native Data pipe requires i64 packet fields".into());
        };
        let slot = fresh_reg(&mut s.next_reg);
        s.body.push(format!(
            "  {slot} = getelementptr inbounds i64, ptr {words}, i64 {i}"
        ));
        s.body
            .push(format!("  store i64 {value}, ptr {slot}, align 8"));
    }
    let status = fresh_reg(&mut s.next_reg);
    let ok = fresh_reg(&mut s.next_reg);
    let next = fresh_block(&mut s.next_block, "pipe.cont");
    let failed = fresh_block(&mut s.next_block, "pipe.fail");
    s.body.push(format!("  {status} = call i32 @nuis_native_data_pipe_event_v1(ptr {site}, i64 {}, ptr {words}, i64 {count})", n.name.len()));
    s.body.extend([
        format!("  {ok} = icmp eq i32 {status}, 0"),
        format!("  br i1 {ok}, label %{next}, label %{failed}"),
        format!("{failed}:"),
        "  call void @llvm.trap()".into(),
        "  unreachable".into(),
        format!("{next}:"),
    ]);
    Ok(())
}

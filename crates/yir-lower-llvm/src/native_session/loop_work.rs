use crate::{fresh_block, fresh_reg};

/// Experimental native-session policy, independent of the per-loop 65536 bound.
pub const DEFAULT_LOOP_WORK_LIMIT: u64 = 1_048_576;
pub(crate) const COUNTER_PARAMETER: &str = "ptr %nuis_loop_work";

// Reserve only after induction admission, before the loop body. The counter is
// owned by the exported callback's stack frame and never reset by a helper.
pub(super) fn reserve(
    trips: &str,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    next_block: &mut usize,
) {
    let remaining = fresh_reg(next_reg);
    let enough = fresh_reg(next_reg);
    let rest = fresh_reg(next_reg);
    let accepted = fresh_block(next_block, "native_loop_work_admitted");
    let rejected = fresh_block(next_block, "native_loop_work_rejected");
    body.extend([
        format!("  {remaining} = load i64, ptr %nuis_loop_work, align 8"),
        format!("  {enough} = icmp uge i64 {remaining}, {trips}"),
        format!("  br i1 {enough}, label %{accepted}, label %{rejected}"),
        format!("{rejected}:"),
        "  call void @llvm.trap()".to_owned(),
        "  unreachable".to_owned(),
        format!("{accepted}:"),
        format!("  {rest} = sub i64 {remaining}, {trips}"),
        format!("  store i64 {rest}, ptr %nuis_loop_work, align 8"),
    ]);
}

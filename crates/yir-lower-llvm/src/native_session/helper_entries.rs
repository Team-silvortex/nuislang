use crate::{fresh_block, fresh_reg};

/// Inclusive YIR function entries per callback, independent of loop reservations.
pub const DEFAULT_HELPER_ENTRY_LIMIT: u64 = 1_048_576;
pub(crate) const HELPER_ENTRY_PARAMETER: &str = "ptr %nuis_helper_entries";

pub(crate) fn enter(body: &mut Vec<String>, next_reg: &mut usize, next_block: &mut usize) {
    let remaining = fresh_reg(next_reg);
    let enough = fresh_reg(next_reg);
    let rest = fresh_reg(next_reg);
    let accepted = fresh_block(next_block, "native_helper_entry_admitted");
    let rejected = fresh_block(next_block, "native_helper_entry_rejected");
    body.extend([
        format!("  {remaining} = load i64, ptr %nuis_helper_entries, align 8"),
        format!("  {enough} = icmp uge i64 {remaining}, 1"),
        format!("  br i1 {enough}, label %{accepted}, label %{rejected}"),
        format!("{rejected}:"),
        "  call void @llvm.trap()".to_owned(),
        "  unreachable".to_owned(),
        format!("{accepted}:"),
        format!("  {rest} = sub i64 {remaining}, 1"),
        format!("  store i64 {rest}, ptr %nuis_helper_entries, align 8"),
    ]);
}

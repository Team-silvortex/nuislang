use super::{fresh_reg, LlvmLoweringState, LlvmValueRef, MutexScalarKind};

pub(crate) fn mutex_scalar_kind(value: &LlvmValueRef) -> Option<MutexScalarKind> {
    match value {
        LlvmValueRef::I32(_) => Some(MutexScalarKind::I32),
        LlvmValueRef::I64(_) => Some(MutexScalarKind::I64),
        _ => None,
    }
}

pub(crate) fn mutex_permit_scalar_kind(ty: &str) -> Option<MutexScalarKind> {
    let payload = ty.strip_prefix("MutexPermit<")?.strip_suffix('>')?;
    match payload {
        "i32" => Some(MutexScalarKind::I32),
        "i64" => Some(MutexScalarKind::I64),
        _ => None,
    }
}

pub(crate) fn emit_mutex_new(
    value: &LlvmValueRef,
    kind: MutexScalarKind,
    state: &mut LlvmLoweringState,
) -> String {
    match (kind, value) {
        (MutexScalarKind::I32, LlvmValueRef::I32(value)) => {
            let bits = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {bits} = sext i32 {value} to i64"));
            let handle = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {handle} = call i64 @nuis_scheduler_mutex_new_scalar_v1(i64 {bits}, i64 {})",
                kind.runtime_tag()
            ));
            handle
        }
        (MutexScalarKind::I64, LlvmValueRef::I64(value)) => {
            let handle = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {handle} = call i64 @nuis_scheduler_mutex_new_i64_v1(i64 {value})"
            ));
            handle
        }
        _ => unreachable!("mutex scalar kind must match its LLVM value"),
    }
}

pub(crate) fn emit_mutex_value(
    guard: &str,
    kind: MutexScalarKind,
    state: &mut LlvmLoweringState,
) -> LlvmValueRef {
    let bits = fresh_reg(&mut state.next_reg);
    match kind {
        MutexScalarKind::I32 => {
            state.body.push(format!(
                "  {bits} = call i64 @nuis_scheduler_mutex_value_scalar_v1(i64 {guard}, i64 {})",
                kind.runtime_tag()
            ));
            let value = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {value} = trunc i64 {bits} to i32"));
            LlvmValueRef::I32(value)
        }
        MutexScalarKind::I64 => {
            state.body.push(format!(
                "  {bits} = call i64 @nuis_scheduler_mutex_value_i64_v1(i64 {guard})"
            ));
            LlvmValueRef::I64(bits)
        }
    }
}

pub(crate) fn emit_mutex_replace(
    guard: &str,
    replacement: &LlvmValueRef,
    kind: MutexScalarKind,
    state: &mut LlvmLoweringState,
) -> Option<LlvmValueRef> {
    match (kind, replacement) {
        (MutexScalarKind::I32, LlvmValueRef::I32(value)) => {
            let bits = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {bits} = sext i32 {value} to i64"));
            let old_bits = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {old_bits} = call i64 @nuis_scheduler_mutex_lease_replace_scalar_v1(i64 {guard}, i64 {bits}, i64 {})",
                kind.runtime_tag()
            ));
            let old = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {old} = trunc i64 {old_bits} to i32"));
            Some(LlvmValueRef::I32(old))
        }
        (MutexScalarKind::I64, LlvmValueRef::I64(value)) => {
            let old_bits = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {old_bits} = call i64 @nuis_scheduler_mutex_lease_replace_i64_v1(i64 {guard}, i64 {value})"
            ));
            Some(LlvmValueRef::I64(old_bits))
        }
        _ => None,
    }
}

impl MutexScalarKind {
    pub(crate) fn runtime_tag(self) -> i64 {
        match self {
            Self::I32 => 1,
            Self::I64 => 2,
        }
    }

    pub(crate) fn staged_zero(self) -> LlvmValueRef {
        match self {
            Self::I32 => LlvmValueRef::I32("0".to_owned()),
            Self::I64 => LlvmValueRef::I64("0".to_owned()),
        }
    }
}

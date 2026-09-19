//! Experimental static CPU bridge for registered, self-contained scalar callbacks.
//! This is not a session scheduler or a replacement for provider admission.

pub use yir_core::native_scalar_session::{
    ScalarKind, ScalarStateLayout, CONTRACT, MAX_SCALAR_SLOTS,
};
use yir_core::YirModule;

mod admission;
pub(crate) mod aggregates;
mod calls;
mod emit;
mod function;
pub(crate) mod helper_entries;
mod loop_work;
pub(crate) mod loops;

pub use helper_entries::DEFAULT_HELPER_ENTRY_LIMIT;
pub(crate) use helper_entries::HELPER_ENTRY_PARAMETER;
pub(crate) use loop_work::COUNTER_PARAMETER;
pub use loop_work::DEFAULT_LOOP_WORK_LIMIT;

#[derive(Debug, Clone)]
pub struct CallbackExport {
    pub role: &'static str,
    pub function: String,
    pub symbol: String,
    pub arguments: Vec<ScalarKind>,
}

#[derive(Debug)]
pub struct NativeSessionBridge {
    /// Complete LLVM unit containing admitted roots, reachable scalar helpers and bridges.
    pub llvm_ir: String,
    pub session_id: String,
    pub callbacks: Vec<CallbackExport>,
    /// ABI slot order, including nested field paths relative to the state parameter.
    pub state_fields: Vec<(String, ScalarKind)>,
    pub state_layout: ScalarStateLayout,
    /// Reserved induction iterations allowed per invocation, not elapsed time or node fuel.
    pub loop_work_limit: u64,
    /// Dynamic YIR function entries, including roots and outlined helpers.
    pub helper_entry_limit: u64,
}

/// Emit static functions with ABI `i32(args: ptr, argc: i64, out: ptr, outc: i64)`.
/// Each slot is one native-endian u64. Status 0 means success, 1 invalid shape,
/// and 2 noncanonical input. Rejections occur before callback entry or output writes.
/// Nonnull pointers must refer to the supplied readable/writable extents. They may
/// overlap: all inputs are read before outputs are written. No pointer escapes.
/// Native traps are process failures, not catchable errors, retries or fuel limits.
pub fn emit_registered(module: &YirModule, id: &str) -> Result<NativeSessionBridge, String> {
    emit_registered_with_loop_work_limit(module, id, DEFAULT_LOOP_WORK_LIMIT)
}

/// Bake an inclusive loop-work reservation limit into each exported callback.
/// Every invocation owns a fresh counter shared by all synchronous helpers. Zero
/// permits loop-free and zero-trip paths. Early exits do not refund reservations;
/// exhaustion traps before the rejected loop body, without publishing output.
/// The independent default function-entry limit still applies.
pub fn emit_registered_with_loop_work_limit(
    module: &YirModule,
    id: &str,
    loop_work_limit: u64,
) -> Result<NativeSessionBridge, String> {
    emit_registered_with_work_limits(module, id, loop_work_limit, DEFAULT_HELPER_ENTRY_LIMIT)
}

/// Bake independent loop-reservation and function-entry limits into each callback.
/// Roots and all selected synchronous YIR helpers consume one entry before their
/// bodies run, including guard helpers that immediately return a neutral value.
/// Arguments have already been evaluated in the caller. Zero entries reject even
/// a loop-free root; calls not entered do not consume entries. Neither
/// counter is refunded, captured by tasks, or shared across exported invocations.
/// These producer policies do not promise elapsed-time, memory or node-work limits.
pub fn emit_registered_with_work_limits(
    module: &YirModule,
    id: &str,
    loop_work_limit: u64,
    helper_entry_limit: u64,
) -> Result<NativeSessionBridge, String> {
    let (selected, callbacks, state_layout) = admission::select(module, id)?;
    let state_fields = state_layout.fields().to_vec();
    let mut llvm_ir = crate::emit_native_scalar_module(&selected)?;
    for callback in &callbacks {
        llvm_ir.push_str(&emit::callback(
            callback,
            state_fields.len(),
            loop_work_limit,
            helper_entry_limit,
        ));
    }
    Ok(NativeSessionBridge {
        llvm_ir,
        session_id: id.to_owned(),
        callbacks,
        state_fields,
        state_layout,
        loop_work_limit,
        helper_entry_limit,
    })
}

//! Experimental static CPU bridge for registered, self-contained scalar callbacks.
//! This is not a session scheduler or a replacement for provider admission.

pub use yir_core::native_scalar_session::{
    ScalarKind, ScalarStateLayout, CONTRACT, MAX_SCALAR_SLOTS,
};
use yir_core::YirModule;

mod admission;
mod calls;
mod emit;
mod function;
pub(crate) mod loops;

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
}

/// Emit static functions with ABI `i32(args: ptr, argc: i64, out: ptr, outc: i64)`.
/// Each slot is one native-endian u64. Status 0 means success, 1 invalid shape,
/// and 2 noncanonical input. Rejections occur before callback entry or output writes.
/// Nonnull pointers must refer to the supplied readable/writable extents. They may
/// overlap: all inputs are read before outputs are written. No pointer escapes.
/// Native traps are process failures, not catchable errors, retries or fuel limits.
pub fn emit_registered(module: &YirModule, id: &str) -> Result<NativeSessionBridge, String> {
    let (selected, callbacks, state_layout) = admission::select(module, id)?;
    let state_fields = state_layout.fields().to_vec();
    let mut llvm_ir = crate::emit_native_scalar_module(&selected)?;
    for callback in &callbacks {
        llvm_ir.push_str(&emit::callback(callback, state_fields.len()));
    }
    Ok(NativeSessionBridge {
        llvm_ir,
        session_id: id.to_owned(),
        callbacks,
        state_fields,
        state_layout,
    })
}

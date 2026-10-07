//! Experimental static CPU bridge for registered, self-contained scalar callbacks.
//! This is not a session scheduler or a replacement for provider admission.

pub use yir_core::native_scalar_session::{
    ScalarKind, ScalarStateLayout, CONTRACT, MAX_SCALAR_SLOTS,
};
use yir_core::YirModule;

mod admission;
pub(crate) mod aggregate_values;
pub(crate) mod aggregates;
mod build_policy;
mod calls;
mod emit;
mod function;
pub(crate) mod helper_entries;
mod literal_prints;
mod loop_work;
pub(crate) mod loops;
pub(crate) mod value_transport;

pub use build_policy::{
    LiteralPrintBuildPolicy, LITERAL_PRINT_BUILD_CONTRACT, MAX_BUILD_POLICY_SITE_BYTES,
    MAX_BUILD_POLICY_TOKEN_BYTES,
};
pub use helper_entries::DEFAULT_HELPER_ENTRY_LIMIT;
pub(crate) use helper_entries::HELPER_ENTRY_PARAMETER;
pub use literal_prints::{LiteralPrintPolicy, MAX_LITERAL_PRINT_SITES};
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
    /// Explicitly checked static literal-print sites; empty for the default pure bridge.
    pub literal_print_sites: Vec<String>,
}

impl NativeSessionBridge {
    /// Conservative bound, not a promise about I/O time or rollback on traps.
    pub fn max_literal_prints_per_invocation(&self) -> u128 {
        self.literal_print_sites.len() as u128 * u128::from(self.helper_entry_limit)
    }
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
    emit_with_policy(module, id, loop_work_limit, helper_entry_limit, None)
}

/// Opt in to bounded literal i64 prints at explicitly named selected YIR nodes.
/// No computed prints, implicit helper grants, resources or arbitrary effects are
/// admitted. Guarded prints require an exact bool. Existing input preflight and
/// fresh work counters apply; successful prints are not rolled back after a trap.
pub fn emit_registered_with_literal_prints(
    module: &YirModule,
    id: &str,
    policy: &LiteralPrintPolicy,
    loop_work_limit: u64,
    helper_entry_limit: u64,
) -> Result<NativeSessionBridge, String> {
    emit_with_policy(
        module,
        id,
        loop_work_limit,
        helper_entry_limit,
        Some(policy),
    )
}

fn emit_with_policy(
    module: &YirModule,
    id: &str,
    loop_work_limit: u64,
    helper_entry_limit: u64,
    policy: Option<&LiteralPrintPolicy>,
) -> Result<NativeSessionBridge, String> {
    let (selected, callbacks, state_layout) = admission::select(module, id, policy)?;
    let literal_print_sites = policy
        .map(|policy| {
            policy.selected_sites(
                &selected
                    .nodes
                    .iter()
                    .filter(|node| matches!(node.op.instruction.as_str(), "print" | "guard_print"))
                    .map(|node| node.name.clone())
                    .collect(),
            )
        })
        .transpose()?
        .unwrap_or_default();
    let state_fields = state_layout.fields().to_vec();
    let roots = callbacks
        .iter()
        .map(|callback| callback.function.as_str())
        .collect::<Vec<_>>();
    let mut llvm_ir = crate::emit_native_scalar_module(&selected, &roots)?;
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
        literal_print_sites,
    })
}

use super::{LiteralPrintPolicy, NativeSessionBridge};
use yir_core::YirModule;

/// Explicit selected sites. Strict native emission rechecks exact i64/bool
/// operands, the admitted closure and effect order; no other effects are granted.
#[derive(Debug, Clone)]
pub struct I64PrintPolicy {
    pub(super) inner: LiteralPrintPolicy,
}

impl I64PrintPolicy {
    pub fn new<I, S>(sites: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut inner = LiteralPrintPolicy::new(sites)?;
        inner.i64_values = true;
        Ok(Self { inner })
    }
}

pub fn emit_registered_with_i64_prints(
    module: &YirModule,
    id: &str,
    policy: &I64PrintPolicy,
    loop_work_limit: u64,
    helper_entry_limit: u64,
) -> Result<NativeSessionBridge, String> {
    super::emit_with_policy(
        module,
        id,
        loop_work_limit,
        helper_entry_limit,
        Some(&policy.inner),
    )
}

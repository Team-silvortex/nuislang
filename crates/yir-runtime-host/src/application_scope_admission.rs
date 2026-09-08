/// Minimal host-scope policy used at callback and finalization boundaries.
/// Implementations decide admission only: they do not perform cleanup, complete
/// devices, publish results or receive a registry/transport. Callers remain
/// responsible for lifecycle validation and their own finalization operation.
pub(crate) trait ScopeAdmission {
    fn checkpoint(&self) -> Result<(), String>;
    fn admit_finalization(&self) -> Result<(), String>;

    /// Seal admission before dropping a failed scope's provider. A drain request
    /// is only intent; the scope must independently validate its transport.
    fn admit_abandonment(&self) -> ScopeAbandonment {
        ScopeAbandonment::Drop
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScopeAbandonment {
    Drop,
    Drain,
}

/// Existing synchronous callers have no additional host admission policy.
/// This does not bypass application or provider completion checks.
impl ScopeAdmission for () {
    fn checkpoint(&self) -> Result<(), String> {
        Ok(())
    }
    fn admit_finalization(&self) -> Result<(), String> {
        Ok(())
    }
}

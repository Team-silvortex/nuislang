use crate::NativeSessionBindings;
use yir_core::Value;
use yir_exec::{FunctionInvocation, FunctionSession};

pub(super) enum SessionExecution<'a> {
    Reference(FunctionSession<'a>),
    Native(NativeSessionBindings<'a>),
}

impl SessionExecution<'_> {
    pub fn admit_limit(&self, limit: Option<usize>) -> Result<(), String> {
        if limit.is_some() && matches!(self, Self::Native(_)) {
            return Err(
                "reference-executor fuel cannot preempt a native session callback".to_owned(),
            );
        }
        Ok(())
    }

    pub fn invoke(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
        limit: Option<usize>,
    ) -> Result<FunctionInvocation, String> {
        self.admit_limit(limit)?;
        match self {
            Self::Reference(session) => match limit {
                Some(limit) => session.invoke_budgeted(name, arguments, limit),
                None => session.invoke(name, arguments),
            },
            Self::Native(binding) => binding.invoke(name, arguments),
        }
    }
}

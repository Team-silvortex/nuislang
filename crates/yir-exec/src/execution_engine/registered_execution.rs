use super::*;
use yir_core::{RegisteredExecution, RegisteredExecutionStep};

impl ExecutionEngine<'_> {
    pub(super) fn execute_registered_node(
        &mut self,
        node: &Node,
        mut execution: Box<dyn RegisteredExecution>,
    ) -> Result<Value, String> {
        let mut call_result = None;
        for _ in 0..MAX_SCOPED_LOOP_ITERATIONS {
            self.charge_step()?;
            match execution.resume(&mut self.state, call_result.take())? {
                RegisteredExecutionStep::Continue => {}
                RegisteredExecutionStep::Call {
                    function,
                    arguments,
                } => {
                    call_result = Some(self.execute_function(&function, arguments)?);
                }
                RegisteredExecutionStep::Complete(value) => return Ok(value),
            }
        }
        Err(format!(
            "registered execution `{}` exceeded {MAX_SCOPED_LOOP_ITERATIONS} reference steps",
            node.name
        ))
    }
}

use yir_core::{YirFunctionParameter, YirValueOwnership};

use super::*;

/// One explicit function call. Events/frames are per-call; completion witnesses
/// are a snapshot of the current context, not a history of every invocation.
#[derive(Debug)]
pub struct FunctionInvocation {
    pub value: Value,
    pub trace: ExecutionTrace,
}

/// A verified reference-execution context shared by independent host calls.
///
/// Construction runs global nodes once, but never invokes entry functions.
/// Heap and provider clock state persist; function locals and delivered traces do
/// not. Ingress currently admits value-owned scalars only, not raw capabilities.
/// Errors do not roll back effects. The caller is responsible for fault policy.
pub struct FunctionSession<'a> {
    engine: ExecutionEngine<'a>,
}

impl<'a> FunctionSession<'a> {
    pub fn new(module: &'a YirModule, registry: &'a ModRegistry) -> Result<Self, String> {
        let (mut engine, order) = ExecutionEngine::prepare(module, registry)?;
        let bodies = module
            .functions
            .iter()
            .flat_map(|function| &function.body_nodes)
            .collect::<BTreeSet<_>>();
        let mut delayed = BTreeMap::new();
        for node in order {
            if !bodies.contains(&node) {
                engine.execute_named_node(&node, &mut delayed)?;
            }
        }
        reject_remaining_delayed(&delayed)?;
        Ok(Self { engine })
    }

    pub fn invoke(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<FunctionInvocation, String> {
        let function = self
            .engine
            .module
            .functions
            .iter()
            .find(|function| function.name == name)
            .ok_or_else(|| format!("unknown session function `{name}`"))?;
        Self::validate_function(function)?;
        Self::validate_arguments(&function.parameters, &arguments)?;
        let result_type = function.result.as_ref().unwrap().ty.clone();
        let result = self.engine.execute_function(name, arguments);
        // Drain even on failure: a failed callback must not retain frame history.
        let trace = self.take_trace();
        let value = result?;
        let compatible = scalar_matches(&result_type, &value)
            || matches!(&value, Value::Struct(value) if value.type_name == result_type);
        if !compatible || !scalar_tree(&value) {
            return Err(format!(
                "session function `{name}` returned an incompatible scalar-state value"
            ));
        }
        Ok(FunctionInvocation { value, trace })
    }

    /// Preflight a boundary without executing any nodes.
    pub fn validate_function(function: &YirFunction) -> Result<(), String> {
        yir_core::validate_session_function(function)
    }

    pub fn validate_arguments(
        parameters: &[YirFunctionParameter],
        arguments: &[Value],
    ) -> Result<(), String> {
        if parameters.len() != arguments.len() {
            return Err(format!(
                "session expects {} scalar arguments, got {}",
                parameters.len(),
                arguments.len()
            ));
        }
        for (parameter, argument) in parameters.iter().zip(arguments) {
            if parameter.ownership != YirValueOwnership::Value
                || !scalar_matches(&parameter.ty, argument)
            {
                return Err(format!(
                    "session parameter `{}` expects value-owned `{}`",
                    parameter.name, parameter.ty
                ));
            }
        }
        Ok(())
    }

    fn take_trace(&mut self) -> ExecutionTrace {
        ExecutionTrace {
            events: std::mem::take(&mut self.engine.state.events),
            lane_events: std::mem::take(&mut self.engine.state.lane_events),
            lane_steps: std::mem::take(&mut self.engine.lane_steps),
            // Persistent values are not an exported snapshot of resource handles.
            values: BTreeMap::new(),
            presented_frames: self.engine.state.take_presented_frames(),
            provider_completion_witnesses: self
                .engine
                .state
                .provider_completion_witnesses()
                .clone(),
        }
    }
}

fn scalar_matches(ty: &str, value: &Value) -> bool {
    matches!(
        (ty, value),
        ("bool", Value::Bool(_))
            | ("i32", Value::I32(_))
            | ("i64", Value::Int(_))
            | ("f32", Value::F32(_))
            | ("f64", Value::F64(_))
    )
}

fn scalar_tree(value: &Value) -> bool {
    match value {
        Value::Bool(_) | Value::I32(_) | Value::Int(_) | Value::F32(_) | Value::F64(_) => true,
        Value::Struct(value) => {
            let mut fields = BTreeSet::new();
            !value.type_name.is_empty()
                && value.fields.iter().all(|(name, value)| {
                    !name.is_empty()
                        && !name.contains('.')
                        && fields.insert(name)
                        && scalar_tree(value)
                })
        }
        _ => false,
    }
}

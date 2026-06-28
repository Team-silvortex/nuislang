use std::collections::BTreeMap;

use yir_core::{Value, YirModule};

use crate::RuntimeError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostYirValueSummary {
    pub node: String,
    pub op: String,
    pub value_kind: String,
    pub display: String,
    pub element_count: usize,
    pub integer_checksum: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostYirExecutionSummary {
    pub nodes_executed: usize,
    pub kernel_nodes_executed: usize,
    pub tensor_values: usize,
    pub scalar_values: usize,
    pub frame_values: usize,
    pub integer_checksum: i64,
    pub kernel_integer_checksum: i64,
    pub kernel_values: Vec<HostYirValueSummary>,
}

pub fn execute_host_yir_source(source: &str) -> Result<HostYirExecutionSummary, RuntimeError> {
    let module = yir_syntax::parse_module(source)
        .map_err(|error| RuntimeError::new(format!("failed to parse host YIR: {error}")))?;
    execute_host_yir_module(&module)
}

pub fn execute_host_yir_module(
    module: &YirModule,
) -> Result<HostYirExecutionSummary, RuntimeError> {
    let trace = yir_exec::execute_module(module)
        .map_err(|error| RuntimeError::new(format!("failed to execute host YIR: {error}")))?;
    Ok(summarize_host_yir_execution(module, &trace.values))
}

fn summarize_host_yir_execution(
    module: &YirModule,
    values: &BTreeMap<String, Value>,
) -> HostYirExecutionSummary {
    let mut tensor_values = 0usize;
    let mut scalar_values = 0usize;
    let mut frame_values = 0usize;
    let mut integer_checksum = 0i64;
    let mut kernel_integer_checksum = 0i64;
    let mut kernel_values = Vec::new();

    for node in &module.nodes {
        let Some(value) = values.get(&node.name) else {
            continue;
        };
        let value_checksum = value_integer_checksum(value);
        integer_checksum = integer_checksum.wrapping_add(value_checksum);
        match value {
            Value::Tensor(_) => tensor_values += 1,
            Value::Bool(_) | Value::I32(_) | Value::Int(_) | Value::F32(_) | Value::F64(_) => {
                scalar_values += 1;
            }
            Value::Frame(_) => frame_values += 1,
            _ => {}
        }
        if node.op.module == "kernel" {
            kernel_integer_checksum = kernel_integer_checksum.wrapping_add(value_checksum);
            kernel_values.push(HostYirValueSummary {
                node: node.name.clone(),
                op: node.op.full_name(),
                value_kind: value_kind(value).to_owned(),
                display: value.to_string(),
                element_count: value_element_count(value),
                integer_checksum: value_checksum,
            });
        }
    }

    HostYirExecutionSummary {
        nodes_executed: values.len(),
        kernel_nodes_executed: kernel_values.len(),
        tensor_values,
        scalar_values,
        frame_values,
        integer_checksum,
        kernel_integer_checksum,
        kernel_values,
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "bool",
        Value::I32(_) => "i32",
        Value::Int(_) => "i64",
        Value::F32(_) => "f32",
        Value::F64(_) => "f64",
        Value::Symbol(_) => "symbol",
        Value::Tensor(_) => "tensor",
        Value::Pointer(_) => "pointer",
        Value::Tuple(_) => "tuple",
        Value::Struct(_) => "struct",
        Value::DataWindow(_) => "data-window",
        Value::DataPipe(_) => "data-pipe",
        Value::DataResult(_) => "data-result",
        Value::DataMarker(_) => "data-marker",
        Value::DataHandleTable(_) => "data-handle-table",
        Value::DataCoreBinding(_) => "data-core-binding",
        Value::ShaderResult(_) => "shader-result",
        Value::KernelResult(_) => "kernel-result",
        Value::NetworkResult(_) => "network-result",
        Value::Target(_) => "target",
        Value::Viewport(_) => "viewport",
        Value::Pipeline(_) => "pipeline",
        Value::VertexLayout(_) => "vertex-layout",
        Value::VertexBuffer(_) => "vertex-buffer",
        Value::IndexBuffer(_) => "index-buffer",
        Value::Texture(_) => "texture",
        Value::Sampler(_) => "sampler",
        Value::Blend(_) => "blend",
        Value::Depth(_) => "depth",
        Value::Raster(_) => "raster",
        Value::RenderState(_) => "render-state",
        Value::Binding(_) => "binding",
        Value::BindingSet(_) => "binding-set",
        Value::RenderPass(_) => "render-pass",
        Value::Frame(_) => "frame",
        Value::Task(_) => "task",
        Value::Thread(_) => "thread",
        Value::TaskResult(_) => "task-result",
        Value::Mutex(_) => "mutex",
        Value::MutexGuard(_) => "mutex-guard",
        Value::Unit => "unit",
    }
}

fn value_element_count(value: &Value) -> usize {
    match value {
        Value::Tensor(tensor) => tensor.elements.len(),
        Value::Tuple(values) => values.len(),
        _ => 1,
    }
}

fn value_integer_checksum(value: &Value) -> i64 {
    match value {
        Value::Bool(value) => i64::from(*value),
        Value::I32(value) => *value as i64,
        Value::Int(value) => *value,
        Value::F32(value) => value.to_bits() as i64,
        Value::F64(value) => value.to_bits() as i64,
        Value::Tensor(tensor) => tensor
            .elements
            .iter()
            .fold(0i64, |acc, value| acc.wrapping_add(*value)),
        Value::Tuple(values) => values.iter().fold(0i64, |acc, value| {
            acc.wrapping_add(value_integer_checksum(value))
        }),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::execute_host_yir_source;

    #[test]
    fn executes_kernel_tensor_pipeline_on_host() {
        let source = r#"
yir 0.1

resource kernel0 kernel.tensor

kernel.tensor input kernel0 1 3 2,4,6
kernel.tensor weights kernel0 3 2 1,-2,3,0,2,1
kernel.matmul projected kernel0 input weights
kernel.reduce_sum total kernel0 projected
"#;

        let summary = execute_host_yir_source(source).expect("host YIR executes");

        assert_eq!(summary.kernel_nodes_executed, 4);
        assert_eq!(summary.tensor_values, 3);
        assert_eq!(summary.scalar_values, 1);
        assert_eq!(summary.integer_checksum, 73);
        assert_eq!(summary.kernel_integer_checksum, 73);

        let total = summary
            .kernel_values
            .iter()
            .find(|value| value.node == "total")
            .expect("total result exists");
        assert_eq!(total.op, "kernel.reduce_sum");
        assert_eq!(total.value_kind, "i64");
        assert_eq!(total.display, "28");
        assert_eq!(total.integer_checksum, 28);
    }
}

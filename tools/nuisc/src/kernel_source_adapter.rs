use std::collections::{BTreeMap, BTreeSet};

use yir_core::{Node, Operation, YirFunction, YirModule};

use crate::kernel_codegen_table::{
    parameter, KernelParameterKind, KernelYirCodegenFunction, KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
};

pub(crate) const KERNEL_YIR_SOURCE_ADAPTER_CONTRACT: &str = "nuis-kernel-yir-source-adapter-v1";
pub(crate) const KERNEL_SOURCE_REQUEST_PROJECTION_CONTRACT: &str =
    "nuis-kernel-source-request-projection-v1";
pub(crate) const KERNEL_SOURCE_RESULT_PROJECTION_CONTRACT: &str =
    "nuis-kernel-source-result-projection-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelSourceAdaptationStatus {
    Adapted,
    Projected,
    Unsupported,
}

impl KernelSourceAdaptationStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Adapted => "adapted",
            Self::Projected => "projected",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelSourceAdaptation {
    pub(crate) contract: &'static str,
    pub(crate) source_function: String,
    pub(crate) source_node: String,
    pub(crate) source_instruction: String,
    pub(crate) status: KernelSourceAdaptationStatus,
    pub(crate) generated_entry: Option<String>,
    pub(crate) request_projection: Option<KernelSourceRequestProjection>,
    pub(crate) result_projection: Option<KernelSourceResultProjection>,
    pub(crate) diagnostic: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelSourceRequestProjection {
    pub(crate) contract: &'static str,
    pub(crate) operation: &'static str,
    pub(crate) element_type: &'static str,
    pub(crate) input_shape: Vec<usize>,
    pub(crate) output_shape: Vec<usize>,
    pub(crate) input_values: Vec<i64>,
    pub(crate) scalar: Option<i64>,
    pub(crate) input_source_node: Option<String>,
    pub(crate) expected_values: Vec<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelSourceResultProjection {
    pub(crate) contract: &'static str,
    pub(crate) element_type: &'static str,
    pub(crate) input_source_node: String,
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) expected_i64: i64,
}

impl KernelSourceAdaptation {
    pub(crate) fn is_adapted(&self) -> bool {
        self.status == KernelSourceAdaptationStatus::Adapted
    }

    pub(crate) fn is_projected(&self) -> bool {
        self.status == KernelSourceAdaptationStatus::Projected
    }
}

pub(crate) fn adapt_project_kernel_functions(
    module: &YirModule,
    source_functions: &[YirFunction],
) -> Result<(Vec<KernelSourceAdaptation>, Vec<KernelYirCodegenFunction>), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut adaptations = Vec::new();
    let mut functions = Vec::new();
    for function in source_functions {
        let body = function.body_nodes.iter().cloned().collect::<BTreeSet<_>>();
        let mut projections = BTreeMap::new();
        for node_name in &function.body_nodes {
            let Some(node) = nodes.get(node_name.as_str()).copied() else {
                return Err(format!(
                    "Kernel source adapter cannot resolve body node `{node_name}`"
                ));
            };
            if node.op.module != "kernel" {
                continue;
            }
            if let Some(adaptation) =
                adapt_element_at_result(function, node, &body, &nodes, &projections)
            {
                adaptations.push(adaptation);
                continue;
            }
            let adapted = adapt_add_scalar_axis(function, node, &body, &nodes)?
                .or_else(|| adapt_reduce_sum_axis(function, node, &body, &projections));
            match adapted {
                Some((adaptation, generated)) => {
                    projections.insert(
                        node.name.clone(),
                        adaptation
                            .request_projection
                            .clone()
                            .expect("adapted Kernel source node must own a request projection"),
                    );
                    adaptations.push(adaptation);
                    functions.push(generated);
                }
                None => adaptations.push(unsupported_adaptation(function, node)),
            }
        }
    }
    Ok((adaptations, functions))
}

fn adapt_element_at_result(
    function: &YirFunction,
    node: &Node,
    body: &BTreeSet<String>,
    nodes: &BTreeMap<&str, &Node>,
    projections: &BTreeMap<String, KernelSourceRequestProjection>,
) -> Option<KernelSourceAdaptation> {
    if node.op.instruction != "element_at" {
        return None;
    }
    let [input, row, col] = node.op.args.as_slice() else {
        return None;
    };
    let input_projection = projections.get(input)?;
    let row = const_i64_index(row, body, nodes)?;
    let col = const_i64_index(col, body, nodes)?;
    if input_projection.element_type != "i64"
        || input_projection.output_shape != [1, 1]
        || input_projection.expected_values.len() != 1
        || row != 0
        || col != 0
    {
        return None;
    }
    Some(KernelSourceAdaptation {
        contract: KERNEL_YIR_SOURCE_ADAPTER_CONTRACT,
        source_function: function.name.clone(),
        source_node: node.name.clone(),
        source_instruction: node.op.instruction.clone(),
        status: KernelSourceAdaptationStatus::Projected,
        generated_entry: None,
        request_projection: None,
        result_projection: Some(KernelSourceResultProjection {
            contract: KERNEL_SOURCE_RESULT_PROJECTION_CONTRACT,
            element_type: "i64",
            input_source_node: input.clone(),
            row,
            col,
            expected_i64: input_projection.expected_values[0],
        }),
        diagnostic: "projected verified 1x1 i64 provider output as a host-visible scalar"
            .to_owned(),
    })
}

fn const_i64_index(
    node_name: &str,
    body: &BTreeSet<String>,
    nodes: &BTreeMap<&str, &Node>,
) -> Option<usize> {
    let node = nodes.get(node_name).copied()?;
    let [value] = node.op.args.as_slice() else {
        return None;
    };
    (body.contains(node_name) && node.op.module == "cpu" && node.op.instruction == "const_i64")
        .then(|| value.parse::<usize>().ok())
        .flatten()
}

fn adapt_reduce_sum_axis(
    function: &YirFunction,
    node: &Node,
    body: &BTreeSet<String>,
    projections: &BTreeMap<String, KernelSourceRequestProjection>,
) -> Option<(KernelSourceAdaptation, KernelYirCodegenFunction)> {
    if node.op.instruction != "reduce_sum_axis" {
        return None;
    }
    let [input, axis] = node.op.args.as_slice() else {
        return None;
    };
    let input_projection = projections.get(input)?;
    if !body.contains(input)
        || input_projection.element_type != "i64"
        || !matches!(axis.as_str(), "rows" | "cols")
    {
        return None;
    }
    let [rows, cols] = input_projection.input_shape.as_slice() else {
        return None;
    };
    let output_shape = match axis.as_str() {
        "rows" => vec![1, *cols],
        "cols" => vec![*rows, 1],
        _ => unreachable!("validated Kernel reduction axis"),
    };
    if output_shape.iter().product::<usize>() != 1 {
        return None;
    }
    let sum = input_projection
        .expected_values
        .iter()
        .try_fold(0i64, |sum, value| sum.checked_add(*value))?;
    let entry = format!(
        "nuis_project_{}_{}_i64",
        identifier_fragment(&function.name),
        identifier_fragment(&node.name)
    );
    let generated = KernelYirCodegenFunction {
        contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
        entry: entry.clone(),
        parameters: vec![
            parameter("input", KernelParameterKind::InputI64),
            parameter("output", KernelParameterKind::OutputI64),
            parameter("element_count", KernelParameterKind::ElementCountU32),
        ],
        nodes: vec![Node {
            name: "adapted_output".to_owned(),
            resource: "kernel0".to_owned(),
            op: Operation {
                module: "kernel".to_owned(),
                instruction: "reduce_sum_i64".to_owned(),
                args: vec!["input".to_owned()],
            },
        }],
        output_node: "adapted_output".to_owned(),
    };
    Some((
        KernelSourceAdaptation {
            contract: KERNEL_YIR_SOURCE_ADAPTER_CONTRACT,
            source_function: function.name.clone(),
            source_node: node.name.clone(),
            source_instruction: node.op.instruction.clone(),
            status: KernelSourceAdaptationStatus::Adapted,
            generated_entry: Some(entry),
            request_projection: Some(KernelSourceRequestProjection {
                contract: KERNEL_SOURCE_REQUEST_PROJECTION_CONTRACT,
                operation: "reduce-sum-i64",
                element_type: "i64",
                input_shape: input_projection.output_shape.clone(),
                output_shape,
                input_values: input_projection.expected_values.clone(),
                scalar: None,
                input_source_node: Some(input.clone()),
                expected_values: vec![sum],
            }),
            result_projection: None,
            diagnostic: format!(
                "selected verified `{axis}` i64 scalar reduction with a project-request dependency"
            ),
        },
        generated,
    ))
}

fn adapt_add_scalar_axis(
    function: &YirFunction,
    node: &Node,
    body: &BTreeSet<String>,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<Option<(KernelSourceAdaptation, KernelYirCodegenFunction)>, String> {
    if node.op.instruction != "add_scalar_axis" {
        return Ok(None);
    }
    let [input, axis, scalar] = node.op.args.as_slice() else {
        return Err(format!(
            "Kernel source node `{}` has malformed add_scalar_axis operands",
            node.name
        ));
    };
    if !matches!(axis.as_str(), "rows" | "cols") || !body.contains(input) || !body.contains(scalar)
    {
        return Err(format!(
            "Kernel source node `{}` cannot prove its axis/input/scalar boundary",
            node.name
        ));
    }
    let input_node = nodes
        .get(input.as_str())
        .copied()
        .filter(|input_node| {
            input_node.op.module == "kernel" && input_node.op.instruction == "tensor"
        })
        .ok_or_else(|| {
            format!(
                "Kernel source node `{}` requires a function-owned tensor input",
                node.name
            )
        })?;
    let scalar_node = nodes
        .get(scalar.as_str())
        .copied()
        .filter(|scalar_node| {
            scalar_node.op.module == "cpu" && scalar_node.op.instruction == "const_i64"
        })
        .ok_or_else(|| {
            format!(
                "Kernel source node `{}` requires a function-owned i64 scalar",
                node.name
            )
        })?;
    let request_projection = request_projection(input_node, scalar_node)?;
    let entry = format!(
        "nuis_project_{}_{}_i64",
        identifier_fragment(&function.name),
        identifier_fragment(&node.name)
    );
    let generated = KernelYirCodegenFunction {
        contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
        entry: entry.clone(),
        parameters: vec![
            parameter("input", KernelParameterKind::InputI64),
            parameter("output", KernelParameterKind::OutputI64),
            parameter("element_count", KernelParameterKind::ElementCountU32),
            parameter("scalar", KernelParameterKind::ScalarI64),
        ],
        nodes: vec![Node {
            name: "adapted_output".to_owned(),
            resource: "kernel0".to_owned(),
            op: Operation {
                module: "kernel".to_owned(),
                instruction: "add_i64".to_owned(),
                args: vec!["input".to_owned(), "scalar".to_owned()],
            },
        }],
        output_node: "adapted_output".to_owned(),
    };
    Ok(Some((
        KernelSourceAdaptation {
            contract: KERNEL_YIR_SOURCE_ADAPTER_CONTRACT,
            source_function: function.name.clone(),
            source_node: node.name.clone(),
            source_instruction: node.op.instruction.clone(),
            status: KernelSourceAdaptationStatus::Adapted,
            generated_entry: Some(entry),
            request_projection: Some(request_projection),
            result_projection: None,
            diagnostic: format!(
                "selected verified `{axis}` i64 scalar-map node with function-owned input and scalar operands"
            ),
        },
        generated,
    )))
}

fn unsupported_adaptation(function: &YirFunction, node: &Node) -> KernelSourceAdaptation {
    KernelSourceAdaptation {
        contract: KERNEL_YIR_SOURCE_ADAPTER_CONTRACT,
        source_function: function.name.clone(),
        source_node: node.name.clone(),
        source_instruction: node.op.instruction.clone(),
        status: KernelSourceAdaptationStatus::Unsupported,
        generated_entry: None,
        request_projection: None,
        result_projection: None,
        diagnostic: format!(
            "Kernel/YIR instruction `{}` has no registered source adapter for CUDA",
            node.op.instruction
        ),
    }
}

fn request_projection(
    input_node: &Node,
    scalar_node: &Node,
) -> Result<KernelSourceRequestProjection, String> {
    let [rows, cols, values] = input_node.op.args.as_slice() else {
        return Err(format!(
            "Kernel tensor node `{}` has malformed shape/value operands",
            input_node.name
        ));
    };
    let rows = positive_dimension(rows, &input_node.name)?;
    let cols = positive_dimension(cols, &input_node.name)?;
    let element_count = rows
        .checked_mul(cols)
        .ok_or_else(|| format!("Kernel tensor node `{}` shape overflows", input_node.name))?;
    let input_values = values
        .split(',')
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| {
            format!(
                "Kernel tensor node `{}` has a non-i64 literal",
                input_node.name
            )
        })?;
    if input_values.len() != element_count {
        return Err(format!(
            "Kernel tensor node `{}` literal count does not match its shape",
            input_node.name
        ));
    }
    let [scalar] = scalar_node.op.args.as_slice() else {
        return Err(format!(
            "Kernel scalar node `{}` has malformed const_i64 operands",
            scalar_node.name
        ));
    };
    let scalar = scalar.parse::<i64>().map_err(|_| {
        format!(
            "Kernel scalar node `{}` has an invalid i64 literal",
            scalar_node.name
        )
    })?;
    let expected_values = input_values
        .iter()
        .map(|value| {
            value.checked_add(scalar).ok_or_else(|| {
                format!(
                    "Kernel source request projection overflows i64 at `{}`",
                    input_node.name
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(KernelSourceRequestProjection {
        contract: KERNEL_SOURCE_REQUEST_PROJECTION_CONTRACT,
        operation: "add-scalar-i64",
        element_type: "i64",
        input_shape: vec![rows, cols],
        output_shape: vec![rows, cols],
        input_values,
        scalar: Some(scalar),
        input_source_node: None,
        expected_values,
    })
}

fn positive_dimension(value: &str, node: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("Kernel tensor node `{node}` has an invalid dimension `{value}`"))
}

fn identifier_fragment(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect()
}

use std::collections::BTreeSet;

use yir_core::{Node, Operation, YirFunction};

use crate::aot_encoding::fnv1a64_hex;

pub(crate) const KERNEL_YIR_CODEGEN_TABLE_CONTRACT: &str = "nuis-kernel-yir-codegen-table-v1";
pub(crate) const KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT: &str = "nuis-kernel-yir-codegen-function-v1";
const PROJECT_YIR_BINDING_CONTRACT: &str = "compiled-project-yir";
const REGISTERED_BINDING_CONTRACT: &str = "registered-provider-kernel-yir";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelParameterKind {
    InputF32,
    OutputF32,
    ElementCountU32,
    ScalarF32,
}

impl KernelParameterKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::InputF32 => "input-f32",
            Self::OutputF32 => "output-f32",
            Self::ElementCountU32 => "element-count-u32",
            Self::ScalarF32 => "scalar-f32",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelParameter {
    pub(crate) name: &'static str,
    pub(crate) kind: KernelParameterKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelYirCodegenFunction {
    pub(crate) contract: &'static str,
    pub(crate) entry: &'static str,
    pub(crate) parameters: &'static [KernelParameter],
    pub(crate) nodes: Vec<Node>,
    pub(crate) output_node: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelYirCodegenTable {
    pub(crate) contract: &'static str,
    pub(crate) source_binding: &'static str,
    pub(crate) source_yir_version: String,
    pub(crate) source_fnv1a64: String,
    pub(crate) source_kernel_node_count: usize,
    pub(crate) source_kernel_body_node_count: usize,
    pub(crate) lowering_target: &'static str,
    pub(crate) source_functions: Vec<YirFunction>,
    pub(crate) functions: Vec<KernelYirCodegenFunction>,
}

const VECTOR_ADD_PARAMETERS: &[KernelParameter] = &[
    KernelParameter {
        name: "input_lhs",
        kind: KernelParameterKind::InputF32,
    },
    KernelParameter {
        name: "input_rhs",
        kind: KernelParameterKind::InputF32,
    },
    KernelParameter {
        name: "output",
        kind: KernelParameterKind::OutputF32,
    },
    KernelParameter {
        name: "element_count",
        kind: KernelParameterKind::ElementCountU32,
    },
];

const SCALE_PARAMETERS: &[KernelParameter] = &[
    KernelParameter {
        name: "input",
        kind: KernelParameterKind::InputF32,
    },
    KernelParameter {
        name: "output",
        kind: KernelParameterKind::OutputF32,
    },
    KernelParameter {
        name: "element_count",
        kind: KernelParameterKind::ElementCountU32,
    },
    KernelParameter {
        name: "scale",
        kind: KernelParameterKind::ScalarF32,
    },
];

pub(crate) fn table_from_compiled_project_yir(
    source: &str,
    lowering_target: &str,
) -> Result<KernelYirCodegenTable, String> {
    if lowering_target != "cuda.nvidia-gpu" {
        return Err(format!(
            "Kernel/YIR codegen table has no registered producer for `{lowering_target}`"
        ));
    }
    let module = yir_syntax::parse_module(source)
        .map_err(|error| format!("failed to parse compiled project YIR: {error}"))?;
    yir_verify::verify_module(&module)
        .map_err(|error| format!("compiled project YIR failed verification: {error}"))?;

    let kernel_resources = module
        .resources
        .iter()
        .filter(|resource| resource.kind.is_family("kernel"))
        .map(|resource| resource.name.as_str())
        .collect::<BTreeSet<_>>();
    let kernel_node_count = module
        .nodes
        .iter()
        .filter(|node| kernel_resources.contains(node.resource.as_str()))
        .count();
    if kernel_resources.is_empty() || kernel_node_count == 0 {
        return Err("compiled project YIR has no Kernel resource/function body".to_owned());
    }
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let source_functions = module
        .functions
        .iter()
        .filter(|function| {
            function.body_nodes.iter().any(|name| {
                nodes
                    .get(name.as_str())
                    .is_some_and(|node| node.op.module == "kernel")
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    let source_kernel_body_node_count = source_functions
        .iter()
        .flat_map(|function| function.body_nodes.iter())
        .filter(|name| {
            nodes
                .get(name.as_str())
                .is_some_and(|node| node.op.module == "kernel")
        })
        .count();
    if source_functions.is_empty() || source_kernel_body_node_count == 0 {
        return Err(
            "compiled project YIR has no function boundary owning Kernel body nodes".to_owned(),
        );
    }
    let target_matches = module.nodes.iter().any(|node| {
        kernel_resources.contains(node.resource.as_str())
            && node.op.module == "kernel"
            && node.op.instruction == "target_config"
            && node.op.args.get(1).is_some_and(|runtime| runtime == "cuda")
    });
    if !target_matches {
        return Err(
            "compiled project YIR does not bind its Kernel target_config to CUDA".to_owned(),
        );
    }

    let table = KernelYirCodegenTable {
        contract: KERNEL_YIR_CODEGEN_TABLE_CONTRACT,
        source_binding: PROJECT_YIR_BINDING_CONTRACT,
        source_yir_version: module.version,
        source_fnv1a64: fnv1a64_hex(source.as_bytes()),
        source_kernel_node_count: kernel_node_count,
        source_kernel_body_node_count,
        lowering_target: "cuda.nvidia-gpu",
        source_functions,
        functions: registered_provider_functions(),
    };
    validate_codegen_table(&table)?;
    Ok(table)
}

pub(crate) fn registered_provider_codegen_table() -> KernelYirCodegenTable {
    let table = KernelYirCodegenTable {
        contract: KERNEL_YIR_CODEGEN_TABLE_CONTRACT,
        source_binding: REGISTERED_BINDING_CONTRACT,
        source_yir_version: "0.1".to_owned(),
        source_fnv1a64: fnv1a64_hex(b"nuis-kernel-registered-arithmetic-v1"),
        source_kernel_node_count: 2,
        source_kernel_body_node_count: 2,
        lowering_target: "cuda.nvidia-gpu",
        source_functions: Vec::new(),
        functions: registered_provider_functions(),
    };
    validate_codegen_table(&table).expect("registered Kernel/YIR table must remain valid");
    table
}

pub(crate) fn validate_codegen_table(table: &KernelYirCodegenTable) -> Result<(), String> {
    if table.contract != KERNEL_YIR_CODEGEN_TABLE_CONTRACT
        || !matches!(
            table.source_binding,
            PROJECT_YIR_BINDING_CONTRACT | REGISTERED_BINDING_CONTRACT
        )
        || table.source_yir_version.is_empty()
        || !valid_fnv1a64(&table.source_fnv1a64)
        || table.source_kernel_node_count == 0
        || table.source_kernel_body_node_count == 0
        || table.lowering_target != "cuda.nvidia-gpu"
        || table.functions.is_empty()
    {
        return Err("Kernel/YIR codegen table header is invalid".to_owned());
    }
    if table.source_binding == PROJECT_YIR_BINDING_CONTRACT && table.source_functions.is_empty() {
        return Err(
            "project Kernel/YIR codegen table has no source function boundaries".to_owned(),
        );
    }
    let mut entries = BTreeSet::new();
    for function in &table.functions {
        if function.contract != KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT
            || !valid_identifier(function.entry)
            || !entries.insert(function.entry)
            || function.parameters.is_empty()
            || function.nodes.is_empty()
            || !function
                .nodes
                .iter()
                .any(|node| node.name == function.output_node)
        {
            return Err(format!(
                "Kernel/YIR codegen function `{}` is invalid",
                function.entry
            ));
        }
    }
    Ok(())
}

pub(crate) fn render_codegen_table(table: &KernelYirCodegenTable) -> Result<String, String> {
    validate_codegen_table(table)?;
    let mut out = format!(
        "schema = \"{}\"\nsource_binding = \"{}\"\nsource_yir_version = \"{}\"\nsource_fnv1a64 = \"{}\"\nsource_kernel_node_count = {}\nsource_kernel_body_node_count = {}\nsource_function_count = {}\nlowering_target = \"{}\"\nfunction_count = {}\n",
        table.contract,
        table.source_binding,
        table.source_yir_version,
        table.source_fnv1a64,
        table.source_kernel_node_count,
        table.source_kernel_body_node_count,
        table.source_functions.len(),
        table.lowering_target,
        table.functions.len()
    );
    for function in &table.source_functions {
        out.push_str("\n[[source_function]]\n");
        out.push_str(&format!(
            "name = \"{}\"\ndomain = \"{}\"\nrole = \"{}\"\n",
            function.name,
            function.domain,
            function.role.as_str()
        ));
        out.push_str("parameters = [");
        for (index, parameter) in function.parameters.iter().enumerate() {
            if index != 0 {
                out.push_str(", ");
            }
            out.push_str(&format!(
                "\"{}:{}:{}:{}\"",
                parameter.name,
                parameter.ty,
                parameter.ownership.as_str(),
                parameter.node
            ));
        }
        out.push_str("]\n");
        if let Some(result) = &function.result {
            out.push_str(&format!(
                "result = \"{}:{}:{}\"\n",
                result.ty,
                result.ownership.as_str(),
                result.node
            ));
        }
        out.push_str(&format!(
            "body_nodes = [{}]\n",
            function
                .body_nodes
                .iter()
                .map(|node| format!("\"{node}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for function in &table.functions {
        out.push_str("\n[[function]]\n");
        out.push_str(&format!(
            "contract = \"{}\"\nentry = \"{}\"\noutput_node = \"{}\"\n",
            function.contract, function.entry, function.output_node
        ));
        out.push_str("parameters = [");
        for (index, parameter) in function.parameters.iter().enumerate() {
            if index != 0 {
                out.push_str(", ");
            }
            out.push_str(&format!(
                "\"{}:{}\"",
                parameter.name,
                parameter.kind.as_str()
            ));
        }
        out.push_str("]\n");
        out.push_str("nodes = [");
        for (index, node) in function.nodes.iter().enumerate() {
            if index != 0 {
                out.push_str(", ");
            }
            out.push_str(&format!(
                "\"{}:{}:{}.{}({})\"",
                node.name,
                node.resource,
                node.op.module,
                node.op.instruction,
                node.op.args.join(",")
            ));
        }
        out.push_str("]\n");
    }
    Ok(out)
}

fn registered_provider_functions() -> Vec<KernelYirCodegenFunction> {
    vec![
        KernelYirCodegenFunction {
            contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
            entry: "nuis_kernel_vector_add_f32",
            parameters: VECTOR_ADD_PARAMETERS,
            nodes: vec![arithmetic_node(
                "sum",
                "add_f32",
                &["input_lhs", "input_rhs"],
            )],
            output_node: "sum",
        },
        KernelYirCodegenFunction {
            contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
            entry: "nuis_kernel_scale_f32",
            parameters: SCALE_PARAMETERS,
            nodes: vec![arithmetic_node("scaled", "mul_f32", &["input", "scale"])],
            output_node: "scaled",
        },
    ]
}

fn arithmetic_node(name: &str, instruction: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: instruction.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        },
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_fnv1a64(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT_YIR: &str = "yir 0.1\n\
resource cpu0 cpu.arm64\n\
resource kernel0 kernel.cuda\n\
function main cpu entry\n\
function-result main f32 value input\n\
function-node main input\n\
function-node main target\n\
kernel.const_f32 input kernel0 1.0\n\
kernel.target_config target kernel0 x86_64 cuda 1 ptx\n";

    #[test]
    fn compiled_project_yir_builds_verified_backend_neutral_table() {
        let table = table_from_compiled_project_yir(PROJECT_YIR, "cuda.nvidia-gpu").unwrap();
        assert_eq!(table.source_binding, PROJECT_YIR_BINDING_CONTRACT);
        assert_eq!(table.source_kernel_node_count, 2);
        assert_eq!(table.source_kernel_body_node_count, 2);
        assert_eq!(table.source_functions.len(), 1);
        assert_eq!(table.functions.len(), 2);
        let rendered = render_codegen_table(&table).unwrap();
        assert!(rendered.contains("schema = \"nuis-kernel-yir-codegen-table-v1\""));
        assert!(rendered.contains("source_binding = \"compiled-project-yir\""));
        assert!(rendered.contains("[[source_function]]"));
        assert!(rendered.contains("body_nodes = [\"input\", \"target\"]"));
        assert!(rendered.contains("kernel.add_f32"));
        assert!(rendered.contains("kernel.mul_f32"));
    }

    #[test]
    fn project_table_rejects_non_cuda_or_unverified_source() {
        assert!(table_from_compiled_project_yir(
            &PROJECT_YIR.replace(" cuda ", " coreml "),
            "cuda.nvidia-gpu"
        )
        .unwrap_err()
        .contains("target_config"));
        assert!(
            table_from_compiled_project_yir(PROJECT_YIR, "missing.target")
                .unwrap_err()
                .contains("no registered producer")
        );
    }
}

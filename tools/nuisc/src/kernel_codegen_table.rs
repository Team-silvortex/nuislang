use std::collections::BTreeSet;

use yir_core::{Node, Operation, YirFunction};

use crate::aot_encoding::fnv1a64_hex;
use crate::kernel_source_adapter::{
    adapt_project_kernel_functions, KernelSourceAdaptation, KernelSourceAdaptationStatus,
    KERNEL_SOURCE_REQUEST_PROJECTION_CONTRACT, KERNEL_SOURCE_RESULT_PROJECTION_CONTRACT,
    KERNEL_YIR_SOURCE_ADAPTER_CONTRACT,
};

pub(crate) const KERNEL_YIR_CODEGEN_TABLE_CONTRACT: &str = "nuis-kernel-yir-codegen-table-v1";
pub(crate) const KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT: &str = "nuis-kernel-yir-codegen-function-v1";
pub(crate) const KERNEL_PROJECT_CODE_ASSET_IDENTITY_CONTRACT: &str =
    "nuis-kernel-project-code-asset-identity-v1";
pub(crate) const KERNEL_CODE_ASSET_IDENTITY_SET_CONTRACT: &str =
    "nuis-provider-code-asset-identity-set-v1";
const PROJECT_YIR_BINDING_CONTRACT: &str = "compiled-project-yir";
const REGISTERED_BINDING_CONTRACT: &str = "registered-provider-kernel-yir";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelParameterKind {
    InputF32,
    OutputF32,
    InputU32,
    OutputU32,
    ElementCountU32,
    ScalarF32,
    InputI64,
    OutputI64,
    ScalarI64,
}

impl KernelParameterKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::InputF32 => "input-f32",
            Self::OutputF32 => "output-f32",
            Self::InputU32 => "input-u32",
            Self::OutputU32 => "output-u32",
            Self::ElementCountU32 => "element-count-u32",
            Self::ScalarF32 => "scalar-f32",
            Self::InputI64 => "input-i64",
            Self::OutputI64 => "output-i64",
            Self::ScalarI64 => "scalar-i64",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelParameter {
    pub(crate) name: String,
    pub(crate) kind: KernelParameterKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelYirCodegenFunction {
    pub(crate) contract: &'static str,
    pub(crate) entry: String,
    pub(crate) parameters: Vec<KernelParameter>,
    pub(crate) nodes: Vec<Node>,
    pub(crate) output_node: String,
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
    pub(crate) source_adaptations: Vec<KernelSourceAdaptation>,
    pub(crate) functions: Vec<KernelYirCodegenFunction>,
}

impl KernelYirCodegenTable {
    pub(crate) fn compiled_project_code_asset_id(&self) -> Option<String> {
        (self.source_binding == PROJECT_YIR_BINDING_CONTRACT).then(|| {
            let entries = self
                .functions
                .iter()
                .map(|function| function.entry.as_str())
                .collect::<Vec<_>>();
            let identity_hash = project_code_asset_identity_hash(
                &self.source_fnv1a64,
                self.lowering_target,
                &entries,
            );
            format!("kernel.cuda.project.{}", &identity_hash[2..])
        })
    }
}

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
    let (source_adaptations, adapted_functions) =
        adapt_project_kernel_functions(&module, &source_functions)?;

    let table = KernelYirCodegenTable {
        contract: KERNEL_YIR_CODEGEN_TABLE_CONTRACT,
        source_binding: PROJECT_YIR_BINDING_CONTRACT,
        source_yir_version: module.version,
        source_fnv1a64: fnv1a64_hex(source.as_bytes()),
        source_kernel_node_count: kernel_node_count,
        source_kernel_body_node_count,
        lowering_target: "cuda.nvidia-gpu",
        source_functions,
        source_adaptations,
        functions: adapted_functions,
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
        source_kernel_node_count: 3,
        source_kernel_body_node_count: 3,
        lowering_target: "cuda.nvidia-gpu",
        source_functions: Vec::new(),
        source_adaptations: Vec::new(),
        functions: registered_provider_functions(),
    };
    validate_codegen_table(&table).expect("registered Kernel/YIR table must remain valid");
    table
}

pub(crate) fn registered_provider_codegen_table_for_entries(
    entries: &[&str],
) -> Result<KernelYirCodegenTable, String> {
    let mut table = registered_provider_codegen_table();
    table
        .functions
        .retain(|function| entries.contains(&function.entry.as_str()));
    if table.functions.len() != entries.len() {
        return Err("registered Kernel/YIR code asset references an unknown entry".to_owned());
    }
    table.source_fnv1a64 = fnv1a64_hex(entries.join("\n").as_bytes());
    table.source_kernel_node_count = table.functions.iter().map(|item| item.nodes.len()).sum();
    table.source_kernel_body_node_count = table.source_kernel_node_count;
    validate_codegen_table(&table)?;
    Ok(table)
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
    if table.source_binding == PROJECT_YIR_BINDING_CONTRACT
        && (table.source_adaptations.len() != table.source_kernel_body_node_count
            || !table
                .source_adaptations
                .iter()
                .any(KernelSourceAdaptation::is_adapted)
            || table
                .source_adaptations
                .iter()
                .any(|adaptation| !valid_source_adaptation(adaptation)))
    {
        return Err(
            "project Kernel/YIR codegen table has no complete source adaptation map".to_owned(),
        );
    }
    if table.source_binding == PROJECT_YIR_BINDING_CONTRACT {
        let projected_entries = table
            .source_adaptations
            .iter()
            .filter_map(|adaptation| adaptation.generated_entry.as_deref())
            .collect::<Vec<_>>();
        let function_entries = table
            .functions
            .iter()
            .map(|function| function.entry.as_str())
            .collect::<Vec<_>>();
        if projected_entries != function_entries {
            return Err(
                "project Kernel/YIR codegen functions do not match source request projections"
                    .to_owned(),
            );
        }
    }
    let mut entries = BTreeSet::new();
    for function in &table.functions {
        if function.contract != KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT
            || !valid_identifier(&function.entry)
            || !entries.insert(function.entry.as_str())
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

fn valid_source_adaptation(adaptation: &KernelSourceAdaptation) -> bool {
    match adaptation.status {
        KernelSourceAdaptationStatus::Adapted => {
            adaptation.generated_entry.is_some()
                && adaptation.result_projection.is_none()
                && adaptation
                    .request_projection
                    .as_ref()
                    .is_some_and(|projection| {
                        projection.contract == KERNEL_SOURCE_REQUEST_PROJECTION_CONTRACT
                            && projection.element_type == "i64"
                            && !projection.input_shape.is_empty()
                            && !projection.output_shape.is_empty()
                            && projection.input_shape.iter().product::<usize>()
                                == projection.input_values.len()
                            && projection.output_shape.iter().product::<usize>()
                                == projection.expected_values.len()
                    })
        }
        KernelSourceAdaptationStatus::Projected => {
            adaptation.generated_entry.is_none()
                && adaptation.request_projection.is_none()
                && adaptation
                    .result_projection
                    .as_ref()
                    .is_some_and(|projection| {
                        projection.contract == KERNEL_SOURCE_RESULT_PROJECTION_CONTRACT
                            && projection.element_type == "i64"
                            && !projection.input_source_node.is_empty()
                    })
        }
        KernelSourceAdaptationStatus::Unsupported => {
            adaptation.generated_entry.is_none()
                && adaptation.request_projection.is_none()
                && adaptation.result_projection.is_none()
        }
    }
}

pub(crate) fn render_codegen_table(table: &KernelYirCodegenTable) -> Result<String, String> {
    validate_codegen_table(table)?;
    let mut out = format!(
        "schema = \"{}\"\nsource_binding = \"{}\"\nsource_yir_version = \"{}\"\nsource_fnv1a64 = \"{}\"\nsource_kernel_node_count = {}\nsource_kernel_body_node_count = {}\nsource_function_count = {}\nsource_adaptation_contract = \"{}\"\nsource_adaptation_count = {}\nsource_adapted_count = {}\nsource_projected_count = {}\nlowering_target = \"{}\"\nfunction_count = {}\n",
        table.contract,
        table.source_binding,
        table.source_yir_version,
        table.source_fnv1a64,
        table.source_kernel_node_count,
        table.source_kernel_body_node_count,
        table.source_functions.len(),
        KERNEL_YIR_SOURCE_ADAPTER_CONTRACT,
        table.source_adaptations.len(),
        table
            .source_adaptations
            .iter()
            .filter(|adaptation| adaptation.is_adapted())
            .count(),
        table
            .source_adaptations
            .iter()
            .filter(|adaptation| adaptation.is_projected())
            .count(),
        table.lowering_target,
        table.functions.len()
    );
    if table.source_binding == PROJECT_YIR_BINDING_CONTRACT {
        let entries = table
            .functions
            .iter()
            .map(|function| function.entry.as_str())
            .collect::<Vec<_>>();
        let identity_hash = project_code_asset_identity_hash(
            &table.source_fnv1a64,
            table.lowering_target,
            &entries,
        );
        let asset_id = format!("kernel.cuda.project.{}", &identity_hash[2..]);
        let identity_set_root_hash = code_asset_identity_set_root_hash(&[(
            &asset_id,
            KERNEL_PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
            &identity_hash,
        )]);
        out.push_str(&format!(
            "project_code_asset_identity_contract = \"{KERNEL_PROJECT_CODE_ASSET_IDENTITY_CONTRACT}\"\nproject_code_asset_id = \"{asset_id}\"\nproject_code_asset_source_fnv1a64 = \"{}\"\nproject_code_asset_lowering_target = \"{}\"\nproject_code_asset_entry_count = {}\nproject_code_asset_entries = [{}]\nproject_code_asset_identity_hash = \"{}\"\nproject_code_asset_identity_set_contract = \"{KERNEL_CODE_ASSET_IDENTITY_SET_CONTRACT}\"\nproject_code_asset_identity_set_count = 1\nproject_code_asset_identity_set_asset_ids = [\"{asset_id}\"]\nproject_code_asset_identity_set_contracts = [\"{KERNEL_PROJECT_CODE_ASSET_IDENTITY_CONTRACT}\"]\nproject_code_asset_identity_set_hashes = [\"{identity_hash}\"]\nproject_code_asset_identity_set_root_hash = \"{identity_set_root_hash}\"\n",
            table.source_fnv1a64,
            table.lowering_target,
            entries.len(),
            entries
                .iter()
                .map(|entry| format!("\"{entry}\""))
                .collect::<Vec<_>>()
                .join(", "),
            identity_hash,
        ));
    }
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
    for adaptation in &table.source_adaptations {
        out.push_str("\n[[source_adaptation]]\n");
        out.push_str(&format!(
            "contract = \"{}\"\nsource_function = \"{}\"\nsource_node = \"{}\"\nsource_instruction = \"{}\"\nstatus = \"{}\"\n",
            adaptation.contract,
            adaptation.source_function,
            adaptation.source_node,
            adaptation.source_instruction,
            adaptation.status.as_str()
        ));
        if let Some(entry) = &adaptation.generated_entry {
            out.push_str(&format!("generated_entry = \"{entry}\"\n"));
        }
        if let Some(projection) = &adaptation.request_projection {
            out.push_str(&format!(
                "request_projection_contract = \"{}\"\nrequest_operation = \"{}\"\nrequest_element_type = \"{}\"\n",
                projection.contract, projection.operation, projection.element_type
            ));
            render_usize_array(&mut out, "request_input_shape", &projection.input_shape);
            render_usize_array(&mut out, "request_output_shape", &projection.output_shape);
            render_i64_array(&mut out, "request_input_values", &projection.input_values);
            if let Some(scalar) = projection.scalar {
                out.push_str(&format!("request_scalar = {scalar}\n"));
            }
            if let Some(input_source_node) = &projection.input_source_node {
                out.push_str(&format!(
                    "request_input_source_node = \"{input_source_node}\"\n"
                ));
            }
            render_i64_array(
                &mut out,
                "request_expected_values",
                &projection.expected_values,
            );
        }
        if let Some(projection) = &adaptation.result_projection {
            out.push_str(&format!(
                "result_projection_contract = \"{}\"\nresult_element_type = \"{}\"\nresult_input_source_node = \"{}\"\nresult_row = {}\nresult_col = {}\nresult_expected_i64 = {}\n",
                projection.contract,
                projection.element_type,
                projection.input_source_node,
                projection.row,
                projection.col,
                projection.expected_i64
            ));
        }
        out.push_str(&format!("diagnostic = \"{}\"\n", adaptation.diagnostic));
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

fn project_code_asset_identity_hash(
    source_fnv1a64: &str,
    lowering_target: &str,
    entries: &[&str],
) -> String {
    fnv1a64_hex(
        format!(
            "{KERNEL_PROJECT_CODE_ASSET_IDENTITY_CONTRACT}\n{source_fnv1a64}\n{lowering_target}\n{}\n{}",
            entries.len(),
            entries.join("\n")
        )
        .as_bytes(),
    )
}

pub(crate) fn code_asset_identity_set_root_hash(items: &[(&str, &str, &str)]) -> String {
    let ordered_items = items
        .iter()
        .map(|(asset_id, contract, identity_hash)| {
            format!("{asset_id}\n{contract}\n{identity_hash}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fnv1a64_hex(
        format!(
            "{KERNEL_CODE_ASSET_IDENTITY_SET_CONTRACT}\n{}\n{ordered_items}",
            items.len()
        )
        .as_bytes(),
    )
}

fn render_usize_array(out: &mut String, key: &str, values: &[usize]) {
    out.push_str(&format!(
        "{key} = [{}]\n",
        values
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    ));
}

fn render_i64_array(out: &mut String, key: &str, values: &[i64]) {
    out.push_str(&format!(
        "{key} = [{}]\n",
        values
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    ));
}

fn registered_provider_functions() -> Vec<KernelYirCodegenFunction> {
    vec![
        KernelYirCodegenFunction {
            contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
            entry: "nuis_kernel_vector_add_f32".to_owned(),
            parameters: vec![
                parameter("input_lhs", KernelParameterKind::InputF32),
                parameter("input_rhs", KernelParameterKind::InputF32),
                parameter("output", KernelParameterKind::OutputF32),
                parameter("element_count", KernelParameterKind::ElementCountU32),
            ],
            nodes: vec![arithmetic_node(
                "sum",
                "add_f32",
                &["input_lhs", "input_rhs"],
            )],
            output_node: "sum".to_owned(),
        },
        KernelYirCodegenFunction {
            contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
            entry: "nuis_kernel_scale_f32".to_owned(),
            parameters: vec![
                parameter("input", KernelParameterKind::InputF32),
                parameter("output", KernelParameterKind::OutputF32),
                parameter("element_count", KernelParameterKind::ElementCountU32),
                parameter("scale", KernelParameterKind::ScalarF32),
            ],
            nodes: vec![arithmetic_node("scaled", "mul_f32", &["input", "scale"])],
            output_node: "scaled".to_owned(),
        },
        KernelYirCodegenFunction {
            contract: KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT,
            entry: "nuis_kernel_copy_u32".to_owned(),
            parameters: vec![
                parameter("input", KernelParameterKind::InputU32),
                parameter("output", KernelParameterKind::OutputU32),
                parameter("element_count", KernelParameterKind::ElementCountU32),
            ],
            nodes: vec![arithmetic_node("copied", "copy_u32", &["input"])],
            output_node: "copied".to_owned(),
        },
    ]
}

pub(crate) fn parameter(name: &str, kind: KernelParameterKind) -> KernelParameter {
    KernelParameter {
        name: name.to_owned(),
        kind,
    }
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
function-result main i64 value selected\n\
function-node main input\n\
function-node main scalar\n\
function-node main mapped\n\
function-node main reduced\n\
function-node main row\n\
function-node main col\n\
function-node main selected\n\
kernel.tensor input kernel0 1 4 1,2,3,4\n\
cpu.const_i64 scalar cpu0 10\n\
kernel.add_scalar_axis mapped kernel0 input cols scalar\n\
kernel.reduce_sum_axis reduced kernel0 mapped cols\n\
cpu.const_i64 row cpu0 0\n\
cpu.const_i64 col cpu0 0\n\
kernel.element_at selected kernel0 reduced row col\n\
kernel.target_config target kernel0 x86_64 cuda 1 ptx\n";

    #[test]
    fn compiled_project_yir_builds_verified_backend_neutral_table() {
        let table = table_from_compiled_project_yir(PROJECT_YIR, "cuda.nvidia-gpu").unwrap();
        assert_eq!(table.source_binding, PROJECT_YIR_BINDING_CONTRACT);
        assert_eq!(table.source_kernel_node_count, 5);
        assert_eq!(table.source_kernel_body_node_count, 4);
        assert_eq!(table.source_functions.len(), 1);
        assert_eq!(table.source_adaptations.len(), 4);
        assert_eq!(
            table
                .source_adaptations
                .iter()
                .filter(|adaptation| adaptation.is_adapted())
                .count(),
            2
        );
        assert_eq!(
            table
                .source_adaptations
                .iter()
                .filter(|adaptation| adaptation.is_projected())
                .count(),
            1
        );
        assert_eq!(table.functions.len(), 2);
        let rendered = render_codegen_table(&table).unwrap();
        assert!(rendered.contains("schema = \"nuis-kernel-yir-codegen-table-v1\""));
        assert!(rendered.contains("source_binding = \"compiled-project-yir\""));
        assert!(rendered.contains("[[source_function]]"));
        assert!(rendered.contains(
            "body_nodes = [\"input\", \"scalar\", \"mapped\", \"reduced\", \"row\", \"col\", \"selected\"]"
        ));
        assert!(
            rendered.contains("source_adaptation_contract = \"nuis-kernel-yir-source-adapter-v1\"")
        );
        assert!(rendered.contains("source_adapted_count = 2"));
        assert!(rendered.contains("source_projected_count = 1"));
        assert!(rendered.contains(
            "project_code_asset_identity_contract = \"nuis-kernel-project-code-asset-identity-v1\""
        ));
        assert!(rendered.contains("project_code_asset_id = \"kernel.cuda.project."));
        assert!(rendered.contains(&format!(
            "project_code_asset_source_fnv1a64 = \"{}\"",
            table.source_fnv1a64
        )));
        assert!(rendered.contains(
            "project_code_asset_entries = [\"nuis_project_main_mapped_i64\", \"nuis_project_main_reduced_i64\"]"
        ));
        assert!(rendered.contains("project_code_asset_identity_hash = \"0x"));
        assert!(rendered.contains(
            "project_code_asset_identity_set_contract = \"nuis-provider-code-asset-identity-set-v1\""
        ));
        assert!(rendered.contains("project_code_asset_identity_set_count = 1"));
        assert!(rendered.contains("project_code_asset_identity_set_root_hash = \"0x"));
        assert!(rendered.contains("status = \"unsupported\""));
        assert!(rendered.contains("status = \"adapted\""));
        assert!(rendered.contains("generated_entry = \"nuis_project_main_mapped_i64\""));
        assert!(rendered.contains(
            "request_projection_contract = \"nuis-kernel-source-request-projection-v1\""
        ));
        assert!(rendered.contains("request_element_type = \"i64\""));
        assert!(rendered.contains("request_operation = \"add-scalar-i64\""));
        assert!(rendered.contains("request_input_shape = [1, 4]"));
        assert!(rendered.contains("request_output_shape = [1, 4]"));
        assert!(rendered.contains("request_input_values = [1, 2, 3, 4]"));
        assert!(rendered.contains("request_scalar = 10"));
        assert!(rendered.contains("request_expected_values = [11, 12, 13, 14]"));
        assert!(rendered.contains("generated_entry = \"nuis_project_main_reduced_i64\""));
        assert!(rendered.contains("request_operation = \"reduce-sum-i64\""));
        assert!(rendered.contains("request_input_source_node = \"mapped\""));
        assert!(rendered.contains("request_output_shape = [1, 1]"));
        assert!(rendered.contains("request_expected_values = [50]"));
        assert!(rendered.contains("status = \"projected\""));
        assert!(rendered
            .contains("result_projection_contract = \"nuis-kernel-source-result-projection-v1\""));
        assert!(rendered.contains("result_input_source_node = \"reduced\""));
        assert!(rendered.contains("result_expected_i64 = 50"));
        assert!(!rendered.contains("kernel.add_f32"));
        assert!(!rendered.contains("kernel.mul_f32"));
        assert!(rendered.contains("kernel.add_i64"));
        assert!(rendered.contains("kernel.reduce_sum_i64"));
        let ptx = crate::kernel_ptx_emitter::lower_cuda_ptx(&table).unwrap();
        assert!(!ptx.contains(".visible .entry nuis_kernel_vector_add_f32"));
        assert!(!ptx.contains(".visible .entry nuis_kernel_scale_f32"));
        assert!(ptx.contains(".visible .entry nuis_project_main_mapped_i64"));
        assert!(ptx.contains("ld.global.s64"));
        assert!(ptx.contains("add.s64"));
        assert!(ptx.contains("st.global.s64"));
        assert!(ptx.contains(".visible .entry nuis_project_main_reduced_i64"));
        assert!(ptx.contains("REDUCE_LOOP_"));
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

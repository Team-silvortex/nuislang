use std::collections::BTreeMap;

use yir_core::{Node, Operation};

pub(crate) const KERNEL_PTX_EMITTER_REGISTRY_CONTRACT: &str = "nuis-kernel-ptx-emitter-registry-v1";
pub(crate) const KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT: &str = "nuis-kernel-yir-codegen-function-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KernelParameterKind {
    InputF32,
    OutputF32,
    ElementCountU32,
    ScalarF32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KernelParameter {
    name: &'static str,
    kind: KernelParameterKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KernelYirCodegenFunction {
    contract: &'static str,
    entry: &'static str,
    parameters: &'static [KernelParameter],
    nodes: Vec<Node>,
    output_node: &'static str,
}

struct KernelPtxEmitterRegistration {
    registry_contract: &'static str,
    target: &'static str,
    module: &'static str,
    supported_instructions: &'static [&'static str],
    emit: fn(&[KernelYirCodegenFunction]) -> Result<String, String>,
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

const CUDA_PTX_EMITTER: KernelPtxEmitterRegistration = KernelPtxEmitterRegistration {
    registry_contract: KERNEL_PTX_EMITTER_REGISTRY_CONTRACT,
    target: "cuda.nvidia-gpu",
    module: "kernel",
    supported_instructions: &["add_f32", "mul_f32"],
    emit: emit_ptx_module,
};

pub(crate) fn lower_registered_cuda_ptx() -> Result<String, String> {
    validate_registration(&CUDA_PTX_EMITTER)?;
    (CUDA_PTX_EMITTER.emit)(&registered_functions())
}

#[cfg(test)]
pub(crate) fn registered_cuda_yir_entries() -> Vec<&'static str> {
    registered_functions()
        .into_iter()
        .map(|function| function.entry)
        .collect()
}

fn registered_functions() -> Vec<KernelYirCodegenFunction> {
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

fn validate_registration(registration: &KernelPtxEmitterRegistration) -> Result<(), String> {
    if registration.registry_contract != KERNEL_PTX_EMITTER_REGISTRY_CONTRACT
        || registration.target != "cuda.nvidia-gpu"
        || registration.module != "kernel"
        || registration.supported_instructions.is_empty()
    {
        return Err("CUDA PTX emitter registration is invalid".to_owned());
    }
    Ok(())
}

fn emit_ptx_module(functions: &[KernelYirCodegenFunction]) -> Result<String, String> {
    if functions.is_empty() {
        return Err("CUDA PTX emitter requires at least one Kernel/YIR function".to_owned());
    }
    let mut output = String::from(".version 8.0\n.target sm_80\n.address_size 64\n\n");
    for (index, function) in functions.iter().enumerate() {
        if index != 0 {
            output.push('\n');
        }
        output.push_str(&emit_ptx_function(function, index)?);
    }
    Ok(output)
}

fn emit_ptx_function(
    function: &KernelYirCodegenFunction,
    function_index: usize,
) -> Result<String, String> {
    validate_function(function)?;
    let mut output = format!(".visible .entry {}(\n", function.entry);
    for (index, parameter) in function.parameters.iter().enumerate() {
        let suffix = if index + 1 == function.parameters.len() {
            ""
        } else {
            ","
        };
        output.push_str(&format!(
            "    .param .{} {}{}\n",
            ptx_parameter_type(parameter.kind),
            parameter.name,
            suffix
        ));
    }
    output.push_str(
        ")\n{\n    .reg .pred %p<2>;\n    .reg .b32 %r<8>;\n    .reg .b64 %rd<24>;\n    .reg .f32 %f<16>;\n\n",
    );

    let mut pointer_registers = BTreeMap::new();
    let mut value_registers = BTreeMap::new();
    let mut next_pointer = 1usize;
    let mut next_value = 1usize;
    let mut output_parameter = None;
    for parameter in function.parameters {
        match parameter.kind {
            KernelParameterKind::InputF32 | KernelParameterKind::OutputF32 => {
                output.push_str(&format!(
                    "    ld.param.u64 %rd{next_pointer}, [{}];\n",
                    parameter.name
                ));
                pointer_registers.insert(parameter.name, next_pointer);
                if parameter.kind == KernelParameterKind::OutputF32 {
                    output_parameter = Some(parameter.name);
                }
                next_pointer += 1;
            }
            KernelParameterKind::ElementCountU32 => {
                output.push_str(&format!("    ld.param.u32 %r1, [{}];\n", parameter.name));
            }
            KernelParameterKind::ScalarF32 => {
                output.push_str(&format!(
                    "    ld.param.f32 %f{next_value}, [{}];\n",
                    parameter.name
                ));
                value_registers.insert(parameter.name, next_value);
                next_value += 1;
            }
        }
    }
    output.push_str(
        "    mov.u32 %r2, %ctaid.x;\n    mov.u32 %r3, %ntid.x;\n    mov.u32 %r4, %tid.x;\n    mad.lo.s32 %r5, %r2, %r3, %r4;\n    setp.ge.u32 %p1, %r5, %r1;\n",
    );
    output.push_str(&format!("    @%p1 bra DONE_{function_index};\n"));
    output.push_str("    mul.wide.u32 %rd8, %r5, 4;\n");

    let mut next_address = 9usize;
    for parameter in function
        .parameters
        .iter()
        .filter(|parameter| parameter.kind == KernelParameterKind::InputF32)
    {
        let pointer = pointer_registers[parameter.name];
        output.push_str(&format!(
            "    add.s64 %rd{next_address}, %rd{pointer}, %rd8;\n    ld.global.f32 %f{next_value}, [%rd{next_address}];\n"
        ));
        value_registers.insert(parameter.name, next_value);
        next_address += 1;
        next_value += 1;
    }

    for node in &function.nodes {
        let lhs = value_register(node, 0, &value_registers)?;
        let rhs = value_register(node, 1, &value_registers)?;
        let instruction = match node.op.instruction.as_str() {
            "add_f32" => "add.rn.f32",
            "mul_f32" => "mul.rn.f32",
            other => return Err(format!("unsupported CUDA Kernel/YIR instruction `{other}`")),
        };
        output.push_str(&format!(
            "    {instruction} %f{next_value}, %f{lhs}, %f{rhs};\n"
        ));
        value_registers.insert(node.name.as_str(), next_value);
        next_value += 1;
    }

    let output_parameter = output_parameter
        .ok_or_else(|| format!("Kernel/YIR function `{}` has no output", function.entry))?;
    let output_pointer = pointer_registers[output_parameter];
    let output_value = value_registers[function.output_node];
    output.push_str(&format!(
        "    add.s64 %rd{next_address}, %rd{output_pointer}, %rd8;\n    st.global.f32 [%rd{next_address}], %f{output_value};\nDONE_{function_index}:\n    ret;\n}}\n"
    ));
    Ok(output)
}

fn validate_function(function: &KernelYirCodegenFunction) -> Result<(), String> {
    if function.contract != KERNEL_YIR_CODEGEN_FUNCTION_CONTRACT
        || !valid_identifier(function.entry)
        || function.nodes.is_empty()
        || function
            .parameters
            .iter()
            .any(|parameter| !valid_identifier(parameter.name))
        || function
            .nodes
            .iter()
            .any(|node| node.resource != "kernel0" || node.op.module != CUDA_PTX_EMITTER.module)
        || function.nodes.iter().any(|node| {
            !CUDA_PTX_EMITTER
                .supported_instructions
                .contains(&node.op.instruction.as_str())
        })
        || !function
            .nodes
            .iter()
            .any(|node| node.name == function.output_node)
        || function
            .parameters
            .iter()
            .filter(|parameter| parameter.kind == KernelParameterKind::OutputF32)
            .count()
            != 1
        || function
            .parameters
            .iter()
            .filter(|parameter| parameter.kind == KernelParameterKind::ElementCountU32)
            .count()
            != 1
    {
        return Err(format!(
            "Kernel/YIR function `{}` does not match the registered PTX emitter",
            function.entry
        ));
    }
    Ok(())
}

fn value_register(
    node: &Node,
    argument_index: usize,
    registers: &BTreeMap<&str, usize>,
) -> Result<usize, String> {
    let argument = node
        .op
        .args
        .get(argument_index)
        .ok_or_else(|| format!("Kernel/YIR node `{}` is missing an operand", node.name))?;
    registers.get(argument.as_str()).copied().ok_or_else(|| {
        format!(
            "Kernel/YIR node `{}` references unavailable value `{argument}`",
            node.name
        )
    })
}

fn ptx_parameter_type(kind: KernelParameterKind) -> &'static str {
    match kind {
        KernelParameterKind::InputF32 | KernelParameterKind::OutputF32 => "u64",
        KernelParameterKind::ElementCountU32 => "u32",
        KernelParameterKind::ScalarF32 => "f32",
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_cuda_emitter_lowers_kernel_yir_arithmetic() {
        let ptx = lower_registered_cuda_ptx().expect("registered CUDA PTX");
        assert_eq!(
            registered_cuda_yir_entries(),
            ["nuis_kernel_vector_add_f32", "nuis_kernel_scale_f32"]
        );
        assert!(ptx.contains(".visible .entry nuis_kernel_vector_add_f32"));
        assert!(ptx.contains(".visible .entry nuis_kernel_scale_f32"));
        assert!(ptx.contains("add.rn.f32"));
        assert!(ptx.contains("mul.rn.f32"));
        assert!(ptx.contains("st.global.f32"));
        assert!(!ptx.contains("nvcc"));
    }

    #[test]
    fn emitter_rejects_unregistered_kernel_yir_instruction() {
        let mut function = registered_functions().remove(0);
        function.nodes[0].op.instruction = "div_f32".to_owned();
        assert!(emit_ptx_function(&function, 0)
            .unwrap_err()
            .contains("registered PTX emitter"));
    }
}

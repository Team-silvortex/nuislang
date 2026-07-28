use std::collections::BTreeMap;

use yir_core::Node;

use crate::kernel_codegen_table::{
    validate_codegen_table, KernelParameterKind, KernelYirCodegenFunction, KernelYirCodegenTable,
};

pub(crate) const KERNEL_PTX_EMITTER_REGISTRY_CONTRACT: &str = "nuis-kernel-ptx-emitter-registry-v1";

struct KernelPtxEmitterRegistration {
    registry_contract: &'static str,
    target: &'static str,
    module: &'static str,
    supported_instructions: &'static [&'static str],
    emit: fn(&[KernelYirCodegenFunction]) -> Result<String, String>,
}

const CUDA_PTX_EMITTER: KernelPtxEmitterRegistration = KernelPtxEmitterRegistration {
    registry_contract: KERNEL_PTX_EMITTER_REGISTRY_CONTRACT,
    target: "cuda.nvidia-gpu",
    module: "kernel",
    supported_instructions: &["add_f32", "mul_f32", "add_i64", "reduce_sum_i64"],
    emit: emit_ptx_module,
};

pub(crate) fn lower_cuda_ptx(table: &KernelYirCodegenTable) -> Result<String, String> {
    validate_registration(&CUDA_PTX_EMITTER)?;
    validate_codegen_table(table)?;
    if table.lowering_target != CUDA_PTX_EMITTER.target {
        return Err(format!(
            "Kernel/YIR codegen table targets `{}` instead of `{}`",
            table.lowering_target, CUDA_PTX_EMITTER.target
        ));
    }
    (CUDA_PTX_EMITTER.emit)(&table.functions)
}

#[cfg(test)]
pub(crate) fn cuda_yir_entries(table: &KernelYirCodegenTable) -> Vec<&str> {
    table
        .functions
        .iter()
        .map(|function| function.entry.as_str())
        .collect()
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
    if function.nodes.len() == 1 && function.nodes[0].op.instruction == "reduce_sum_i64" {
        return emit_reduce_sum_i64(function, function_index);
    }
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
        ")\n{\n    .reg .pred %p<2>;\n    .reg .b32 %r<8>;\n    .reg .b64 %rd<24>;\n    .reg .b64 %l<16>;\n    .reg .f32 %f<16>;\n\n",
    );

    let mut pointer_registers = BTreeMap::new();
    let mut value_registers = BTreeMap::new();
    let mut next_pointer = 1usize;
    let mut next_value = 1usize;
    let mut output_parameter = None;
    for parameter in &function.parameters {
        match parameter.kind {
            KernelParameterKind::InputF32
            | KernelParameterKind::OutputF32
            | KernelParameterKind::InputI64
            | KernelParameterKind::OutputI64 => {
                output.push_str(&format!(
                    "    ld.param.u64 %rd{next_pointer}, [{}];\n",
                    parameter.name
                ));
                pointer_registers.insert(parameter.name.as_str(), next_pointer);
                if matches!(
                    parameter.kind,
                    KernelParameterKind::OutputF32 | KernelParameterKind::OutputI64
                ) {
                    output_parameter = Some(parameter.name.as_str());
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
                value_registers.insert(parameter.name.as_str(), next_value);
                next_value += 1;
            }
            KernelParameterKind::ScalarI64 => {
                output.push_str(&format!(
                    "    ld.param.s64 %l{next_value}, [{}];\n",
                    parameter.name
                ));
                value_registers.insert(parameter.name.as_str(), next_value);
                next_value += 1;
            }
        }
    }
    output.push_str(
        "    mov.u32 %r2, %ctaid.x;\n    mov.u32 %r3, %ntid.x;\n    mov.u32 %r4, %tid.x;\n    mad.lo.s32 %r5, %r2, %r3, %r4;\n    setp.ge.u32 %p1, %r5, %r1;\n",
    );
    output.push_str(&format!("    @%p1 bra DONE_{function_index};\n"));
    let integer_function = function
        .parameters
        .iter()
        .any(|parameter| is_i64_parameter(parameter.kind));
    let element_size = if integer_function { 8 } else { 4 };
    output.push_str(&format!("    mul.wide.u32 %rd8, %r5, {element_size};\n"));

    let mut next_address = 9usize;
    for parameter in function.parameters.iter().filter(|parameter| {
        matches!(
            parameter.kind,
            KernelParameterKind::InputF32 | KernelParameterKind::InputI64
        )
    }) {
        let pointer = pointer_registers[parameter.name.as_str()];
        let load = match parameter.kind {
            KernelParameterKind::InputF32 => {
                format!("ld.global.f32 %f{next_value}, [%rd{next_address}];")
            }
            KernelParameterKind::InputI64 => {
                format!("ld.global.s64 %l{next_value}, [%rd{next_address}];")
            }
            _ => unreachable!(),
        };
        output.push_str(&format!(
            "    add.s64 %rd{next_address}, %rd{pointer}, %rd8;\n    {load}\n"
        ));
        value_registers.insert(parameter.name.as_str(), next_value);
        next_address += 1;
        next_value += 1;
    }

    for node in &function.nodes {
        let lhs = value_register(node, 0, &value_registers)?;
        let rhs = value_register(node, 1, &value_registers)?;
        let (instruction, register) = match node.op.instruction.as_str() {
            "add_f32" => ("add.rn.f32", "%f"),
            "mul_f32" => ("mul.rn.f32", "%f"),
            "add_i64" => ("add.s64", "%l"),
            other => return Err(format!("unsupported CUDA Kernel/YIR instruction `{other}`")),
        };
        output.push_str(&format!(
            "    {instruction} {register}{next_value}, {register}{lhs}, {register}{rhs};\n"
        ));
        value_registers.insert(node.name.as_str(), next_value);
        next_value += 1;
    }

    let output_parameter = output_parameter
        .ok_or_else(|| format!("Kernel/YIR function `{}` has no output", function.entry))?;
    let output_pointer = pointer_registers[output_parameter];
    let output_value = value_registers
        .get(function.output_node.as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "Kernel/YIR function `{}` has no output value `{}`",
                function.entry, function.output_node
            )
        })?;
    let store = if integer_function {
        format!("st.global.s64 [%rd{next_address}], %l{output_value};")
    } else {
        format!("st.global.f32 [%rd{next_address}], %f{output_value};")
    };
    output.push_str(&format!(
        "    add.s64 %rd{next_address}, %rd{output_pointer}, %rd8;\n    {store}\nDONE_{function_index}:\n    ret;\n}}\n"
    ));
    Ok(output)
}

fn emit_reduce_sum_i64(
    function: &KernelYirCodegenFunction,
    function_index: usize,
) -> Result<String, String> {
    let expected_parameters = [
        ("input", KernelParameterKind::InputI64),
        ("output", KernelParameterKind::OutputI64),
        ("element_count", KernelParameterKind::ElementCountU32),
    ];
    if function.parameters.len() != expected_parameters.len()
        || function
            .parameters
            .iter()
            .zip(expected_parameters)
            .any(|(actual, expected)| (actual.name.as_str(), actual.kind) != expected)
    {
        return Err(format!(
            "Kernel/YIR reduction function `{}` does not match the registered PTX ABI",
            function.entry
        ));
    }
    Ok(format!(
        ".visible .entry {}(\n\
         \x20   .param .u64 input,\n\
         \x20   .param .u64 output,\n\
         \x20   .param .u32 element_count\n\
         )\n\
         {{\n\
         \x20   .reg .pred %p<3>;\n\
         \x20   .reg .b32 %r<8>;\n\
         \x20   .reg .b64 %rd<8>;\n\
         \x20   .reg .b64 %l<4>;\n\n\
         \x20   ld.param.u64 %rd1, [input];\n\
         \x20   ld.param.u64 %rd2, [output];\n\
         \x20   ld.param.u32 %r1, [element_count];\n\
         \x20   mov.u32 %r2, %ctaid.x;\n\
         \x20   mov.u32 %r3, %ntid.x;\n\
         \x20   mov.u32 %r4, %tid.x;\n\
         \x20   mad.lo.s32 %r5, %r2, %r3, %r4;\n\
         \x20   setp.ne.u32 %p1, %r5, 0;\n\
         \x20   @%p1 bra REDUCE_DONE_{function_index};\n\
         \x20   mov.u32 %r6, 0;\n\
         \x20   mov.b64 %l1, 0;\n\
         REDUCE_LOOP_{function_index}:\n\
         \x20   setp.ge.u32 %p2, %r6, %r1;\n\
         \x20   @%p2 bra REDUCE_STORE_{function_index};\n\
         \x20   mul.wide.u32 %rd3, %r6, 8;\n\
         \x20   add.s64 %rd4, %rd1, %rd3;\n\
         \x20   ld.global.s64 %l2, [%rd4];\n\
         \x20   add.s64 %l1, %l1, %l2;\n\
         \x20   add.u32 %r6, %r6, 1;\n\
         \x20   bra REDUCE_LOOP_{function_index};\n\
         REDUCE_STORE_{function_index}:\n\
         \x20   st.global.s64 [%rd2], %l1;\n\
         REDUCE_DONE_{function_index}:\n\
         \x20   ret;\n\
         }}\n",
        function.entry
    ))
}

fn validate_function(function: &KernelYirCodegenFunction) -> Result<(), String> {
    let has_f32_parameters = function
        .parameters
        .iter()
        .any(|parameter| is_f32_parameter(parameter.kind));
    let has_i64_parameters = function
        .parameters
        .iter()
        .any(|parameter| is_i64_parameter(parameter.kind));
    if !valid_identifier(&function.entry)
        || function.nodes.is_empty()
        || function
            .parameters
            .iter()
            .any(|parameter| !valid_identifier(&parameter.name))
        || has_f32_parameters == has_i64_parameters
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
            .filter(|parameter| {
                matches!(
                    parameter.kind,
                    KernelParameterKind::OutputF32 | KernelParameterKind::OutputI64
                )
            })
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
        KernelParameterKind::InputF32
        | KernelParameterKind::OutputF32
        | KernelParameterKind::InputI64
        | KernelParameterKind::OutputI64 => "u64",
        KernelParameterKind::ElementCountU32 => "u32",
        KernelParameterKind::ScalarF32 => "f32",
        KernelParameterKind::ScalarI64 => "s64",
    }
}

fn is_f32_parameter(kind: KernelParameterKind) -> bool {
    matches!(
        kind,
        KernelParameterKind::InputF32
            | KernelParameterKind::OutputF32
            | KernelParameterKind::ScalarF32
    )
}

fn is_i64_parameter(kind: KernelParameterKind) -> bool {
    matches!(
        kind,
        KernelParameterKind::InputI64
            | KernelParameterKind::OutputI64
            | KernelParameterKind::ScalarI64
    )
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
        let table = crate::kernel_codegen_table::registered_provider_codegen_table();
        let ptx = lower_cuda_ptx(&table).expect("registered CUDA PTX");
        assert_eq!(
            cuda_yir_entries(&table),
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
        let mut function = crate::kernel_codegen_table::registered_provider_codegen_table()
            .functions
            .remove(0);
        function.nodes[0].op.instruction = "div_f32".to_owned();
        assert!(emit_ptx_function(&function, 0)
            .unwrap_err()
            .contains("registered PTX emitter"));
    }
}

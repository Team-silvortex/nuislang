use std::collections::{BTreeMap, BTreeSet};

use yir_core::Node;

use super::{
    call_return::cpu_scalar_kind_llvm_type,
    fresh_reg,
    value_ref::{get_bool, get_f32, get_f64, get_i32, get_i64},
    CpuCallScalarKind, CpuHelperSignature, LlvmValueRef, TaskThunkArgument,
};

pub(crate) fn lower_cpu_call_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    deferred_task_calls: &BTreeSet<String>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }
    if !matches!(
        node.op.instruction.as_str(),
        "call_bool" | "call_i32" | "call_i64" | "call_f32" | "call_f64" | "call_owned_struct"
    ) {
        return Ok(false);
    }

    let callee = &node.op.args[0];
    let Some(signature) = helper_signatures.get(callee) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because helper signature `{}` is unavailable",
            node.op.instruction, node.name, callee
        ));
        return Ok(true);
    };

    let argument_offset = usize::from(node.op.instruction == "call_owned_struct") + 1;
    let lowered_args = node.op.args[argument_offset..]
        .iter()
        .zip(signature.params.iter())
        .map(|(arg, kind)| match kind {
            CpuCallScalarKind::Bool => get_bool(registers, arg).map(|value| format!("i1 {value}")),
            CpuCallScalarKind::I32 => get_i32(registers, arg).map(|value| format!("i32 {value}")),
            CpuCallScalarKind::I64 => get_i64(registers, arg).map(|value| format!("i64 {value}")),
            CpuCallScalarKind::F32 => get_f32(registers, arg).map(|value| format!("float {value}")),
            CpuCallScalarKind::F64 => {
                get_f64(registers, arg).map(|value| format!("double {value}"))
            }
        })
        .collect::<Option<Vec<_>>>();
    let Some(lowered_args) = lowered_args else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };

    if node.op.instruction == "call_owned_struct" && deferred_task_calls.contains(&node.name) {
        let template = parse_owned_struct_layout(&node.op.args[1])?;
        let arguments = lowered_args
            .iter()
            .zip(signature.params.iter().copied())
            .map(|(argument, kind)| TaskThunkArgument {
                kind,
                value: argument
                    .strip_prefix(cpu_scalar_kind_llvm_type(kind))
                    .and_then(|value| value.strip_prefix(' '))
                    .expect("owned struct helper argument should carry its LLVM ABI type")
                    .to_owned(),
            })
            .collect();
        registers.insert(
            node.name.clone(),
            LlvmValueRef::DeferredTaskThunkOwnedStruct {
                callee: callee.clone(),
                arguments,
                template,
            },
        );
        return Ok(true);
    }

    if deferred_task_calls.contains(&node.name)
        && matches!(
            signature.ret,
            CpuCallScalarKind::Bool
                | CpuCallScalarKind::I32
                | CpuCallScalarKind::I64
                | CpuCallScalarKind::F32
                | CpuCallScalarKind::F64
        )
        && signature.params.iter().all(|kind| {
            matches!(
                kind,
                CpuCallScalarKind::Bool
                    | CpuCallScalarKind::I32
                    | CpuCallScalarKind::I64
                    | CpuCallScalarKind::F32
                    | CpuCallScalarKind::F64
            )
        })
    {
        let arguments = lowered_args
            .iter()
            .zip(signature.params.iter().copied())
            .map(|(argument, kind)| TaskThunkArgument {
                kind,
                value: argument
                    .strip_prefix(cpu_scalar_kind_llvm_type(kind))
                    .and_then(|value| value.strip_prefix(' '))
                    .expect("scalar helper argument should carry its LLVM ABI type")
                    .to_owned(),
            })
            .collect();
        registers.insert(
            node.name.clone(),
            LlvmValueRef::DeferredTaskThunkScalar {
                callee: callee.clone(),
                arguments,
                return_kind: signature.ret,
            },
        );
        return Ok(true);
    }

    let reg = fresh_reg(next_reg);
    let symbol = format!("nuis_fn_{callee}");
    let call = match lowered_args.as_slice() {
        [] => format!(
            "call {} @{symbol}()",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        [a0] => format!(
            "call {} @{symbol}({a0})",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        [a0, a1] => format!(
            "call {} @{symbol}({a0}, {a1})",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        [a0, a1, a2] => format!(
            "call {} @{symbol}({a0}, {a1}, {a2})",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        _ => {
            body.push(format!(
                "  ; deferred lowering for cpu.{} `{}` because callee `{}` has unsupported arity {}",
                node.op.instruction,
                node.name,
                callee,
                lowered_args.len()
            ));
            return Ok(true);
        }
    };
    body.push(format!("  {reg} = {call}"));

    if node.op.instruction == "call_owned_struct" {
        let template = parse_owned_struct_layout(&node.op.args[1])?;
        let value = unpack_immediate_owned_struct(&reg, &template, body, next_reg);
        registers.insert(node.name.clone(), LlvmValueRef::Struct(value));
        return Ok(true);
    }

    match signature.ret {
        CpuCallScalarKind::Bool => {
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {reg} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: reg.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        CpuCallScalarKind::I32 => {
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        CpuCallScalarKind::I64 => {
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        CpuCallScalarKind::F32 => {
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fpext float {reg} to double"));
            let as_i64 = fresh_reg(next_reg);
            body.push(format!("  {as_i64} = fptosi double {widened} to i64"));
            *last_cpu_value = Some(as_i64);
        }
        CpuCallScalarKind::F64 => {
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let as_i64 = fresh_reg(next_reg);
            body.push(format!("  {as_i64} = fptosi double {reg} to i64"));
            *last_cpu_value = Some(as_i64);
        }
    }

    Ok(true)
}

fn unpack_immediate_owned_struct(
    pointer_bits: &str,
    template: &super::StructLlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> super::StructLlvmValueRef {
    let data = fresh_reg(next_reg);
    body.push(format!("  {data} = inttoptr i64 {pointer_bits} to ptr"));
    let mut fields = Vec::with_capacity(template.fields.len());
    for (index, (name, kind)) in template.fields.iter().enumerate() {
        let slot = if index == 0 {
            data.clone()
        } else {
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr i8, ptr {data}, i64 {}",
                index * 8
            ));
            slot
        };
        let packed = fresh_reg(next_reg);
        body.push(format!("  {packed} = load i64, ptr {slot}, align 8"));
        let value = match kind {
            LlvmValueRef::Bool { .. } => {
                let i1 = fresh_reg(next_reg);
                body.push(format!("  {i1} = trunc i64 {packed} to i1"));
                LlvmValueRef::Bool { i1, i64: packed }
            }
            LlvmValueRef::I32(_) => {
                let value = fresh_reg(next_reg);
                body.push(format!("  {value} = trunc i64 {packed} to i32"));
                LlvmValueRef::I32(value)
            }
            LlvmValueRef::I64(_) => LlvmValueRef::I64(packed),
            LlvmValueRef::F32(_) => {
                let bits = fresh_reg(next_reg);
                body.push(format!("  {bits} = trunc i64 {packed} to i32"));
                let value = fresh_reg(next_reg);
                body.push(format!("  {value} = bitcast i32 {bits} to float"));
                LlvmValueRef::F32(value)
            }
            LlvmValueRef::F64(_) => {
                let value = fresh_reg(next_reg);
                body.push(format!("  {value} = bitcast i64 {packed} to double"));
                LlvmValueRef::F64(value)
            }
            _ => unreachable!("owned struct layout parser only admits scalar fields"),
        };
        fields.push((name.clone(), value));
    }
    body.push(format!("  call void @free(ptr {data})"));
    super::StructLlvmValueRef {
        type_name: template.type_name.clone(),
        fields,
    }
}

fn parse_owned_struct_layout(layout: &str) -> Result<super::StructLlvmValueRef, String> {
    let (type_name, fields_source) = layout
        .split_once('|')
        .ok_or_else(|| format!("invalid owned struct layout `{layout}`"))?;
    let fields = fields_source
        .split(',')
        .map(|field| {
            let (name, kind) = field
                .split_once(':')
                .ok_or_else(|| format!("invalid owned struct field layout `{field}`"))?;
            let value = match kind {
                "bool" => LlvmValueRef::Bool {
                    i1: "false".to_owned(),
                    i64: "0".to_owned(),
                },
                "i32" => LlvmValueRef::I32("0".to_owned()),
                "i64" => LlvmValueRef::I64("0".to_owned()),
                "f32" => LlvmValueRef::F32("0.0".to_owned()),
                "f64" => LlvmValueRef::F64("0.0".to_owned()),
                _ => return Err(format!("unsupported owned struct field kind `{kind}`")),
            };
            Ok((name.to_owned(), value))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if type_name.is_empty() || fields.is_empty() {
        return Err(format!("owned struct layout `{layout}` cannot be empty"));
    }
    Ok(super::StructLlvmValueRef {
        type_name: type_name.to_owned(),
        fields,
    })
}

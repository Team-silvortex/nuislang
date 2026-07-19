use std::collections::{BTreeMap, BTreeSet};

use yir_core::{parse_branch_owned_call_args, Node};

use super::{
    call_return::cpu_scalar_kind_llvm_type,
    fresh_block, fresh_reg,
    value_ref::{coerce_to_i64, get_bool, get_f32, get_f64, get_i32, get_i64, get_ptr},
    CpuCallScalarKind, CpuHelperSignature, LlvmValueRef, TaskThunkArgument,
};

pub(crate) fn lower_cpu_branch_owned_call_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<bool, String> {
    if node.op.module != "cpu" || node.op.instruction != "branch_call_owned_bytes" {
        return Ok(false);
    }
    let Some(args) = parse_branch_owned_call_args(&node.op.args) else {
        return Err(format!(
            "cpu.branch_call_owned_bytes `{}` has invalid scalar argument segments",
            node.name
        ));
    };
    let then_callee = args.then_callee;
    let else_callee = args.else_callee;
    let then_signature = branch_owned_helper_signature(
        node,
        then_callee,
        args.then_scalar_args.len(),
        helper_signatures,
    )?;
    let else_signature = branch_owned_helper_signature(
        node,
        else_callee,
        args.else_scalar_args.len(),
        helper_signatures,
    )?;
    let Some(condition) = registers.get(args.condition) else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_call_owned_bytes `{}` because its condition is outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let Some(condition) = coerce_to_i64(condition, body, next_reg) else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_call_owned_bytes `{}` because its condition is not coercible to i64",
            node.name
        ));
        return Ok(true);
    };
    let Some(LlvmValueRef::OwnedBytes { blob: owner }) = registers.get(args.owner) else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_call_owned_bytes `{}` because its owner is outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let owner = owner.clone();
    let Some(then_scalar_args) = lower_branch_scalar_args(
        registers,
        args.then_scalar_args,
        &then_signature.params[1..],
    ) else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_call_owned_bytes `{}` because one or more then scalar inputs are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let Some(else_scalar_args) = lower_branch_scalar_args(
        registers,
        args.else_scalar_args,
        &else_signature.params[1..],
    ) else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_call_owned_bytes `{}` because one or more else scalar inputs are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let then_call_args = std::iter::once(format!("ptr {owner}"))
        .chain(then_scalar_args)
        .collect::<Vec<_>>()
        .join(", ");
    let else_call_args = std::iter::once(format!("ptr {owner}"))
        .chain(else_scalar_args)
        .collect::<Vec<_>>()
        .join(", ");
    let condition_i1 = fresh_reg(next_reg);
    let then_result = fresh_reg(next_reg);
    let else_result = fresh_reg(next_reg);
    let result = fresh_reg(next_reg);
    let then_label = fresh_block(next_block, "branch_owned_call_then");
    let else_label = fresh_block(next_block, "branch_owned_call_else");
    let merge_label = fresh_block(next_block, "branch_owned_call_merge");
    body.push(format!("  {condition_i1} = icmp ne i64 {condition}, 0"));
    body.push(format!(
        "  br i1 {condition_i1}, label %{then_label}, label %{else_label}"
    ));
    body.push(format!("{then_label}:"));
    body.push(format!(
        "  {then_result} = call ptr @nuis_fn_{then_callee}({then_call_args})"
    ));
    body.push(format!("  br label %{merge_label}"));
    body.push(format!("{else_label}:"));
    body.push(format!(
        "  {else_result} = call ptr @nuis_fn_{else_callee}({else_call_args})"
    ));
    body.push(format!("  br label %{merge_label}"));
    body.push(format!("{merge_label}:"));
    body.push(format!(
        "  {result} = phi ptr [ {then_result}, %{then_label} ], [ {else_result}, %{else_label} ]"
    ));
    registers.insert(node.name.clone(), LlvmValueRef::OwnedBytes { blob: result });
    Ok(true)
}

pub(crate) fn branch_owned_helper_signature<'a>(
    node: &Node,
    callee: &str,
    scalar_count: usize,
    helper_signatures: &'a BTreeMap<String, CpuHelperSignature>,
) -> Result<&'a CpuHelperSignature, String> {
    let signature = helper_signatures.get(callee).ok_or_else(|| {
        format!(
            "{} `{}` references unavailable owned helper `{callee}`",
            node.op.full_name(),
            node.name
        )
    })?;
    if signature.ret != CpuCallScalarKind::OwnedBytes
        || signature.params.first() != Some(&CpuCallScalarKind::OwnedBytes)
        || signature.params.len() != scalar_count + 1
        || signature.params[1..].iter().any(|kind| {
            matches!(
                kind,
                CpuCallScalarKind::BorrowedBuffer | CpuCallScalarKind::OwnedBytes
            )
        })
    {
        return Err(format!(
            "{} `{}` requires `{callee}` to have signature (Bytes, scalar...) -> Bytes matching its encoded arguments",
            node.op.full_name(), node.name
        ));
    }
    Ok(signature)
}

pub(crate) fn lower_branch_scalar_args(
    registers: &BTreeMap<String, LlvmValueRef>,
    args: &[String],
    kinds: &[CpuCallScalarKind],
) -> Option<Vec<String>> {
    args.iter()
        .zip(kinds)
        .map(|(arg, kind)| match kind {
            CpuCallScalarKind::Bool => get_bool(registers, arg).map(|value| format!("i1 {value}")),
            CpuCallScalarKind::I32 => get_i32(registers, arg).map(|value| format!("i32 {value}")),
            CpuCallScalarKind::I64 => get_i64(registers, arg).map(|value| format!("i64 {value}")),
            CpuCallScalarKind::F32 => get_f32(registers, arg).map(|value| format!("float {value}")),
            CpuCallScalarKind::F64 => {
                get_f64(registers, arg).map(|value| format!("double {value}"))
            }
            CpuCallScalarKind::BorrowedBuffer | CpuCallScalarKind::OwnedBytes => None,
        })
        .collect()
}

pub(crate) fn lower_cpu_call_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    buffer_lengths: &BTreeMap<String, String>,
    deferred_task_calls: &BTreeSet<String>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }
    if !matches!(
        node.op.instruction.as_str(),
        "call_bool"
            | "call_i32"
            | "call_i64"
            | "call_f32"
            | "call_f64"
            | "call_owned_bytes"
            | "call_owned_struct"
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
            CpuCallScalarKind::Bool => {
                get_bool(registers, arg).map(|value| vec![format!("i1 {value}")])
            }
            CpuCallScalarKind::I32 => {
                get_i32(registers, arg).map(|value| vec![format!("i32 {value}")])
            }
            CpuCallScalarKind::I64 => {
                get_i64(registers, arg).map(|value| vec![format!("i64 {value}")])
            }
            CpuCallScalarKind::F32 => {
                get_f32(registers, arg).map(|value| vec![format!("float {value}")])
            }
            CpuCallScalarKind::F64 => {
                get_f64(registers, arg).map(|value| vec![format!("double {value}")])
            }
            CpuCallScalarKind::BorrowedBuffer => get_ptr(registers, arg)
                .zip(buffer_lengths.get(arg))
                .map(|(ptr, len)| vec![format!("ptr {ptr}"), format!("i64 {len}")]),
            CpuCallScalarKind::OwnedBytes => match registers.get(arg) {
                Some(LlvmValueRef::OwnedBytes { blob }) => Some(vec![format!("ptr {blob}")]),
                _ => None,
            },
        })
        .collect::<Option<Vec<_>>>()
        .map(|args| args.into_iter().flatten().collect::<Vec<_>>());
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
        CpuCallScalarKind::BorrowedBuffer => unreachable!("borrowed refs cannot return"),
        CpuCallScalarKind::OwnedBytes => {
            registers.insert(node.name.clone(), LlvmValueRef::OwnedBytes { blob: reg });
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
    body.push(format!(
        "  call void @nuis_scheduler_owned_aggregate_require_v1(ptr {data})"
    ));
    let mut leaf_index = 0;
    let fields = unpack_immediate_fields(template, &data, &mut leaf_index, body, next_reg);
    body.push(format!(
        "  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr {data})"
    ));
    super::StructLlvmValueRef {
        type_name: template.type_name.clone(),
        fields,
    }
}

fn unpack_immediate_fields(
    template: &super::StructLlvmValueRef,
    data: &str,
    leaf_index: &mut usize,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Vec<(String, LlvmValueRef)> {
    template
        .fields
        .iter()
        .map(|(name, value)| {
            let value = match value {
                LlvmValueRef::Struct(nested) => LlvmValueRef::Struct(super::StructLlvmValueRef {
                    type_name: nested.type_name.clone(),
                    fields: unpack_immediate_fields(nested, data, leaf_index, body, next_reg),
                }),
                scalar => unpack_immediate_scalar(scalar, data, leaf_index, body, next_reg),
            };
            (name.clone(), value)
        })
        .collect()
}

fn unpack_immediate_scalar(
    template: &LlvmValueRef,
    data: &str,
    leaf_index: &mut usize,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> LlvmValueRef {
    if matches!(template, LlvmValueRef::OwnedBytes { .. }) {
        let blob = fresh_reg(next_reg);
        body.push(format!(
            "  {blob} = call ptr @nuis_scheduler_owned_aggregate_take_blob_v1(ptr {data}, i64 {leaf_index})"
        ));
        *leaf_index += 1;
        return LlvmValueRef::OwnedBytes { blob };
    }
    let packed = fresh_reg(next_reg);
    body.push(format!(
        "  {packed} = call i64 @nuis_scheduler_owned_aggregate_get_v1(ptr {data}, i64 {leaf_index})"
    ));
    *leaf_index += 1;
    match template {
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
        LlvmValueRef::TextHandle { .. } => {
            let blob = fresh_reg(next_reg);
            body.push(format!("  {blob} = inttoptr i64 {packed} to ptr"));
            let handle = fresh_reg(next_reg);
            body.push(format!(
                "  {handle} = call i64 @nuis_scheduler_owned_blob_text_lift_v1(ptr {blob})"
            ));
            let ptr = fresh_reg(next_reg);
            body.push(format!(
                "  {ptr} = call ptr @nuis_host_text_ptr(i64 {handle})"
            ));
            LlvmValueRef::TextHandle { ptr, handle }
        }
        _ => unreachable!("owned struct layout parser only admits scalar leaves"),
    }
}

fn parse_owned_struct_layout(layout: &str) -> Result<super::StructLlvmValueRef, String> {
    let mut parser = OwnedStructLayoutParser::new(layout);
    let parsed = parser.parse_struct()?;
    if parser.position != parser.source.len() {
        return Err(format!("trailing data in owned struct layout `{layout}`"));
    }
    Ok(parsed)
}

struct OwnedStructLayoutParser<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> OwnedStructLayoutParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            position: 0,
        }
    }

    fn parse_struct(&mut self) -> Result<super::StructLlvmValueRef, String> {
        let type_name = self.parse_name()?;
        self.parse_struct_body(type_name)
    }

    fn parse_struct_body(
        &mut self,
        type_name: String,
    ) -> Result<super::StructLlvmValueRef, String> {
        self.expect(b'{')?;
        let mut fields = Vec::new();
        loop {
            if self.consume(b'}') {
                break;
            }
            let name = self.parse_name()?;
            self.expect(b':')?;
            let kind = self.parse_name()?;
            let value = match scalar_template(&kind) {
                Some(value) => value,
                None => LlvmValueRef::Struct(self.parse_struct_body(kind)?),
            };
            fields.push((name, value));
            if self.consume(b'}') {
                break;
            }
            self.expect(b';')?;
        }
        if fields.is_empty() {
            return Err(format!("owned struct layout `{type_name}` cannot be empty"));
        }
        Ok(super::StructLlvmValueRef { type_name, fields })
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let start = self.position;
        while self.position < self.source.len()
            && !matches!(self.source[self.position], b'{' | b'}' | b':' | b';')
        {
            self.position += 1;
        }
        if start == self.position {
            return Err("expected name in owned struct layout".to_owned());
        }
        String::from_utf8(self.source[start..self.position].to_vec())
            .map_err(|_| "owned struct layout names must be UTF-8".to_owned())
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.consume(byte) {
            Ok(())
        } else {
            Err(format!(
                "expected `{}` in owned struct layout",
                byte as char
            ))
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.source.get(self.position) == Some(&byte) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

fn scalar_template(kind: &str) -> Option<LlvmValueRef> {
    match kind {
        "bool" => Some(LlvmValueRef::Bool {
            i1: "false".to_owned(),
            i64: "0".to_owned(),
        }),
        "i32" => Some(LlvmValueRef::I32("0".to_owned())),
        "i64" => Some(LlvmValueRef::I64("0".to_owned())),
        "f32" => Some(LlvmValueRef::F32("0.0".to_owned())),
        "f64" => Some(LlvmValueRef::F64("0.0".to_owned())),
        "String" => Some(LlvmValueRef::TextHandle {
            ptr: "null".to_owned(),
            handle: "0".to_owned(),
        }),
        "Bytes" => Some(LlvmValueRef::OwnedBytes {
            blob: "null".to_owned(),
        }),
        _ => None,
    }
}

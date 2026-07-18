use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    call_return::{
        can_emit_typed_return_from_value, cpu_scalar_kind_llvm_type, emit_typed_return_from_value,
    },
    fresh_block, fresh_reg, guard_host_call,
    value_ref::{coerce_to_cstr, coerce_to_i64, get_bool, get_f32, get_f64, get_i32},
    CpuCallScalarKind, LlvmValueRef,
};

pub(crate) enum GuardReturnLoweringOutcome {
    NotGuard,
    Continue,
    TerminalReturn,
}

pub(crate) fn lower_cpu_guard_return_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    next_block: &mut usize,
    function_return_kind: CpuCallScalarKind,
) -> Result<GuardReturnLoweringOutcome, String> {
    if node.op.module != "cpu" {
        return Ok(GuardReturnLoweringOutcome::NotGuard);
    }

    match node.op.instruction.as_str() {
        "branch_drop_owned_bytes_return" => {
            if node.op.args.len() != 5 {
                return Err(format!(
                    "cpu.branch_drop_owned_bytes_return `{}` expects condition and two bytes/return pairs",
                    node.name
                ));
            }
            let values = node
                .op
                .args
                .iter()
                .map(|arg| registers.get(arg).cloned())
                .collect::<Option<Vec<_>>>();
            let Some(values) = values else {
                body.push(format!(
                    "  ; deferred lowering for cpu.branch_drop_owned_bytes_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let Some(cond) = coerce_to_i64(&values[0], body, next_reg) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.branch_drop_owned_bytes_return `{}` because its condition is not coercible to i64",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let (
                LlvmValueRef::OwnedBytes { blob: then_blob },
                LlvmValueRef::OwnedBytes { blob: else_blob },
            ) = (&values[1], &values[3])
            else {
                body.push(format!(
                    "  ; deferred lowering for cpu.branch_drop_owned_bytes_return `{}` because an owned bytes input is unavailable",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            if !can_emit_typed_return_from_value(function_return_kind, &values[2])
                || !can_emit_typed_return_from_value(function_return_kind, &values[4])
            {
                body.push(format!(
                    "  ; deferred lowering for cpu.branch_drop_owned_bytes_return `{}` because a return value is not coercible to {}",
                    node.name,
                    cpu_scalar_kind_llvm_type(function_return_kind)
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }

            let cond_bool = fresh_reg(next_reg);
            body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
            let then_label = fresh_block(next_block, "branch_drop_bytes_return_then");
            let else_label = fresh_block(next_block, "branch_drop_bytes_return_else");
            body.push(format!(
                "  br i1 {cond_bool}, label %{then_label}, label %{else_label}"
            ));
            body.push(format!("{then_label}:"));
            body.push(format!(
                "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {then_blob})"
            ));
            emit_typed_return_from_value(body, next_reg, function_return_kind, &values[2]);
            body.push(format!("{else_label}:"));
            body.push(format!(
                "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {else_blob})"
            ));
            emit_typed_return_from_value(body, next_reg, function_return_kind, &values[4]);
            return Ok(GuardReturnLoweringOutcome::TerminalReturn);
        }
        "guard_drop_owned_bytes_return" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "cpu.guard_drop_owned_bytes_return `{}` expects condition, bytes, and return inputs",
                    node.name
                ));
            }
            let cond_value = registers.get(&node.op.args[0]).cloned();
            let bytes_value = registers.get(&node.op.args[1]).cloned();
            let return_value = registers.get(&node.op.args[2]).cloned();
            let (Some(cond_value), Some(bytes_value), Some(return_value)) =
                (cond_value, bytes_value, return_value)
            else {
                body.push(format!(
                    "  ; deferred lowering for cpu.guard_drop_owned_bytes_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.guard_drop_owned_bytes_return `{}` because its condition is not coercible to i64",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let LlvmValueRef::OwnedBytes { blob } = bytes_value else {
                body.push(format!(
                    "  ; deferred lowering for cpu.guard_drop_owned_bytes_return `{}` because its owned bytes input is unavailable",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            if !can_emit_typed_return_from_value(function_return_kind, &return_value) {
                body.push(format!(
                    "  ; deferred lowering for cpu.guard_drop_owned_bytes_return `{}` because its return value is not coercible to {}",
                    node.name,
                    cpu_scalar_kind_llvm_type(function_return_kind)
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }

            let cond_bool = fresh_reg(next_reg);
            body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
            let then_label = fresh_block(next_block, "guard_drop_bytes_return_then");
            let cont_label = fresh_block(next_block, "guard_drop_bytes_return_cont");
            body.push(format!(
                "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
            ));
            body.push(format!("{then_label}:"));
            body.push(format!(
                "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {blob})"
            ));
            if !emit_typed_return_from_value(body, next_reg, function_return_kind, &return_value) {
                unreachable!("return compatibility checked before branch emission");
            }
            body.push(format!("{cont_label}:"));
        }
        "guard_return" => {
            let cond_value = registers.get(&node.op.args[0]).cloned();
            let return_value = registers.get(&node.op.args[1]).cloned();
            let (Some(cond_value), Some(return_value)) = (cond_value, return_value) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_return `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            if matches!(
                return_value,
                LlvmValueRef::Struct(_) | LlvmValueRef::VariantUnion(_)
            ) {
                body.push(format!(
                    "  ; structural cpu.guard_return `{}` is resolved by downstream fieldwise selection",
                    node.name
                ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            let Some(returned) = coerce_to_i64(&return_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to i64",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let cond_bool = fresh_reg(next_reg);
            body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
            let then_label = fresh_block(next_block, "guard_return_then");
            let cont_label = fresh_block(next_block, "guard_return_cont");
            body.push(format!(
                "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
            ));
            body.push(format!("{then_label}:"));
            match function_return_kind {
                CpuCallScalarKind::Bool => {
                    let Some(returned_bool) = get_bool(registers, &node.op.args[1]) else {
                        body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to bool",
                                node.name
                            ));
                        return Ok(GuardReturnLoweringOutcome::Continue);
                    };
                    body.push(format!("  ret i1 {returned_bool}"));
                }
                CpuCallScalarKind::I32 => {
                    let Some(returned_i32) = get_i32(registers, &node.op.args[1]) else {
                        body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to i32",
                                node.name
                            ));
                        return Ok(GuardReturnLoweringOutcome::Continue);
                    };
                    body.push(format!("  ret i32 {returned_i32}"));
                }
                CpuCallScalarKind::I64 => {
                    body.push(format!("  ret i64 {returned}"));
                }
                CpuCallScalarKind::F32 => {
                    if let Some(returned_f32) = get_f32(registers, &node.op.args[1]) {
                        body.push(format!("  ret float {returned_f32}"));
                    } else if let Some(returned_f64) = get_f64(registers, &node.op.args[1]) {
                        let as_f32 = fresh_reg(next_reg);
                        body.push(format!(
                            "  {as_f32} = fptrunc double {returned_f64} to float"
                        ));
                        body.push(format!("  ret float {as_f32}"));
                    } else {
                        body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to f32",
                                node.name
                            ));
                        return Ok(GuardReturnLoweringOutcome::Continue);
                    }
                }
                CpuCallScalarKind::F64 => {
                    if let Some(returned_f64) = get_f64(registers, &node.op.args[1]) {
                        body.push(format!("  ret double {returned_f64}"));
                    } else if let Some(returned_f32) = get_f32(registers, &node.op.args[1]) {
                        let as_f64 = fresh_reg(next_reg);
                        body.push(format!("  {as_f64} = fpext float {returned_f32} to double"));
                        body.push(format!("  ret double {as_f64}"));
                    } else {
                        body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to f64",
                                node.name
                            ));
                        return Ok(GuardReturnLoweringOutcome::Continue);
                    }
                }
            }
            body.push(format!("{cont_label}:"));
        }
        "guard_print" => {
            let cond_value = registers.get(&node.op.args[0]).cloned();
            let print_value = registers.get(&node.op.args[1]).cloned();
            let (Some(cond_value), Some(print_value)) = (cond_value, print_value) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_print `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_print `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let cond_bool = fresh_reg(next_reg);
            body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
            let then_label = fresh_block(next_block, "guard_print_then");
            let cont_label = fresh_block(next_block, "guard_print_cont");
            body.push(format!(
                "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
            ));
            body.push(format!("{then_label}:"));
            if let Some(input) = coerce_to_cstr(&print_value, body, next_reg) {
                let print_reg = fresh_reg(next_reg);
                body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
            } else if let Some(input) = coerce_to_i64(&print_value, body, next_reg) {
                body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
            } else {
                body.push(format!(
                        "  ; deferred lowering inside cpu.guard_print `{}` because its print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
            }
            body.push(format!("  br label %{cont_label}"));
            body.push(format!("{cont_label}:"));
        }
        "guard_print_return" => {
            let cond_value = registers.get(&node.op.args[0]).cloned();
            let print_value = registers.get(&node.op.args[1]).cloned();
            let return_value = registers.get(&node.op.args[2]).cloned();
            let (Some(cond_value), Some(print_value), Some(return_value)) =
                (cond_value, print_value, return_value)
            else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            if !can_emit_typed_return_from_value(function_return_kind, &return_value) {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because its return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            let cond_bool = fresh_reg(next_reg);
            body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
            let then_label = fresh_block(next_block, "guard_print_return_then");
            let cont_label = fresh_block(next_block, "guard_print_return_cont");
            body.push(format!(
                "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
            ));
            body.push(format!("{then_label}:"));
            if let Some(input) = coerce_to_cstr(&print_value, body, next_reg) {
                let print_reg = fresh_reg(next_reg);
                body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
            } else if let Some(input) = coerce_to_i64(&print_value, body, next_reg) {
                body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
            } else {
                body.push(format!(
                        "  ; deferred lowering inside cpu.guard_print_return `{}` because its print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
            }
            if !emit_typed_return_from_value(body, next_reg, function_return_kind, &return_value) {
                body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because its return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            body.push(format!("{cont_label}:"));
        }
        "guard_host_call_return" => {
            if !guard_host_call::lower_guard_host_call_return(
                node,
                registers,
                body,
                next_reg,
                next_block,
                function_return_kind,
            ) {
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
        }
        "branch_host_call_return" => {
            if !guard_host_call::lower_branch_host_call_return(
                node,
                registers,
                body,
                next_reg,
                next_block,
                function_return_kind,
            ) {
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            return Ok(GuardReturnLoweringOutcome::TerminalReturn);
        }
        "branch_print_return" => {
            let cond_value = registers.get(&node.op.args[0]).cloned();
            let then_print_value = registers.get(&node.op.args[1]).cloned();
            let then_return_value = registers.get(&node.op.args[2]).cloned();
            let else_print_value = registers.get(&node.op.args[3]).cloned();
            let else_return_value = registers.get(&node.op.args[4]).cloned();
            let (
                Some(cond_value),
                Some(then_print_value),
                Some(then_return_value),
                Some(else_print_value),
                Some(else_return_value),
            ) = (
                cond_value,
                then_print_value,
                then_return_value,
                else_print_value,
                else_return_value,
            )
            else {
                body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            };
            if !can_emit_typed_return_from_value(function_return_kind, &then_return_value) {
                body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its then-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            if !can_emit_typed_return_from_value(function_return_kind, &else_return_value) {
                body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its else-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            let cond_bool = fresh_reg(next_reg);
            body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
            let then_label = fresh_block(next_block, "branch_print_return_then");
            let else_label = fresh_block(next_block, "branch_print_return_else");
            body.push(format!(
                "  br i1 {cond_bool}, label %{then_label}, label %{else_label}"
            ));
            body.push(format!("{then_label}:"));
            if let Some(input) = coerce_to_cstr(&then_print_value, body, next_reg) {
                let print_reg = fresh_reg(next_reg);
                body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
            } else if let Some(input) = coerce_to_i64(&then_print_value, body, next_reg) {
                body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
            } else {
                body.push(format!(
                        "  ; deferred lowering inside cpu.branch_print_return `{}` because its then-print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
            }
            if !emit_typed_return_from_value(
                body,
                next_reg,
                function_return_kind,
                &then_return_value,
            ) {
                body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its then-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            body.push(format!("{else_label}:"));
            if let Some(input) = coerce_to_cstr(&else_print_value, body, next_reg) {
                let print_reg = fresh_reg(next_reg);
                body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
            } else if let Some(input) = coerce_to_i64(&else_print_value, body, next_reg) {
                body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
            } else {
                body.push(format!(
                        "  ; deferred lowering inside cpu.branch_print_return `{}` because its else-print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
            }
            if !emit_typed_return_from_value(
                body,
                next_reg,
                function_return_kind,
                &else_return_value,
            ) {
                body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its else-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                return Ok(GuardReturnLoweringOutcome::Continue);
            }
            return Ok(GuardReturnLoweringOutcome::TerminalReturn);
        }
        _ => return Ok(GuardReturnLoweringOutcome::NotGuard),
    }

    Ok(GuardReturnLoweringOutcome::Continue)
}

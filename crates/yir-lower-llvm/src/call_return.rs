use super::{fresh_reg, value_ref::coerce_to_i64, CpuCallScalarKind, LlvmValueRef};

pub(crate) fn cpu_call_scalar_kind_for_instruction(instruction: &str) -> Option<CpuCallScalarKind> {
    match instruction {
        "param_bool" | "call_bool" | "return_bool" => Some(CpuCallScalarKind::Bool),
        "param_i32" | "call_i32" | "return_i32" => Some(CpuCallScalarKind::I32),
        "param_i64" | "call_i64" | "return_i64" => Some(CpuCallScalarKind::I64),
        "param_f32" | "call_f32" | "return_f32" => Some(CpuCallScalarKind::F32),
        "param_f64" | "call_f64" | "return_f64" => Some(CpuCallScalarKind::F64),
        "param_buffer_ref" => Some(CpuCallScalarKind::BorrowedBuffer),
        _ => None,
    }
}

pub(crate) fn cpu_scalar_kind_llvm_type(kind: CpuCallScalarKind) -> &'static str {
    match kind {
        CpuCallScalarKind::Bool => "i1",
        CpuCallScalarKind::I32 => "i32",
        CpuCallScalarKind::I64 => "i64",
        CpuCallScalarKind::F32 => "float",
        CpuCallScalarKind::F64 => "double",
        CpuCallScalarKind::BorrowedBuffer => "ptr",
    }
}

pub(crate) fn cpu_param_binding(kind: CpuCallScalarKind, index: usize) -> LlvmValueRef {
    let arg = format!("%arg{index}");
    match kind {
        CpuCallScalarKind::Bool => {
            let widened = format!("%arg{index}_i64");
            LlvmValueRef::Bool {
                i1: arg,
                i64: widened,
            }
        }
        CpuCallScalarKind::I32 => LlvmValueRef::I32(arg),
        CpuCallScalarKind::I64 => LlvmValueRef::I64(arg),
        CpuCallScalarKind::F32 => LlvmValueRef::F32(arg),
        CpuCallScalarKind::F64 => LlvmValueRef::F64(arg),
        CpuCallScalarKind::BorrowedBuffer => LlvmValueRef::Ptr(arg),
    }
}

pub(crate) fn emit_typed_return_from_value(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    function_return_kind: CpuCallScalarKind,
    return_value: &LlvmValueRef,
) -> bool {
    match function_return_kind {
        CpuCallScalarKind::Bool => match return_value {
            LlvmValueRef::Bool { i1, .. } => {
                body.push(format!("  ret i1 {i1}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_bool = fresh_reg(next_reg);
                body.push(format!("  {as_bool} = icmp ne i64 {value}, 0"));
                body.push(format!("  ret i1 {as_bool}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::I32 => match return_value {
            LlvmValueRef::I32(value) => {
                body.push(format!("  ret i32 {value}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_i32 = fresh_reg(next_reg);
                body.push(format!("  {as_i32} = trunc i64 {value} to i32"));
                body.push(format!("  ret i32 {as_i32}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::I64 => {
            if let Some(returned) = coerce_to_i64(return_value, body, next_reg) {
                body.push(format!("  ret i64 {returned}"));
                true
            } else {
                false
            }
        }
        CpuCallScalarKind::F32 => match return_value {
            LlvmValueRef::F32(value) => {
                body.push(format!("  ret float {value}"));
                true
            }
            LlvmValueRef::F64(value) => {
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_f32} = fptrunc double {value} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_f32} = sitofp i64 {value} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            LlvmValueRef::I32(value) => {
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_f32} = sitofp i32 {value} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {as_f32} = sitofp i64 {as_i64} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::F64 => match return_value {
            LlvmValueRef::F64(value) => {
                body.push(format!("  ret double {value}"));
                true
            }
            LlvmValueRef::F32(value) => {
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_f64} = fpext float {value} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_f64} = sitofp i64 {value} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            LlvmValueRef::I32(value) => {
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_f64} = sitofp i32 {value} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {as_f64} = sitofp i64 {as_i64} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::BorrowedBuffer => false,
    }
}

pub(crate) fn can_emit_typed_return_from_value(
    function_return_kind: CpuCallScalarKind,
    return_value: &LlvmValueRef,
) -> bool {
    match function_return_kind {
        CpuCallScalarKind::Bool => {
            matches!(
                return_value,
                LlvmValueRef::Bool { .. } | LlvmValueRef::I64(_)
            )
        }
        CpuCallScalarKind::I32 => {
            matches!(return_value, LlvmValueRef::I32(_) | LlvmValueRef::I64(_))
        }
        CpuCallScalarKind::I64 => matches!(
            return_value,
            LlvmValueRef::I64(_)
                | LlvmValueRef::TextHandle { .. }
                | LlvmValueRef::I32(_)
                | LlvmValueRef::Bool { .. }
                | LlvmValueRef::F32(_)
                | LlvmValueRef::F64(_)
        ),
        CpuCallScalarKind::F32 => matches!(
            return_value,
            LlvmValueRef::F32(_)
                | LlvmValueRef::F64(_)
                | LlvmValueRef::I64(_)
                | LlvmValueRef::I32(_)
                | LlvmValueRef::Bool { .. }
        ),
        CpuCallScalarKind::F64 => matches!(
            return_value,
            LlvmValueRef::F64(_)
                | LlvmValueRef::F32(_)
                | LlvmValueRef::I64(_)
                | LlvmValueRef::I32(_)
                | LlvmValueRef::Bool { .. }
        ),
        CpuCallScalarKind::BorrowedBuffer => false,
    }
}

pub(crate) fn emit_typed_return_from_last_value(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    function_return_kind: CpuCallScalarKind,
    last_value: &str,
) {
    match function_return_kind {
        CpuCallScalarKind::Bool => {
            let as_bool = fresh_reg(next_reg);
            body.push(format!("  {as_bool} = icmp ne i64 {last_value}, 0"));
            body.push(format!("  ret i1 {as_bool}"));
        }
        CpuCallScalarKind::I32 => {
            let as_i32 = fresh_reg(next_reg);
            body.push(format!("  {as_i32} = trunc i64 {last_value} to i32"));
            body.push(format!("  ret i32 {as_i32}"));
        }
        CpuCallScalarKind::I64 => {
            body.push(format!("  ret i64 {last_value}"));
        }
        CpuCallScalarKind::F32 => {
            let as_f32 = fresh_reg(next_reg);
            body.push(format!("  {as_f32} = sitofp i64 {last_value} to float"));
            body.push(format!("  ret float {as_f32}"));
        }
        CpuCallScalarKind::F64 => {
            let as_f64 = fresh_reg(next_reg);
            body.push(format!("  {as_f64} = sitofp i64 {last_value} to double"));
            body.push(format!("  ret double {as_f64}"));
        }
        CpuCallScalarKind::BorrowedBuffer => {
            unreachable!("borrowed buffers cannot be function return values")
        }
    }
}

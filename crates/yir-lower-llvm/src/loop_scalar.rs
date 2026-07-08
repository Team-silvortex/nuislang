use super::{fresh_reg, value_ref::coerce_to_i64, CpuLoopScalarKind, LlvmValueRef};

pub(crate) fn infer_loop_scalar_kind<'a, I>(values: I) -> Option<CpuLoopScalarKind>
where
    I: IntoIterator<Item = &'a LlvmValueRef>,
{
    let mut saw_f32 = false;
    for value in values {
        match value {
            LlvmValueRef::F64(_) => return Some(CpuLoopScalarKind::F64),
            LlvmValueRef::F32(_) => saw_f32 = true,
            LlvmValueRef::I64(_)
            | LlvmValueRef::I32(_)
            | LlvmValueRef::Bool { .. }
            | LlvmValueRef::TextHandle { .. } => {}
            _ => return None,
        }
    }
    if saw_f32 {
        Some(CpuLoopScalarKind::F32)
    } else {
        Some(CpuLoopScalarKind::I64)
    }
}

pub(crate) fn loop_scalar_llvm_type(kind: CpuLoopScalarKind) -> &'static str {
    match kind {
        CpuLoopScalarKind::I64 => "i64",
        CpuLoopScalarKind::F32 => "float",
        CpuLoopScalarKind::F64 => "double",
    }
}

pub(crate) fn loop_scalar_value_ref(kind: CpuLoopScalarKind, value: String) -> LlvmValueRef {
    match kind {
        CpuLoopScalarKind::I64 => LlvmValueRef::I64(value),
        CpuLoopScalarKind::F32 => LlvmValueRef::F32(value),
        CpuLoopScalarKind::F64 => LlvmValueRef::F64(value),
    }
}

pub(crate) fn coerce_to_loop_scalar(
    value: &LlvmValueRef,
    kind: CpuLoopScalarKind,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match kind {
        CpuLoopScalarKind::I64 => coerce_to_i64(value, body, next_reg),
        CpuLoopScalarKind::F32 => match value {
            LlvmValueRef::F32(value) => Some(value.clone()),
            LlvmValueRef::F64(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fptrunc double {value} to float"));
                Some(reg)
            }
            LlvmValueRef::I64(value) | LlvmValueRef::TextHandle { handle: value, .. } => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i64 {value} to float"));
                Some(reg)
            }
            LlvmValueRef::I32(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i32 {value} to float"));
                Some(reg)
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let reg = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {reg} = sitofp i64 {as_i64} to float"));
                Some(reg)
            }
            _ => None,
        },
        CpuLoopScalarKind::F64 => match value {
            LlvmValueRef::F64(value) => Some(value.clone()),
            LlvmValueRef::F32(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fpext float {value} to double"));
                Some(reg)
            }
            LlvmValueRef::I64(value) | LlvmValueRef::TextHandle { handle: value, .. } => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i64 {value} to double"));
                Some(reg)
            }
            LlvmValueRef::I32(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i32 {value} to double"));
                Some(reg)
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let reg = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {reg} = sitofp i64 {as_i64} to double"));
                Some(reg)
            }
            _ => None,
        },
    }
}

pub(crate) fn emit_loop_compare(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    kind: CpuLoopScalarKind,
    compare: &str,
    lhs: &str,
    rhs: &str,
) -> Result<String, String> {
    let reg = fresh_reg(next_reg);
    match kind {
        CpuLoopScalarKind::I64 => {
            let pred = match compare {
                "eq" => "eq",
                "ne" => "ne",
                "lt" => "slt",
                "le" => "sle",
                "gt" => "sgt",
                "ge" => "sge",
                other => return Err(format!("unsupported integer loop compare kind `{other}`")),
            };
            body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F32 => {
            let pred = match compare {
                "eq" => "oeq",
                "ne" => "one",
                "lt" => "olt",
                "le" => "ole",
                "gt" => "ogt",
                "ge" => "oge",
                other => return Err(format!("unsupported float loop compare kind `{other}`")),
            };
            body.push(format!("  {reg} = fcmp {pred} float {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F64 => {
            let pred = match compare {
                "eq" => "oeq",
                "ne" => "one",
                "lt" => "olt",
                "le" => "ole",
                "gt" => "ogt",
                "ge" => "oge",
                other => return Err(format!("unsupported float loop compare kind `{other}`")),
            };
            body.push(format!("  {reg} = fcmp {pred} double {lhs}, {rhs}"));
        }
    }
    Ok(reg)
}

pub(crate) fn emit_loop_numeric_op(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    kind: CpuLoopScalarKind,
    op: &str,
    lhs: &str,
    rhs: &str,
) -> Result<String, String> {
    let reg = fresh_reg(next_reg);
    match kind {
        CpuLoopScalarKind::I64 => {
            let instr = match op {
                "add" => "add",
                "sub" => "sub",
                "mul" => "mul",
                "div" => "sdiv",
                other => return Err(format!("unsupported integer loop op `{other}`")),
            };
            body.push(format!("  {reg} = {instr} i64 {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F32 => {
            let instr = match op {
                "add" => "fadd",
                "sub" => "fsub",
                "mul" => "fmul",
                "div" => "fdiv",
                other => return Err(format!("unsupported float loop op `{other}`")),
            };
            body.push(format!("  {reg} = {instr} float {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F64 => {
            let instr = match op {
                "add" => "fadd",
                "sub" => "fsub",
                "mul" => "fmul",
                "div" => "fdiv",
                other => return Err(format!("unsupported float loop op `{other}`")),
            };
            body.push(format!("  {reg} = {instr} double {lhs}, {rhs}"));
        }
    }
    Ok(reg)
}

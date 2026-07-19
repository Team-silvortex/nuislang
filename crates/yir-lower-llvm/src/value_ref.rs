use std::collections::BTreeMap;

use super::{
    fresh_reg, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef, NetworkResultLlvmValueRef,
    StructLlvmValueRef, TaskLlvmValueRef, TaskResultLlvmValueRef, ThreadLlvmValueRef,
};

pub(crate) fn get_i64<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I64(value)) => Some(value.as_str()),
        Some(LlvmValueRef::Bool { i64, .. }) => Some(i64.as_str()),
        Some(LlvmValueRef::TextHandle { handle, .. }) => Some(handle.as_str()),
        _ => None,
    }
}

pub(crate) fn get_i32<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I32(value)) => Some(value.as_str()),
        _ => None,
    }
}

pub(crate) fn get_bool<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Bool { i1, .. }) => Some(i1.as_str()),
        _ => None,
    }
}

pub(crate) fn get_f32<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::F32(value)) => Some(value.as_str()),
        _ => None,
    }
}

pub(crate) fn get_f64<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::F64(value)) => Some(value.as_str()),
        _ => None,
    }
}

pub(crate) fn get_struct<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a StructLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Struct(value)) => Some(value),
        _ => None,
    }
}

pub(crate) fn get_network_result<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a NetworkResultLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::NetworkResult(result)) => Some(result),
        _ => None,
    }
}

pub(crate) fn get_task<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a TaskLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Task(task)) => Some(task),
        _ => None,
    }
}

pub(crate) fn get_thread<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a ThreadLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Thread(thread)) => Some(thread),
        _ => None,
    }
}

pub(crate) fn get_task_result<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a TaskResultLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::TaskResult(result)) => Some(result),
        _ => None,
    }
}

pub(crate) fn get_mutex<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a MutexLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Mutex(mutex)) => Some(mutex),
        _ => None,
    }
}

pub(crate) fn get_mutex_guard<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a MutexGuardLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::MutexGuard(guard)) => Some(guard),
        _ => None,
    }
}

pub(crate) fn coerce_to_i64(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::I64(value) => Some(value.clone()),
        LlvmValueRef::TextHandle { handle, .. } => Some(handle.clone()),
        LlvmValueRef::Ptr(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = ptrtoint ptr {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::BorrowedBuffer { ptr, .. } => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = ptrtoint ptr {ptr} to i64"));
            Some(reg)
        }
        LlvmValueRef::I32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sext i32 {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::Bool { i64, .. } => Some(i64.clone()),
        LlvmValueRef::F32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi float {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::F64(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi double {value} to i64"));
            Some(reg)
        }
        _ => None,
    }
}

pub(crate) fn coerce_to_i32(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::I32(value) => Some(value.clone()),
        LlvmValueRef::I64(value) | LlvmValueRef::TextHandle { handle: value, .. } => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = trunc i64 {value} to i32"));
            Some(reg)
        }
        LlvmValueRef::Bool { i1, .. } => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = zext i1 {i1} to i32"));
            Some(reg)
        }
        LlvmValueRef::F32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi float {value} to i32"));
            Some(reg)
        }
        LlvmValueRef::F64(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi double {value} to i32"));
            Some(reg)
        }
        _ => None,
    }
}

pub(crate) fn coerce_to_cstr<'a>(
    value: &'a LlvmValueRef,
    _body: &mut Vec<String>,
    _next_reg: &mut usize,
) -> Option<&'a str> {
    match value {
        LlvmValueRef::TextHandle { ptr, .. } => Some(ptr.as_str()),
        _ => None,
    }
}

pub(crate) fn get_ptr<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Ptr(value)) => Some(value.as_str()),
        Some(LlvmValueRef::BorrowedBuffer { ptr, .. }) => Some(ptr.as_str()),
        _ => None,
    }
}

pub(crate) fn borrowed_buffer_parts(
    registers: &BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
    name: &str,
) -> Option<(String, String)> {
    match registers.get(name)? {
        LlvmValueRef::BorrowedBuffer { ptr, len } => Some((ptr.clone(), len.clone())),
        LlvmValueRef::Ptr(ptr) => Some((ptr.clone(), buffer_lengths.get(name)?.clone())),
        _ => None,
    }
}

pub(crate) fn get_cstr<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::TextHandle { ptr, .. }) => Some(ptr.as_str()),
        _ => None,
    }
}

use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) enum LlvmValueRef {
    Bool { i1: String, i64: String },
    I32(String),
    I64(String),
    F32(String),
    F64(String),
    Task(TaskLlvmValueRef),
    Thread(ThreadLlvmValueRef),
    TaskResult(TaskResultLlvmValueRef),
    Mutex(MutexLlvmValueRef),
    MutexGuard(MutexGuardLlvmValueRef),
    NetworkResult(NetworkResultLlvmValueRef),
    Struct(StructLlvmValueRef),
    VariantUnion(VariantUnionLlvmValueRef),
    Ptr(String),
    TextHandle { ptr: String, handle: String },
    Void,
}
#[derive(Clone)]
pub(crate) struct StructLlvmValueRef {
    pub(crate) type_name: String,
    pub(crate) fields: Vec<(String, LlvmValueRef)>,
}
#[derive(Clone)]
pub(crate) struct VariantUnionLlvmValueRef {
    pub(crate) parent_type_name: String,
    pub(crate) tag_i64: String,
    pub(crate) variants: BTreeMap<String, StructLlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct NetworkResultLlvmValueRef {
    pub(crate) state: String,
    pub(crate) value: Box<LlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct TaskLlvmValueRef {
    pub(crate) value: Box<LlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct ThreadLlvmValueRef {
    pub(crate) value: Box<LlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct TaskResultLlvmValueRef {
    pub(crate) state: String,
    pub(crate) value: Option<Box<LlvmValueRef>>,
}
#[derive(Clone)]
pub(crate) struct MutexLlvmValueRef {
    pub(crate) value: Box<LlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct MutexGuardLlvmValueRef {
    pub(crate) value: Box<LlvmValueRef>,
}
pub(crate) struct LlvmLoweringState {
    pub(crate) body: Vec<String>,
    pub(crate) globals: Vec<String>,
    pub(crate) registers: BTreeMap<String, LlvmValueRef>,
    pub(crate) delayed_registers: BTreeMap<String, String>,
    pub(crate) known_bool_values: BTreeMap<String, bool>,
    pub(crate) known_i64_values: BTreeMap<String, i64>,
    pub(crate) buffer_lengths: BTreeMap<String, String>,
    pub(crate) next_reg: usize,
    pub(crate) next_global: usize,
    pub(crate) next_block: usize,
    pub(crate) last_cpu_value: Option<String>,
    pub(crate) ends_with_terminal_return: bool,
}
pub(crate) struct EmittedCpuFunction {
    pub(crate) globals: Vec<String>,
    pub(crate) body: String,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CpuCallScalarKind {
    Bool,
    I32,
    I64,
    F32,
    F64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CpuLoopScalarKind {
    I64,
    F32,
    F64,
}
pub(crate) struct CpuHelperSignature {
    pub(crate) params: Vec<CpuCallScalarKind>,
    pub(crate) ret: CpuCallScalarKind,
}

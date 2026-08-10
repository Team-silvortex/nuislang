use std::collections::BTreeMap;

use super::KnownFacts;

#[derive(Clone)]
pub(crate) enum LlvmValueRef {
    Bool {
        i1: String,
        i64: String,
    },
    I32(String),
    I64(String),
    F32(String),
    F64(String),
    DeferredTaskThunkScalar {
        callee: String,
        arguments: Vec<TaskThunkArgument>,
        return_kind: CpuCallScalarKind,
    },
    DeferredTaskThunkOwnedStruct {
        callee: String,
        arguments: Vec<TaskThunkArgument>,
        template: StructLlvmValueRef,
    },
    Task(TaskLlvmValueRef),
    Thread(ThreadLlvmValueRef),
    TaskResult(TaskResultLlvmValueRef),
    Mutex(MutexLlvmValueRef),
    MutexPermit(MutexPermitLlvmValueRef),
    MutexGuard(MutexGuardLlvmValueRef),
    NetworkResult(NetworkResultLlvmValueRef),
    Struct(StructLlvmValueRef),
    VariantUnion(VariantUnionLlvmValueRef),
    Ptr(String),
    BorrowedBuffer {
        ptr: String,
        len: String,
    },
    OwnedExternalBuffer {
        ptr: String,
        len: String,
        abi: String,
        destructor: String,
        destructor_signature_hash: String,
    },
    TextHandle {
        ptr: String,
        handle: String,
    },
    OwnedBytes {
        blob: String,
    },
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
    pub(crate) runtime_handle: Option<String>,
    pub(crate) value: Box<LlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct ThreadLlvmValueRef {
    pub(crate) value: Box<LlvmValueRef>,
}
#[derive(Clone)]
pub(crate) struct TaskResultLlvmValueRef {
    pub(crate) state: String,
    pub(crate) runtime_state: Option<String>,
    pub(crate) runtime_handle: Option<String>,
    pub(crate) value: Option<Box<LlvmValueRef>>,
}
#[derive(Clone)]
pub(crate) struct MutexLlvmValueRef {
    pub(crate) runtime_handle: Option<String>,
    pub(crate) value: Box<LlvmValueRef>,
    pub(crate) scalar_kind: Option<MutexScalarKind>,
}
#[derive(Clone)]
pub(crate) struct MutexPermitLlvmValueRef {
    pub(crate) runtime_token: String,
    pub(crate) scalar_kind: MutexScalarKind,
}
#[derive(Clone)]
pub(crate) struct MutexGuardLlvmValueRef {
    pub(crate) runtime_guard: Option<String>,
    pub(crate) value: Box<LlvmValueRef>,
    pub(crate) scalar_kind: Option<MutexScalarKind>,
}
pub(crate) struct LlvmLoweringState {
    pub(crate) body: Vec<String>,
    pub(crate) globals: Vec<String>,
    pub(crate) registers: BTreeMap<String, LlvmValueRef>,
    pub(crate) delayed_registers: BTreeMap<String, String>,
    pub(crate) facts: KnownFacts,
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
    BorrowedBuffer,
    TraversalPointer,
    OwnedBytes,
    OwnedExternalBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MutexScalarKind {
    I32,
    I64,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CpuOwnedExternalBufferAbi {
    pub(crate) abi: String,
    pub(crate) destructor: String,
    pub(crate) destructor_signature_hash: String,
}
#[derive(Clone)]
pub(crate) struct TaskThunkArgument {
    pub(crate) kind: CpuCallScalarKind,
    pub(crate) value: String,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CpuLoopScalarKind {
    I64,
    F32,
    F64,
}
pub(crate) struct CpuHelperSignature {
    pub(crate) params: Vec<CpuCallScalarKind>,
    pub(crate) mutex_permit_params: Vec<Option<MutexScalarKind>>,
    pub(crate) ret: CpuCallScalarKind,
    pub(crate) owned_external_buffer_return: Option<CpuOwnedExternalBufferAbi>,
}

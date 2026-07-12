use std::fmt;

use crate::{Operation, SemanticOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmValueClass {
    Val,
    Res,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmSketchValueClass {
    Bridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmBridgeObjectKind {
    TaskExternalHandle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmUseMode {
    Own,
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmEffect {
    None,
    DomainMove,
    LifetimeEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlmAccess {
    pub input: String,
    pub class: GlmValueClass,
    pub mode: GlmUseMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlmNodeProfile {
    pub result_class: GlmValueClass,
    pub accesses: Vec<GlmAccess>,
    pub effect: GlmEffect,
}

impl fmt::Display for GlmValueClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Val => f.write_str("val"),
            Self::Res => f.write_str("res"),
        }
    }
}

impl fmt::Display for GlmSketchValueClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bridge => f.write_str("bridge"),
        }
    }
}

impl fmt::Display for GlmBridgeObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskExternalHandle => f.write_str("task-external-handle"),
        }
    }
}

impl fmt::Display for GlmUseMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Own => f.write_str("Own"),
            Self::Read => f.write_str("Read"),
            Self::Write => f.write_str("Write"),
        }
    }
}

impl fmt::Display for GlmEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("none"),
            Self::DomainMove => f.write_str("domain-move"),
            Self::LifetimeEnd => f.write_str("lifetime-end"),
        }
    }
}

pub fn glm_profile_for_operation(op: &Operation) -> GlmNodeProfile {
    match op.semantic_op() {
        SemanticOp::CpuAllocNode | SemanticOp::CpuAllocBuffer => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: op
                .args
                .iter()
                .map(|input| GlmAccess {
                    input: input.clone(),
                    class: GlmValueClass::Val,
                    mode: GlmUseMode::Read,
                })
                .collect(),
            effect: GlmEffect::None,
        },
        SemanticOp::CpuBorrow => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::CpuBorrowEnd => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::CpuProjectProfileRef | SemanticOp::CpuInstantiateUnit => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: Vec::new(),
            effect: GlmEffect::None,
        },
        SemanticOp::CpuMovePtr => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::DomainMove,
        },
        SemanticOp::CpuLoadValue
        | SemanticOp::CpuLoadNext
        | SemanticOp::CpuBufferLen
        | SemanticOp::CpuLoadAt => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::CpuStoreValue | SemanticOp::CpuStoreNext | SemanticOp::CpuStoreAt => {
            GlmNodeProfile {
                result_class: GlmValueClass::Val,
                accesses: vec![GlmAccess {
                    input: op.args[0].clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Write,
                }],
                effect: GlmEffect::None,
            }
        }
        SemanticOp::CpuFree => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::LifetimeEnd,
        },
        SemanticOp::DataMove => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Val,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::DomainMove,
        },
        SemanticOp::DataCopyWindow
        | SemanticOp::DataReadWindow
        | SemanticOp::DataWriteWindow
        | SemanticOp::DataFreezeWindow
        | SemanticOp::DataImmutableWindow => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::DataOutputPipe | SemanticOp::DataInputPipe => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        _ if op.is_async_core_op() => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: op
                .args
                .iter()
                .map(|input| GlmAccess {
                    input: input.clone(),
                    class: GlmValueClass::Val,
                    mode: GlmUseMode::Read,
                })
                .collect(),
            effect: GlmEffect::None,
        },
        _ => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: op
                .args
                .iter()
                .map(|input| GlmAccess {
                    input: input.clone(),
                    class: GlmValueClass::Val,
                    mode: GlmUseMode::Read,
                })
                .collect(),
            effect: GlmEffect::None,
        },
    }
}

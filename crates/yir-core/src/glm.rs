use std::fmt;

use crate::{
    owned_select_tree_conditions, owned_select_tree_scalar_args, owned_select_tree_transfers,
    parse_branch_effect_args, parse_branch_owned_call_args, parse_owned_select_tree_args,
    Operation, SemanticOp,
};

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
        _ if op.module == "cpu" && op.instruction == "loop_while_i64_effect" => {
            cpu_effect_loop_profile(op)
        }
        _ if op.module == "cpu" && op.instruction == "select_owned_bytes" => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![
                value_read(&op.args[0]),
                GlmAccess {
                    input: op.args[1].clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Own,
                },
                GlmAccess {
                    input: op.args[2].clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Own,
                },
            ],
            effect: GlmEffect::None,
        },
        _ if op.module == "cpu" && op.instruction == "select_owned_bytes_drop_unselected" => {
            GlmNodeProfile {
                result_class: GlmValueClass::Res,
                accesses: vec![
                    value_read(&op.args[0]),
                    GlmAccess {
                        input: op.args[1].clone(),
                        class: GlmValueClass::Res,
                        mode: GlmUseMode::Own,
                    },
                    GlmAccess {
                        input: op.args[2].clone(),
                        class: GlmValueClass::Res,
                        mode: GlmUseMode::Own,
                    },
                ],
                effect: GlmEffect::DomainMove,
            }
        }
        _ if op.module == "cpu" && op.instruction == "branch_call_owned_bytes" => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: {
                let mut accesses = Vec::new();
                if let Some(args) = parse_branch_owned_call_args(&op.args) {
                    accesses.push(value_read_str(args.condition));
                    accesses.push(GlmAccess {
                        input: args.owner.to_owned(),
                        class: GlmValueClass::Res,
                        mode: GlmUseMode::Own,
                    });
                    accesses.extend(
                        args.then_scalar_args
                            .iter()
                            .chain(args.else_scalar_args)
                            .map(value_read),
                    );
                }
                accesses
            },
            effect: GlmEffect::DomainMove,
        },
        _ if op.module == "cpu" && op.instruction == "select_owned_bytes_tree" => {
            let mut accesses = Vec::new();
            if let Some(args) = parse_owned_select_tree_args(&op.args) {
                let mut conditions = Vec::new();
                owned_select_tree_conditions(&args.tree, &mut conditions);
                accesses.extend(conditions.into_iter().map(value_read_str));
                let mut transfers = Vec::new();
                owned_select_tree_transfers(&args.tree, &mut transfers);
                let mut scalar_args = Vec::new();
                owned_select_tree_scalar_args(&args.tree, &mut scalar_args);
                accesses.extend(
                    scalar_args
                        .into_iter()
                        .filter(|arg| !transfers.contains(arg))
                        .map(value_read_str),
                );
                accesses.extend(transfers.into_iter().map(|input| GlmAccess {
                    input: input.to_owned(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Own,
                }));
                accesses.extend(args.owners.iter().map(|owner| GlmAccess {
                    input: owner.clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Own,
                }));
            }
            GlmNodeProfile {
                result_class: GlmValueClass::Res,
                accesses,
                effect: GlmEffect::DomainMove,
            }
        }
        _ if op.module == "cpu" && op.instruction == "branch_effect" => {
            let mut accesses = Vec::new();
            if let Some(args) = parse_branch_effect_args(&op.args) {
                accesses.push(value_read_str(args.condition));
                for action in args.then_actions.iter().chain(&args.else_actions) {
                    for operand in &action.operands {
                        let access = match operand.access {
                            crate::BranchEffectAccess::ValueRead => value_read_str(operand.value),
                            crate::BranchEffectAccess::ResourceRead => GlmAccess {
                                input: operand.value.to_owned(),
                                class: GlmValueClass::Res,
                                mode: GlmUseMode::Read,
                            },
                            crate::BranchEffectAccess::ResourceOwn => GlmAccess {
                                input: operand.value.to_owned(),
                                class: GlmValueClass::Res,
                                mode: GlmUseMode::Own,
                            },
                        };
                        if !accesses.iter().any(|known| {
                            known.input == access.input
                                && known.class == access.class
                                && known.mode == access.mode
                        }) {
                            accesses.push(access);
                        }
                    }
                }
            }
            GlmNodeProfile {
                result_class: GlmValueClass::Val,
                accesses,
                effect: GlmEffect::DomainMove,
            }
        }
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

fn cpu_effect_loop_profile(op: &Operation) -> GlmNodeProfile {
    let mut accesses = op.args.iter().take(3).map(value_read).collect::<Vec<_>>();
    let operand_start = match op.args.get(6).map(String::as_str) {
        Some("owned_bytes_copy_drop") => 8,
        Some("scoped_call") => 9,
        Some("scoped_call_owned_return") => 10,
        _ => op.args.len(),
    };
    let mut moves_owned = false;
    for operand in op.args.iter().skip(operand_start) {
        if operand == "$current" {
            continue;
        }
        let (input, class, mode) = if let Some(input) = operand.strip_prefix("copy_owned:") {
            (input, GlmValueClass::Res, GlmUseMode::Read)
        } else if let Some(input) = operand.strip_prefix("move_owned:") {
            moves_owned = true;
            (input, GlmValueClass::Res, GlmUseMode::Own)
        } else {
            (operand.as_str(), GlmValueClass::Val, GlmUseMode::Read)
        };
        accesses.push(GlmAccess {
            input: input.to_owned(),
            class,
            mode,
        });
    }
    GlmNodeProfile {
        result_class: GlmValueClass::Val,
        accesses,
        effect: if moves_owned {
            GlmEffect::DomainMove
        } else {
            GlmEffect::None
        },
    }
}

fn value_read(input: &String) -> GlmAccess {
    GlmAccess {
        input: input.clone(),
        class: GlmValueClass::Val,
        mode: GlmUseMode::Read,
    }
}

fn value_read_str(input: &str) -> GlmAccess {
    GlmAccess {
        input: input.to_owned(),
        class: GlmValueClass::Val,
        mode: GlmUseMode::Read,
    }
}

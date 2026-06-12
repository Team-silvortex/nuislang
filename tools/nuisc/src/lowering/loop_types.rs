use super::*;

pub(super) enum LoweredIfOutcome {
    Continued,
    Bind { name: String, value: String },
    Printed,
    Returned(String),
}

pub(super) enum PreparedTerminalBranch {
    Return(NirExpr),
    PrintReturn { print: NirExpr, returned: NirExpr },
}

#[derive(Clone)]
pub(super) struct InlineablePureHelper {
    pub(super) params: Vec<String>,
    pub(super) expr: NirExpr,
}

#[derive(Clone)]
pub(super) struct PureHelperBlock {
    pub(super) params: Vec<String>,
    pub(super) body: Vec<NirStmt>,
}

pub(super) enum PreparedLoopBody {
    ExitOnly,
    PrintExit {
        print: NirExpr,
    },
    Return {
        returned: NirExpr,
    },
    PrintReturn {
        print: NirExpr,
        returned: NirExpr,
    },
    Branch {
        condition: NirExpr,
        then_body: Box<PreparedLoopBody>,
        else_body: Box<PreparedLoopBody>,
    },
}

#[derive(Clone, Copy)]
pub(super) enum PreparedLoopCompare {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Copy)]
pub(super) enum PreparedLoopStepKind {
    Add,
    Sub,
}

pub(super) struct PreparedCountedWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) step: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_kind: PreparedLoopStepKind,
}

pub(super) struct PreparedChainedWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) step: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_kind: PreparedLoopStepKind,
    pub(super) carries: Vec<PreparedCarryUpdate>,
}

pub(super) struct PreparedAsyncChainedWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_callee: String,
    pub(super) carries: Vec<PreparedCarryUpdate>,
}

pub(super) struct PreparedAsyncFlowWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_callee: String,
    pub(super) control: PreparedLoopFlowControl,
    pub(super) carries: Vec<PreparedCarryUpdate>,
}

pub(super) struct PreparedAsyncPostFlowWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_callee: String,
    pub(super) carries: Vec<PreparedCarryUpdate>,
    pub(super) control: PreparedLoopFlowControl,
}

pub(super) struct PreparedFlowWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) step: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_kind: PreparedLoopStepKind,
    pub(super) control: PreparedLoopFlowControl,
    pub(super) carries: Vec<PreparedCarryUpdate>,
}

pub(super) struct PreparedPostFlowWhile {
    pub(super) binding_name: String,
    pub(super) limit: NirExpr,
    pub(super) step: NirExpr,
    pub(super) compare: PreparedLoopCompare,
    pub(super) step_kind: PreparedLoopStepKind,
    pub(super) carries: Vec<PreparedCarryUpdate>,
    pub(super) control: PreparedLoopFlowControl,
}

pub(super) struct PreparedCarryUpdate {
    pub(super) binding_name: String,
    pub(super) kind: PreparedCarryUpdateKind,
}

pub(super) const TAIL_RECURSIVE_PREV_CURRENT_BINDING: &str = "__tailrec_prev_current";
pub(super) const TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX: &str = "__tailrec_prev_carry_";

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PreparedCarrySource {
    Current,
    PreviousCurrent,
    PreviousCarry(usize),
    Carry(usize),
    FixedRead(PreparedFixedReadCarrySource),
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PreparedFixedReadCarrySource {
    Value(NirExpr),
    At { buffer: NirExpr, index: NirExpr },
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PreparedReadableCarrySourceCandidate {
    Fixed(PreparedFixedReadCarrySource),
    DynamicIndexAt { buffer: NirExpr, index: NirExpr },
    TraversalNext { base: NirExpr },
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedReadableCarrySourceFamily {
    Fixed,
    DynamicIndexAt,
    TraversalNext,
}

impl PreparedReadableCarrySourceCandidate {
    pub(super) fn family(&self) -> PreparedReadableCarrySourceFamily {
        match self {
            Self::Fixed(_) => PreparedReadableCarrySourceFamily::Fixed,
            Self::DynamicIndexAt { .. } => PreparedReadableCarrySourceFamily::DynamicIndexAt,
            Self::TraversalNext { .. } => PreparedReadableCarrySourceFamily::TraversalNext,
        }
    }

    #[cfg(test)]
    pub(super) fn family_name(&self) -> &'static str {
        match self.family() {
            PreparedReadableCarrySourceFamily::Fixed => "fixed_read",
            PreparedReadableCarrySourceFamily::DynamicIndexAt => "dynamic_index_at",
            PreparedReadableCarrySourceFamily::TraversalNext => "traversal_next",
        }
    }

    pub(super) fn fixed_read(&self) -> Option<&PreparedFixedReadCarrySource> {
        match self {
            Self::Fixed(source) => Some(source),
            _ => None,
        }
    }
}

impl PreparedCarrySource {
    pub(super) fn is_fixed_read(&self) -> bool {
        matches!(self, Self::FixedRead(_))
    }

    pub(super) fn fixed_read(&self) -> Option<&PreparedFixedReadCarrySource> {
        match self {
            Self::FixedRead(source) => Some(source),
            _ => None,
        }
    }

    pub(super) fn contract_kind(&self, op: PreparedCarryLinearOp) -> String {
        match (op, self) {
            (PreparedCarryLinearOp::Add, Self::Current) => "add_current".to_owned(),
            (PreparedCarryLinearOp::Add, Self::PreviousCurrent) => "add_prev_current".to_owned(),
            (PreparedCarryLinearOp::Add, Self::PreviousCarry(index)) => {
                format!("add_prev_carry{index}")
            }
            (PreparedCarryLinearOp::Add, Self::Carry(index)) => format!("add_carry{index}"),
            (
                PreparedCarryLinearOp::Add,
                Self::FixedRead(PreparedFixedReadCarrySource::Value(_)),
            ) => "add_read_value_fixed".to_owned(),
            (
                PreparedCarryLinearOp::Add,
                Self::FixedRead(PreparedFixedReadCarrySource::At { .. }),
            ) => "add_read_at_fixed".to_owned(),
            (PreparedCarryLinearOp::Mul, Self::Current) => "mul_current".to_owned(),
            (PreparedCarryLinearOp::Mul, Self::PreviousCurrent) => "mul_prev_current".to_owned(),
            (PreparedCarryLinearOp::Mul, Self::PreviousCarry(index)) => {
                format!("mul_prev_carry{index}")
            }
            (PreparedCarryLinearOp::Mul, Self::Carry(index)) => format!("mul_carry{index}"),
            (
                PreparedCarryLinearOp::Mul,
                Self::FixedRead(PreparedFixedReadCarrySource::Value(_)),
            ) => "mul_read_value_fixed".to_owned(),
            (
                PreparedCarryLinearOp::Mul,
                Self::FixedRead(PreparedFixedReadCarrySource::At { .. }),
            ) => "mul_read_at_fixed".to_owned(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedCarryLinearOp {
    Add,
    Mul,
}

pub(super) enum PreparedCarryUpdateKind {
    Linear {
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    },
    Conditional {
        condition: PreparedLoopFlowCondition,
        then_source: PreparedCarryBranchSource,
        else_source: PreparedCarryBranchSource,
    },
}

#[derive(Clone)]
pub(super) struct PreparedLoopCarryCondition {
    pub(super) lhs: PreparedCarryCondSource,
    pub(super) compare: PreparedLoopCompare,
    pub(super) rhs: NirExpr,
}

#[derive(Clone, Copy)]
pub(super) enum PreparedCarryCondSource {
    Current,
    PreviousCurrent,
    PreviousCarry(usize),
    Carry(usize),
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PreparedCarryBranchSource {
    Keep,
    Source {
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    },
}

impl PreparedCarryBranchSource {
    pub(super) fn is_keep(&self) -> bool {
        matches!(self, Self::Keep)
    }

    pub(super) fn source_parts(&self) -> Option<(PreparedCarryLinearOp, &PreparedCarrySource)> {
        match self {
            Self::Keep => None,
            Self::Source { op, source } => Some((*op, source)),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum PreparedLoopFlowAction {
    Break,
    Continue,
}

pub(super) struct PreparedLoopFlowControl {
    pub(super) condition: PreparedLoopFlowCondition,
    pub(super) action: PreparedLoopFlowAction,
}

#[derive(Clone)]
pub(super) enum PreparedLoopFlowCondition {
    Simple(PreparedLoopCarryCondition),
    Compound {
        op: PreparedLoopLogicOp,
        lhs: Box<PreparedLoopFlowCondition>,
        rhs: Box<PreparedLoopFlowCondition>,
    },
}

#[derive(Clone, Copy)]
pub(super) enum PreparedLoopLogicOp {
    And,
    Or,
}

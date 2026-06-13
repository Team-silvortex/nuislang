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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreparedLoopStateRef {
    Current,
    PreviousCurrent,
    PreviousCarry(usize),
    Carry(usize),
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum PreparedCarrySource {
    Current,
    PreviousCurrent,
    PreviousCarry(usize),
    Carry(usize),
    FixedRead(PreparedFixedReadCarrySource),
    DynamicReadAt {
        buffer: NirExpr,
        index_source: Box<PreparedCarrySource>,
    },
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

    pub(super) fn is_dynamic_read_at(&self) -> bool {
        matches!(self, Self::DynamicReadAt { .. })
    }

    pub(super) fn fixed_read(&self) -> Option<&PreparedFixedReadCarrySource> {
        match self {
            Self::FixedRead(source) => Some(source),
            _ => None,
        }
    }

    pub(super) fn dynamic_read_at(&self) -> Option<(&NirExpr, &PreparedCarrySource)> {
        match self {
            Self::DynamicReadAt {
                buffer,
                index_source,
            } => Some((buffer, index_source.as_ref())),
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
            (PreparedCarryLinearOp::Add, Self::DynamicReadAt { index_source, .. }) => {
                match index_source.as_ref() {
                    Self::Current => "add_read_at_dynamic_current".to_owned(),
                    Self::PreviousCurrent => "add_read_at_dynamic_prev_current".to_owned(),
                    Self::PreviousCarry(index) => format!("add_read_at_dynamic_prev_carry{index}"),
                    Self::Carry(index) => format!("add_read_at_dynamic_carry{index}"),
                    Self::FixedRead(_) | Self::DynamicReadAt { .. } => {
                        unreachable!("dynamic read index sources must be simple loop-state sources")
                    }
                }
            }
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
            (PreparedCarryLinearOp::Mul, Self::DynamicReadAt { index_source, .. }) => {
                match index_source.as_ref() {
                    Self::Current => "mul_read_at_dynamic_current".to_owned(),
                    Self::PreviousCurrent => "mul_read_at_dynamic_prev_current".to_owned(),
                    Self::PreviousCarry(index) => format!("mul_read_at_dynamic_prev_carry{index}"),
                    Self::Carry(index) => format!("mul_read_at_dynamic_carry{index}"),
                    Self::FixedRead(_) | Self::DynamicReadAt { .. } => {
                        unreachable!("dynamic read index sources must be simple loop-state sources")
                    }
                }
            }
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
    KeepCurrentValue,
    KeepPreviousValue,
    Source {
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    },
}

pub(super) enum PreparedCarryBranchValueKind {
    KeepCurrentValue,
    KeepPreviousValue,
    #[allow(dead_code)]
    LinearSource {
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    },
}

pub(super) enum PreparedCarryBranchView<'a> {
    KeepCurrentValue,
    KeepPreviousValue,
    Source {
        op: PreparedCarryLinearOp,
        source: &'a PreparedCarrySource,
    },
}

impl PreparedCarryBranchSource {
    pub(super) fn keep() -> Self {
        Self::KeepCurrentValue
    }

    pub(super) fn keep_previous_value() -> Self {
        Self::KeepPreviousValue
    }

    pub(super) fn from_linear_source(op: PreparedCarryLinearOp, source: PreparedCarrySource) -> Self {
        Self::Source { op, source }
    }

    pub(super) fn value_kind(&self) -> PreparedCarryBranchValueKind {
        match self {
            Self::KeepCurrentValue => PreparedCarryBranchValueKind::KeepCurrentValue,
            Self::KeepPreviousValue => PreparedCarryBranchValueKind::KeepPreviousValue,
            Self::Source { op, source } => PreparedCarryBranchValueKind::LinearSource {
                op: *op,
                source: source.clone(),
            },
        }
    }

    pub(super) fn view(&self) -> PreparedCarryBranchView<'_> {
        match self.value_kind() {
            PreparedCarryBranchValueKind::KeepCurrentValue => {
                PreparedCarryBranchView::KeepCurrentValue
            }
            PreparedCarryBranchValueKind::KeepPreviousValue => {
                PreparedCarryBranchView::KeepPreviousValue
            }
            PreparedCarryBranchValueKind::LinearSource { op, source: _ } => {
                PreparedCarryBranchView::Source {
                    op,
                    source: match self {
                        Self::Source { source, .. } => source,
                        Self::KeepCurrentValue | Self::KeepPreviousValue => {
                            unreachable!("keep branch cannot expose source view")
                        }
                    },
                }
            }
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

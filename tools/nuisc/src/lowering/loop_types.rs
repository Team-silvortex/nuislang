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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedCarrySource {
    Current,
    PreviousCurrent,
    PreviousCarry(usize),
    Carry(usize),
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedCarryBranchSource {
    Keep,
    Source {
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    },
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

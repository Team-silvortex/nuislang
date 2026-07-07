use super::*;

#[path = "loop_types_accessors.rs"]
mod loop_types_accessors;
#[path = "loop_types_contract.rs"]
mod loop_types_contract;

pub(super) enum LoweredIfOutcome {
    Continued,
    Bind { name: String, value: String },
    Printed,
    Returned(String),
}

pub(super) type EncodedLoopArgs = (Vec<String>, Vec<String>, Vec<String>);
pub(super) type EncodedLoopControlArgs = (Vec<String>, Vec<String>, Vec<String>, bool);
pub(super) type LoopTempPrefixedSplit<'a> = (Vec<(String, NirExpr)>, &'a NirStmt, &'a [NirStmt]);
pub(super) type LoopTrailingTempSplit<'a> = (&'a [NirStmt], Vec<(String, NirExpr)>);
pub(super) type ConditionalTempExprBuilder = Box<dyn Fn(&NirExpr) -> NirExpr>;
pub(super) type PreparedFactorGroupProduct = (
    Vec<PreparedLoopStateRef>,
    Option<NirExpr>,
    Vec<PreparedLoopStateRef>,
    Option<NirExpr>,
);
pub(super) type PreparedScaledFactorGroupProduct = (
    Vec<PreparedLoopStateRef>,
    Option<NirExpr>,
    Vec<PreparedLoopStateRef>,
    Option<NirExpr>,
    NirExpr,
);
pub(in crate::lowering) type PreparedFactorStateListView<'a> = (
    &'a [PreparedLoopStateRef],
    &'a [PreparedLoopStateRef],
    Option<&'a NirExpr>,
    Option<&'a NirExpr>,
);
pub(in crate::lowering) type PreparedScaledFactorStateListView<'a> = (
    &'a [PreparedLoopStateRef],
    &'a [PreparedLoopStateRef],
    &'a NirExpr,
    Option<&'a NirExpr>,
    Option<&'a NirExpr>,
);
pub(in crate::lowering) type PreparedFactorGroupProductView<'a> = (
    &'a [PreparedLoopStateRef],
    &'a [PreparedLoopStateRef],
    Option<&'a NirExpr>,
    &'a [PreparedLoopStateRef],
    Option<&'a NirExpr>,
    Option<&'a NirExpr>,
);
pub(in crate::lowering) type PreparedScaledFactorGroupProductView<'a> = (
    &'a [PreparedLoopStateRef],
    &'a [PreparedLoopStateRef],
    Option<&'a NirExpr>,
    &'a [PreparedLoopStateRef],
    Option<&'a NirExpr>,
    &'a NirExpr,
    Option<&'a NirExpr>,
);

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
    AddStateList {
        terms: Vec<PreparedLoopStateRef>,
        offset: Option<NirExpr>,
    },
    ScaledStateList {
        terms: Vec<PreparedLoopStateRef>,
        factor: NirExpr,
        offset: Option<NirExpr>,
    },
    ScaledStateListByState {
        terms: Vec<PreparedLoopStateRef>,
        factor: PreparedLoopStateRef,
        offset: Option<NirExpr>,
    },
    ScaledStateListByStatePlusInvariant {
        terms: Vec<PreparedLoopStateRef>,
        factor: PreparedLoopStateRef,
        factor_offset: NirExpr,
        offset: Option<NirExpr>,
    },
    ScaledStateListByFactorStateList {
        terms: Vec<PreparedLoopStateRef>,
        factor_terms: Vec<PreparedLoopStateRef>,
        factor_offset: Option<NirExpr>,
        offset: Option<NirExpr>,
    },
    ScaledStateListByFactorStateListTimesInvariant {
        terms: Vec<PreparedLoopStateRef>,
        factor_terms: Vec<PreparedLoopStateRef>,
        factor_scale: NirExpr,
        factor_offset: Option<NirExpr>,
        offset: Option<NirExpr>,
    },
    ScaledStateListByFactorGroupProduct {
        terms: Vec<PreparedLoopStateRef>,
        lhs_factor_terms: Vec<PreparedLoopStateRef>,
        lhs_factor_offset: Option<NirExpr>,
        rhs_factor_terms: Vec<PreparedLoopStateRef>,
        rhs_factor_offset: Option<NirExpr>,
        offset: Option<NirExpr>,
    },
    ScaledStateListByFactorGroupProductTimesInvariant {
        terms: Vec<PreparedLoopStateRef>,
        lhs_factor_terms: Vec<PreparedLoopStateRef>,
        lhs_factor_offset: Option<NirExpr>,
        rhs_factor_terms: Vec<PreparedLoopStateRef>,
        rhs_factor_offset: Option<NirExpr>,
        factor_scale: NirExpr,
        offset: Option<NirExpr>,
    },
    InvariantExpr(NirExpr),
    AddInvariant {
        base: Box<PreparedCarrySource>,
        offset: NirExpr,
    },
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedCarryLinearOp {
    Add,
    Mul,
}

pub(super) enum PreparedCarryUpdateKind {
    Linear {
        op: PreparedCarryLinearOp,
        source: Box<PreparedCarrySource>,
    },
    Conditional {
        condition: PreparedLoopFlowCondition,
        then_source: Box<PreparedCarryBranchSource>,
        else_source: Box<PreparedCarryBranchSource>,
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
        source: Box<PreparedCarrySource>,
    },
}

pub(super) enum PreparedCarryBranchValueKind {
    KeepCurrentValue,
    KeepPreviousValue,
    #[allow(dead_code)]
    LinearSource {
        op: PreparedCarryLinearOp,
        source: Box<PreparedCarrySource>,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedLoopFlowAction {
    Break,
    Continue,
}

#[derive(Clone)]
pub(super) enum PreparedLoopFlowControl {
    Terminal {
        condition: PreparedLoopFlowCondition,
        action: PreparedLoopFlowAction,
    },
    Compound {
        op: PreparedLoopLogicOp,
        lhs: Box<PreparedLoopFlowControl>,
        rhs: Box<PreparedLoopFlowControl>,
    },
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

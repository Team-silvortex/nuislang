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
    fn state_ref_contract_fragment(state_ref: PreparedLoopStateRef) -> String {
        match state_ref {
            PreparedLoopStateRef::Current => "current".to_owned(),
            PreparedLoopStateRef::PreviousCurrent => "prev_current".to_owned(),
            PreparedLoopStateRef::PreviousCarry(index) => format!("prev_carry{index}"),
            PreparedLoopStateRef::Carry(index) => format!("carry{index}"),
        }
    }

    pub(super) fn is_fixed_read(&self) -> bool {
        matches!(self, Self::FixedRead(_))
    }

    pub(super) fn is_dynamic_read_at(&self) -> bool {
        matches!(self, Self::DynamicReadAt { .. })
    }

    pub(super) fn invariant_expr(&self) -> Option<&NirExpr> {
        match self {
            Self::InvariantExpr(expr) => Some(expr),
            _ => None,
        }
    }

    pub(super) fn add_invariant(&self) -> Option<(&PreparedCarrySource, &NirExpr)> {
        match self {
            Self::AddInvariant { base, offset } => Some((base.as_ref(), offset)),
            _ => None,
        }
    }

    pub(super) fn add_state_list(&self) -> Option<(&[PreparedLoopStateRef], Option<&NirExpr>)> {
        match self {
            Self::AddStateList { terms, offset } => Some((terms.as_slice(), offset.as_ref())),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list(
        &self,
    ) -> Option<(&[PreparedLoopStateRef], &NirExpr, Option<&NirExpr>)> {
        match self {
            Self::ScaledStateList {
                terms,
                factor,
                offset,
            } => Some((terms.as_slice(), factor, offset.as_ref())),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list_by_state(
        &self,
    ) -> Option<(
        &[PreparedLoopStateRef],
        PreparedLoopStateRef,
        Option<&NirExpr>,
    )> {
        match self {
            Self::ScaledStateListByState {
                terms,
                factor,
                offset,
            } => Some((terms.as_slice(), *factor, offset.as_ref())),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list_by_state_plus_invariant(
        &self,
    ) -> Option<(
        &[PreparedLoopStateRef],
        PreparedLoopStateRef,
        &NirExpr,
        Option<&NirExpr>,
    )> {
        match self {
            Self::ScaledStateListByStatePlusInvariant {
                terms,
                factor,
                factor_offset,
                offset,
            } => Some((terms.as_slice(), *factor, factor_offset, offset.as_ref())),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list_by_factor_state_list(
        &self,
    ) -> Option<(
        &[PreparedLoopStateRef],
        &[PreparedLoopStateRef],
        Option<&NirExpr>,
        Option<&NirExpr>,
    )> {
        match self {
            Self::ScaledStateListByFactorStateList {
                terms,
                factor_terms,
                factor_offset,
                offset,
            } => Some((
                terms.as_slice(),
                factor_terms.as_slice(),
                factor_offset.as_ref(),
                offset.as_ref(),
            )),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list_by_factor_state_list_times_invariant(
        &self,
    ) -> Option<(
        &[PreparedLoopStateRef],
        &[PreparedLoopStateRef],
        &NirExpr,
        Option<&NirExpr>,
        Option<&NirExpr>,
    )> {
        match self {
            Self::ScaledStateListByFactorStateListTimesInvariant {
                terms,
                factor_terms,
                factor_scale,
                factor_offset,
                offset,
            } => Some((
                terms.as_slice(),
                factor_terms.as_slice(),
                factor_scale,
                factor_offset.as_ref(),
                offset.as_ref(),
            )),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list_by_factor_group_product(
        &self,
    ) -> Option<(
        &[PreparedLoopStateRef],
        &[PreparedLoopStateRef],
        Option<&NirExpr>,
        &[PreparedLoopStateRef],
        Option<&NirExpr>,
        Option<&NirExpr>,
    )> {
        match self {
            Self::ScaledStateListByFactorGroupProduct {
                terms,
                lhs_factor_terms,
                lhs_factor_offset,
                rhs_factor_terms,
                rhs_factor_offset,
                offset,
            } => Some((
                terms.as_slice(),
                lhs_factor_terms.as_slice(),
                lhs_factor_offset.as_ref(),
                rhs_factor_terms.as_slice(),
                rhs_factor_offset.as_ref(),
                offset.as_ref(),
            )),
            _ => None,
        }
    }

    pub(super) fn scaled_state_list_by_factor_group_product_times_invariant(
        &self,
    ) -> Option<(
        &[PreparedLoopStateRef],
        &[PreparedLoopStateRef],
        Option<&NirExpr>,
        &[PreparedLoopStateRef],
        Option<&NirExpr>,
        &NirExpr,
        Option<&NirExpr>,
    )> {
        match self {
            Self::ScaledStateListByFactorGroupProductTimesInvariant {
                terms,
                lhs_factor_terms,
                lhs_factor_offset,
                rhs_factor_terms,
                rhs_factor_offset,
                factor_scale,
                offset,
            } => Some((
                terms.as_slice(),
                lhs_factor_terms.as_slice(),
                lhs_factor_offset.as_ref(),
                rhs_factor_terms.as_slice(),
                rhs_factor_offset.as_ref(),
                factor_scale,
                offset.as_ref(),
            )),
            _ => None,
        }
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
            (PreparedCarryLinearOp::Add, Self::AddStateList { terms, offset }) => {
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!("add_{rendered_terms}_plus_invariant")
                } else {
                    format!("add_{rendered_terms}")
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateList {
                    terms,
                    factor: _,
                    offset,
                },
            ) => {
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!("add_scaled_{rendered_terms}_plus_invariant")
                } else {
                    format!("add_scaled_{rendered_terms}")
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateListByState {
                    terms,
                    factor,
                    offset,
                },
            ) => {
                let factor = Self::state_ref_contract_fragment(*factor);
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!("add_scaled_by_{factor}_{rendered_terms}_plus_invariant")
                } else {
                    format!("add_scaled_by_{factor}_{rendered_terms}")
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateListByStatePlusInvariant {
                    terms,
                    factor,
                    factor_offset: _,
                    offset,
                },
            ) => {
                let factor = Self::state_ref_contract_fragment(*factor);
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!(
                        "add_scaled_by_{factor}_plus_factor_invariant_{rendered_terms}_plus_invariant"
                    )
                } else {
                    format!("add_scaled_by_{factor}_plus_factor_invariant_{rendered_terms}")
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateListByFactorStateList {
                    terms,
                    factor_terms,
                    factor_offset,
                    offset,
                },
            ) => {
                let rendered_factor_terms = factor_terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                match (factor_offset.is_some(), offset.is_some()) {
                    (true, true) => format!(
                        "add_scaled_by_{rendered_factor_terms}_plus_factor_invariant_times_{rendered_terms}_plus_invariant"
                    ),
                    (true, false) => format!(
                        "add_scaled_by_{rendered_factor_terms}_plus_factor_invariant_times_{rendered_terms}"
                    ),
                    (false, true) => format!(
                        "add_scaled_by_{rendered_factor_terms}_times_{rendered_terms}_plus_invariant"
                    ),
                    (false, false) => {
                        format!("add_scaled_by_{rendered_factor_terms}_times_{rendered_terms}")
                    }
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateListByFactorStateListTimesInvariant {
                    terms,
                    factor_terms,
                    factor_scale: _,
                    factor_offset,
                    offset,
                },
            ) => {
                let rendered_factor_terms = factor_terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                match (factor_offset.is_some(), offset.is_some()) {
                    (true, true) => format!(
                        "add_scaled_by_{rendered_factor_terms}_plus_factor_invariant_times_factor_invariant_times_{rendered_terms}_plus_invariant"
                    ),
                    (true, false) => format!(
                        "add_scaled_by_{rendered_factor_terms}_plus_factor_invariant_times_factor_invariant_times_{rendered_terms}"
                    ),
                    (false, true) => format!(
                        "add_scaled_by_{rendered_factor_terms}_times_factor_invariant_times_{rendered_terms}_plus_invariant"
                    ),
                    (false, false) => format!(
                        "add_scaled_by_{rendered_factor_terms}_times_factor_invariant_times_{rendered_terms}"
                    ),
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateListByFactorGroupProduct {
                    terms,
                    lhs_factor_terms,
                    lhs_factor_offset,
                    rhs_factor_terms,
                    rhs_factor_offset,
                    offset,
                },
            ) => {
                let render_group =
                    |terms: &[PreparedLoopStateRef], group_offset: &Option<NirExpr>| -> String {
                        let rendered_terms = terms
                            .iter()
                            .map(|term| Self::state_ref_contract_fragment(*term))
                            .collect::<Vec<_>>()
                            .join("_plus_");
                        if group_offset.is_some() {
                            format!("{rendered_terms}_plus_factor_invariant")
                        } else {
                            rendered_terms
                        }
                    };
                let lhs_group = render_group(lhs_factor_terms, lhs_factor_offset);
                let rhs_group = render_group(rhs_factor_terms, rhs_factor_offset);
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!(
                        "add_scaled_by_{lhs_group}_times_factor_group_{rhs_group}_times_terms_{rendered_terms}_plus_invariant"
                    )
                } else {
                    format!(
                        "add_scaled_by_{lhs_group}_times_factor_group_{rhs_group}_times_terms_{rendered_terms}"
                    )
                }
            }
            (
                PreparedCarryLinearOp::Add,
                Self::ScaledStateListByFactorGroupProductTimesInvariant {
                    terms,
                    lhs_factor_terms,
                    lhs_factor_offset,
                    rhs_factor_terms,
                    rhs_factor_offset,
                    factor_scale: _,
                    offset,
                },
            ) => {
                let render_group =
                    |terms: &[PreparedLoopStateRef], group_offset: &Option<NirExpr>| -> String {
                        let rendered_terms = terms
                            .iter()
                            .map(|term| Self::state_ref_contract_fragment(*term))
                            .collect::<Vec<_>>()
                            .join("_plus_");
                        if group_offset.is_some() {
                            format!("{rendered_terms}_plus_factor_invariant")
                        } else {
                            rendered_terms
                        }
                    };
                let lhs_group = render_group(lhs_factor_terms, lhs_factor_offset);
                let rhs_group = render_group(rhs_factor_terms, rhs_factor_offset);
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!(
                        "add_scaled_by_{lhs_group}_times_factor_group_{rhs_group}_times_factor_invariant_times_terms_{rendered_terms}_plus_invariant"
                    )
                } else {
                    format!(
                        "add_scaled_by_{lhs_group}_times_factor_group_{rhs_group}_times_factor_invariant_times_terms_{rendered_terms}"
                    )
                }
            }
            (PreparedCarryLinearOp::Add, Self::InvariantExpr(_)) => "add_invariant".to_owned(),
            (PreparedCarryLinearOp::Add, Self::AddInvariant { base, offset: _ }) => {
                match base.as_ref() {
                    Self::Current => "add_current_plus_invariant".to_owned(),
                    Self::PreviousCurrent => "add_prev_current_plus_invariant".to_owned(),
                    Self::PreviousCarry(index) => format!("add_prev_carry{index}_plus_invariant"),
                    Self::Carry(index) => format!("add_carry{index}_plus_invariant"),
                    Self::InvariantExpr(_)
                    | Self::AddInvariant { .. }
                    | Self::AddStateList { .. }
                    | Self::ScaledStateList { .. }
                    | Self::ScaledStateListByState { .. }
                    | Self::ScaledStateListByStatePlusInvariant { .. }
                    | Self::ScaledStateListByFactorStateList { .. }
                    | Self::ScaledStateListByFactorStateListTimesInvariant { .. }
                    | Self::ScaledStateListByFactorGroupProduct { .. }
                    | Self::ScaledStateListByFactorGroupProductTimesInvariant { .. }
                    | Self::FixedRead(_)
                    | Self::DynamicReadAt { .. } => "add_source_plus_invariant".to_owned(),
                }
            }
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
                    Self::InvariantExpr(_)
                    | Self::AddInvariant { .. }
                    | Self::AddStateList { .. }
                    | Self::ScaledStateList { .. }
                    | Self::ScaledStateListByState { .. }
                    | Self::ScaledStateListByStatePlusInvariant { .. }
                    | Self::ScaledStateListByFactorStateList { .. }
                    | Self::ScaledStateListByFactorStateListTimesInvariant { .. }
                    | Self::ScaledStateListByFactorGroupProduct { .. }
                    | Self::ScaledStateListByFactorGroupProductTimesInvariant { .. }
                    | Self::FixedRead(_)
                    | Self::DynamicReadAt { .. } => {
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
                Self::ScaledStateList {
                    terms,
                    factor: _,
                    offset,
                },
            ) => {
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!("mul_scaled_{rendered_terms}_plus_invariant")
                } else {
                    format!("mul_scaled_{rendered_terms}")
                }
            }
            (
                PreparedCarryLinearOp::Mul,
                Self::ScaledStateListByState {
                    terms,
                    factor,
                    offset,
                },
            ) => {
                let factor = Self::state_ref_contract_fragment(*factor);
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!("mul_scaled_by_{factor}_{rendered_terms}_plus_invariant")
                } else {
                    format!("mul_scaled_by_{factor}_{rendered_terms}")
                }
            }
            (
                PreparedCarryLinearOp::Mul,
                Self::ScaledStateListByStatePlusInvariant {
                    terms,
                    factor,
                    factor_offset: _,
                    offset,
                },
            ) => {
                let factor = Self::state_ref_contract_fragment(*factor);
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!(
                        "mul_scaled_by_{factor}_plus_factor_invariant_{rendered_terms}_plus_invariant"
                    )
                } else {
                    format!("mul_scaled_by_{factor}_plus_factor_invariant_{rendered_terms}")
                }
            }
            (PreparedCarryLinearOp::Mul, Self::ScaledStateListByFactorStateList { .. })
            | (
                PreparedCarryLinearOp::Mul,
                Self::ScaledStateListByFactorStateListTimesInvariant { .. },
            )
            | (PreparedCarryLinearOp::Mul, Self::ScaledStateListByFactorGroupProduct { .. })
            | (
                PreparedCarryLinearOp::Mul,
                Self::ScaledStateListByFactorGroupProductTimesInvariant { .. },
            ) => {
                unreachable!("invariant/affine carry sources are currently add-only")
            }
            (PreparedCarryLinearOp::Mul, Self::AddStateList { terms, offset }) => {
                let rendered_terms = terms
                    .iter()
                    .map(|term| Self::state_ref_contract_fragment(*term))
                    .collect::<Vec<_>>()
                    .join("_plus_");
                if offset.is_some() {
                    format!("mul_{rendered_terms}_plus_invariant")
                } else {
                    format!("mul_{rendered_terms}")
                }
            }
            (PreparedCarryLinearOp::Mul, Self::InvariantExpr(_)) => "mul_invariant".to_owned(),
            (PreparedCarryLinearOp::Mul, Self::AddInvariant { base, offset: _ }) => {
                match base.as_ref() {
                    Self::Current => "mul_current_plus_invariant".to_owned(),
                    Self::PreviousCurrent => "mul_prev_current_plus_invariant".to_owned(),
                    Self::PreviousCarry(index) => format!("mul_prev_carry{index}_plus_invariant"),
                    Self::Carry(index) => format!("mul_carry{index}_plus_invariant"),
                    Self::InvariantExpr(_)
                    | Self::AddInvariant { .. }
                    | Self::AddStateList { .. }
                    | Self::ScaledStateList { .. }
                    | Self::ScaledStateListByState { .. }
                    | Self::ScaledStateListByStatePlusInvariant { .. }
                    | Self::ScaledStateListByFactorStateList { .. }
                    | Self::ScaledStateListByFactorStateListTimesInvariant { .. }
                    | Self::ScaledStateListByFactorGroupProduct { .. }
                    | Self::ScaledStateListByFactorGroupProductTimesInvariant { .. }
                    | Self::FixedRead(_)
                    | Self::DynamicReadAt { .. } => "mul_source_plus_invariant".to_owned(),
                }
            }
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
                    Self::InvariantExpr(_)
                    | Self::AddInvariant { .. }
                    | Self::AddStateList { .. }
                    | Self::ScaledStateList { .. }
                    | Self::ScaledStateListByState { .. }
                    | Self::ScaledStateListByStatePlusInvariant { .. }
                    | Self::ScaledStateListByFactorStateList { .. }
                    | Self::ScaledStateListByFactorStateListTimesInvariant { .. }
                    | Self::ScaledStateListByFactorGroupProduct { .. }
                    | Self::ScaledStateListByFactorGroupProductTimesInvariant { .. }
                    | Self::FixedRead(_)
                    | Self::DynamicReadAt { .. } => {
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

    pub(super) fn from_linear_source(
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    ) -> Self {
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

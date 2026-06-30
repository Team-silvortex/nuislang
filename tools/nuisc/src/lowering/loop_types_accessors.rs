use super::*;

impl PreparedReadableCarrySourceCandidate {
    pub(in crate::lowering) fn family(&self) -> PreparedReadableCarrySourceFamily {
        match self {
            Self::Fixed(_) => PreparedReadableCarrySourceFamily::Fixed,
            Self::DynamicIndexAt { .. } => PreparedReadableCarrySourceFamily::DynamicIndexAt,
            Self::TraversalNext { .. } => PreparedReadableCarrySourceFamily::TraversalNext,
        }
    }

    #[cfg(test)]
    pub(in crate::lowering) fn family_name(&self) -> &'static str {
        match self.family() {
            PreparedReadableCarrySourceFamily::Fixed => "fixed_read",
            PreparedReadableCarrySourceFamily::DynamicIndexAt => "dynamic_index_at",
            PreparedReadableCarrySourceFamily::TraversalNext => "traversal_next",
        }
    }

    pub(in crate::lowering) fn fixed_read(&self) -> Option<&PreparedFixedReadCarrySource> {
        match self {
            Self::Fixed(source) => Some(source),
            _ => None,
        }
    }
}

impl PreparedCarrySource {
    pub(in crate::lowering) fn is_fixed_read(&self) -> bool {
        matches!(self, Self::FixedRead(_))
    }

    pub(in crate::lowering) fn is_dynamic_read_at(&self) -> bool {
        matches!(self, Self::DynamicReadAt { .. })
    }

    pub(in crate::lowering) fn invariant_expr(&self) -> Option<&NirExpr> {
        match self {
            Self::InvariantExpr(expr) => Some(expr),
            _ => None,
        }
    }

    pub(in crate::lowering) fn add_invariant(&self) -> Option<(&PreparedCarrySource, &NirExpr)> {
        match self {
            Self::AddInvariant { base, offset } => Some((base.as_ref(), offset)),
            _ => None,
        }
    }

    pub(in crate::lowering) fn add_state_list(
        &self,
    ) -> Option<(&[PreparedLoopStateRef], Option<&NirExpr>)> {
        match self {
            Self::AddStateList { terms, offset } => Some((terms.as_slice(), offset.as_ref())),
            _ => None,
        }
    }

    pub(in crate::lowering) fn scaled_state_list(
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

    pub(in crate::lowering) fn scaled_state_list_by_state(
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

    pub(in crate::lowering) fn scaled_state_list_by_state_plus_invariant(
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

    pub(in crate::lowering) fn scaled_state_list_by_factor_state_list(
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

    pub(in crate::lowering) fn scaled_state_list_by_factor_state_list_times_invariant(
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

    pub(in crate::lowering) fn scaled_state_list_by_factor_group_product(
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

    pub(in crate::lowering) fn scaled_state_list_by_factor_group_product_times_invariant(
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

    pub(in crate::lowering) fn fixed_read(&self) -> Option<&PreparedFixedReadCarrySource> {
        match self {
            Self::FixedRead(source) => Some(source),
            _ => None,
        }
    }

    pub(in crate::lowering) fn dynamic_read_at(&self) -> Option<(&NirExpr, &PreparedCarrySource)> {
        match self {
            Self::DynamicReadAt {
                buffer,
                index_source,
            } => Some((buffer, index_source.as_ref())),
            _ => None,
        }
    }
}

impl PreparedCarryBranchSource {
    pub(in crate::lowering) fn keep() -> Self {
        Self::KeepCurrentValue
    }

    pub(in crate::lowering) fn keep_previous_value() -> Self {
        Self::KeepPreviousValue
    }

    pub(in crate::lowering) fn from_linear_source(
        op: PreparedCarryLinearOp,
        source: PreparedCarrySource,
    ) -> Self {
        Self::Source { op, source }
    }

    pub(in crate::lowering) fn value_kind(&self) -> PreparedCarryBranchValueKind {
        match self {
            Self::KeepCurrentValue => PreparedCarryBranchValueKind::KeepCurrentValue,
            Self::KeepPreviousValue => PreparedCarryBranchValueKind::KeepPreviousValue,
            Self::Source { op, source } => PreparedCarryBranchValueKind::LinearSource {
                op: *op,
                source: source.clone(),
            },
        }
    }

    pub(in crate::lowering) fn view(&self) -> PreparedCarryBranchView<'_> {
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

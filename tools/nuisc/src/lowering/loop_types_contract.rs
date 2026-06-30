use super::*;

impl PreparedCarrySource {
    fn state_ref_contract_fragment(state_ref: PreparedLoopStateRef) -> String {
        match state_ref {
            PreparedLoopStateRef::Current => "current".to_owned(),
            PreparedLoopStateRef::PreviousCurrent => "prev_current".to_owned(),
            PreparedLoopStateRef::PreviousCarry(index) => format!("prev_carry{index}"),
            PreparedLoopStateRef::Carry(index) => format!("carry{index}"),
        }
    }

    pub(in crate::lowering) fn contract_kind(&self, op: PreparedCarryLinearOp) -> String {
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
                    Self::FixedRead(PreparedFixedReadCarrySource::Value(_)) => {
                        "add_read_value_fixed_plus_invariant".to_owned()
                    }
                    Self::FixedRead(PreparedFixedReadCarrySource::At { .. }) => {
                        "add_read_at_fixed_plus_invariant".to_owned()
                    }
                    Self::DynamicReadAt { index_source, .. } => match index_source.as_ref() {
                        Self::Current => "add_read_at_dynamic_current_plus_invariant".to_owned(),
                        Self::PreviousCurrent => {
                            "add_read_at_dynamic_prev_current_plus_invariant".to_owned()
                        }
                        Self::PreviousCarry(index) => {
                            format!("add_read_at_dynamic_prev_carry{index}_plus_invariant")
                        }
                        Self::Carry(index) => {
                            format!("add_read_at_dynamic_carry{index}_plus_invariant")
                        }
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
                        | Self::DynamicReadAt { .. } => unreachable!(
                            "dynamic read index sources must be simple loop-state sources"
                        ),
                    },
                    Self::InvariantExpr(_)
                    | Self::AddInvariant { .. }
                    | Self::AddStateList { .. }
                    | Self::ScaledStateList { .. }
                    | Self::ScaledStateListByState { .. }
                    | Self::ScaledStateListByStatePlusInvariant { .. }
                    | Self::ScaledStateListByFactorStateList { .. }
                    | Self::ScaledStateListByFactorStateListTimesInvariant { .. }
                    | Self::ScaledStateListByFactorGroupProduct { .. }
                    | Self::ScaledStateListByFactorGroupProductTimesInvariant { .. } => {
                        "add_source_plus_invariant".to_owned()
                    }
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
                    Self::FixedRead(PreparedFixedReadCarrySource::Value(_)) => {
                        "mul_read_value_fixed_plus_invariant".to_owned()
                    }
                    Self::FixedRead(PreparedFixedReadCarrySource::At { .. }) => {
                        "mul_read_at_fixed_plus_invariant".to_owned()
                    }
                    Self::DynamicReadAt { index_source, .. } => match index_source.as_ref() {
                        Self::Current => "mul_read_at_dynamic_current_plus_invariant".to_owned(),
                        Self::PreviousCurrent => {
                            "mul_read_at_dynamic_prev_current_plus_invariant".to_owned()
                        }
                        Self::PreviousCarry(index) => {
                            format!("mul_read_at_dynamic_prev_carry{index}_plus_invariant")
                        }
                        Self::Carry(index) => {
                            format!("mul_read_at_dynamic_carry{index}_plus_invariant")
                        }
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
                        | Self::DynamicReadAt { .. } => unreachable!(
                            "dynamic read index sources must be simple loop-state sources"
                        ),
                    },
                    Self::InvariantExpr(_)
                    | Self::AddInvariant { .. }
                    | Self::AddStateList { .. }
                    | Self::ScaledStateList { .. }
                    | Self::ScaledStateListByState { .. }
                    | Self::ScaledStateListByStatePlusInvariant { .. }
                    | Self::ScaledStateListByFactorStateList { .. }
                    | Self::ScaledStateListByFactorStateListTimesInvariant { .. }
                    | Self::ScaledStateListByFactorGroupProduct { .. }
                    | Self::ScaledStateListByFactorGroupProductTimesInvariant { .. } => {
                        "mul_source_plus_invariant".to_owned()
                    }
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

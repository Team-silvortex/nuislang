use super::*;

#[test]
fn readable_carry_candidate_recognizes_fixed_load_value() {
    let expr = NirExpr::LoadValue(Box::new(NirExpr::Var("head".to_owned())));
    let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
        .expect("expected readable carry candidate");
    assert_eq!(candidate.family_name(), "fixed_read");
    assert!(matches!(
        candidate.fixed_read(),
        Some(PreparedFixedReadCarrySource::Value(_))
    ));
}

#[test]
fn readable_carry_candidate_recognizes_fixed_load_at() {
    let expr = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Int(0)),
    };
    let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
        .expect("expected readable carry candidate");
    assert_eq!(candidate.family_name(), "fixed_read");
    assert!(matches!(
        candidate.fixed_read(),
        Some(PreparedFixedReadCarrySource::At { .. })
    ));
}

#[test]
fn readable_carry_candidate_recognizes_dynamic_index_load_at_separately() {
    let expr = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Var("current".to_owned())),
    };
    let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
        .expect("expected readable carry candidate");
    assert_eq!(candidate.family_name(), "dynamic_index_at");
    assert!(candidate.fixed_read().is_none());
}

#[test]
fn readable_carry_candidate_recognizes_load_next_traversal_separately() {
    let expr = NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned())));
    let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
        .expect("expected readable carry candidate");
    assert_eq!(candidate.family_name(), "traversal_next");
    assert!(candidate.fixed_read().is_none());
}

#[test]
fn parse_prepared_dynamic_read_carry_source_accepts_current_index_driver() {
    let expr = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Var("current".to_owned())),
    };
    let source = parse_prepared_dynamic_read_carry_source(&expr, "current", &[])
        .expect("expected dynamic read carry source");
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_dynamic_current"
    );
}

#[test]
fn parse_prepared_dynamic_read_carry_source_accepts_prior_carry_index_driver() {
    let expr = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Var("slot".to_owned())),
    };
    let carries = vec![PreparedCarryUpdate {
        binding_name: "slot".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    let source = parse_prepared_dynamic_read_carry_source(&expr, "current", &carries)
        .expect("expected dynamic read carry source");
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_dynamic_carry0"
    );
}

#[test]
fn parse_prepared_dynamic_read_carry_source_rejects_non_direct_index_expr() {
    let expr = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("current".to_owned())),
            rhs: Box::new(NirExpr::Int(1)),
        }),
    };
    assert!(parse_prepared_dynamic_read_carry_source(&expr, "current", &[]).is_none());
}

#[test]
fn dynamic_read_contract_kind_supports_prev_current_and_prev_carry_index_drivers() {
    let prev_current = PreparedCarrySource::DynamicReadAt {
        buffer: NirExpr::Var("buffer".to_owned()),
        index_source: Box::new(PreparedCarrySource::PreviousCurrent),
    };
    let prev_carry = PreparedCarrySource::DynamicReadAt {
        buffer: NirExpr::Var("buffer".to_owned()),
        index_source: Box::new(PreparedCarrySource::PreviousCarry(0)),
    };
    assert_eq!(
        prev_current.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_dynamic_prev_current"
    );
    assert_eq!(
        prev_carry.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_dynamic_prev_carry0"
    );
}

#[test]
fn add_invariant_contract_kind_distinguishes_fixed_read_shapes() {
    let fixed_value = PreparedCarrySource::AddInvariant {
        base: Box::new(PreparedCarrySource::FixedRead(
            PreparedFixedReadCarrySource::Value(NirExpr::Var("head".to_owned())),
        )),
        offset: NirExpr::Int(1),
    };
    let fixed_at = PreparedCarrySource::AddInvariant {
        base: Box::new(PreparedCarrySource::FixedRead(
            PreparedFixedReadCarrySource::At {
                buffer: NirExpr::Var("buffer".to_owned()),
                index: NirExpr::Int(0),
            },
        )),
        offset: NirExpr::Int(1),
    };
    assert_eq!(
        fixed_value.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_value_fixed_plus_invariant"
    );
    assert_eq!(
        fixed_at.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_fixed_plus_invariant"
    );
}

#[test]
fn add_invariant_contract_kind_distinguishes_dynamic_read_shapes() {
    let dynamic_current = PreparedCarrySource::AddInvariant {
        base: Box::new(PreparedCarrySource::DynamicReadAt {
            buffer: NirExpr::Var("buffer".to_owned()),
            index_source: Box::new(PreparedCarrySource::Current),
        }),
        offset: NirExpr::Int(1),
    };
    let dynamic_prev_carry = PreparedCarrySource::AddInvariant {
        base: Box::new(PreparedCarrySource::DynamicReadAt {
            buffer: NirExpr::Var("buffer".to_owned()),
            index_source: Box::new(PreparedCarrySource::PreviousCarry(0)),
        }),
        offset: NirExpr::Int(1),
    };
    assert_eq!(
        dynamic_current.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_dynamic_current_plus_invariant"
    );
    assert_eq!(
        dynamic_prev_carry.contract_kind(PreparedCarryLinearOp::Add),
        "add_read_at_dynamic_prev_carry0_plus_invariant"
    );
}

#[test]
fn parse_loop_carry_linear_accepts_multiplicative_state_plus_invariant_source() {
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("current".to_owned())),
            rhs: Box::new(NirExpr::Int(1)),
        }),
    };
    let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &[], &BTreeMap::new())
        .expect("expected multiplicative additive carry source");
    assert!(matches!(op, PreparedCarryLinearOp::Mul));
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Mul),
        "mul_current_plus_invariant"
    );
}

#[test]
fn parse_loop_carry_linear_accepts_multiplicative_multi_state_additive_source() {
    let carries = vec![PreparedCarryUpdate {
        binding_name: "slot".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("current".to_owned())),
            rhs: Box::new(NirExpr::Var("slot".to_owned())),
        }),
    };
    let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &carries, &BTreeMap::new())
        .expect("expected multiplicative additive carry source");
    assert!(matches!(op, PreparedCarryLinearOp::Mul));
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Mul),
        "mul_current_plus_carry0"
    );
}

#[test]
fn parse_loop_carry_linear_accepts_scaled_multiplicative_additive_source() {
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            }),
            rhs: Box::new(NirExpr::Int(2)),
        }),
    };
    let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &[], &BTreeMap::new())
        .expect("expected scaled multiplicative additive carry source");
    assert!(matches!(op, PreparedCarryLinearOp::Mul));
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Mul),
        "mul_scaled_current_plus_invariant"
    );
}

#[test]
fn parse_loop_carry_linear_accepts_state_scaled_multiplicative_additive_source() {
    let carries = vec![PreparedCarryUpdate {
        binding_name: "sum".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Var("sum".to_owned())),
            }),
            rhs: Box::new(NirExpr::Var("current".to_owned())),
        }),
    };
    let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &carries, &BTreeMap::new())
        .expect("expected state-scaled multiplicative additive carry source");
    assert!(matches!(op, PreparedCarryLinearOp::Mul));
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Mul),
        "mul_scaled_by_current_current_plus_carry0"
    );
}

#[test]
fn parse_loop_carry_linear_accepts_state_plus_invariant_scaled_multiplicative_additive_source() {
    let carries = vec![PreparedCarryUpdate {
        binding_name: "sum".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Var("sum".to_owned())),
            }),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            }),
        }),
    };
    let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &carries, &BTreeMap::new())
        .expect("expected state-plus-invariant scaled multiplicative additive carry source");
    assert!(matches!(op, PreparedCarryLinearOp::Mul));
    assert_eq!(
        source.contract_kind(PreparedCarryLinearOp::Mul),
        "mul_scaled_by_current_plus_factor_invariant_current_plus_carry0"
    );
}

#[test]
fn parses_loop_state_refs_for_current_prev_and_carry_slots() {
    let carries = vec![PreparedCarryUpdate {
        binding_name: "slot".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    assert_eq!(
        parse_prepared_loop_state_ref_name("current", "current", &carries),
        Some(PreparedLoopStateRef::Current)
    );
    assert_eq!(
        parse_prepared_loop_state_ref_name("__tailrec_prev_current", "current", &carries),
        Some(PreparedLoopStateRef::PreviousCurrent)
    );
    assert_eq!(
        parse_prepared_loop_state_ref_name("__tailrec_prev_carry_0", "current", &carries),
        Some(PreparedLoopStateRef::PreviousCarry(0))
    );
    assert_eq!(
        parse_prepared_loop_state_ref_name("slot", "current", &carries),
        Some(PreparedLoopStateRef::Carry(0))
    );
}

#[test]
fn parses_loop_state_refs_directly_from_carry_binding_names() {
    let carry_binding_names = vec!["slot".to_owned(), "acc".to_owned()];
    assert_eq!(
        parse_prepared_loop_state_ref_name_from_carry_names(
            "current",
            "current",
            &carry_binding_names,
        ),
        Some(PreparedLoopStateRef::Current)
    );
    assert_eq!(
        parse_prepared_loop_state_ref_name_from_carry_names(
            "slot",
            "current",
            &carry_binding_names,
        ),
        Some(PreparedLoopStateRef::Carry(0))
    );
    assert_eq!(
        parse_prepared_loop_state_ref_name_from_carry_names("acc", "current", &carry_binding_names,),
        Some(PreparedLoopStateRef::Carry(1))
    );
}

#[test]
fn parses_loop_state_refs_directly_from_var_exprs() {
    let carries = vec![PreparedCarryUpdate {
        binding_name: "slot".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    assert_eq!(
        parse_prepared_loop_state_ref_expr(
            &NirExpr::Var("__tailrec_prev_current".to_owned()),
            "current",
            &carries,
        ),
        Some(PreparedLoopStateRef::PreviousCurrent)
    );
    assert_eq!(
        parse_prepared_loop_state_ref_expr(&NirExpr::Var("slot".to_owned()), "current", &carries,),
        Some(PreparedLoopStateRef::Carry(0))
    );
    assert_eq!(
        parse_prepared_loop_state_ref_expr(&NirExpr::Int(1), "current", &carries),
        None
    );
}

#[test]
fn parses_loop_carry_keep_source_for_identity_and_add_zero_forms() {
    assert!(matches!(
        parse_loop_carry_keep_source("acc", &NirExpr::Var("acc".to_owned()), &[]),
        Some(source) if matches!(source.view(), PreparedCarryBranchView::KeepCurrentValue)
    ));
    assert!(matches!(
        parse_loop_carry_keep_source(
            "acc",
            &NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("acc".to_owned())),
                rhs: Box::new(NirExpr::Int(0)),
            }
        , &[]),
        Some(source) if matches!(source.view(), PreparedCarryBranchView::KeepCurrentValue)
    ));
    assert!(parse_loop_carry_keep_source(
        "acc",
        &NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Int(1)),
        },
        &[],
    )
    .is_none());
}

#[test]
fn parses_loop_carry_keep_source_for_explicit_previous_value_placeholder() {
    let carries = vec![PreparedCarryUpdate {
        binding_name: "acc".to_owned(),
        kind: PreparedCarryUpdateKind::Linear {
            op: PreparedCarryLinearOp::Add,
            source: Box::new(PreparedCarrySource::Current),
        },
    }];
    let previous_name = tail_recursive_prev_carry_binding(carries.len());
    assert!(matches!(
        parse_loop_carry_keep_source("slot", &NirExpr::Var(previous_name), &carries),
        Some(source) if matches!(source.view(), PreparedCarryBranchView::KeepPreviousValue)
    ));
}

#[test]
fn prepared_carry_branch_source_helpers_round_trip_keep_and_source_variants() {
    let keep = PreparedCarryBranchSource::keep();
    assert!(matches!(
        keep.view(),
        PreparedCarryBranchView::KeepCurrentValue
    ));
    assert_eq!(encode_branch_view_name(keep.view()), "keep_current_value");
    assert!(matches!(
        keep.value_kind(),
        PreparedCarryBranchValueKind::KeepCurrentValue
    ));

    let keep_previous = PreparedCarryBranchSource::keep_previous_value();
    assert!(matches!(
        keep_previous.view(),
        PreparedCarryBranchView::KeepPreviousValue
    ));
    assert_eq!(
        encode_branch_view_name(keep_previous.view()),
        "keep_previous_value"
    );
    assert!(matches!(
        keep_previous.value_kind(),
        PreparedCarryBranchValueKind::KeepPreviousValue
    ));

    let source = PreparedCarryBranchSource::from_linear_source(
        PreparedCarryLinearOp::Add,
        PreparedCarrySource::Current,
    );
    match source.value_kind() {
        PreparedCarryBranchValueKind::LinearSource { op, source } => {
            assert!(op == PreparedCarryLinearOp::Add);
            assert!(matches!(*source, PreparedCarrySource::Current));
        }
        _ => panic!("expected linear source branch value kind"),
    }
    match source.view() {
        PreparedCarryBranchView::Source { op, source } => {
            assert!(op == PreparedCarryLinearOp::Add);
            assert!(matches!(source, PreparedCarrySource::Current));
        }
        _ => panic!("expected linear source branch view"),
    }
}

#[test]
fn previous_value_branch_view_is_constructible_and_distinct_from_current_keep() {
    let source = PreparedCarryBranchSource::keep_previous_value();
    assert!(matches!(
        source.value_kind(),
        PreparedCarryBranchValueKind::KeepPreviousValue
    ));
    assert!(matches!(
        source.view(),
        PreparedCarryBranchView::KeepPreviousValue
    ));
    assert!(
        parse_loop_carry_keep_source("acc", &NirExpr::Var("acc".to_owned()), &[]).is_some_and(
            |parsed| matches!(
                parsed.value_kind(),
                PreparedCarryBranchValueKind::KeepCurrentValue
            )
        )
    );
}

fn encode_branch_view_name(view: PreparedCarryBranchView<'_>) -> &'static str {
    match view {
        PreparedCarryBranchView::KeepCurrentValue => "keep_current_value",
        PreparedCarryBranchView::KeepPreviousValue => "keep_previous_value",
        PreparedCarryBranchView::Source { .. } => "source",
    }
}

#[test]
fn keep_like_loop_carry_exprs_do_not_report_unsupported_diagnostics() {
    assert!(diagnose_unsupported_loop_carry_expr(
        "acc",
        &NirExpr::Var("acc".to_owned()),
        "current",
        &[],
        &BTreeMap::new(),
    )
    .is_none());
    assert!(diagnose_unsupported_loop_carry_expr(
        "acc",
        &NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Int(0)),
        },
        "current",
        &[],
        &BTreeMap::new(),
    )
    .is_none());
}

#[test]
fn diagnose_unsupported_loop_carry_expr_reports_non_direct_dynamic_index_reads() {
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer".to_owned())),
            index: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            }),
        }),
    };
    let diagnostic =
        diagnose_unsupported_loop_carry_expr("acc", &expr, "current", &[], &BTreeMap::new())
            .expect("expected unsupported carry diagnostic");
    assert!(diagnostic.contains("dynamic `load_at(buffer, index)` reads currently support only direct loop-state index drivers"));
}

#[test]
fn diagnose_unsupported_loop_carry_expr_reports_traversal_next_reads() {
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Var("acc".to_owned())),
        rhs: Box::new(NirExpr::LoadNext(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
    };
    let diagnostic =
        diagnose_unsupported_loop_carry_expr("acc", &expr, "current", &[], &BTreeMap::new())
            .expect("expected unsupported carry diagnostic");
    assert!(diagnostic.contains("`load_next(...)` traversal reads"));
}

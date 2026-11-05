use super::*;

#[test]
fn dev_tensor_native_iteration_registration_preserves_order_and_uniqueness() {
    let ids = checks().map(|spec| spec.id).collect::<Vec<_>>();
    assert_eq!(
        &ids[..CHECKS.len()],
        CHECKS.iter().map(|spec| spec.id).collect::<Vec<_>>()
    );
    let mut previous_end = CHECKS.len();
    for group in [
        logical_trees::CHECKS,
        return_print_prefixes::CHECKS,
        return_print_aliases::CHECKS,
        staged_return_effects::CHECKS,
        logical_staged_initializers::CHECKS,
        continuing_effect_regions::CHECKS,
        effect_region_exits::CHECKS,
        exit_only_regions::CHECKS,
        effect_result_joins::CHECKS,
        partial_effect_joins::CHECKS,
        one_sided_effect_joins::CHECKS,
        effect_join_captures::CHECKS,
        equal_effect_joins::CHECKS,
        typed_effect_snapshots::CHECKS,
        float_literal_negation::CHECKS,
        float_sign_negation::CHECKS,
        typed_effect_returns::CHECKS,
        literal_print_policy::CHECKS,
        literal_print_build::CHECKS,
        effectful_selections::CHECKS,
        argument_print::I32_NEGATION_ARGUMENT_PRINT,
        argument_print::I32_WIDENING_ARGUMENT_PRINT,
        argument_print::I64_ARITHMETIC_ARGUMENT_PRINT,
        argument_print::I64_XOR_ARGUMENT_PRINT,
        argument_print::LOCAL_SCALAR_ARGUMENT_PRINT,
        argument_print::EPOCH_LOCAL_PLAN,
        argument_print::EPOCH_LOCAL_ARGUMENT_PRINT,
        argument_print::CHILD_EPOCH_LOCAL_ARGUMENT_PRINT,
        argument_print::PARENT_REENTRY_ARGUMENT_PRINT,
        argument_print::CHILD_PROGRESS_ARGUMENT_PRINT,
        argument_print::PARENT_BOUND_ARGUMENT_PRINT,
        argument_print::FALLIBLE_BOUND_ARGUMENT_PRINT,
        argument_print::STATE_BOUND_ARGUMENT_PRINT,
        argument_print::STATE_BREAK_ARGUMENT_PRINT,
        argument_print::STATE_CONTINUE_ARGUMENT_PRINT,
        argument_print::STATE_MIXED_ARGUMENT_PRINT,
        argument_print::STATE_PREDICATE_ARGUMENT_PRINT,
        argument_print::STATE_SHORT_CIRCUIT_ARGUMENT_PRINT,
        argument_print::STATE_GROUPED_ARGUMENT_PRINT,
        argument_print::STATE_FIELD_EXIT_ARGUMENT_PRINT,
        argument_print::STATE_DIVISION_EXIT_ARGUMENT_PRINT,
        computed_return_prints::CHECKS,
        caller_records::CHECKS,
        caller_spills::CHECKS,
        artifact_publication::CHECKS,
    ] {
        let start = ids.iter().position(|id| *id == group[0].id).unwrap();
        let end = start + group.len();
        assert!(start >= previous_end);
        assert_eq!(
            &ids[start..end],
            group.iter().map(|spec| spec.id).collect::<Vec<_>>()
        );
        previous_end = end;
    }
    assert_eq!(previous_end, ids.len());
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        ids.len()
    );
}

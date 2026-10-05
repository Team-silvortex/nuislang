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

use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_scalar_join_admits_typed_values_without_pure_effect_authority() {
    for variant in cases::scalar_join_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let before = module.clone();
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        for name in ["record_marker", "record_total", "project_record", "work"] {
            assert!(catalog.contains_key(name), "{name}");
            assert!(!pure.contains_key(name), "{name}");
        }
        assert!(!module.functions.iter().any(|f| f.name == "record_select"));
        for leaf in ["record_gate", "record_rhs"] {
            let mut denied = module.clone();
            denied.functions.retain(|f| f.name != leaf);
            let denied = scalar_helpers::collect_iteration_calls(&denied, &layouts);
            for name in [
                leaf,
                "record_marker",
                "record_total",
                "project_record",
                "work",
            ] {
                assert!(!denied.contains_key(name), "{name}");
            }
        }
        assert_eq!(module, before);
    }
}

#[test]
fn effect_return_storage_scalar_join_full_helpers_fit_native_bound() {
    for variant in cases::scalar_join_variants() {
        let (mut module, _, _) = plans(&cases::source(variant), "work");
        outline_buffer_loops(&mut module).unwrap();
        assert!(
            module.functions.len() <= 64,
            "{variant:?}: {}",
            module.functions.len()
        );
        for f in module
            .functions
            .iter()
            .filter(|f| matches!(f.name.as_str(), "record_marker" | "record_total"))
        {
            assert!(f.body.iter().any(|s| matches!(s, NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. } if name == "selected_root" && callee.starts_with("__nuis_conditional_value_"))));
        }
    }
}

#[test]
fn effect_return_storage_scalar_join_matches_independent_execution_and_carry_width() {
    super::payload_call_tests::check_execution(cases::scalar_join_variants());
}

#[test]
fn effect_return_storage_scalar_join_preserves_caller_certificate_controls() {
    super::projected_payload_tests::check_certificate(cases::scalar_join_variants());
}

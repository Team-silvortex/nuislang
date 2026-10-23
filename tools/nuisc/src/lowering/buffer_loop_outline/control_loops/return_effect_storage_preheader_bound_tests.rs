use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_preheader_bounds_admit_dominating_record_without_storage_authority() {
    for variant in cases::preheader_variants() {
        let source = cases::source(variant)
            .replace("frozen", "saved_extent")
            .replace("carry", "working_copy");
        let (module, layouts, catalog) = plans(&source, "work");
        let before = module.clone();
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
        assert!(verify(
            function,
            &independent,
            &shared,
            "working_copy",
            &layouts,
            &catalog
        )
        .is_some());
        let invalid = rewrite(
            function,
            &layouts,
            Some("saved_extent".into()),
            validation::LiteralPrints::Allow,
        )
        .unwrap();
        assert!(verify(
            function,
            &independent,
            &invalid,
            "saved_extent",
            &layouts,
            &catalog
        )
        .is_none());
        for callee in ["advance", "bound_seed", "bound_extent"] {
            let mut denied = catalog.clone();
            denied.remove(callee);
            assert!(prepare_effects(function, &layouts, &denied).is_none());
        }
        assert_eq!(module, before);
    }
}

#[test]
fn effect_return_storage_preheader_bounds_oracle_freezes_once_before_parent_backedges() {
    for variant in cases::preheader_variants() {
        let call = cases::base(variant);
        let (state, trace) = call.execute(true, None).unwrap();
        assert_eq!(state.total, if variant.leading { 123 } else { 107 });
        assert_eq!(trace.iter().filter(|&&v| v == 930).count(), 1);
        assert_eq!(trace.iter().filter(|&&v| v == 941).count(), 2);
        assert_eq!(
            trace
                .windows(5)
                .filter(|w| *w == [932, 933, 4, 1, 16])
                .count(),
            4
        );
        assert_eq!(&trace[..5], &[950, 940, 930, 931, 941]);
        let mut inactive = call;
        inactive.selected = false;
        inactive.root = 0;
        inactive.trips = i64::MAX;
        inactive.divisor = 0;
        let (_, trace) = inactive.execute(true, None).unwrap();
        assert!(!trace.contains(&930));
        for mode in [1, 4, 5] {
            let mut exit = call;
            exit.mode = mode;
            exit.second = -1;
            assert!(!exit.execute(true, None).unwrap().1.contains(&945));
        }
    }
}

#[test]
fn effect_return_storage_preheader_bounds_reject_typed_recomputation_and_effect_reordering() {
    for variant in cases::preheader_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..4 {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(super::sibling_bound_tests::mutate(
                &mut shared.body,
                mutation
            ));
            assert!(scalar_helpers::validate_iteration_body(
                function,
                &shared.body,
                &catalog,
                &layouts
            )
            .is_some());
            assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
        }
    }
}

#[test]
fn effect_return_storage_preheader_bounds_match_independent_values_effects_and_carry_reduction() {
    use super::execution_tests::{lowered, maximum_carry_width};
    use super::sibling_bound_tests::run_entry;
    for variant in cases::preheader_variants() {
        let mut calls = (0..=9)
            .map(|mode| {
                let mut call = cases::base(variant);
                call.mode = mode;
                call
            })
            .collect::<Vec<_>>();
        for (_, mut failure) in cases::failures(variant) {
            failure.input = failure.previous.take().unwrap();
            calls.push(failure);
        }
        let mut inactive = cases::base(variant);
        inactive.selected = false;
        inactive.root = 0;
        inactive.divisor = 0;
        inactive.trips = i64::MAX;
        calls.push(inactive);
        for call in calls {
            let args = call
                .words()
                .into_iter()
                .enumerate()
                .map(|(slot, word)| match slot {
                    0 | 4 => (word != 0).to_string(),
                    2 => format!("{:?}", f32::from_bits(word as u32)),
                    3 => format!("{:?}", f64::from_bits(word as u64)),
                    _ if word == i64::MIN => "(-9223372036854775807 - 1)".into(),
                    _ => word.to_string(),
                })
                .collect::<Vec<_>>()
                .join(",");
            let source = cases::source(variant).replace(
                "fn main() -> i64 { return 0; }",
                &format!("fn main() -> i64 {{ let result = start({args}); return result.total; }}"),
            );
            let independent = lowered(&source, true);
            let shared = lowered(&source, false);
            assert_eq!(
                maximum_carry_width(&shared) + 4,
                maximum_carry_width(&independent)
            );
            for reversed in [false, true] {
                let actual = run_entry(shared.clone(), reversed);
                assert_eq!(actual, run_entry(independent.clone(), reversed));
                assert_eq!(
                    actual,
                    call.execute(false, None)
                        .map(|(state, trace)| (yir_core::Value::Int(state.total), trace))
                        .map_err(|_| ()),
                    "{call:?}"
                );
            }
        }
    }
}

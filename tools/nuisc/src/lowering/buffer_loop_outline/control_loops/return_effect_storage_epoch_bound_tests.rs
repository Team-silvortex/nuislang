use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_epoch_bounds_admit_distinct_readonly_records_without_storage_authority() {
    for variant in cases::epoch_variants() {
        let source = cases::source(variant)
            .replace("frozen", "older")
            .replace("recent", "newer")
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
        for name in ["older", "newer"] {
            let invalid = rewrite(
                function,
                &layouts,
                Some(name.into()),
                validation::LiteralPrints::Allow,
            )
            .unwrap();
            assert!(verify(function, &independent, &invalid, name, &layouts, &catalog).is_none());
        }
        for callee in ["advance", "bound_seed", "bound_extent"] {
            let mut denied = catalog.clone();
            denied.remove(callee);
            assert!(prepare_effects(function, &layouts, &denied).is_none());
        }
        assert_eq!(module, before);
    }
}

fn substitute(body: &mut [NirStmt], binding: &str, replacement: &str) -> bool {
    for stmt in body {
        match stmt {
            NirStmt::Let {
                name,
                value: NirExpr::Call { args, .. },
                ..
            } if name == binding => {
                args[0] = NirExpr::Var(replacement.into());
                return true;
            }
            NirStmt::While { body, .. } => {
                if substitute(body, binding, replacement) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if substitute(then_body, binding, replacement)
                    || substitute(else_body, binding, replacement)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_epoch_bounds_reject_typed_wrong_epoch_recomputation_and_reordering() {
    for variant in cases::epoch_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..5 {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(match mutation {
                0 => substitute(&mut shared.body, "first_width", "frozen"),
                1 => substitute(&mut shared.body, "second_width", "recent"),
                2 => substitute(&mut shared.body, "recent", "frozen"),
                3 => super::sibling_bound_tests::mutate(&mut shared.body, 0),
                4 => super::sibling_bound_tests::mutate(&mut shared.body, 2),
                _ => unreachable!(),
            });
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
fn effect_return_storage_epoch_bounds_oracle_distinguishes_epochs_reconstruction_and_fault_order() {
    for variant in cases::epoch_variants() {
        let call = cases::base(variant);
        let (state, trace) = call.execute(true, None).unwrap();
        assert_eq!(state.total, if variant.leading { 116 } else { 102 });
        assert_eq!(trace.iter().filter(|&&v| v == 930).count(), 3);
        assert_eq!(trace.iter().filter(|&&v| v == 934).count(), 2);
        let widths = trace
            .windows(5)
            .filter(|w| w[0] == 932 && w[1] == 933 && w[4] == 16)
            .map(|w| w[2])
            .collect::<Vec<_>>();
        assert_eq!(widths, [2, 4, 4, 4]);
        for (prefix, failure) in cases::failures(variant) {
            assert_eq!(prefix.len(), 3);
            let trace = failure.execute(true, None).unwrap_err();
            if matches!(failure.second, -3 | -2) {
                assert!(!trace.contains(&943));
                assert_eq!(
                    trace.last(),
                    Some(&if failure.second == -2 { 934 } else { 930 })
                );
            }
        }
        for mode in [1, 4, 5] {
            let mut exit = call;
            exit.mode = mode;
            exit.second = -1;
            assert!(!exit.execute(true, None).unwrap().1.contains(&945));
        }
        let mut inactive = call;
        inactive.selected = false;
        inactive.trips = i64::MAX;
        inactive.root = 0;
        inactive.second = -2;
        assert!(!inactive.execute(true, None).unwrap().1.contains(&934));
    }
}

#[test]
fn effect_return_storage_epoch_bounds_match_independent_values_effects_and_four_word_reduction() {
    use super::execution_tests::{lowered, maximum_carry_width};
    use super::sibling_bound_tests::run_entry;
    for variant in cases::epoch_variants() {
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
        inactive.second = -2;
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

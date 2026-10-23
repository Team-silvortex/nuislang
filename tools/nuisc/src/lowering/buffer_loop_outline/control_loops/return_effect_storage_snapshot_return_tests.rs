use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_snapshot_returns_admit_both_records_without_mutable_snapshot_authority() {
    for variant in cases::snapshot_return_variants() {
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

#[test]
fn effect_return_storage_snapshot_returns_oracle_keeps_complete_fields_and_skips_later_effects() {
    for variant in cases::snapshot_return_variants() {
        for mode in [1, 6, 10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            if mode >= 10 {
                call.trips = 5;
                call.divisor = 2;
            }
            let (state, trace) = call.execute(true, None).unwrap();
            let newer = matches!(mode, 1 | 10);
            assert_eq!(state.result, newer);
            assert_eq!(state.gain, if newer { 0 } else { 0x80000000 });
            assert_eq!(state.marker, if newer { 0 } else { 0x8000000000000000 });
            assert_eq!(
                state.total,
                match mode {
                    1 => 2,
                    6 => 4,
                    10 if variant.leading => 2,
                    _ => 5,
                }
            );
            assert_eq!(
                trace.iter().filter(|&&v| v == 941).count(),
                if mode >= 10 { 2 } else { 1 }
            );
            assert!(!trace.contains(&949));
            if mode == 1 {
                assert!(!trace.contains(&945));
            }
            assert_eq!(trace.last(), Some(&960));
        }
    }
}

fn mutate(body: &mut Vec<NirStmt>, mutation: usize) -> bool {
    for index in 0..body.len() {
        let target = if mutation == 1 { "frozen" } else { "recent" };
        if matches!(&body[index], NirStmt::Let { name, value: NirExpr::Var(value), .. } if name == "carry" && value == target)
        {
            if mutation == 3 {
                assert!(matches!(
                    &body[index + 1],
                    NirStmt::Let {
                        value: NirExpr::Int(1),
                        ..
                    }
                ));
                body.swap(index, index + 1);
            } else {
                let NirStmt::Let { value, .. } = &mut body[index] else {
                    unreachable!()
                };
                *value = NirExpr::Var(
                    match mutation {
                        0 => "frozen",
                        1 => "recent",
                        2 => "carry",
                        _ => unreachable!(),
                    }
                    .into(),
                );
            }
            return true;
        }
        match &mut body[index] {
            NirStmt::While { body, .. } => {
                if mutate(body, mutation) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if mutate(then_body, mutation) || mutate(else_body, mutation) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_snapshot_returns_reject_wrong_record_live_carry_and_early_pending() {
    for variant in cases::snapshot_return_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..4 {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(mutate(&mut shared.body, mutation));
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
fn effect_return_storage_snapshot_returns_match_independent_effects_values_and_carry_reduction() {
    use super::execution_tests::{lowered, maximum_carry_width};
    use super::sibling_bound_tests::run_entry;
    for variant in cases::snapshot_return_variants() {
        let mut calls = (0..=11)
            .map(|mode| {
                let mut call = cases::base(variant);
                call.mode = mode;
                if mode >= 10 {
                    call.trips = 5;
                    call.divisor = 2;
                }
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

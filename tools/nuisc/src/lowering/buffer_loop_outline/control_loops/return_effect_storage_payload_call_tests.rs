use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_payload_calls_require_catalog_and_preserve_snapshot_authority() {
    for variant in cases::payload_call_variants() {
        let source = cases::source(variant).replace("carry", "working_copy");
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
        for callee in ["payload_first", "payload_second", "payload_finish"] {
            let mut denied = catalog.clone();
            assert!(denied.remove(callee).is_some());
            assert!(prepare_effects(function, &layouts, &denied).is_none());
            assert!(verify(
                function,
                &independent,
                &shared,
                "working_copy",
                &layouts,
                &denied
            )
            .is_none());
        }
        for snapshot in ["recent", "frozen"] {
            let invalid = rewrite(
                function,
                &layouts,
                Some(snapshot.into()),
                validation::LiteralPrints::Allow,
            )
            .unwrap();
            assert!(verify(
                function,
                &independent,
                &invalid,
                snapshot,
                &layouts,
                &catalog
            )
            .is_none());
        }
        assert_eq!(module, before);
    }
}

#[test]
fn effect_return_storage_payload_calls_oracle_distinguishes_ordered_failures_and_raw_fields() {
    for variant in cases::payload_call_variants() {
        for mode in [1, 6, 10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            if mode >= 10 {
                call.trips = 5;
                call.divisor = 2;
            }
            let (state, trace) = call.execute(true, None).unwrap();
            let mut bare = call;
            bare.variant.payload_calls = false;
            let (snapshot, before) = bare.execute(true, None).unwrap();
            assert_eq!(state.total, snapshot.total + 10);
            assert_eq!(
                (state.result, state.gain, state.marker),
                (snapshot.result, snapshot.gain, snapshot.marker)
            );
            assert_eq!(&trace[..before.len()], before);
            assert_eq!(&trace[before.len()..], [970, 971, 972, 973]);
        }
        for (_, call) in cases::failures(variant)
            .into_iter()
            .filter(|(_, c)| c.mode >= 12)
        {
            let trace = call.execute(true, None).unwrap_err();
            assert!(trace.contains(&960));
            assert!(!trace.contains(&949));
            let effects = trace
                .iter()
                .copied()
                .filter(|n| (970..=973).contains(n))
                .collect::<Vec<_>>();
            let expected: &[i64] = match call.mode {
                12 | 15 => &[],
                13 | 16 => &[970],
                _ => &[970, 971, 972],
            };
            assert_eq!(effects, expected);
            assert_eq!(
                trace.iter().filter(|&&n| n == 941).count(),
                if call.mode >= 18 { 2 } else { 1 }
            );
            assert_eq!(
                trace.last(),
                Some(&match call.mode {
                    12 | 15 => 960,
                    13 | 16 => 970,
                    _ => 972,
                })
            );
        }
    }
}

fn mutate(body: &mut Vec<NirStmt>, mutation: usize) -> bool {
    for index in 0..body.len() {
        if matches!(&body[index], NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. } if name == "carry" && callee == "payload_finish")
        {
            if mutation == 2 {
                body.insert(index, body[index].clone());
            } else if mutation == 3 {
                assert!(matches!(
                    body[index + 1],
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
                let NirExpr::Call { args, .. } = value else {
                    unreachable!()
                };
                match mutation {
                    0 => args[0] = NirExpr::Var("frozen".into()),
                    1 => args.swap(1, 2),
                    4 => *value = args[0].clone(),
                    5 => args[1] = NirExpr::Int(1),
                    _ => unreachable!(),
                }
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
fn effect_return_storage_payload_calls_reject_reordering_duplication_bypass_and_early_pending() {
    for variant in cases::payload_call_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..6 {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(mutate(&mut shared.body, mutation));
            assert!(
                scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
                    .is_some(),
                "mutation {mutation}"
            );
            assert!(
                verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none(),
                "mutation {mutation}"
            );
        }
    }
}

#[test]
fn effect_return_storage_payload_calls_match_independent_execution_and_carry_width() {
    check_execution(cases::payload_call_variants());
}

pub(super) fn check_execution(variants: Vec<cases::Variant>) {
    use super::execution_tests::{lowered, maximum_carry_width};
    use super::sibling_bound_tests::run_entry;
    for variant in variants {
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
        calls.extend(cases::failures(variant).into_iter().map(|(_, mut call)| {
            call.input = call.previous.take().unwrap();
            call
        }));
        for mode in cases::payload_fault_modes(variant) {
            let mut call = cases::base(variant);
            call.mode = mode;
            call.selected = false;
            calls.push(call);
        }
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
                        .map(|(s, t)| (yir_core::Value::Int(s.total), t))
                        .map_err(|_| ()),
                    "{call:?}"
                );
            }
        }
    }
}

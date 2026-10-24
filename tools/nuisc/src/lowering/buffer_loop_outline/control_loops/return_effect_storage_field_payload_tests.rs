use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_field_payload_requires_each_field_call_grant() {
    check_authority(cases::field_payload_variants());
}

pub(super) fn check_authority(variants: Vec<cases::Variant>) {
    for variant in variants {
        let source = cases::source(variant).replace("carry", "working_copy");
        assert!(!source.contains("fn payload_finish"));
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
        let mut callees = vec![
            "field_marker",
            "field_result",
            "field_gain",
            "field_total",
            "payload_first",
            "payload_second",
        ];
        if variant.mixed_payload {
            callees.push("mixed_bias");
        }
        if variant.projected_payload {
            callees.extend([
                "project_record",
                "project_marker_arg",
                "project_result_arg",
                "project_gain_arg",
                "project_total_arg",
            ]);
        }
        for callee in callees {
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
fn effect_return_storage_field_payload_oracle_keeps_once_only_prefix_and_raw_fields() {
    for variant in cases::field_payload_variants() {
        for mode in [1, 6, 10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            if mode >= 10 {
                call.trips = 5;
                call.divisor = 2;
            }
            let (state, trace) = call.execute(true, None).unwrap();
            let mut bare = call;
            bare.variant.field_payload = false;
            bare.variant.payload_calls = false;
            let (snapshot, before) = bare.execute(true, None).unwrap();
            assert_eq!(state.total, snapshot.total + 10);
            assert_eq!(
                (state.result, state.gain, state.marker),
                (snapshot.result, snapshot.gain, snapshot.marker)
            );
            assert_eq!(&trace[..before.len()], before);
            assert_eq!(&trace[before.len()..], [980, 981, 982, 970, 971, 972, 973]);
        }
        for (_, call) in cases::failures(variant)
            .into_iter()
            .filter(|(_, c)| c.mode >= 12)
        {
            let trace = call.execute(true, None).unwrap_err();
            let effects = trace
                .iter()
                .copied()
                .filter(|n| *n >= 970)
                .collect::<Vec<_>>();
            let expected: &[i64] = match call.mode {
                12 | 15 => &[980, 981, 982],
                13 | 16 => &[980, 981, 982, 970],
                _ => &[980, 981, 982, 970, 971, 972],
            };
            assert_eq!(effects, expected);
            assert!(!trace.contains(&949));
            assert_eq!(
                trace.iter().filter(|&&n| n == 941).count(),
                if call.mode >= 18 { 2 } else { 1 }
            );
        }
    }
}

fn mutate(body: &mut Vec<NirStmt>, mutation: usize) -> bool {
    for index in 0..body.len() {
        if matches!(&body[index], NirStmt::Let { name, value: NirExpr::StructLiteral { .. }, .. } if name == "carry")
        {
            if mutation == 1 {
                body.insert(index, body[index].clone());
            } else if mutation == 2 {
                assert!(matches!(
                    body[index + 1],
                    NirStmt::Let {
                        value: NirExpr::Int(1),
                        ..
                    }
                ));
                body.swap(index, index + 1);
            } else {
                let NirStmt::Let {
                    value: NirExpr::StructLiteral { fields, .. },
                    ..
                } = &mut body[index]
                else {
                    unreachable!()
                };
                assert_eq!(
                    fields
                        .iter()
                        .map(|(name, _)| name.as_str())
                        .collect::<Vec<_>>(),
                    ["marker", "result", "gain", "total"]
                );
                match mutation {
                    // Reordering whole field pairs remains type-correct but changes effects.
                    0 => fields.swap(0, 3),
                    3 => {
                        let NirExpr::Call { args, .. } = &mut fields[3].1 else {
                            unreachable!()
                        };
                        args.swap(1, 2);
                    }
                    4 => {
                        let NirExpr::Call { args, .. } = &fields[3].1 else {
                            unreachable!()
                        };
                        fields[3].1 = args[0].clone();
                    }
                    5 => {
                        let NirExpr::Call { args, .. } = &mut fields[0].1 else {
                            unreachable!()
                        };
                        let NirExpr::FieldAccess { base, .. } = &mut args[0] else {
                            unreachable!()
                        };
                        let wrong = if matches!(base.as_ref(), NirExpr::Var(name) if name == "frozen")
                        {
                            "recent"
                        } else {
                            "frozen"
                        };
                        *base = Box::new(NirExpr::Var(wrong.into()));
                    }
                    6 => {
                        let NirExpr::Call { args, .. } = &fields[0].1 else {
                            unreachable!()
                        };
                        fields[0].1 = args[0].clone();
                    }
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
fn effect_return_storage_field_payload_rejects_reordering_duplication_and_early_publication() {
    check_certificate(cases::field_payload_variants());
}

pub(super) fn check_certificate(variants: Vec<cases::Variant>) {
    for variant in variants {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..7 {
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
fn effect_return_storage_field_payload_matches_independent_execution_and_carry_width() {
    super::payload_call_tests::check_execution(cases::field_payload_variants());
}

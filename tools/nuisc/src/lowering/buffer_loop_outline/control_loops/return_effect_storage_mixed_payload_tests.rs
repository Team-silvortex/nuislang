use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_mixed_payload_keeps_catalog_and_snapshot_authority() {
    super::field_payload_tests::check_authority(cases::mixed_payload_variants());
}

#[test]
fn effect_return_storage_mixed_payload_oracle_distinguishes_epochs_and_original_self_reads() {
    for variant in cases::mixed_payload_variants() {
        for mode in [1, 6, 10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            if mode >= 10 {
                call.trips = 5;
                call.divisor = 2;
            }
            let (state, trace) = call.execute(true, None).unwrap();
            // Hand-derived totals include live work, 100 * older, 10 * newer,
            // and the original live bool's bias, not the returned bool's bias.
            let expected = match (variant.leading, mode) {
                (false, 1) => 2431,
                (true, 1) => 2432,
                (false, 6) => 2442,
                (true, 6) => 2445,
                (false, 10) => 2584,
                (true, 10) => 2558,
                (false, 11) => 2595,
                (true, 11) => 1569,
                _ => unreachable!(),
            };
            assert_eq!(state.total, expected, "{variant:?}, {mode}");
            assert_eq!(state.result, !(variant.leading && mode == 11));
            let first_child = matches!(mode, 1 | 10);
            assert_eq!(state.gain, if first_child { 0 } else { 0x80000000 });
            assert_eq!(
                state.marker,
                if first_child { 0x8000000000000000 } else { 0 }
            );
            let effects = trace
                .iter()
                .copied()
                .filter(|n| *n >= 970)
                .collect::<Vec<_>>();
            assert_eq!(effects, [980, 981, 982, 984, 970, 971, 972, 973]);
            assert!(!trace.contains(&949));
            assert_eq!(
                trace.iter().filter(|&&n| n == 941).count(),
                if mode >= 10 { 2 } else { 1 }
            );
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
                12 | 15 => &[980, 981, 982, 984],
                13 | 16 => &[980, 981, 982, 984, 970],
                _ => &[980, 981, 982, 984, 970, 971, 972],
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

fn mutate_read(value: &mut NirExpr, mutation: usize) -> bool {
    match value {
        NirExpr::FieldAccess { base, field } if field == "total" => {
            let target = if mutation == 0 { "carry" } else { "frozen" };
            if mutation != 1 && matches!(base.as_ref(), NirExpr::Var(name) if name == target) {
                *base = Box::new(NirExpr::Var("recent".into()));
                return true;
            }
        }
        NirExpr::Call { callee, args } if mutation == 1 && callee == "mixed_bias" => {
            args[0] = NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(args[0].clone()),
                rhs: Box::new(NirExpr::Bool(true)),
            };
            return true;
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            return mutate_read(lhs, mutation) || mutate_read(rhs, mutation)
        }
        NirExpr::Call { args, .. } => return args.iter_mut().any(|arg| mutate_read(arg, mutation)),
        _ => {}
    }
    false
}

fn mutate_body(body: &mut [NirStmt], mutation: usize) -> bool {
    for stmt in body {
        match stmt {
            NirStmt::Let {
                name,
                value: NirExpr::StructLiteral { fields, .. },
                ..
            } if name == "carry" => {
                let (_, total) = fields.iter_mut().find(|(name, _)| name == "total").unwrap();
                return mutate_read(total, mutation);
            }
            NirStmt::While { body, .. } => {
                if mutate_body(body, mutation) {
                    return true;
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if mutate_body(then_body, mutation) || mutate_body(else_body, mutation) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn effect_return_storage_mixed_payload_rejects_wrong_origins_and_forwarded_return_fields() {
    super::field_payload_tests::check_certificate(cases::mixed_payload_variants());
    for variant in cases::mixed_payload_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..3 {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(mutate_body(&mut shared.body, mutation));
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
fn effect_return_storage_mixed_payload_matches_independent_execution_and_carry_width() {
    super::payload_call_tests::check_execution(cases::mixed_payload_variants());
}

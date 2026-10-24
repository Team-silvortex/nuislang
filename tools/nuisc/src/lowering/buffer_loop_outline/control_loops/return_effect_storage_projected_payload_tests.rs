use super::sibling_bound_tests::cases;
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_projected_payload_requires_record_and_argument_call_authority() {
    super::field_payload_tests::check_authority(cases::projected_payload_variants());
    let (module, layouts, _) = plans(
        &cases::source(cases::projected_payload_variants()[0]),
        "work",
    );
    let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
    assert!(!pure.contains_key("project_record"));
    assert!(!pure.contains_key("work"));
}

const FIELDS: &[i64] = &[
    985, 986, 987, 980, 988, 986, 987, 981, 989, 986, 987, 982, 990, 986, 987, 984,
];

#[test]
fn effect_return_storage_projected_payload_oracle_distinguishes_each_call_and_fault_stage() {
    for variant in cases::projected_payload_variants() {
        for mode in [1, 6, 10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            if mode >= 10 {
                call.trips = 5;
                call.divisor = 2;
            }
            let (state, trace) = call.execute(true, None).unwrap();
            let expected = match (variant.leading, mode) {
                (false, 1) => 2435,
                (true, 1) => 2436,
                (false, 6) => 2446,
                (true, 6) => 2449,
                (false, 10) => 2588,
                (true, 10) => 2562,
                (false, 11) => 2599,
                (true, 11) => 1573,
                _ => unreachable!(),
            };
            assert_eq!(state.total, expected, "{variant:?}, {mode}");
            assert_eq!(state.result, !(variant.leading && mode == 11));
            let first = matches!(mode, 1 | 10);
            assert_eq!(state.gain, if first { 0x80000000 } else { 0 });
            assert_eq!(state.marker, if first { 0 } else { 0x8000000000000000 });
            let effects = trace
                .iter()
                .copied()
                .filter(|n| *n >= 970)
                .collect::<Vec<_>>();
            let mut expected = FIELDS.to_vec();
            expected.extend([970, 971, 972, 973]);
            assert_eq!(effects, expected);
            assert_eq!(effects.iter().filter(|&&n| n == 986).count(), 4);
            assert!(!trace.contains(&949));
        }
        assert_eq!(cases::failures(variant).len(), 23);
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
            let mut expected = if call.mode >= 20 {
                FIELDS[..8].to_vec()
            } else {
                FIELDS.to_vec()
            };
            match call.mode {
                12 | 15 | 20 | 22 | 24 | 26 => {}
                13 | 16 => expected.push(970),
                21 | 23 | 25 | 27 => expected.extend([989, 986]),
                _ => expected.extend([970, 971, 972]),
            }
            assert_eq!(effects, expected, "{variant:?}, {}", call.mode);
            assert!(!trace.contains(&949));
            assert_eq!(
                trace.iter().filter(|&&n| n == 941).count(),
                if matches!(call.mode, 18 | 19) || call.mode >= 24 {
                    2
                } else {
                    1
                }
            );
        }
    }
}

fn mutate_projection(value: &mut NirExpr, mutation: usize) -> bool {
    match value {
        NirExpr::FieldAccess { base, field }
            if matches!(base.as_ref(), NirExpr::Call { callee, .. } if callee == "project_record")
                && (mutation < 5 || field == "gain") =>
        {
            let duplicate = base.as_ref().clone();
            let NirExpr::Call { args, .. } = base.as_mut() else {
                unreachable!()
            };
            match mutation {
                0 => args[0] = NirExpr::Var("recent".into()),
                1 => {
                    let NirExpr::Call { callee, .. } = &mut args[1] else {
                        unreachable!()
                    };
                    *callee = "project_result_arg".into();
                }
                2 => args[0] = duplicate,
                3 => *base = Box::new(args[0].clone()),
                4 => args[1] = NirExpr::Int(1),
                5 => args[2] = NirExpr::Int(1),
                6 => args[3] = NirExpr::Int(1),
                7 => args[4] = NirExpr::Int(1),
                _ => unreachable!(),
            }
            true
        }
        NirExpr::FieldAccess { base, .. } => mutate_projection(base, mutation),
        NirExpr::Call { args, .. } => args.iter_mut().any(|arg| mutate_projection(arg, mutation)),
        NirExpr::Binary { lhs, rhs, .. } => {
            mutate_projection(lhs, mutation) || mutate_projection(rhs, mutation)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter_mut()
            .any(|(_, value)| mutate_projection(value, mutation)),
        _ => false,
    }
}

pub(super) fn mutate_body(body: &mut [NirStmt], mutation: usize) -> bool {
    for stmt in body {
        match stmt {
            NirStmt::Let {
                name,
                value: value @ NirExpr::StructLiteral { .. },
                ..
            } if name == "carry" => {
                return mutate_projection(value, mutation);
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
fn effect_return_storage_projected_payload_rejects_duplication_bypass_and_removed_checks() {
    check_certificate(cases::projected_payload_variants());
}

pub(super) fn check_certificate(variants: Vec<cases::Variant>) {
    super::field_payload_tests::check_certificate(variants.clone());
    for variant in variants {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in 0..6 {
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
fn effect_return_storage_projected_payload_matches_independent_execution_and_carry_width() {
    super::payload_call_tests::check_execution(cases::projected_payload_variants());
}

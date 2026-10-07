use super::*;

// This is return-value authority, separate from legacy print captures and
// bool-only liveness. New typed results enter only through selected regions.
pub(super) fn admitted(ty: &NirTypeRef) -> bool {
    ty == &scalar_type(&ty.name)
        && matches!(ty.name.as_str(), "bool" | "i64" | "i32" | "f32" | "f64")
}

pub(super) fn capture(result: &NirTypeRef, ty: &NirTypeRef) -> bool {
    admitted(result)
        && if scalar(result) {
            scalar(ty)
        } else {
            admitted(ty)
        }
}

pub(super) fn local(result: &NirTypeRef, ty: &NirTypeRef) -> bool {
    admitted(result) && (scalar(result) || admitted(ty))
}

pub(super) fn seed(result: &NirTypeRef) -> NirExpr {
    assert!(admitted(result));
    control_values::zero_value(result, &control_values::TypedLayouts::default())
}

#[test]
fn conditional_return_typed_handoff_keeps_exact_owned_profile_and_legacy_capture_limits() {
    for name in ["bool", "i64", "i32", "f32", "f64"] {
        let result = scalar_type(name);
        assert!(admitted(&result));
        assert_eq!(
            control_values::value_type(
                &seed(&result),
                &Scope::new(),
                &ScalarHelpers::new(),
                &control_values::TypedLayouts::default()
            ),
            Some(result.clone())
        );
        for input in ["bool", "i64", "i32", "f32", "f64"] {
            assert_eq!(
                capture(&result, &scalar_type(input)),
                !scalar(&result) || matches!(input, "bool" | "i64")
            );
        }
        for mutation in ["reference", "optional", "generic"] {
            let mut changed = result.clone();
            match mutation {
                "reference" => changed.is_ref = true,
                "optional" => changed.is_optional = true,
                "generic" => changed.generic_args.push(scalar_type("i64")),
                _ => unreachable!(),
            }
            assert!(!admitted(&changed));
            assert!(!capture(&result, &changed));
            assert!(!capture(&changed, &result));
            assert!(!local(&changed, &result));
        }
        for excluded in ["Buffer", "Bytes", "Node", "Packet", "u64"] {
            let excluded = scalar_type(excluded);
            assert!(!admitted(&excluded));
            assert!(!capture(&result, &excluded));
            assert_eq!(local(&result, &excluded), scalar(&result));
        }
    }
}

#[test]
fn conditional_return_typed_handoff_keeps_real_zero_exits_distinct_from_continuation_seeds() {
    for name in ["i32", "f32", "f64"] {
        let result = scalar_type(name);
        let signal = scalar_type("Exit");
        let inactive = signal::wrap(vec![], &signal, &result, &mut BTreeSet::new());
        let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) = &inactive[0] else {
            panic!()
        };
        assert_eq!(
            fields,
            &vec![
                ("exited".into(), NirExpr::Bool(false)),
                ("value".into(), seed(&result))
            ]
        );
        let active = signal::wrap(
            vec![NirStmt::Return(Some(seed(&result)))],
            &signal,
            &result,
            &mut BTreeSet::new(),
        );
        let NirStmt::Let { name, ty, value } = &active[0] else {
            panic!()
        };
        assert_eq!(ty.as_ref(), Some(&result));
        assert_eq!(value, &seed(&result));
        let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) = &active[1] else {
            panic!()
        };
        assert_eq!(
            fields,
            &vec![
                ("exited".into(), NirExpr::Bool(true)),
                ("value".into(), NirExpr::Var(name.clone()))
            ]
        );
    }
}

use super::*;

// Data snapshots have a separate profile from bool path masks and the legacy
// bool/i64 return-tail profile. References and aggregate/resource kinds stay out.
pub(super) fn admitted(ty: &NirTypeRef) -> bool {
    ty == &scalar_type(&ty.name)
        && matches!(ty.name.as_str(), "bool" | "i64" | "i32" | "f32" | "f64")
}

pub(super) fn seed(ty: &NirTypeRef) -> NirExpr {
    assert!(admitted(ty));
    control_values::zero_value(ty, &control_values::TypedLayouts::default())
}

#[test]
fn conditional_return_typed_snapshots_keep_exact_owned_data_profile_and_seed_kinds() {
    let scalars = &control_values::TypedLayouts::default();
    for name in ["bool", "i64", "i32", "f32", "f64"] {
        let ty = scalar_type(name);
        assert!(admitted(&ty));
        assert_eq!(
            control_values::value_type(&seed(&ty), &Scope::new(), &ScalarHelpers::new(), scalars),
            Some(ty.clone())
        );
        let mut borrowed = ty;
        borrowed.is_ref = true;
        assert!(!admitted(&borrowed));
    }
    for name in ["Buffer", "Bytes", "Node", "Packet", "u64"] {
        assert!(!admitted(&scalar_type(name)));
    }
}

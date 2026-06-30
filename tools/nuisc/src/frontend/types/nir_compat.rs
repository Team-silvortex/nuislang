use super::*;

pub(crate) fn compatible_types(expected: &NirTypeRef, actual: &NirTypeRef) -> bool {
    if expected.window_mode() == Some(NirWindowMode::Immutable)
        && actual.window_mode() == Some(NirWindowMode::Mutable)
        && expected.is_optional == actual.is_optional
        && expected.is_ref == actual.is_ref
        && expected.generic_args.len() == actual.generic_args.len()
    {
        return expected
            .generic_args
            .iter()
            .zip(&actual.generic_args)
            .all(|(lhs, rhs)| compatible_types(lhs, rhs));
    }
    if expected.name == actual.name
        && !expected.is_ref
        && !actual.is_ref
        && !expected.is_optional
        && !actual.is_optional
        && matches!(expected.name.as_str(), "Marker" | "HandleTable")
    {
        return expected.generic_args.is_empty()
            || actual.generic_args.is_empty()
            || (expected.generic_args.len() == actual.generic_args.len()
                && expected
                    .generic_args
                    .iter()
                    .zip(&actual.generic_args)
                    .all(|(lhs, rhs)| compatible_types(lhs, rhs)));
    }
    if expected.name != actual.name
        || expected.is_ref != actual.is_ref
        || expected.is_optional != actual.is_optional
        || expected.generic_args.len() != actual.generic_args.len()
    {
        if enum_variant_matches_parent(expected, actual) {
            return true;
        }
        return expected.is_ref && actual.is_ref && expected.generic_args.is_empty();
    }
    expected
        .generic_args
        .iter()
        .zip(&actual.generic_args)
        .all(|(lhs, rhs)| compatible_types(lhs, rhs))
}

pub(super) fn enum_variant_matches_parent(expected: &NirTypeRef, actual: &NirTypeRef) -> bool {
    if expected.is_ref != actual.is_ref || expected.is_optional != actual.is_optional {
        return false;
    }
    let Some((parent, _variant)) = actual.name.rsplit_once('.') else {
        return false;
    };
    expected.name == parent
        && expected.generic_args.len() == actual.generic_args.len()
        && expected
            .generic_args
            .iter()
            .zip(&actual.generic_args)
            .all(|(lhs, rhs)| compatible_types(lhs, rhs))
}

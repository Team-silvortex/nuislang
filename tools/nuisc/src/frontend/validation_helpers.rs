use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{NirFunction, NirModule, NirStructDef, NirTypeRef};

use super::named_type;

pub(super) fn async_boundary_violation_detail(
    ty: &NirTypeRef,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<String> {
    let mut visiting = BTreeSet::new();
    async_boundary_violation_detail_inner(ty, struct_table, &mut visiting)
}

pub(super) fn async_parameter_violation_detail(
    ty: &NirTypeRef,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<String> {
    let mut visiting = BTreeSet::new();
    async_parameter_violation_detail_inner(ty, struct_table, &mut visiting)
}

fn async_parameter_violation_detail_inner(
    ty: &NirTypeRef,
    struct_table: &BTreeMap<String, NirStructDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    if let Some(result_family) = ty.result_family() {
        match result_family {
            nuis_semantics::model::NirResultFamily::Shader
            | nuis_semantics::model::NirResultFamily::Kernel
            | nuis_semantics::model::NirResultFamily::Network => {
                let payload = ty.result_payload().expect("result payload");
                if !payload.is_async_boundary_safe() {
                    return Some(format!(
                        "directly carries async-unsafe `{}` payload through `{}`",
                        payload.render(),
                        ty.render()
                    ));
                }
                if is_async_resource_bearing_payload(payload) {
                    return Some(format!(
                        "directly carries resource-bearing `{}` payload through `{}`",
                        payload.render(),
                        ty.render()
                    ));
                }
            }
            _ => return Some(format!("directly carries `{}`", ty.render())),
        }
    } else {
        if !ty.is_async_boundary_safe() {
            return Some(format!("directly carries `{}`", ty.render()));
        }
        if is_async_resource_bearing_payload(ty) {
            return Some(format!(
                "directly carries resource-bearing `{}`",
                ty.render()
            ));
        }
    }
    let Some(definition) = struct_table.get(&ty.name) else {
        return None;
    };
    let visit_key = ty.render();
    if !visiting.insert(visit_key.clone()) {
        return None;
    }
    for field in &definition.fields {
        if let Some(nested) =
            async_parameter_violation_detail_inner(&field.ty, struct_table, visiting)
        {
            visiting.remove(&visit_key);
            return Some(format!(
                "nested field `{}.{}` {}",
                definition.name, field.name, nested
            ));
        }
    }
    visiting.remove(&visit_key);
    None
}

fn async_boundary_violation_detail_inner(
    ty: &NirTypeRef,
    struct_table: &BTreeMap<String, NirStructDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    if !ty.is_async_boundary_safe() {
        return Some(format!("directly carries `{}`", ty.render()));
    }
    if is_async_resource_bearing_payload(ty) {
        return Some(format!(
            "directly carries resource-bearing `{}`",
            ty.render()
        ));
    }
    let Some(definition) = struct_table.get(&ty.name) else {
        return None;
    };
    let visit_key = ty.render();
    if !visiting.insert(visit_key.clone()) {
        return None;
    }
    for field in &definition.fields {
        if let Some(nested) =
            async_boundary_violation_detail_inner(&field.ty, struct_table, visiting)
        {
            visiting.remove(&visit_key);
            return Some(format!(
                "nested field `{}.{}` {}",
                definition.name, field.name, nested
            ));
        }
    }
    visiting.remove(&visit_key);
    None
}

fn is_async_resource_bearing_payload(ty: &NirTypeRef) -> bool {
    matches!(
        ty.container_kind(),
        Some(
            nuis_semantics::model::NirContainerKind::Window
                | nuis_semantics::model::NirContainerKind::Pipe
        )
    ) || ty.is_marker_family()
        || ty.is_handle_table_family()
}

pub(super) fn validate_test_function_signature(
    module: &NirModule,
    function: &NirFunction,
) -> Result<(), String> {
    let label = function.test_name.as_deref().unwrap_or(&function.name);
    if module.domain != "cpu" {
        return Err(format!(
            "test function `{}::{}` ({}) is only supported in `mod cpu` for now",
            module.unit, function.name, label
        ));
    }
    if !function.params.is_empty() {
        return Err(format!(
            "test function `{}` ({}) cannot take parameters in the current front-door runner shape",
            function.name, label
        ));
    }
    let Some(return_type) = function.return_type.as_ref() else {
        return Err(format!(
            "test function `{}` ({}) must return `bool` or `i64` in the current MVP",
            function.name, label
        ));
    };
    if !(return_type.is_bool_scalar() || return_type.is_integer_scalar()) {
        return Err(format!(
            "test function `{}` ({}) must return `bool` or integer scalar, found `{}`",
            function.name,
            label,
            return_type.render()
        ));
    }
    if function.test_ignored && function.test_should_fail {
        return Err(format!(
            "test function `{}` ({}) cannot be both `ignored` and `should_fail` in the current MVP",
            function.name, label
        ));
    }
    if function.test_reason.is_some() && !function.test_should_fail {
        return Err(format!(
            "test function `{}` ({}) can only use `reason=\"...\"` together with `should_fail=true` in the current MVP",
            function.name, label
        ));
    }
    if let Some(timeout_ms) = function.test_timeout_ms {
        if timeout_ms <= 0 {
            return Err(format!(
                "test function `{}` ({}) must use `timeout_ms` > 0, found `{}`",
                function.name, label, timeout_ms
            ));
        }
    }
    if let Some(clock_domain) = function.test_clock_domain {
        if function.test_timeout_ms.is_none() {
            return Err(format!(
                "test function `{}` ({}) can only use `clock_domain=\"...\"` together with `timeout_ms=...` in the current MVP",
                function.name, label
            ));
        }
        if clock_domain == nuis_semantics::model::TestClockDomain::Wall && function.is_async {
            return Err(format!(
                "test function `{}` ({}) cannot use `clock_domain=\"wall\"` on `async fn` in the current MVP; use `monotonic` or `global`",
                function.name, label
            ));
        }
        if !matches!(
            clock_domain,
            nuis_semantics::model::TestClockDomain::Monotonic
                | nuis_semantics::model::TestClockDomain::Wall
                | nuis_semantics::model::TestClockDomain::Global
        ) {
            return Err(format!(
                "test function `{}` ({}) uses unsupported `clock_domain=\"{}\"`; expected `monotonic`, `wall`, or `global`",
                function.name,
                label,
                clock_domain.as_str()
            ));
        }
    }
    if let Some(clock_policy) = function.test_clock_policy {
        if function.test_timeout_ms.is_none() {
            return Err(format!(
                "test function `{}` ({}) can only use `clock_policy=\"...\"` together with `timeout_ms=...` in the current MVP",
                function.name, label
            ));
        }
        if function.test_clock_domain != Some(nuis_semantics::model::TestClockDomain::Global) {
            return Err(format!(
                "test function `{}` ({}) can only use `clock_policy=\"{}\"` together with `clock_domain=\"global\"` in the current MVP",
                function.name,
                label,
                clock_policy.as_str()
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_type_ref(ty: &NirTypeRef) -> Result<(), String> {
    ty.validate_container_contract()
        .map_err(|error| format!("invalid type `{}`: {error}", ty.render()))
}

pub(super) fn select_expected_semantic_token_type(
    expected: Option<&NirTypeRef>,
    token_name: &str,
) -> NirTypeRef {
    match expected {
        Some(expected)
            if expected.name == token_name
                && !expected.is_ref
                && !expected.is_optional
                && expected.generic_args.len() <= 1 =>
        {
            expected.clone()
        }
        _ => named_type(token_name),
    }
}

pub(super) fn render_type_name(ty: &NirTypeRef) -> String {
    ty.render()
}

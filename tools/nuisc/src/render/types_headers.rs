use super::*;

pub(super) fn render_ast_type(ty: &nuis_semantics::model::AstTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_ast_type)
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('>');
    }
    if ty.is_optional {
        out.push('?');
    }
    out
}

pub(super) fn render_ast_generic_params(params: &[AstGenericParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let parts = params
        .iter()
        .map(render_ast_generic_param)
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", parts)
}

pub(super) fn render_nir_type(ty: &nuis_semantics::model::NirTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_nir_type)
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('>');
    }
    if ty.is_optional {
        out.push('?');
    }
    out
}

pub(super) fn render_nir_generic_params(params: &[NirGenericParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let parts = params
        .iter()
        .map(render_nir_generic_param)
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", parts)
}

pub(super) fn render_ast_generic_param(param: &AstGenericParam) -> String {
    if param.bounds.is_empty() {
        return param.name.clone();
    }
    format!(
        "{}: {}",
        param.name,
        param
            .bounds
            .iter()
            .map(render_ast_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

pub(super) fn render_nir_generic_param(param: &NirGenericParam) -> String {
    if param.bounds.is_empty() {
        return param.name.clone();
    }
    format!(
        "{}: {}",
        param.name,
        param
            .bounds
            .iter()
            .map(render_nir_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

pub(super) fn render_ast_function_header(function: &AstFunction) -> String {
    let mut out = render_ast_doc_comments("  ", &function.attributes);
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = function
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type(ty)))
        .unwrap_or_default();
    let test_prefix = function
        .test_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if function.test_ignored {
                parts.push("ignored=true".to_owned());
            }
            if function.test_should_fail {
                parts.push("should_fail=true".to_owned());
            }
            if let Some(reason) = &function.test_reason {
                parts.push(format!("reason=\"{}\"", reason));
            }
            if let Some(timeout_ms) = function.test_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.test_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.test_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("test({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let benchmark_prefix = function
        .benchmark_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if let Some(warmup_iters) = function.benchmark_warmup_iters {
                parts.push(format!("warmup_iters={warmup_iters}"));
            }
            if let Some(measure_iters) = function.benchmark_measure_iters {
                parts.push(format!("measure_iters={measure_iters}"));
            }
            if let Some(timeout_ms) = function.benchmark_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.benchmark_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.benchmark_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("benchmark({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    let attribute_prefix = render_ast_attributes(&function.attributes);
    let visibility_prefix = render_ast_visibility(function.visibility);
    let where_suffix = render_ast_where_clause(&function.where_bounds);
    out.push_str(&format!(
        "  {}{}{}{}{}fn {}{}({}){}{}\n",
        attribute_prefix,
        visibility_prefix,
        test_prefix,
        benchmark_prefix,
        async_prefix,
        function.name,
        render_ast_generic_params(&function.generic_params),
        params,
        return_suffix,
        where_suffix
    ));
    out
}

pub(super) fn render_nir_function_header(function: &NirFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = function
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_nir_type(ty)))
        .unwrap_or_default();
    let test_prefix = function
        .test_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if function.test_ignored {
                parts.push("ignored=true".to_owned());
            }
            if function.test_should_fail {
                parts.push("should_fail=true".to_owned());
            }
            if let Some(reason) = &function.test_reason {
                parts.push(format!("reason=\"{}\"", reason));
            }
            if let Some(timeout_ms) = function.test_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.test_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.test_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("test({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let benchmark_prefix = function
        .benchmark_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if let Some(warmup_iters) = function.benchmark_warmup_iters {
                parts.push(format!("warmup_iters={warmup_iters}"));
            }
            if let Some(measure_iters) = function.benchmark_measure_iters {
                parts.push(format!("measure_iters={measure_iters}"));
            }
            if let Some(timeout_ms) = function.benchmark_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.benchmark_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.benchmark_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("benchmark({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    let annotation_prefix = render_nir_annotations(&function.annotations);
    let visibility_prefix = render_nir_visibility(function.visibility);
    let where_suffix = render_nir_where_clause(&function.where_bounds);
    format!(
        "  {}{}{}{}{}fn {}{}({}){}{}\n",
        annotation_prefix,
        visibility_prefix,
        test_prefix,
        benchmark_prefix,
        async_prefix,
        function.name,
        render_nir_generic_params(&function.generic_params),
        params,
        return_suffix,
        where_suffix
    )
}

pub(super) fn render_ast_where_clause(predicates: &[AstWherePredicate]) -> String {
    if predicates.is_empty() {
        return String::new();
    }
    format!(
        " where {}",
        predicates
            .iter()
            .map(render_ast_where_predicate)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(super) fn render_nir_where_clause(predicates: &[NirWherePredicate]) -> String {
    if predicates.is_empty() {
        return String::new();
    }
    format!(
        " where {}",
        predicates
            .iter()
            .map(render_nir_where_predicate)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(super) fn render_ast_where_predicate(predicate: &AstWherePredicate) -> String {
    format!(
        "{}: {}",
        predicate.param_name,
        predicate
            .bounds
            .iter()
            .map(render_ast_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

pub(super) fn render_nir_where_predicate(predicate: &NirWherePredicate) -> String {
    format!(
        "{}: {}",
        predicate.param_name,
        predicate
            .bounds
            .iter()
            .map(render_nir_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

pub(super) fn render_ast_visibility(visibility: AstVisibility) -> &'static str {
    match visibility {
        AstVisibility::Private => "",
        AstVisibility::Public => "pub ",
    }
}

pub(super) fn render_nir_visibility(visibility: NirVisibility) -> &'static str {
    match visibility {
        NirVisibility::Private => "",
        NirVisibility::Public => "pub ",
    }
}

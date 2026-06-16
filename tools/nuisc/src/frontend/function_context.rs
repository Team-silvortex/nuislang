pub(super) fn lambda_owner_name(function_name: &str) -> Option<&str> {
    let remainder = function_name.strip_prefix("__lambda_")?;
    let unspecialized = remainder
        .split_once("__")
        .map(|(base, _)| base)
        .unwrap_or(remainder);
    let (owner, counter) = unspecialized.rsplit_once('_')?;
    counter
        .chars()
        .all(|ch| ch.is_ascii_digit())
        .then_some(owner)
}

pub(super) fn higher_order_template_name(function_name: &str) -> Option<&str> {
    let remainder = function_name.strip_prefix("__hof_")?;
    remainder
        .split_once("___lambda_")
        .map(|(template, _)| template)
}

pub(super) fn render_function_context(
    function_name: &str,
    lambda_context_suffix: &str,
    plain_function_suffix: &str,
) -> String {
    if let Some(owner) = lambda_owner_name(function_name) {
        format!("function `{owner}` {lambda_context_suffix}")
    } else if let Some(template) = higher_order_template_name(function_name) {
        format!("function `{template}` body higher-order specialization")
    } else if plain_function_suffix.is_empty() {
        format!("function `{function_name}`")
    } else {
        format!("function `{function_name}` {plain_function_suffix}")
    }
}

pub(super) fn render_function_body_context(function_name: &str) -> String {
    render_function_context(function_name, "body lambda body", "body")
}

pub(super) fn render_function_validation_context(function_name: &str) -> String {
    render_function_context(function_name, "body lambda", "")
}

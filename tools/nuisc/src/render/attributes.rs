use super::*;

pub(super) fn render_ast_attributes(attributes: &[AstAttribute]) -> String {
    let rendered = attributes
        .iter()
        .filter(|attribute| !is_doc_attribute(attribute))
        .map(render_ast_attribute)
        .collect::<Vec<_>>();
    if rendered.is_empty() {
        return String::new();
    }
    format!("{} ", rendered.join(" "))
}

pub(super) fn render_ast_doc_comments(indent: &str, attributes: &[AstAttribute]) -> String {
    let mut out = String::new();
    for attribute in attributes {
        if !is_doc_attribute(attribute) {
            continue;
        }
        let Some(AstAttributeArg {
            name: None,
            value: AstAttributeValue::String(value),
        }) = attribute.args.first()
        else {
            continue;
        };
        out.push_str(&format!("{indent}/// {value}\n"));
    }
    out
}

pub(super) fn is_doc_attribute(attribute: &AstAttribute) -> bool {
    attribute.name == "doc"
        && attribute.args.len() == 1
        && matches!(
            attribute.args.first(),
            Some(AstAttributeArg {
                name: None,
                value: AstAttributeValue::String(_),
            })
        )
}

pub(super) fn render_ast_attribute(attribute: &AstAttribute) -> String {
    if attribute.args.is_empty() {
        return format!("@{}", attribute.name);
    }
    format!(
        "@{}({})",
        attribute.name,
        attribute
            .args
            .iter()
            .map(render_ast_attribute_arg)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(super) fn render_ast_attribute_arg(arg: &AstAttributeArg) -> String {
    let value = match &arg.value {
        AstAttributeValue::Bool(value) => value.to_string(),
        AstAttributeValue::Int(value) => value.to_string(),
        AstAttributeValue::String(value) => format!("\"{}\"", escape_debug(value)),
        AstAttributeValue::Ident(value) => value.clone(),
    };
    match &arg.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

pub(super) fn render_nir_annotations(annotations: &[NirAnnotation]) -> String {
    if annotations.is_empty() {
        return String::new();
    }
    format!(
        "{} ",
        annotations
            .iter()
            .map(render_nir_annotation)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

pub(super) fn render_nir_annotation(annotation: &NirAnnotation) -> String {
    if annotation.args.is_empty() {
        return format!("@{}", annotation.name);
    }
    format!(
        "@{}({})",
        annotation.name,
        annotation
            .args
            .iter()
            .map(render_nir_annotation_arg)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(super) fn render_nir_annotation_arg(arg: &NirAttributeArg) -> String {
    let value = match &arg.value {
        NirAttributeValue::Bool(value) => value.to_string(),
        NirAttributeValue::Int(value) => value.to_string(),
        NirAttributeValue::String(value) => format!("\"{}\"", escape_debug(value)),
        NirAttributeValue::Ident(value) => value.clone(),
    };
    match &arg.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

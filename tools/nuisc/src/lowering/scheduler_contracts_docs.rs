use super::{push_scheduler_contract_text_node, YirModule};
use nuis_semantics::model::{NirAnnotation, NirAttributeValue, NirFunction, NirModule};

pub(crate) fn materialize_doc_contract_nodes(yir: &mut YirModule, module: &NirModule) {
    let cpu_resource = yir
        .resources
        .iter()
        .find(|resource| resource.kind.family() == "cpu")
        .map(|resource| resource.name.clone())
        .unwrap_or_else(|| "cpu0".to_owned());
    let module_path = format!("{}.{}", module.domain, module.unit);
    let module_docs = doc_lines_from_annotations(&module.annotations);
    if !module_docs.is_empty() {
        push_doc_contract_text_node(
            yir,
            &format!(
                "doc_contract_module_{}",
                sanitize_doc_contract_name(&module_path)
            ),
            &cpu_resource,
            render_doc_contract("module", &module_path, None, &module_docs),
        );
    }
    for function in &module.functions {
        let docs = doc_lines_from_annotations(&function.annotations);
        if docs.is_empty() {
            continue;
        }
        let path = format!("{module_path}::{}", function.name);
        push_doc_contract_text_node(
            yir,
            &format!(
                "doc_contract_function_{}",
                sanitize_doc_contract_name(&path)
            ),
            &cpu_resource,
            render_doc_contract(
                "function",
                &path,
                Some(render_function_doc_signature(function)),
                &docs,
            ),
        );
    }
}

fn doc_lines_from_annotations(annotations: &[NirAnnotation]) -> Vec<String> {
    annotations
        .iter()
        .filter(|annotation| annotation.name == "doc")
        .filter_map(|annotation| match annotation.args.first() {
            Some(arg) if arg.name.is_none() => match &arg.value {
                NirAttributeValue::String(value) => Some(value.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn render_doc_contract(
    scope: &str,
    path: &str,
    signature: Option<String>,
    docs: &[String],
) -> String {
    let mut fields = vec![
        "schema=nuis-yir-doc-contract-v1".to_owned(),
        format!("scope={scope}"),
        format!("path={}", escape_doc_contract_value(path)),
        format!("line_count={}", docs.len()),
        format!("docs={}", escape_doc_contract_value(&docs.join("\\n"))),
    ];
    if let Some(signature) = signature {
        fields.push(format!(
            "signature={}",
            escape_doc_contract_value(&signature)
        ));
    }
    fields.join(";")
}

fn render_function_doc_signature(function: &NirFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.ty.name))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = function
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", ty.name))
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    format!(
        "{async_prefix}fn {}({params}){return_suffix}",
        function.name
    )
}

fn escape_doc_contract_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

fn sanitize_doc_contract_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn push_doc_contract_text_node(module: &mut YirModule, name: &str, resource: &str, value: String) {
    push_scheduler_contract_text_node(module, name, resource, value);
}

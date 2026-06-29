use nuis_semantics::model::{
    AstAttribute, AstAttributeArg, AstAttributeValue, AstEnumVariant, AstModule, AstStructField,
    AstTraitMethodSig, AstTypeAlias, AstTypeRef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDocIndex {
    pub module_path: String,
    pub items: Vec<AstDocIndexItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDocIndexItem {
    pub kind: String,
    pub path: String,
    pub docs: Vec<String>,
    pub signature: Option<String>,
}

pub fn extract_ast_doc_index(module: &AstModule) -> AstDocIndex {
    let module_path = format!("{}.{}", module.domain, module.unit);
    let mut items = Vec::new();

    push_doc_index_item(
        &mut items,
        "module",
        module_path.clone(),
        doc_lines(&module.attributes),
        Some(format!("mod {} {}", module.domain, module.unit)),
    );

    for constant in &module.consts {
        push_doc_index_item(
            &mut items,
            "const",
            format!("{module_path}::{}", constant.name),
            doc_lines(&constant.attributes),
            Some(render_const_signature(constant)),
        );
    }
    for alias in &module.type_aliases {
        push_doc_index_item(
            &mut items,
            "type",
            format!("{module_path}::{}", alias.name),
            doc_lines(&alias.attributes),
            Some(render_type_alias_signature(alias)),
        );
    }
    for definition in &module.structs {
        let struct_path = format!("{module_path}::{}", definition.name);
        push_doc_index_item(
            &mut items,
            "struct",
            struct_path.clone(),
            doc_lines(&definition.attributes),
            Some(render_struct_signature(definition)),
        );
        for field in &definition.fields {
            push_doc_index_item(
                &mut items,
                "struct_field",
                format!("{struct_path}::{}", field.name),
                doc_lines(&field.attributes),
                Some(render_struct_field_signature(field)),
            );
        }
    }
    for definition in &module.enums {
        let enum_path = format!("{module_path}::{}", definition.name);
        push_doc_index_item(
            &mut items,
            "enum",
            enum_path.clone(),
            doc_lines(&definition.attributes),
            Some(render_enum_signature(definition)),
        );
        for variant in &definition.variants {
            push_doc_index_item(
                &mut items,
                "enum_variant",
                format!("{enum_path}::{}", variant.name),
                doc_lines(&variant.attributes),
                Some(render_enum_variant_signature(variant)),
            );
        }
    }
    for definition in &module.traits {
        let trait_path = format!("{module_path}::{}", definition.name);
        push_doc_index_item(
            &mut items,
            "trait",
            trait_path.clone(),
            doc_lines(&definition.attributes),
            Some(format!("trait {}", definition.name)),
        );
        for method in &definition.methods {
            push_doc_index_item(
                &mut items,
                "trait_method",
                format!("{trait_path}::{}", method.name),
                doc_lines(&method.attributes),
                Some(render_trait_method_signature(method)),
            );
        }
    }
    for function in &module.functions {
        push_doc_index_item(
            &mut items,
            "function",
            format!("{module_path}::{}", function.name),
            doc_lines(&function.attributes),
            Some(render_function_signature(function)),
        );
    }

    AstDocIndex { module_path, items }
}

fn push_doc_index_item(
    items: &mut Vec<AstDocIndexItem>,
    kind: &str,
    path: String,
    docs: Vec<String>,
    signature: Option<String>,
) {
    if docs.is_empty() {
        return;
    }
    items.push(AstDocIndexItem {
        kind: kind.to_owned(),
        path,
        docs,
        signature,
    });
}

fn doc_lines(attributes: &[AstAttribute]) -> Vec<String> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "doc")
        .filter_map(|attribute| match attribute.args.first() {
            Some(AstAttributeArg {
                name: None,
                value: AstAttributeValue::String(value),
            }) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn render_const_signature(constant: &nuis_semantics::model::AstConstItem) -> String {
    let ty = constant
        .ty
        .as_ref()
        .map(render_ast_type_name)
        .unwrap_or_else(|| "_".to_owned());
    format!("const {}: {}", constant.name, ty)
}

fn render_type_alias_signature(alias: &AstTypeAlias) -> String {
    format!(
        "type {} = {}",
        alias.name,
        render_ast_type_name(&alias.target)
    )
}

fn render_struct_signature(definition: &nuis_semantics::model::AstStructDef) -> String {
    format!("struct {}", definition.name)
}

fn render_struct_field_signature(field: &AstStructField) -> String {
    format!("field {}: {}", field.name, render_ast_type_name(&field.ty))
}

fn render_enum_signature(definition: &nuis_semantics::model::AstEnumDef) -> String {
    format!("enum {}", definition.name)
}

fn render_enum_variant_signature(variant: &AstEnumVariant) -> String {
    match &variant.kind {
        nuis_semantics::model::AstEnumVariantKind::Unit => format!("variant {}", variant.name),
        nuis_semantics::model::AstEnumVariantKind::Tuple(fields) => format!(
            "variant {}({})",
            variant.name,
            fields
                .iter()
                .map(render_ast_type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        nuis_semantics::model::AstEnumVariantKind::Struct(fields) => format!(
            "variant {} {{ {} }}",
            variant.name,
            fields
                .iter()
                .map(render_struct_field_signature)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_trait_method_signature(method: &AstTraitMethodSig) -> String {
    let params = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type_name(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type_name(ty)))
        .unwrap_or_default();
    format!("fn {}({}){}", method.name, params, return_suffix)
}

fn render_function_signature(function: &nuis_semantics::model::AstFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type_name(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = function
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type_name(ty)))
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    format!(
        "{}fn {}({}){}",
        async_prefix, function.name, params, return_suffix
    )
}

fn render_ast_type_name(ty: &AstTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push('&');
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_ast_type_name)
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

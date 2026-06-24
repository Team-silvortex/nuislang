use nuis_semantics::model::{
    AstEnumDef, AstEnumVariantKind, AstGenericParam, AstStructDef, AstWherePredicate, NirEnumDef,
    NirEnumVariantKind, NirGenericParam, NirStructDef, NirTypeRef, NirWherePredicate,
};

pub(super) fn render_ast_struct(definition: &AstStructDef) -> String {
    let mut out = String::new();
    out.push_str(&super::render_ast_doc_comments("  ", &definition.attributes));
    let attribute_prefix = super::render_ast_attributes(&definition.attributes);
    let visibility_prefix = super::render_ast_visibility(definition.visibility);
    out.push_str(&format!(
        "  {}{}struct {}{}{}\n",
        attribute_prefix,
        visibility_prefix,
        definition.name,
        render_ast_generic_param_suffix(&definition.generic_params),
        render_ast_where_clause(&definition.where_bounds)
    ));
    for field in &definition.fields {
        out.push_str(&super::render_ast_doc_comments("    ", &field.attributes));
        let field_prefix = super::render_ast_attributes(&field.attributes);
        let field_visibility = super::render_ast_visibility(field.visibility);
        out.push_str(&format!(
            "    {}{}field {}: {}\n",
            field_prefix,
            field_visibility,
            field.name,
            super::render_ast_type(&field.ty)
        ));
    }
    out
}

pub(super) fn render_nir_struct(definition: &NirStructDef) -> String {
    let mut out = String::new();
    let annotation_prefix = super::render_nir_annotations(&definition.annotations);
    let visibility_prefix = super::render_nir_visibility(definition.visibility);
    out.push_str(&format!(
        "  {}{}struct {}{}{}\n",
        annotation_prefix,
        visibility_prefix,
        definition.name,
        render_nir_generic_param_suffix(&definition.generic_params),
        render_nir_where_clause(&definition.where_bounds)
    ));
    for field in &definition.fields {
        let field_prefix = super::render_nir_annotations(&field.annotations);
        let field_visibility = super::render_nir_visibility(field.visibility);
        out.push_str(&format!(
            "    {}{}field {}: {}\n",
            field_prefix,
            field_visibility,
            field.name,
            super::render_nir_type(&field.ty)
        ));
    }
    out
}

pub(super) fn render_ast_enum(definition: &AstEnumDef) -> String {
    let mut out = String::new();
    out.push_str(&super::render_ast_doc_comments("  ", &definition.attributes));
    let attribute_prefix = super::render_ast_attributes(&definition.attributes);
    let visibility_prefix = super::render_ast_visibility(definition.visibility);
    out.push_str(&format!(
        "  {}{}enum {}{}{}\n",
        attribute_prefix,
        visibility_prefix,
        definition.name,
        render_ast_generic_param_suffix(&definition.generic_params),
        render_ast_where_clause(&definition.where_bounds)
    ));
    for variant in &definition.variants {
        out.push_str(&super::render_ast_doc_comments("    ", &variant.attributes));
        match &variant.kind {
            AstEnumVariantKind::Unit => {
                out.push_str(&format!("    variant {}\n", variant.name));
            }
            AstEnumVariantKind::Tuple(fields) => {
                out.push_str(&format!(
                    "    variant {}({})\n",
                    variant.name,
                    fields
                        .iter()
                        .map(super::render_ast_type)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            AstEnumVariantKind::Struct(fields) => {
                out.push_str(&format!("    variant {} {{\n", variant.name));
                for field in fields {
                    out.push_str(&super::render_ast_doc_comments("      ", &field.attributes));
                    let field_prefix = super::render_ast_attributes(&field.attributes);
                    let field_visibility = super::render_ast_visibility(field.visibility);
                    out.push_str(&format!(
                        "      {}{}field {}: {}\n",
                        field_prefix,
                        field_visibility,
                        field.name,
                        super::render_ast_type(&field.ty)
                    ));
                }
                out.push_str("    }\n");
            }
        }
    }
    out
}

pub(super) fn render_nir_enum(definition: &NirEnumDef) -> String {
    let mut out = String::new();
    let annotation_prefix = super::render_nir_annotations(&definition.annotations);
    let visibility_prefix = super::render_nir_visibility(definition.visibility);
    out.push_str(&format!(
        "  {}{}enum {}{}{}\n",
        annotation_prefix,
        visibility_prefix,
        definition.name,
        render_nir_generic_param_suffix(&definition.generic_params),
        render_nir_where_clause(&definition.where_bounds)
    ));
    for variant in &definition.variants {
        match &variant.kind {
            NirEnumVariantKind::Unit => {
                out.push_str(&format!("    variant {}\n", variant.name));
            }
            NirEnumVariantKind::Tuple(fields) => {
                out.push_str(&format!(
                    "    variant {}({})\n",
                    variant.name,
                    fields
                        .iter()
                        .map(super::render_nir_type)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            NirEnumVariantKind::Struct(fields) => {
                out.push_str(&format!("    variant {} {{\n", variant.name));
                for field in fields {
                    let field_prefix = super::render_nir_annotations(&field.annotations);
                    let field_visibility = super::render_nir_visibility(field.visibility);
                    out.push_str(&format!(
                        "      {}{}field {}: {}\n",
                        field_prefix,
                        field_visibility,
                        field.name,
                        super::render_nir_type(&field.ty)
                    ));
                }
                out.push_str("    }\n");
            }
        }
    }
    out
}

pub(super) fn render_nir_type_arg_suffix(type_args: &[NirTypeRef]) -> String {
    if type_args.is_empty() {
        return String::new();
    }
    format!(
        "<{}>",
        type_args
            .iter()
            .map(super::render_nir_type)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_ast_generic_param_suffix(params: &[AstGenericParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    format!(
        "<{}>",
        params
            .iter()
            .map(render_ast_generic_param)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_nir_generic_param_suffix(params: &[NirGenericParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    format!(
        "<{}>",
        params
            .iter()
            .map(render_nir_generic_param)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_ast_generic_param(param: &AstGenericParam) -> String {
    if param.bounds.is_empty() {
        return param.name.clone();
    }
    format!(
        "{}: {}",
        param.name,
        param
            .bounds
            .iter()
            .map(super::render_ast_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

fn render_nir_generic_param(param: &NirGenericParam) -> String {
    if param.bounds.is_empty() {
        return param.name.clone();
    }
    format!(
        "{}: {}",
        param.name,
        param
            .bounds
            .iter()
            .map(super::render_nir_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

fn render_ast_where_clause(predicates: &[AstWherePredicate]) -> String {
    if predicates.is_empty() {
        return String::new();
    }
    format!(
        " where {}",
        predicates
            .iter()
            .map(|predicate| {
                format!(
                    "{}: {}",
                    predicate.param_name,
                    predicate
                        .bounds
                        .iter()
                        .map(super::render_ast_type)
                        .collect::<Vec<_>>()
                        .join(" + ")
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_nir_where_clause(predicates: &[NirWherePredicate]) -> String {
    if predicates.is_empty() {
        return String::new();
    }
    format!(
        " where {}",
        predicates
            .iter()
            .map(|predicate| {
                format!(
                    "{}: {}",
                    predicate.param_name,
                    predicate
                        .bounds
                        .iter()
                        .map(super::render_nir_type)
                        .collect::<Vec<_>>()
                        .join(" + ")
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

use nuis_semantics::model::{
    AstGenericParam, AstStructDef, NirGenericParam, NirStructDef, NirTypeRef,
};

pub(super) fn render_ast_struct(definition: &AstStructDef) -> String {
    let mut out = String::new();
    let attribute_prefix = super::render_ast_attributes(&definition.attributes);
    let visibility_prefix = super::render_ast_visibility(definition.visibility);
    out.push_str(&format!(
        "  {}{}struct {}{}\n",
        attribute_prefix,
        visibility_prefix,
        definition.name,
        render_ast_generic_param_suffix(&definition.generic_params)
    ));
    for field in &definition.fields {
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
        "  {}{}struct {}{}\n",
        annotation_prefix,
        visibility_prefix,
        definition.name,
        render_nir_generic_param_suffix(&definition.generic_params)
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
            .map(|param| match &param.bound {
                Some(bound) => format!("{}: {}", param.name, super::render_ast_type(bound)),
                None => param.name.clone(),
            })
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
            .map(|param| match &param.bound {
                Some(bound) => format!("{}: {}", param.name, super::render_nir_type(bound)),
                None => param.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

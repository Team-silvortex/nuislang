use nuis_semantics::model::{
    AstAttribute, AstAttributeValue, AstStructDef, NirAnnotation, NirAttributeArg,
    NirAttributeValue, NirExpr, NirStructDef, NirTypeRef, NirVisibility,
};

use super::is_public_visibility;

#[derive(Clone)]
pub(crate) struct ModuleConstValue {
    pub(crate) visibility: NirVisibility,
    pub(crate) ty: NirTypeRef,
    pub(crate) value: NirExpr,
}

pub(crate) fn lower_ast_attributes(attributes: &[AstAttribute]) -> Vec<NirAnnotation> {
    attributes
        .iter()
        .map(|attribute| NirAnnotation {
            name: attribute.name.clone(),
            args: attribute
                .args
                .iter()
                .map(|arg| NirAttributeArg {
                    name: arg.name.clone(),
                    value: match &arg.value {
                        AstAttributeValue::Bool(value) => NirAttributeValue::Bool(*value),
                        AstAttributeValue::Int(value) => NirAttributeValue::Int(*value),
                        AstAttributeValue::String(value) => {
                            NirAttributeValue::String(value.clone())
                        }
                        AstAttributeValue::Ident(value) => NirAttributeValue::Ident(value.clone()),
                    },
                })
                .collect(),
        })
        .collect()
}

pub(crate) fn helper_visible_struct_annotations(definition: &AstStructDef) -> Vec<NirAnnotation> {
    let mut annotations = lower_ast_attributes(&definition.attributes);
    let hidden_private_fields = definition
        .fields
        .iter()
        .filter(|field| !is_public_visibility(field.visibility))
        .count();
    if hidden_private_fields > 0 {
        annotations.push(NirAnnotation {
            name: "visibility_hidden_fields".to_owned(),
            args: vec![NirAttributeArg {
                name: None,
                value: NirAttributeValue::Int(hidden_private_fields as i64),
            }],
        });
    }
    annotations
}

pub(crate) fn hidden_private_field_count(definition: &NirStructDef) -> usize {
    definition
        .annotations
        .iter()
        .find(|annotation| annotation.name == "visibility_hidden_fields")
        .and_then(|annotation| annotation.args.first())
        .and_then(|arg| match arg.value {
            NirAttributeValue::Int(value) if value > 0 => Some(value as usize),
            _ => None,
        })
        .unwrap_or(0)
}

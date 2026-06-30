use std::collections::BTreeMap;

use nuis_semantics::model::nir_expr_effect_class;
use nuis_semantics::model::{AstDestructureBinding, AstDestructureField, NirExpr};

use super::metadata::ModuleConstValue;
use super::validation_helpers::validate_type_ref;
use super::{
    infer_nir_expr_type, instantiate_struct_field_type, lower_expr_with_async,
    lower_type_ref_with_aliases, resolve_declared_or_inferred, AstStmt, AstTypeAlias,
    FunctionSignature, NirStmt, NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_destructure_let_stmt_with_async(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    let AstStmt::DestructureLet {
        type_ref,
        fields,
        value,
    } = stmt
    else {
        return Err("internal error: expected destructuring let statement".to_owned());
    };
    let expected = type_ref
        .as_ref()
        .map(|type_ref| lower_type_ref_with_aliases(type_ref, type_aliases))
        .transpose()?;
    if let Some(expected) = expected.as_ref() {
        validate_type_ref(expected)?;
    }
    let lowered = lower_expr_with_async(
        value,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected.as_ref(),
        false,
    )?;
    match nir_expr_effect_class(&lowered) {
        nuis_semantics::model::NirExprEffectClass::Pure
        | nuis_semantics::model::NirExprEffectClass::LocalReadOnly => {}
        _ => {
            return Err(
                "minimal destructuring let currently requires a pure or local-read-only source expression"
                    .to_owned(),
            )
        }
    }
    let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
    let final_type = resolve_declared_or_inferred("destructuring let source", expected, inferred)?;
    let mut lowered_stmts = Vec::new();
    emit_destructure_bindings(
        &lowered,
        &final_type,
        fields,
        bindings,
        type_aliases,
        struct_table,
        &mut lowered_stmts,
    )?;
    Ok(lowered_stmts)
}

pub(super) fn emit_destructure_bindings(
    base: &NirExpr,
    base_type: &NirTypeRef,
    fields: &[AstDestructureField],
    bindings: &mut BTreeMap<String, NirTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, NirStructDef>,
    lowered_stmts: &mut Vec<NirStmt>,
) -> Result<(), String> {
    let definition = struct_table.get(&base_type.name).ok_or_else(|| {
        format!(
            "destructuring let requires a visible struct type, found `{}`",
            base_type.render()
        )
    })?;
    for field in fields {
        let field_def = definition.field(&field.field).ok_or_else(|| {
            format!(
                "destructuring let `{}` does not have field `{}`",
                base_type.render(),
                field.field
            )
        })?;
        let field_ty = instantiate_struct_field_type(base_type, definition, &field_def.ty);
        let field_expr = NirExpr::FieldAccess {
            base: Box::new(base.clone()),
            field: field.field.clone(),
        };
        match &field.binding {
            AstDestructureBinding::Bind(name) => {
                bindings.insert(name.clone(), field_ty.clone());
                lowered_stmts.push(NirStmt::Let {
                    name: name.clone(),
                    ty: Some(field_ty),
                    value: field_expr,
                });
            }
            AstDestructureBinding::Ignore => {}
            AstDestructureBinding::Nested {
                type_ref,
                fields: nested_fields,
            } => {
                if let Some(type_ref) = type_ref {
                    let expected = lower_type_ref_with_aliases(type_ref, type_aliases)?;
                    validate_type_ref(&expected)?;
                    if expected != field_ty {
                        return Err(format!(
                            "nested destructuring field `{}` expects `{}`, found `{}`",
                            field.field,
                            expected.render(),
                            field_ty.render()
                        ));
                    }
                }
                emit_destructure_bindings(
                    &field_expr,
                    &field_ty,
                    nested_fields,
                    bindings,
                    type_aliases,
                    struct_table,
                    lowered_stmts,
                )?;
            }
        }
    }
    Ok(())
}

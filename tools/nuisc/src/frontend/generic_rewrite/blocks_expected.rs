use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::FunctionSignature;

pub(super) fn contains_unresolved_struct_placeholder(
    ty: &AstTypeRef,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> bool {
    if let Some(definition) = struct_table.get(&ty.name) {
        let placeholder_names = definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        if ty.generic_args.iter().any(|arg| {
            contains_definition_placeholder(arg, &placeholder_names)
                || contains_unresolved_struct_placeholder(arg, struct_table)
        }) {
            return true;
        }
    }
    ty.generic_args
        .iter()
        .any(|arg| contains_unresolved_struct_placeholder(arg, struct_table))
}

fn contains_definition_placeholder(ty: &AstTypeRef, placeholder_names: &BTreeSet<String>) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_definition_placeholder(arg, placeholder_names))
}

pub(super) fn let_binding_expected_type_from_following_use(
    stmt: &AstStmt,
    following_stmts: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let AstStmt::Let { name, ty, .. } = stmt else {
        return None;
    };
    if ty.is_some() {
        return None;
    }
    expected_type_for_var_from_following_stmts(
        name,
        following_stmts,
        current_return_type,
        generic_templates,
        signatures,
        visible_type_aliases,
        struct_table,
    )
}

fn expected_type_for_var_from_following_stmts(
    current_name: &str,
    following_stmts: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let (stmt, rest) = following_stmts.split_first()?;
    match stmt {
        AstStmt::Let {
            name,
            value: AstExpr::Var(source_name),
            ..
        } if source_name == current_name => expected_type_for_var_from_following_stmts(
            name,
            rest,
            current_return_type,
            generic_templates,
            signatures,
            visible_type_aliases,
            struct_table,
        ),
        AstStmt::Let {
            name,
            ty,
            mutable: _,
            value:
                AstExpr::Call {
                    callee,
                    generic_args,
                    args,
                },
        } => {
            let index = args.iter().position(
                |arg| matches!(arg, AstExpr::Var(var_name) if var_name == current_name),
            )?;
            let call_expected = ty.clone().or_else(|| {
                expected_type_for_var_from_following_stmts(
                    name,
                    rest,
                    current_return_type,
                    generic_templates,
                    signatures,
                    visible_type_aliases,
                    struct_table,
                )
            });
            super::exprs::call_arg_expected_type(super::exprs::CallArgExpectedTypeInput {
                callee,
                generic_args,
                index,
                expected: call_expected.as_ref().or(current_return_type),
                generic_templates,
                signatures,
                visible_type_aliases,
                struct_table,
            })
        }
        AstStmt::Return(Some(AstExpr::Call {
            callee,
            generic_args,
            args,
        })) => args.iter().enumerate().find_map(|(index, arg)| {
            matches!(arg, AstExpr::Var(var_name) if var_name == current_name).then(|| {
                super::exprs::call_arg_expected_type(super::exprs::CallArgExpectedTypeInput {
                    callee,
                    generic_args,
                    index,
                    expected: current_return_type,
                    generic_templates,
                    signatures,
                    visible_type_aliases,
                    struct_table,
                })
            })?
        }),
        _ => None,
    }
}

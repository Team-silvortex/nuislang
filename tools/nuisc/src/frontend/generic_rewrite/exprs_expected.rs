use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstStructDef, AstTypeAlias, AstTypeRef};

use super::super::generics::{specialize_ast_type_ref, unify_generic_type_pattern};
use super::super::types::ast_type_from_nir;
use super::super::{
    lower_type_ref, resolve_ast_type_ref_aliases, substitute_ast_type_alias_target,
    FunctionSignature,
};
use super::exprs_alias_expected::{
    contains_ast_placeholder_generic_name, infer_alias_struct_target_from_expected,
};

pub(super) struct CallArgExpectedTypeInput<'a> {
    pub(super) callee: &'a str,
    pub(super) generic_args: &'a [AstTypeRef],
    pub(super) index: usize,
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) generic_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
}

pub(super) fn call_arg_expected_type(input: CallArgExpectedTypeInput<'_>) -> Option<AstTypeRef> {
    let CallArgExpectedTypeInput {
        callee,
        generic_args,
        index,
        expected,
        generic_templates,
        signatures,
        visible_type_aliases,
        struct_table,
    } = input;
    if let Some(from_explicit_generics) = generic_template_arg_expected_type_from_explicit_args(
        callee,
        generic_args,
        index,
        generic_templates,
        visible_type_aliases,
    ) {
        return Some(from_explicit_generics);
    }
    if let Some(from_template_expected) = generic_template_arg_expected_type_from_return(
        callee,
        index,
        expected,
        generic_templates,
        visible_type_aliases,
    ) {
        return Some(from_template_expected);
    }
    if let Some(from_signature_expected) = generic_signature_arg_expected_type_from_return(
        callee,
        index,
        expected,
        signatures,
        visible_type_aliases,
        struct_table,
    ) {
        return Some(from_signature_expected);
    }
    if let Some(from_task_builtin) = task_builtin_arg_expected_type(callee, index, expected) {
        return Some(from_task_builtin);
    }
    if let Some(from_signature) = signatures
        .get(callee)
        .and_then(|signature| signature.params.get(index))
        .map(ast_type_from_nir)
    {
        return Some(from_signature);
    }
    constructor_value_expected_type(
        callee,
        generic_args,
        index,
        expected,
        visible_type_aliases,
        struct_table,
    )
}

fn generic_template_arg_expected_type_from_explicit_args(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    generic_templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    if generic_args.is_empty() {
        return None;
    }
    let template = generic_templates.get(callee)?;
    if generic_args.len() != template.generic_params.len() {
        return None;
    }
    let param = template.params.get(index)?;
    let substitutions = template
        .generic_params
        .iter()
        .zip(generic_args.iter())
        .map(|(generic, arg)| {
            Some((
                generic.name.clone(),
                lower_type_ref(&resolve_ast_type_ref_aliases(arg, visible_type_aliases).ok()?),
            ))
        })
        .collect::<Option<BTreeMap<_, _>>>()?;
    specialize_ast_type_ref(&param.ty, &substitutions).ok()
}

fn generic_template_arg_expected_type_from_return(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let template = generic_templates.get(callee)?;
    let return_pattern = template.return_type.as_ref()?;
    let param = template.params.get(index)?;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    if unify_generic_type_pattern(
        return_pattern,
        expected,
        &generic_names,
        &mut substitutions,
        &template.name,
    )
    .is_err()
    {
        let resolved_return_pattern =
            resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases).ok()?;
        let resolved_expected =
            resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
        substitutions.clear();
        unify_generic_type_pattern(
            &resolved_return_pattern,
            &resolved_expected,
            &generic_names,
            &mut substitutions,
            &template.name,
        )
        .ok()?;
    }
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok()
}

fn generic_signature_arg_expected_type_from_return(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let signature = signatures.get(callee)?;
    let return_ty = ast_type_from_nir(signature.return_type.as_ref()?);
    let param_ty = ast_type_from_nir(signature.params.get(index)?);
    let mut generic_names = BTreeSet::new();
    collect_signature_generic_placeholders(
        &return_ty,
        visible_type_aliases,
        struct_table,
        &mut generic_names,
    );
    collect_signature_generic_placeholders(
        &param_ty,
        visible_type_aliases,
        struct_table,
        &mut generic_names,
    );
    if generic_names.is_empty() {
        return None;
    }
    let resolved_return_pattern =
        resolve_ast_type_ref_aliases(&return_ty, visible_type_aliases).ok()?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    unify_generic_type_pattern(
        &resolved_return_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        callee,
    )
    .ok()?;
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    let specialized = specialize_ast_type_ref(&param_ty, &lowered_substitutions).ok()?;
    (!contains_ast_placeholder_generic_name(&specialized, &generic_names)).then_some(specialized)
}

fn task_builtin_arg_expected_type(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    match callee {
        "spawn" if index == 0 && expected.name == "Task" && expected.generic_args.len() == 1 => {
            expected.generic_args.first().cloned()
        }
        "join" if index == 0 && expected.name != "Task" => Some(AstTypeRef {
            name: "Task".to_owned(),
            generic_args: vec![expected.clone()],
            is_optional: false,
            is_ref: false,
        }),
        _ => None,
    }
}

fn collect_signature_generic_placeholders(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
    out: &mut BTreeSet<String>,
) {
    if ty.generic_args.is_empty()
        && ty
            .name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        && !visible_type_aliases.contains_key(&ty.name)
        && !struct_table.contains_key(&ty.name)
        && ty.name != "String"
    {
        out.insert(ty.name.clone());
    }
    for arg in &ty.generic_args {
        collect_signature_generic_placeholders(arg, visible_type_aliases, struct_table, out);
    }
}

fn constructor_value_expected_type(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    if index != 0 {
        return None;
    }
    let concrete_head = if let Some(definition) = struct_table.get(callee) {
        if definition.fields.len() != 1 {
            return None;
        }
        if generic_args.is_empty() {
            expected
                .filter(|expected| expected.name == callee)
                .cloned()
                .or_else(|| {
                    let (parent_name, _) = callee.rsplit_once('.')?;
                    let resolved_expected =
                        resolve_ast_type_ref_aliases(expected?, visible_type_aliases).ok()?;
                    (resolved_expected.name == parent_name).then_some(AstTypeRef {
                        name: callee.to_owned(),
                        generic_args: resolved_expected.generic_args,
                        is_optional: false,
                        is_ref: false,
                    })
                })
        } else {
            Some(AstTypeRef {
                name: callee.to_owned(),
                generic_args: generic_args.to_vec(),
                is_optional: false,
                is_ref: false,
            })
        }
    } else if generic_args.is_empty() {
        infer_alias_struct_target_from_expected(
            callee,
            expected,
            visible_type_aliases,
            struct_table,
        )
        .ok()
        .flatten()
        .map(|(name, generic_args)| AstTypeRef {
            name,
            generic_args,
            is_optional: false,
            is_ref: false,
        })
    } else {
        let alias_definition = visible_type_aliases.get(callee)?;
        let substituted = substitute_ast_type_alias_target(
            &alias_definition.target,
            &alias_definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .zip(generic_args.iter().cloned())
                .collect::<BTreeMap<_, _>>(),
        )
        .ok()?;
        let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases).ok()?;
        Some(resolved)
    }?;
    let definition = struct_table.get(&concrete_head.name)?;
    if definition.fields.len() != 1 {
        return None;
    }
    Some(
        super::super::validation_binding_env::instantiate_ast_struct_field_type(
            &concrete_head,
            definition,
            &definition.fields[0].ty,
        ),
    )
}

pub(super) fn struct_field_expected_type(
    literal_ty: &AstTypeRef,
    field_name: &str,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let definition = struct_table.get(&literal_ty.name)?;
    let field = definition
        .fields
        .iter()
        .find(|field| field.name == field_name)?;
    Some(
        super::super::validation_binding_env::instantiate_ast_struct_field_type(
            literal_ty, definition, &field.ty,
        ),
    )
}

pub(super) fn field_access_base_expected_type(
    field_expected: Option<&AstTypeRef>,
    field_name: &str,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let resolved_expected =
        resolve_ast_type_ref_aliases(field_expected?, visible_type_aliases).ok()?;
    let mut candidates = Vec::new();
    for definition in struct_table.values() {
        let Some(field) = definition
            .fields
            .iter()
            .find(|field| field.name == field_name)
        else {
            continue;
        };
        let resolved_field_ty =
            resolve_ast_type_ref_aliases(&field.ty, visible_type_aliases).ok()?;
        let generic_names = definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
        if unify_generic_type_pattern(
            &resolved_field_ty,
            &resolved_expected,
            &generic_names,
            &mut substitutions,
            &definition.name,
        )
        .is_err()
        {
            continue;
        }
        let Some(generic_args) = definition
            .generic_params
            .iter()
            .map(|param| substitutions.get(&param.name).cloned())
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        if generic_args
            .iter()
            .any(|arg| contains_ast_placeholder_generic_name(arg, &generic_names))
        {
            continue;
        }
        candidates.push(AstTypeRef {
            name: definition.name.clone(),
            generic_args,
            is_optional: false,
            is_ref: false,
        });
    }
    (candidates.len() == 1).then(|| candidates.pop()).flatten()
}

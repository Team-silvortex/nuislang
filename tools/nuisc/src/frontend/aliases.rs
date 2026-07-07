use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstModule, AstParam, AstTypeAlias, AstTypeRef, AstVisibility, NirConstItem,
    NirParam, NirStructDef, NirTypeRef,
};

use super::types::ast_type_from_nir;
use super::{
    infer_nir_expr_type, is_public_visibility, lower_expr_with_async, lower_visibility,
    resolve_declared_or_inferred, validate_type_ref, ExprWithAsyncInput, FunctionSignature,
    ModuleConstValue,
};

pub(crate) fn lower_param_with_aliases(
    param: &AstParam,
    aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<NirParam, String> {
    Ok(NirParam {
        name: param.name.clone(),
        ty: lower_type_ref_with_aliases(&param.ty, aliases)?,
    })
}

pub(crate) fn lower_type_ref(ty: &AstTypeRef) -> NirTypeRef {
    NirTypeRef {
        name: ty.name.clone(),
        generic_args: ty.generic_args.iter().map(lower_type_ref).collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

pub(crate) fn lower_type_ref_with_aliases(
    ty: &AstTypeRef,
    aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<NirTypeRef, String> {
    Ok(lower_type_ref(&resolve_ast_type_ref_aliases(ty, aliases)?))
}

pub(crate) fn build_visible_type_alias_map(
    module: &AstModule,
    local_helpers: &[&AstModule],
) -> Result<BTreeMap<String, AstTypeAlias>, String> {
    let mut aliases = BTreeMap::new();
    for helper in local_helpers {
        for alias in &helper.type_aliases {
            if !is_public_visibility(alias.visibility) {
                continue;
            }
            aliases.insert(format!("{}.{}", helper.unit, alias.name), alias.clone());
            aliases
                .entry(alias.name.clone())
                .or_insert_with(|| alias.clone());
        }
    }
    for alias in &module.type_aliases {
        aliases.insert(alias.name.clone(), alias.clone());
    }
    Ok(aliases)
}

pub(crate) fn resolve_ast_type_ref_aliases(
    ty: &AstTypeRef,
    aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<AstTypeRef, String> {
    let mut visiting = BTreeSet::new();
    resolve_ast_type_ref_aliases_inner(ty, aliases, &mut visiting)
}

pub(crate) fn resolve_ast_type_ref_aliases_inner(
    ty: &AstTypeRef,
    aliases: &BTreeMap<String, AstTypeAlias>,
    visiting: &mut BTreeSet<String>,
) -> Result<AstTypeRef, String> {
    let resolved_generic_args = ty
        .generic_args
        .iter()
        .map(|arg| resolve_ast_type_ref_aliases_inner(arg, aliases, visiting))
        .collect::<Result<Vec<_>, _>>()?;
    let raw = AstTypeRef {
        name: ty.name.clone(),
        generic_args: resolved_generic_args,
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    };
    let Some(alias_definition) = aliases.get(&raw.name) else {
        return Ok(raw);
    };
    if alias_definition.generic_params.len() != raw.generic_args.len() {
        return Err(format!(
            "type alias `{}` expects {} generic argument(s), found {}",
            raw.name,
            alias_definition.generic_params.len(),
            raw.generic_args.len()
        ));
    }
    let visit_key = lower_type_ref(&raw).render();
    if !visiting.insert(visit_key.clone()) {
        return Err(format!("type alias `{}` is cyclic", raw.name));
    }
    let substitutions = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .zip(raw.generic_args.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    let substituted = substitute_ast_type_alias_target(&alias_definition.target, &substitutions)?;
    let mut expanded = resolve_ast_type_ref_aliases_inner(&substituted, aliases, visiting)?;
    visiting.remove(&visit_key);
    if ty.is_optional {
        expanded.is_optional = true;
    }
    if ty.is_ref {
        expanded.is_ref = true;
    }
    Ok(expanded)
}

pub(crate) fn substitute_ast_type_alias_target(
    ty: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> Result<AstTypeRef, String> {
    if ty.generic_args.is_empty() {
        if let Some(substitution) = substitutions.get(&ty.name) {
            let mut substituted = substitution.clone();
            if ty.is_optional {
                substituted.is_optional = true;
            }
            if ty.is_ref {
                substituted.is_ref = true;
            }
            return Ok(substituted);
        }
    }
    Ok(AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| substitute_ast_type_alias_target(arg, substitutions))
            .collect::<Result<Vec<_>, _>>()?,
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    })
}

pub(crate) fn const_type_bindings(
    values: &BTreeMap<String, ModuleConstValue>,
) -> BTreeMap<String, NirTypeRef> {
    values
        .iter()
        .map(|(name, constant)| (name.clone(), constant.ty.clone()))
        .collect()
}

pub(crate) fn ast_const_type_env(
    values: &BTreeMap<String, ModuleConstValue>,
) -> BTreeMap<String, AstTypeRef> {
    values
        .iter()
        .map(|(name, constant)| (name.clone(), ast_type_from_nir(&constant.ty)))
        .collect()
}

pub(crate) fn lower_module_const_items(
    module: &AstModule,
    imported_consts: &BTreeMap<String, ModuleConstValue>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(Vec<NirConstItem>, BTreeMap<String, ModuleConstValue>), String> {
    let mut available_consts = imported_consts.clone();
    let mut lowered_items = Vec::new();
    let mut local_consts = BTreeMap::new();
    for constant in &module.consts {
        let expected = constant
            .ty
            .as_ref()
            .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
            .transpose()?;
        if let Some(expected) = &expected {
            validate_type_ref(expected)?;
        }
        let type_bindings = const_type_bindings(&available_consts);
        let lowered_value = lower_expr_with_async(ExprWithAsyncInput {
            expr: &constant.value,
            current_domain: &module.domain,
            current_function_is_async: false,
            bindings: &type_bindings,
            module_consts: &available_consts,
            signatures,
            struct_table,
            expected: expected.as_ref(),
            allow_async_calls: false,
        })?;
        let inferred =
            infer_nir_expr_type(&lowered_value, &type_bindings, signatures, struct_table);
        let final_type = resolve_declared_or_inferred(&constant.name, expected, inferred)?;
        let lowered = ModuleConstValue {
            visibility: lower_visibility(constant.visibility),
            ty: final_type.clone(),
            value: lowered_value.clone(),
        };
        available_consts.insert(constant.name.clone(), lowered.clone());
        local_consts.insert(constant.name.clone(), lowered.clone());
        lowered_items.push(NirConstItem {
            visibility: lowered.visibility,
            name: constant.name.clone(),
            ty: final_type,
            value: lowered_value,
        });
    }
    Ok((lowered_items, local_consts))
}

pub(crate) fn build_function_return_type_table(
    module: &AstModule,
    concrete_module_functions: &[AstFunction],
    generic_templates: &BTreeMap<String, AstFunction>,
    local_cpu_helpers: &[&AstModule],
    aliases: &BTreeMap<String, AstTypeAlias>,
) -> BTreeMap<String, Option<AstTypeRef>> {
    let mut table = BTreeMap::new();
    for function in &module.externs {
        table.insert(
            function.name.clone(),
            Some(
                resolve_ast_type_ref_aliases(&function.return_type, aliases)
                    .unwrap_or_else(|_| function.return_type.clone()),
            ),
        );
    }
    for function in concrete_module_functions {
        table.insert(
            function.name.clone(),
            function
                .return_type
                .as_ref()
                .map(|ty| resolve_ast_type_ref_aliases(ty, aliases).unwrap_or_else(|_| ty.clone())),
        );
    }
    for (name, function) in generic_templates {
        table.insert(
            name.clone(),
            function
                .return_type
                .as_ref()
                .map(|ty| resolve_ast_type_ref_aliases(ty, aliases).unwrap_or_else(|_| ty.clone())),
        );
    }
    for helper in local_cpu_helpers {
        for function in helper
            .functions
            .iter()
            .filter(|function| matches!(function.visibility, AstVisibility::Public))
        {
            table.insert(
                function.name.clone(),
                function.return_type.as_ref().map(|ty| {
                    resolve_ast_type_ref_aliases(ty, aliases).unwrap_or_else(|_| ty.clone())
                }),
            );
            table.insert(
                format!("{}.{}", helper.unit, function.name),
                function.return_type.as_ref().map(|ty| {
                    resolve_ast_type_ref_aliases(ty, aliases).unwrap_or_else(|_| ty.clone())
                }),
            );
        }
    }
    table
}

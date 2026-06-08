use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstModule, AstStmt, AstTraitDef, AstTypeAlias, AstTypeRef};

use super::{lower_type_ref, resolve_ast_type_ref_aliases, substitute_ast_type_alias_target};

pub(super) fn collect_visible_trait_methods(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut methods = BTreeMap::new();
    for definition in &module.traits {
        insert_trait_methods(&mut methods, definition.name.clone(), definition);
    }
    for helper in local_cpu_helpers {
        for definition in helper
            .traits
            .iter()
            .filter(|definition| super::is_public_visibility(definition.visibility))
        {
            insert_trait_methods(&mut methods, definition.name.clone(), definition);
            insert_trait_methods(
                &mut methods,
                format!("{}.{}", helper.unit, definition.name),
                definition,
            );
        }
    }
    methods
}

pub(super) fn validate_expr_generic_method_bounds(
    expr: &AstExpr,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match expr {
        AstExpr::Bool(_) | AstExpr::Text(_) | AstExpr::Int(_) | AstExpr::Var(_) => {}
        AstExpr::Lambda { .. } | AstExpr::Instantiate { .. } => {}
        AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Call { args, .. } | AstExpr::Invoke { args, .. } => {
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            validate_expr_generic_method_bounds(
                receiver,
                visible_type_aliases,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
            if let Some(receiver_ty) = simple_local_expr_type(receiver, local_type_env) {
                validate_generic_receiver_method_bound(
                    &receiver_ty,
                    method,
                    visible_type_aliases,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    context,
                )?;
            }
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_expr_generic_method_bounds(
                    value,
                    visible_type_aliases,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_expr_generic_method_bounds(
                lhs,
                visible_type_aliases,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                rhs,
                visible_type_aliases,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
    }
    Ok(())
}

pub(super) fn simple_local_stmt_type(
    stmt: &AstStmt,
    local_type_env: &BTreeMap<String, AstTypeRef>,
) -> Option<(String, AstTypeRef)> {
    match stmt {
        AstStmt::Let { name, ty, value } | AstStmt::Const { name, ty, value } => ty
            .clone()
            .or_else(|| simple_local_expr_type(value, local_type_env))
            .map(|ty| (name.clone(), ty)),
        _ => None,
    }
}

fn insert_trait_methods(
    methods: &mut BTreeMap<String, BTreeSet<String>>,
    name: String,
    definition: &AstTraitDef,
) {
    methods.insert(
        name,
        definition
            .methods
            .iter()
            .map(|method| method.name.clone())
            .collect(),
    );
}

pub(super) fn simple_local_expr_type(
    expr: &AstExpr,
    local_type_env: &BTreeMap<String, AstTypeRef>,
) -> Option<AstTypeRef> {
    match expr {
        AstExpr::Var(name) => local_type_env.get(name).cloned(),
        _ => None,
    }
}

fn validate_generic_receiver_method_bound(
    receiver_ty: &AstTypeRef,
    method: &str,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    context: &str,
) -> Result<(), String> {
    let Some((generic_name, receiver_context)) = resolve_generic_receiver_context(
        receiver_ty,
        visible_type_aliases,
        generic_param_names,
        &mut BTreeSet::new(),
    )?
    else {
        return Ok(());
    };
    let context = format!("{context}{receiver_context}");

    let candidates = trait_methods
        .iter()
        .filter(|(_, methods)| methods.contains(method))
        .map(|(trait_name, _)| trait_name.clone())
        .collect::<Vec<_>>();

    if let Some(bound) = generic_bounds.get(&generic_name) {
        if trait_methods
            .get(bound)
            .is_some_and(|methods| methods.contains(method))
        {
            return Ok(());
        }
        if candidates.is_empty() {
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{bound}` does not define that method"
            ));
        }
        if candidates.len() == 1 {
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{bound}` does not define that method; consider bound `{}`",
                candidates[0]
            ));
        }
        return Err(format!(
            "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{bound}` does not define that method; candidate bounds: {}",
            candidates.join(", ")
        ));
    }

    if candidates.len() == 1 {
        return Err(format!(
            "{context} calls method `{method}` on generic parameter `{generic_name}` without required bound `{}`",
            candidates[0]
        ));
    }
    if candidates.len() > 1 {
        return Err(format!(
            "{context} calls method `{method}` on generic parameter `{generic_name}` without a trait bound; candidate bounds: {}",
            candidates.join(", ")
        ));
    }
    Ok(())
}

fn resolve_generic_receiver_context(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_param_names: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> Result<Option<(String, String)>, String> {
    if let Some(alias_definition) = visible_type_aliases.get(&ty.name) {
        if alias_definition.generic_params.len() == ty.generic_args.len() {
            let visit_key = format!("{}::{}", ty.name, lower_type_signature(ty));
            if !visiting.insert(visit_key.clone()) {
                return Ok(None);
            }

            let substitutions = alias_definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .zip(ty.generic_args.iter().cloned())
                .collect::<BTreeMap<_, _>>();
            let expanded =
                substitute_ast_type_alias_target(&alias_definition.target, &substitutions)?;
            let nested = resolve_generic_receiver_context(
                &expanded,
                visible_type_aliases,
                generic_param_names,
                visiting,
            )?;
            visiting.remove(&visit_key);
            if let Some((name, context)) = nested {
                return Ok(Some((
                    name,
                    format!(
                        "{context} via type alias `{}` target",
                        alias_definition.name
                    ),
                )));
            }
        }
    }

    let resolved = resolve_ast_type_ref_aliases(ty, visible_type_aliases)?;
    if resolved.generic_args.is_empty() && !resolved.is_optional && !resolved.is_ref {
        if generic_param_names.contains(&resolved.name) {
            return Ok(Some((resolved.name.clone(), String::new())));
        }
    }
    Ok(None)
}

fn lower_type_signature(ty: &AstTypeRef) -> String {
    lower_type_ref(ty).render()
}

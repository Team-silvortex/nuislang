use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstStmt, AstTypeAlias, AstTypeRef, NirExpr, NirModule, NirStmt, NirTypeRef,
};

use super::{sanitize_ident, split_domain_unit, LoadedProject};

pub(super) fn find_profile_call_declared_type(
    body: &[AstStmt],
    aliases: &[AstTypeAlias],
    callee: &str,
    marker_tag: Option<&str>,
) -> Option<AstTypeRef> {
    for stmt in body {
        match stmt {
            AstStmt::Let {
                ty: Some(ty),
                value:
                    AstExpr::Call {
                        callee: stmt_callee,
                        generic_args: _,
                        args,
                    },
                ..
            }
            | AstStmt::Const {
                ty: Some(ty),
                value:
                    AstExpr::Call {
                        callee: stmt_callee,
                        generic_args: _,
                        args,
                    },
                ..
            } if stmt_callee == callee => {
                if let Some(expected_tag) = marker_tag {
                    let Some(AstExpr::Text(actual_tag)) = args.first() else {
                        continue;
                    };
                    if actual_tag != expected_tag {
                        continue;
                    }
                }
                return Some(
                    resolve_project_type_aliases(ty, aliases).unwrap_or_else(|| ty.clone()),
                );
            }
            _ => {}
        }
    }
    None
}

pub(super) fn require_profile_semantic_type(
    ty: &AstTypeRef,
    family: &str,
    require_generic: bool,
    unit: &str,
    slot: &str,
) -> Result<(), String> {
    if ty.name != family || ty.is_ref || ty.is_optional {
        return Err(format!(
            "project data unit `data.{}` requires `{}` binding `{}` to use `{}` type, found `{}`",
            unit,
            family,
            slot,
            family,
            render_ast_type_name(ty)
        ));
    }
    if require_generic && ty.generic_args.len() != 1 {
        return Err(format!(
            "project data unit `data.{}` requires `{}` binding `{}` to use typed form `{}<...>`",
            unit, family, slot, family
        ));
    }
    Ok(())
}

pub(super) fn require_marker_semantic_payload_name(
    marker_ty: &AstTypeRef,
    expected: &str,
    unit: &str,
    slot: &str,
) -> Result<(), String> {
    require_profile_semantic_type(marker_ty, "Marker", true, unit, slot)?;
    let actual = marker_ty
        .generic_args
        .first()
        .map(render_ast_type_name)
        .unwrap_or_default();
    if actual != expected {
        return Err(format!(
            "project data unit `data.{}` requires marker `{}` to use `Marker<{}>`, found `Marker<{}>`",
            unit, slot, expected, actual
        ));
    }
    Ok(())
}

pub(super) fn infer_project_route_payload_type(
    project: &LoadedProject,
    endpoint: &str,
    data_unit: &str,
    uplink: bool,
) -> Result<Option<NirTypeRef>, String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    let Some(project_module) = project
        .modules
        .iter()
        .find(|module| module.ast.domain == domain && module.ast.unit == unit)
    else {
        return Ok(None);
    };
    let nir = super::lower_project_module_to_nir(project, project_module)?;
    Ok(find_route_payload_type_in_nir(&nir, data_unit, uplink))
}

pub(super) fn payload_class_marker_name(ty: &NirTypeRef) -> String {
    let suffix = if ty.container_kind().is_some() {
        "Window"
    } else if ty.is_marker_family() {
        "Marker"
    } else if ty.is_handle_table_family() {
        "HandleTable"
    } else {
        "Value"
    };
    format!("PayloadClass{suffix}")
}

pub(super) fn payload_shape_marker_name(ty: &NirTypeRef) -> String {
    format!("PayloadShape{}", payload_shape_type_suffix(ty))
}

pub(super) fn merge_project_payload_contract(
    existing: Option<NirTypeRef>,
    next: NirTypeRef,
    domain: &str,
    unit: &str,
    direction: &str,
) -> Result<NirTypeRef, String> {
    match existing {
        Some(existing) if existing != next => Err(format!(
            "project {} unit `{}.{}` has inconsistent {} payload contracts: `{}` vs `{}`",
            domain,
            domain,
            unit,
            direction,
            existing.render(),
            next.render()
        )),
        Some(existing) => Ok(existing),
        None => Ok(next),
    }
}

pub(super) fn infer_data_handle_table_schema(
    project: &LoadedProject,
    unit: &str,
) -> Result<Option<String>, String> {
    let Some(project_module) = project
        .modules
        .iter()
        .find(|module| module.ast.domain == "data" && module.ast.unit == unit)
    else {
        return Ok(None);
    };
    let Some(profile_fn) = project_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
    else {
        return Ok(None);
    };
    Ok(find_profile_call_declared_type(
        &profile_fn.body,
        &project_module.ast.type_aliases,
        "data_handle_table",
        None,
    )
    .and_then(|ty| ty.generic_args.first().map(render_ast_type_name)))
}

fn resolve_project_type_aliases(ty: &AstTypeRef, aliases: &[AstTypeAlias]) -> Option<AstTypeRef> {
    let alias_map = aliases
        .iter()
        .map(|alias| (alias.name.clone(), alias))
        .collect::<BTreeMap<_, _>>();
    resolve_project_type_aliases_inner(ty, &alias_map, &mut BTreeSet::new())
}

fn resolve_project_type_aliases_inner(
    ty: &AstTypeRef,
    aliases: &BTreeMap<String, &AstTypeAlias>,
    visiting: &mut BTreeSet<String>,
) -> Option<AstTypeRef> {
    let key = render_ast_type_name(ty);
    if !visiting.insert(key.clone()) {
        return None;
    }

    let result = if let Some(alias) = aliases.get(&ty.name) {
        if alias.generic_params.len() != ty.generic_args.len() {
            None
        } else {
            let bindings = alias
                .generic_params
                .iter()
                .zip(ty.generic_args.iter())
                .map(|(param, arg)| (param.name.clone(), arg.clone()))
                .collect::<BTreeMap<_, _>>();
            let substituted = substitute_project_type_alias_target(&alias.target, &bindings);
            resolve_project_type_aliases_inner(&substituted, aliases, visiting)
        }
    } else {
        let mut resolved = ty.clone();
        let mut args = Vec::with_capacity(ty.generic_args.len());
        for arg in &ty.generic_args {
            args.push(resolve_project_type_aliases_inner(arg, aliases, visiting)?);
        }
        resolved.generic_args = args;
        Some(resolved)
    };

    visiting.remove(&key);
    result
}

fn substitute_project_type_alias_target(
    ty: &AstTypeRef,
    bindings: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if let Some(bound) = bindings.get(&ty.name) {
        let mut substituted = bound.clone();
        substituted.is_optional = substituted.is_optional || ty.is_optional;
        substituted.is_ref = substituted.is_ref || ty.is_ref;
        return substituted;
    }
    AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| substitute_project_type_alias_target(arg, bindings))
            .collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

fn find_route_payload_type_in_nir(
    module: &NirModule,
    data_unit: &str,
    uplink: bool,
) -> Option<NirTypeRef> {
    if let Some(function) = module.functions.iter().find(|function| function.name == "main") {
        if let Some(ty) = find_route_payload_type_in_stmts(
            &function.body,
            function.return_type.as_ref(),
            data_unit,
            uplink,
        ) {
            return Some(ty);
        }
    }
    module.functions.iter().find_map(|function| {
        find_route_payload_type_in_stmts(
            &function.body,
            function.return_type.as_ref(),
            data_unit,
            uplink,
        )
    })
}

fn find_route_payload_type_in_stmts(
    body: &[NirStmt],
    current_return_type: Option<&NirTypeRef>,
    data_unit: &str,
    uplink: bool,
) -> Option<NirTypeRef> {
    for stmt in body {
        match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value,
                ..
            }
            | NirStmt::Const { ty, value, .. } => {
                if route_payload_expr_matches(value, data_unit, uplink) {
                    return Some(ty.clone());
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if let Some(ty) =
                    find_route_payload_type_in_stmts(then_body, current_return_type, data_unit, uplink)
                {
                    return Some(ty);
                }
                if let Some(ty) =
                    find_route_payload_type_in_stmts(else_body, current_return_type, data_unit, uplink)
                {
                    return Some(ty);
                }
            }
            NirStmt::While { body, .. } => {
                if let Some(ty) =
                    find_route_payload_type_in_stmts(body, current_return_type, data_unit, uplink)
                {
                    return Some(ty);
                }
            }
            NirStmt::Return(Some(value)) => {
                if route_payload_expr_matches(value, data_unit, uplink) {
                    if let Some(ty) = current_return_type {
                        return Some(ty.clone());
                    }
                }
            }
            NirStmt::Print(_)
            | NirStmt::Await(_)
            | NirStmt::Expr(_)
            | NirStmt::Return(None)
            | NirStmt::Break
            | NirStmt::Continue => {}
            NirStmt::Let { ty: None, .. } => {}
        }
    }
    None
}

fn route_payload_expr_matches(expr: &NirExpr, data_unit: &str, uplink: bool) -> bool {
    match (uplink, expr) {
        (true, NirExpr::DataProfileSendUplink { unit, .. }) => unit == data_unit,
        (false, NirExpr::DataProfileSendDownlink { unit, .. }) => unit == data_unit,
        _ => false,
    }
}

fn payload_shape_type_suffix(ty: &NirTypeRef) -> String {
    if let Some(kind) = ty.container_kind() {
        let prefix = match kind {
            nuis_semantics::model::NirContainerKind::Window => "Window",
            nuis_semantics::model::NirContainerKind::Pipe => "Pipe",
            nuis_semantics::model::NirContainerKind::Instance => "Instance",
            nuis_semantics::model::NirContainerKind::Task => "Task",
        };
        let inner = ty
            .container_payload()
            .map(payload_shape_type_suffix)
            .unwrap_or_else(|| "Unknown".to_owned());
        return format!("{prefix}{inner}");
    }
    sanitize_ident(&ty.name)
}

fn render_ast_type_name(ty: &AstTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
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

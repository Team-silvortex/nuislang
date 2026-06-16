use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use nuis_semantics::model::nir_expr_effect_class;
use nuis_semantics::model::{AstDestructureBinding, AstDestructureField, AstMatchArm, NirExpr};

use super::match_lowering::lower_match_stmt_with_async;
use super::metadata::ModuleConstValue;
use super::validation_helpers::validate_type_ref;
use super::{
    ast_type_from_nir, bool_type, compatible_types, infer_nir_expr_type,
    instantiate_struct_field_type, lower_expr_with_async,
    lower_type_ref_with_aliases, resolve_declared_or_inferred, AstStmt, AstTypeAlias, AstTypeRef,
    FunctionSignature, NirStmt, NirStructDef, NirTypeRef,
};

static TRY_EXPANSION_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_stmt_with_async(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    Ok(match stmt {
        AstStmt::Let {
            name, ty, value, ..
        } => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                let expected = ty
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                    .transpose()?;
                if let Some(expected_ty) = expected.as_ref() {
                    validate_type_ref(expected_ty)?;
                }
                let final_type = expected.clone().ok_or_else(|| {
                    format!(
                        "`if` expression let binding `{name}` currently requires an explicit type annotation"
                    )
                })?;
                bindings.insert(name.clone(), final_type.clone());
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: Some(ty.clone().unwrap_or_else(|| AstTypeRef {
                            name: final_type.name.clone(),
                            generic_args: Vec::new(),
                            is_ref: final_type.is_ref,
                            is_optional: final_type.is_optional,
                        })),
                        value,
                    },
                );
            }
            if let super::AstExpr::Match { value, arms } = value {
                let expected = ty
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                    .transpose()?;
                if let Some(expected_ty) = expected.as_ref() {
                    validate_type_ref(expected_ty)?;
                }
                let final_type = expected.clone().ok_or_else(|| {
                    format!(
                        "`match` expression let binding `{name}` currently requires an explicit type annotation"
                    )
                })?;
                bindings.insert(name.clone(), final_type.clone());
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: Some(ty.clone().unwrap_or_else(|| AstTypeRef {
                            name: final_type.name.clone(),
                            generic_args: Vec::new(),
                            is_ref: final_type.is_ref,
                            is_optional: final_type.is_optional,
                        })),
                        value,
                    },
                );
            }
            let expected = ty
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
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
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
                name: name.clone(),
                ty: Some(final_type),
                value: lowered,
            }
        }
        AstStmt::AssignLocal { name, value } => {
            let current_ty = bindings
                .get(name)
                .cloned()
                .ok_or_else(|| format!("cannot assign to unknown local `{name}`"))?;
            let lowered = lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                Some(&current_ty),
                false,
            )?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type =
                resolve_declared_or_inferred(name, Some(current_ty.clone()), inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
                name: name.clone(),
                ty: Some(final_type),
                value: lowered,
            }
        }
        AstStmt::Const { name, ty, value } => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                if ty.is_none() {
                    return Err(format!(
                        "`if` expression const binding `{name}` currently requires an explicit type annotation"
                    ));
                }
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Const {
                        name: name.clone(),
                        ty: ty.clone(),
                        value,
                    },
                );
            }
            if let super::AstExpr::Match { value, arms } = value {
                if ty.is_none() {
                    return Err(format!(
                        "`match` expression const binding `{name}` currently requires an explicit type annotation"
                    ));
                }
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Const {
                        name: name.clone(),
                        ty: ty.clone(),
                        value,
                    },
                );
            }
            let expected = ty
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
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
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Const {
                name: name.clone(),
                ty: final_type,
                value: lowered,
            }
        }
        AstStmt::Print(value) => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &AstStmt::Print,
                );
            }
            if let super::AstExpr::Match { value, arms } = value {
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &AstStmt::Print,
                );
            }
            NirStmt::Print(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
                false,
            )?)
        }
        AstStmt::Await(value) => {
            if !current_function_is_async {
                return Err("`await` is only allowed inside `async fn`".to_owned());
            }
            NirStmt::Await(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
                true,
            )?)
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut then_bindings = bindings.clone();
            let mut else_bindings = bindings.clone();
            NirStmt::If {
                condition: lower_expr_with_async(
                    condition,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    Some(&bool_type()),
                    false,
                )?,
                then_body: lower_stmt_block_with_async(
                    then_body,
                    current_domain,
                    current_function_is_async,
                    &mut then_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )?,
                else_body: lower_stmt_block_with_async(
                    else_body,
                    current_domain,
                    current_function_is_async,
                    &mut else_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )?,
            }
        }
        AstStmt::Match { value, arms } => lower_match_stmt_with_async(
            value,
            arms,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?,
        AstStmt::While { condition, body } => {
            let mut loop_bindings = bindings.clone();
            NirStmt::While {
                condition: lower_expr_with_async(
                    condition,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    Some(&bool_type()),
                    false,
                )?,
                body: lower_stmt_block_with_async(
                    body,
                    current_domain,
                    current_function_is_async,
                    &mut loop_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )?,
            }
        }
        AstStmt::Break => NirStmt::Break,
        AstStmt::Continue => NirStmt::Continue,
        AstStmt::Expr(expr) => NirStmt::Expr(lower_expr_with_async(
            expr,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            None,
            false,
        )?),
        AstStmt::Return(value) => {
            if let Some(super::AstExpr::If {
                condition,
                then_body,
                else_body,
            }) = value
            {
                return lower_if_expr_stmt_with_async(
                    condition,
                    then_body,
                    else_body,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Return(Some(value)),
                );
            }
            if let Some(super::AstExpr::Match { value, arms }) = value {
                return lower_match_expr_stmt_with_async(
                    value,
                    arms,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                    &|value| AstStmt::Return(Some(value)),
                );
            }
            let expected = return_type
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            NirStmt::Return(match value {
                Some(value) => Some(lower_expr_with_async(
                    value,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    expected.as_ref(),
                    false,
                )?),
                None => None,
            })
        }
        AstStmt::DestructureLet { .. } => return Err(
            "internal error: destructuring let must be lowered through statement-sequence lowering"
                .to_owned(),
        ),
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_if_expr_stmt_with_async(
    condition: &super::AstExpr,
    then_body: &[AstStmt],
    else_body: &[AstStmt],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap_terminal: &dyn Fn(super::AstExpr) -> AstStmt,
) -> Result<NirStmt, String> {
    let condition = lower_expr_with_async(
        condition,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        Some(&bool_type()),
        false,
    )?;
    let mut then_bindings = bindings.clone();
    let mut else_bindings = bindings.clone();
    Ok(NirStmt::If {
        condition,
        then_body: lower_if_expr_branch_with_async(
            then_body,
            current_domain,
            current_function_is_async,
            &mut then_bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
            wrap_terminal,
        )?,
        else_body: lower_if_expr_branch_with_async(
            else_body,
            current_domain,
            current_function_is_async,
            &mut else_bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
            wrap_terminal,
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_if_expr_branch_with_async(
    body: &[AstStmt],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap_terminal: &dyn Fn(super::AstExpr) -> AstStmt,
) -> Result<Vec<NirStmt>, String> {
    let rewritten_body = rewrite_control_expr_terminal_branch(
        body,
        wrap_terminal,
        ControlExprKind::If,
    )?;
    lower_stmt_block_with_async(
        &rewritten_body,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )
}

#[allow(clippy::too_many_arguments)]
fn lower_match_expr_stmt_with_async(
    value: &super::AstExpr,
    arms: &[AstMatchArm],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap_terminal: &dyn Fn(super::AstExpr) -> AstStmt,
) -> Result<NirStmt, String> {
    let rewritten_arms = arms
        .iter()
        .map(|arm| {
            Ok(AstMatchArm {
                pattern: arm.pattern.clone(),
                guard: arm.guard.clone(),
                body: rewrite_control_expr_terminal_branch(
                    &arm.body,
                    wrap_terminal,
                    ControlExprKind::Match,
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    lower_match_stmt_with_async(
        value,
        &rewritten_arms,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ControlExprKind {
    If,
    Match,
}

impl ControlExprKind {
    fn keyword(self) -> &'static str {
        match self {
            Self::If => "if",
            Self::Match => "match",
        }
    }

    fn branch_name(self) -> &'static str {
        match self {
            Self::If => "branch",
            Self::Match => "arm",
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_stmt_sequence_with_async(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    for kind in [ControlExprKind::If, ControlExprKind::Match] {
        if let Some(expanded) = expand_nested_control_expr_stmt(stmt, kind)? {
            return lower_expanded_stmt_sequence_with_async(
                stmt,
                expanded,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            );
        }
    }
    if let Some(expanded) = expand_try_stmt(
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )? {
        return lower_expanded_stmt_sequence_with_async(
            stmt,
            expanded,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        );
    }
    if let Some(expanded) = expand_nested_try_stmt(
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )? {
        return lower_expanded_stmt_sequence_with_async(
            stmt,
            expanded,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        );
    }
    if let AstStmt::DestructureLet {
        type_ref,
        fields,
        value,
    } = stmt
    {
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
        let final_type =
            resolve_declared_or_inferred("destructuring let source", expected, inferred)?;
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
        return Ok(lowered_stmts);
    }
    Ok(vec![lower_stmt_with_async(
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )?])
}

#[allow(clippy::too_many_arguments)]
fn lower_expanded_stmt_sequence_with_async(
    original_stmt: &AstStmt,
    expanded: Vec<AstStmt>,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    seed_expanded_stmt_bindings(original_stmt, bindings, type_aliases)?;
    let mut lowered = Vec::new();
    for stmt in expanded {
        lowered.extend(lower_stmt_sequence_with_async(
            &stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?);
    }
    Ok(lowered)
}

#[allow(clippy::too_many_arguments)]
fn expand_try_stmt(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<Vec<AstStmt>>, String> {
    let (inner, expansion) = match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value: super::AstExpr::Try(inner),
        } => (
            inner.as_ref(),
            TryConsumer::Let {
                mutable: *mutable,
                name: name.clone(),
                declared_ty: ty.clone(),
            },
        ),
        AstStmt::Const {
            name,
            ty,
            value: super::AstExpr::Try(inner),
        } => (
            inner.as_ref(),
            TryConsumer::Const {
                name: name.clone(),
                declared_ty: ty.clone(),
            },
        ),
        AstStmt::Print(super::AstExpr::Try(inner)) => (inner.as_ref(), TryConsumer::Print),
        AstStmt::Expr(super::AstExpr::Try(inner)) => (inner.as_ref(), TryConsumer::Expr),
        AstStmt::Return(Some(super::AstExpr::Try(inner))) => (inner.as_ref(), TryConsumer::Return),
        _ => return Ok(None),
    };

    let function_result_ty = current_function_result_type(return_type, type_aliases)?;
    let lowered_inner = lower_expr_with_async(
        inner,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
        false,
    )?;
    let inner_ty = infer_nir_expr_type(&lowered_inner, bindings, signatures, struct_table)
        .ok_or_else(|| "could not infer operand type for `?`".to_owned())?;
    let (payload_ty, error_ty) = split_result_type(&inner_ty)?;
    if !compatible_types(&function_result_ty.1, &error_ty) {
        return Err(format!(
            "`?` error type `{}` does not match enclosing function error type `{}`",
            error_ty.render(),
            function_result_ty.1.render()
        ));
    }

    let expansion = match expansion {
        TryConsumer::Let {
            mutable,
            name,
            declared_ty,
        } => {
            let final_payload_ty = match declared_ty {
                Some(declared_ty) => {
                    let lowered_declared = lower_type_ref_with_aliases(&declared_ty, type_aliases)?;
                    if !compatible_types(&lowered_declared, &payload_ty) {
                        return Err(format!(
                            "`?` payload type `{}` does not match declared type `{}` for `{}`",
                            payload_ty.render(),
                            lowered_declared.render(),
                            name
                        ));
                    }
                    ast_type_from_nir(&lowered_declared)
                }
                None => ast_type_from_nir(&payload_ty),
            };
            synthesize_try_statements(
                lowered_inner,
                inner_ty,
                AstStmt::Let {
                    mutable,
                    name,
                    ty: Some(final_payload_ty),
                    value: super::AstExpr::Var("__nuis_try_payload".to_owned()),
                },
            )
        }
        TryConsumer::Const { name, declared_ty } => {
            let final_payload_ty = match declared_ty {
                Some(declared_ty) => {
                    let lowered_declared = lower_type_ref_with_aliases(&declared_ty, type_aliases)?;
                    if !compatible_types(&lowered_declared, &payload_ty) {
                        return Err(format!(
                            "`?` payload type `{}` does not match declared type `{}` for `{}`",
                            payload_ty.render(),
                            lowered_declared.render(),
                            name
                        ));
                    }
                    ast_type_from_nir(&lowered_declared)
                }
                None => ast_type_from_nir(&payload_ty),
            };
            synthesize_try_statements(
                lowered_inner,
                inner_ty,
                AstStmt::Const {
                    name,
                    ty: Some(final_payload_ty),
                    value: super::AstExpr::Var("__nuis_try_payload".to_owned()),
                },
            )
        }
        TryConsumer::Print => synthesize_try_statements(
            lowered_inner,
            inner_ty,
            AstStmt::Print(super::AstExpr::Var("__nuis_try_payload".to_owned())),
        ),
        TryConsumer::Expr => synthesize_try_expr_statements(lowered_inner, inner_ty),
        TryConsumer::Return => synthesize_try_statements(
            lowered_inner,
            inner_ty,
            AstStmt::Return(Some(super::AstExpr::Var("__nuis_try_payload".to_owned()))),
        ),
    }?;

    Ok(Some(expansion))
}

#[allow(clippy::too_many_arguments)]
fn expand_nested_try_stmt(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<Vec<AstStmt>>, String> {
    match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::AssignLocal { name, value } => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::AssignLocal {
                name: name.clone(),
                value,
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Const { name, ty, value } => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Print(value) => expand_nested_try_expr_as_stmt(
            value,
            &AstStmt::Print,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Expr(value) => expand_nested_try_expr_as_stmt(
            value,
            &AstStmt::Expr,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Return(Some(value)) => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::Return(Some(value)),
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        _ => Ok(None),
    }
}

enum TryConsumer {
    Let {
        mutable: bool,
        name: String,
        declared_ty: Option<AstTypeRef>,
    },
    Const {
        name: String,
        declared_ty: Option<AstTypeRef>,
    },
    Print,
    Expr,
    Return,
}

fn current_function_result_type<'a>(
    return_type: Option<&'a AstTypeRef>,
    type_aliases: &'a BTreeMap<String, AstTypeAlias>,
) -> Result<(NirTypeRef, NirTypeRef), String> {
    let return_type = return_type.ok_or_else(|| {
        "`?` currently requires an enclosing function with explicit `Result<Payload, Error>` return type"
            .to_owned()
    })?;
    let lowered = lower_type_ref_with_aliases(return_type, type_aliases)?;
    let (payload, error) = split_result_type(&lowered)?;
    Ok((payload, error))
}

fn split_result_type(ty: &NirTypeRef) -> Result<(NirTypeRef, NirTypeRef), String> {
    if ty.name == "Result" && ty.generic_args.len() == 2 && !ty.is_ref && !ty.is_optional {
        return Ok((ty.generic_args[0].clone(), ty.generic_args[1].clone()));
    }
    Err(format!(
        "`?` currently requires a `Result<Payload, Error>` operand, found `{}`",
        ty.render()
    ))
}

fn synthesize_try_statements(
    lowered_inner: NirExpr,
    inner_ty: NirTypeRef,
    ok_terminal: AstStmt,
) -> Result<Vec<AstStmt>, String> {
    let id = TRY_EXPANSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let result_name = format!("__nuis_try_result_{id}");
    let payload_name = format!("__nuis_try_payload_{id}");
    let error_name = format!("__nuis_try_error_{id}");
    let result_ty = ast_type_from_nir(&inner_ty);
    let ok_stmt = rewrite_try_payload_placeholder(ok_terminal, &payload_name)?;

    Ok(vec![
        AstStmt::Let {
            mutable: false,
            name: result_name.clone(),
            ty: Some(result_ty),
            value: ast_expr_from_nir(lowered_inner),
        },
        AstStmt::Match {
            value: super::AstExpr::Var(result_name),
            arms: vec![
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Ok".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Bind(
                            payload_name,
                        )),
                    },
                    guard: None,
                    body: vec![ok_stmt],
                },
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Err".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Bind(
                            error_name.clone(),
                        )),
                    },
                    guard: None,
                    body: vec![AstStmt::Return(Some(super::AstExpr::Call {
                        callee: "Result.Err".to_owned(),
                        generic_args: Vec::new(),
                        args: vec![super::AstExpr::Var(error_name)],
                    }))],
                },
            ],
        },
    ])
}

fn synthesize_try_expr_statements(
    lowered_inner: NirExpr,
    inner_ty: NirTypeRef,
) -> Result<Vec<AstStmt>, String> {
    let id = TRY_EXPANSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let result_name = format!("__nuis_try_result_{id}");
    let error_name = format!("__nuis_try_error_{id}");
    Ok(vec![
        AstStmt::Let {
            mutable: false,
            name: result_name.clone(),
            ty: Some(ast_type_from_nir(&inner_ty)),
            value: ast_expr_from_nir(lowered_inner),
        },
        AstStmt::Match {
            value: super::AstExpr::Var(result_name),
            arms: vec![
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Ok".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Wildcard),
                    },
                    guard: None,
                    body: Vec::new(),
                },
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Err".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Bind(
                            error_name.clone(),
                        )),
                    },
                    guard: None,
                    body: vec![AstStmt::Return(Some(super::AstExpr::Call {
                        callee: "Result.Err".to_owned(),
                        generic_args: Vec::new(),
                        args: vec![super::AstExpr::Var(error_name)],
                    }))],
                },
            ],
        },
    ])
}

#[allow(clippy::too_many_arguments)]
fn synthesize_try_wrapped_statements(
    inner: &super::AstExpr,
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<AstStmt>, String> {
    let function_result_ty = current_function_result_type(return_type, type_aliases)?;
    let lowered_inner = lower_expr_with_async(
        inner,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
        false,
    )?;
    let inner_ty = infer_nir_expr_type(&lowered_inner, bindings, signatures, struct_table)
        .ok_or_else(|| "could not infer operand type for `?`".to_owned())?;
    let (_payload_ty, error_ty) = split_result_type(&inner_ty)?;
    if !compatible_types(&function_result_ty.1, &error_ty) {
        return Err(format!(
            "`?` error type `{}` does not match enclosing function error type `{}`",
            error_ty.render(),
            function_result_ty.1.render()
        ));
    }
    synthesize_try_statements(
        lowered_inner,
        inner_ty,
        wrap(super::AstExpr::Var("__nuis_try_payload".to_owned())),
    )
}

#[allow(clippy::too_many_arguments)]
fn expand_nested_try_expr_as_stmt(
    expr: &super::AstExpr,
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<Vec<AstStmt>>, String> {
    match expr {
        super::AstExpr::Try(inner) => Ok(Some(synthesize_try_wrapped_statements(
            inner,
            wrap,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?)),
        super::AstExpr::Await(value) => expand_nested_try_expr_as_stmt(
            value,
            &|rewritten| wrap(super::AstExpr::Await(Box::new(rewritten))),
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        super::AstExpr::Unary { op, operand } => expand_nested_try_expr_as_stmt(
            operand,
            &|rewritten| {
                wrap(super::AstExpr::Unary {
                    op: *op,
                    operand: Box::new(rewritten),
                })
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        super::AstExpr::Invoke { callee, args } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(
                callee,
                &|rewritten| {
                    wrap(super::AstExpr::Invoke {
                        callee: Box::new(rewritten),
                        args: args.clone(),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::Invoke {
                            callee: callee.clone(),
                            args: rewritten_args,
                        })
                    },
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::Call {
                            callee: callee.clone(),
                            generic_args: generic_args.clone(),
                            args: rewritten_args,
                        })
                    },
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(
                receiver,
                &|rewritten| {
                    wrap(super::AstExpr::MethodCall {
                        receiver: Box::new(rewritten),
                        method: method.clone(),
                        generic_args: generic_args.clone(),
                        args: args.clone(),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::MethodCall {
                            receiver: receiver.clone(),
                            method: method.clone(),
                            generic_args: generic_args.clone(),
                            args: rewritten_args,
                        })
                    },
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
                    value,
                    &|rewritten| {
                        let mut rewritten_fields = fields.clone();
                        rewritten_fields[index].1 = rewritten;
                        wrap(super::AstExpr::StructLiteral {
                            type_name: type_name.clone(),
                            type_args: type_args.clone(),
                            fields: rewritten_fields,
                        })
                    },
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::FieldAccess { base, field } => expand_nested_try_expr_as_stmt(
            base,
            &|rewritten| {
                wrap(super::AstExpr::FieldAccess {
                    base: Box::new(rewritten),
                    field: field.clone(),
                })
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        super::AstExpr::Binary { op, lhs, rhs } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(
                lhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: Box::new(rewritten),
                        rhs: rhs.clone(),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )? {
                return Ok(Some(expanded));
            }
            expand_nested_try_expr_as_stmt(
                rhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: lhs.clone(),
                        rhs: Box::new(rewritten),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )
        }
        super::AstExpr::Bool(_)
        | super::AstExpr::Text(_)
        | super::AstExpr::Int(_)
        | super::AstExpr::Float(_)
        | super::AstExpr::Var(_)
        | super::AstExpr::Lambda { .. }
        | super::AstExpr::Instantiate { .. }
        | super::AstExpr::If { .. }
        | super::AstExpr::Match { .. } => Ok(None),
    }
}

fn rewrite_try_payload_placeholder(stmt: AstStmt, payload_name: &str) -> Result<AstStmt, String> {
    match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } => Ok(AstStmt::Let {
            mutable,
            name,
            ty,
            value: rewrite_try_payload_placeholder_expr(value, payload_name),
        }),
        AstStmt::Const {
            name,
            ty,
            value,
        } => Ok(AstStmt::Const {
            name,
            ty,
            value: rewrite_try_payload_placeholder_expr(value, payload_name),
        }),
        AstStmt::Print(value) => Ok(AstStmt::Print(rewrite_try_payload_placeholder_expr(
            value,
            payload_name,
        ))),
        AstStmt::Expr(value) => Ok(AstStmt::Expr(rewrite_try_payload_placeholder_expr(
            value,
            payload_name,
        ))),
        AstStmt::Return(Some(value)) => Ok(AstStmt::Return(Some(
            rewrite_try_payload_placeholder_expr(value, payload_name),
        ))),
        other => Err(format!(
            "internal error: unsupported `?` payload rewrite target `{other:?}`"
        )),
    }
}

fn rewrite_try_payload_placeholder_expr(
    expr: super::AstExpr,
    payload_name: &str,
) -> super::AstExpr {
    match expr {
        super::AstExpr::Var(var) if var == "__nuis_try_payload" => {
            super::AstExpr::Var(payload_name.to_owned())
        }
        super::AstExpr::Await(value) => super::AstExpr::Await(Box::new(
            rewrite_try_payload_placeholder_expr(*value, payload_name),
        )),
        super::AstExpr::Try(value) => super::AstExpr::Try(Box::new(
            rewrite_try_payload_placeholder_expr(*value, payload_name),
        )),
        super::AstExpr::Call {
            callee,
            generic_args,
            args,
        } => super::AstExpr::Call {
            callee,
            generic_args,
            args: args
                .into_iter()
                .map(|arg| rewrite_try_payload_placeholder_expr(arg, payload_name))
                .collect(),
        },
        super::AstExpr::Invoke { callee, args } => super::AstExpr::Invoke {
            callee: Box::new(rewrite_try_payload_placeholder_expr(*callee, payload_name)),
            args: args
                .into_iter()
                .map(|arg| rewrite_try_payload_placeholder_expr(arg, payload_name))
                .collect(),
        },
        super::AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => super::AstExpr::MethodCall {
            receiver: Box::new(rewrite_try_payload_placeholder_expr(*receiver, payload_name)),
            method,
            generic_args,
            args: args
                .into_iter()
                .map(|arg| rewrite_try_payload_placeholder_expr(arg, payload_name))
                .collect(),
        },
        super::AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => super::AstExpr::StructLiteral {
            type_name,
            type_args,
            fields: fields
                .into_iter()
                .map(|(field, value)| {
                    (
                        field,
                        rewrite_try_payload_placeholder_expr(value, payload_name),
                    )
                })
                .collect(),
        },
        super::AstExpr::FieldAccess { base, field } => super::AstExpr::FieldAccess {
            base: Box::new(rewrite_try_payload_placeholder_expr(*base, payload_name)),
            field,
        },
        super::AstExpr::Unary { op, operand } => super::AstExpr::Unary {
            op,
            operand: Box::new(rewrite_try_payload_placeholder_expr(*operand, payload_name)),
        },
        super::AstExpr::Binary { op, lhs, rhs } => super::AstExpr::Binary {
            op,
            lhs: Box::new(rewrite_try_payload_placeholder_expr(*lhs, payload_name)),
            rhs: Box::new(rewrite_try_payload_placeholder_expr(*rhs, payload_name)),
        },
        other => other,
    }
}

fn ast_expr_from_nir(expr: NirExpr) -> super::AstExpr {
    match expr {
        NirExpr::Bool(value) => super::AstExpr::Bool(value),
        NirExpr::Text(text) => super::AstExpr::Text(text),
        NirExpr::Int(value) => super::AstExpr::Int(value),
        NirExpr::F32(value) | NirExpr::F64(value) => super::AstExpr::Float(value),
        NirExpr::Var(name) => super::AstExpr::Var(name),
        NirExpr::Await(value) => super::AstExpr::Await(Box::new(ast_expr_from_nir(*value))),
        NirExpr::Call { callee, args } => super::AstExpr::Call {
            callee,
            generic_args: Vec::new(),
            args: args.into_iter().map(ast_expr_from_nir).collect(),
        },
        NirExpr::FieldAccess { base, field } => super::AstExpr::FieldAccess {
            base: Box::new(ast_expr_from_nir(*base)),
            field,
        },
        NirExpr::Binary { op, lhs, rhs } => super::AstExpr::Binary {
            op: match op {
                nuis_semantics::model::NirBinaryOp::And => nuis_semantics::model::AstBinaryOp::And,
                nuis_semantics::model::NirBinaryOp::Or => nuis_semantics::model::AstBinaryOp::Or,
                nuis_semantics::model::NirBinaryOp::Add => nuis_semantics::model::AstBinaryOp::Add,
                nuis_semantics::model::NirBinaryOp::Sub => nuis_semantics::model::AstBinaryOp::Sub,
                nuis_semantics::model::NirBinaryOp::Mul => nuis_semantics::model::AstBinaryOp::Mul,
                nuis_semantics::model::NirBinaryOp::Div => nuis_semantics::model::AstBinaryOp::Div,
                nuis_semantics::model::NirBinaryOp::Rem => nuis_semantics::model::AstBinaryOp::Rem,
                nuis_semantics::model::NirBinaryOp::Eq => nuis_semantics::model::AstBinaryOp::Eq,
                nuis_semantics::model::NirBinaryOp::Ne => nuis_semantics::model::AstBinaryOp::Ne,
                nuis_semantics::model::NirBinaryOp::Lt => nuis_semantics::model::AstBinaryOp::Lt,
                nuis_semantics::model::NirBinaryOp::Le => nuis_semantics::model::AstBinaryOp::Le,
                nuis_semantics::model::NirBinaryOp::Gt => nuis_semantics::model::AstBinaryOp::Gt,
                nuis_semantics::model::NirBinaryOp::Ge => nuis_semantics::model::AstBinaryOp::Ge,
            },
            lhs: Box::new(ast_expr_from_nir(*lhs)),
            rhs: Box::new(ast_expr_from_nir(*rhs)),
        },
        other => panic!("internal error: unsupported NIR-to-AST try expansion expr {other:?}"),
    }
}

fn seed_expanded_stmt_bindings(
    stmt: &AstStmt,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let {
            name, ty: Some(ty), ..
        }
        | AstStmt::Const {
            name, ty: Some(ty), ..
        } => {
            let lowered_ty = lower_type_ref_with_aliases(ty, type_aliases)?;
            validate_type_ref(&lowered_ty)?;
            bindings.insert(name.clone(), lowered_ty);
        }
        _ => {}
    }
    Ok(())
}

fn expand_nested_control_expr_stmt(
    stmt: &AstStmt,
    kind: ControlExprKind,
) -> Result<Option<Vec<AstStmt>>, String> {
    match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            kind,
            false,
        ),
        AstStmt::AssignLocal { name, value } => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::AssignLocal {
                name: name.clone(),
                value,
            },
            kind,
            false,
        ),
        AstStmt::Const { name, ty, value } => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            kind,
            false,
        ),
        AstStmt::Print(value) => {
            expand_nested_control_expr_as_stmt(value, &AstStmt::Print, kind, false)
        }
        AstStmt::Expr(value) => {
            expand_nested_control_expr_as_stmt(value, &AstStmt::Expr, kind, false)
        }
        AstStmt::Return(Some(value)) => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::Return(Some(value)),
            kind,
            false,
        ),
        _ => Ok(None),
    }
}

fn expand_nested_control_expr_as_stmt(
    expr: &super::AstExpr,
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    kind: ControlExprKind,
    allow_root_control: bool,
) -> Result<Option<Vec<AstStmt>>, String> {
    if allow_root_control {
        match (kind, expr) {
            (
                ControlExprKind::If,
                super::AstExpr::If {
                    condition,
                    then_body,
                    else_body,
                },
            ) => {
                return Ok(Some(vec![AstStmt::If {
                    condition: *condition.clone(),
                    then_body: rewrite_control_expr_terminal_branch(then_body, wrap, kind)?,
                    else_body: rewrite_control_expr_terminal_branch(else_body, wrap, kind)?,
                }]));
            }
            (ControlExprKind::Match, super::AstExpr::Match { value, arms }) => {
                return Ok(Some(vec![AstStmt::Match {
                    value: *value.clone(),
                    arms: arms
                        .iter()
                        .map(|arm| {
                            Ok(AstMatchArm {
                                pattern: arm.pattern.clone(),
                                guard: arm.guard.clone(),
                                body: rewrite_control_expr_terminal_branch(&arm.body, wrap, kind)?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                }]));
            }
            _ => {}
        }
    }

    match expr {
        super::AstExpr::If { .. } if kind == ControlExprKind::If => Ok(None),
        super::AstExpr::Match { .. } if kind == ControlExprKind::Match => Ok(None),
        super::AstExpr::Await(value) => expand_nested_control_expr_as_stmt(
            value,
            &|rewritten| wrap(super::AstExpr::Await(Box::new(rewritten))),
            kind,
            true,
        ),
        super::AstExpr::Try(value) => expand_nested_control_expr_as_stmt(
            value,
            &|rewritten| wrap(super::AstExpr::Try(Box::new(rewritten))),
            kind,
            true,
        ),
        super::AstExpr::Unary { op, operand } => expand_nested_control_expr_as_stmt(
            operand,
            &|rewritten| {
                wrap(super::AstExpr::Unary {
                    op: *op,
                    operand: Box::new(rewritten),
                })
            },
            kind,
            true,
        ),
        super::AstExpr::Invoke { callee, args } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                callee,
                &|rewritten| {
                    wrap(super::AstExpr::Invoke {
                        callee: Box::new(rewritten),
                        args: args.clone(),
                    })
                },
                kind,
                true,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::Invoke {
                            callee: callee.clone(),
                            args: rewritten_args,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::Call {
                            callee: callee.clone(),
                            generic_args: generic_args.clone(),
                            args: rewritten_args,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                receiver,
                &|rewritten| {
                    wrap(super::AstExpr::MethodCall {
                        receiver: Box::new(rewritten),
                        method: method.clone(),
                        generic_args: generic_args.clone(),
                        args: args.clone(),
                    })
                },
                kind,
                true,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::MethodCall {
                            receiver: receiver.clone(),
                            method: method.clone(),
                            generic_args: generic_args.clone(),
                            args: rewritten_args,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    value,
                    &|rewritten| {
                        let mut rewritten_fields = fields.clone();
                        rewritten_fields[index].1 = rewritten;
                        wrap(super::AstExpr::StructLiteral {
                            type_name: type_name.clone(),
                            type_args: type_args.clone(),
                            fields: rewritten_fields,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::FieldAccess { base, field } => expand_nested_control_expr_as_stmt(
            base,
            &|rewritten| {
                wrap(super::AstExpr::FieldAccess {
                    base: Box::new(rewritten),
                    field: field.clone(),
                })
            },
            kind,
            true,
        ),
        super::AstExpr::Binary { op, lhs, rhs } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                lhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: Box::new(rewritten),
                        rhs: rhs.clone(),
                    })
                },
                kind,
                true,
            )? {
                return Ok(Some(expanded));
            }
            expand_nested_control_expr_as_stmt(
                rhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: lhs.clone(),
                        rhs: Box::new(rewritten),
                    })
                },
                kind,
                true,
            )
        }
        super::AstExpr::Bool(_)
        | super::AstExpr::Text(_)
        | super::AstExpr::Int(_)
        | super::AstExpr::Float(_)
        | super::AstExpr::Var(_)
        | super::AstExpr::Lambda { .. }
        | super::AstExpr::Instantiate { .. } => Ok(None),
        super::AstExpr::If { .. } | super::AstExpr::Match { .. } => Ok(None),
    }
}

fn rewrite_control_expr_terminal_branch(
    body: &[AstStmt],
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    kind: ControlExprKind,
) -> Result<Vec<AstStmt>, String> {
    let Some((last, prefix)) = body.split_last() else {
        return Err(format!(
            "`{}` expression {} cannot be empty",
            kind.keyword(),
            kind.branch_name()
        ));
    };
    let mut rewritten = prefix.to_vec();
    match last {
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            rewritten.push(AstStmt::If {
                condition: condition.clone(),
                then_body: rewrite_control_expr_terminal_branch(then_body, wrap, ControlExprKind::If)?,
                else_body: rewrite_control_expr_terminal_branch(else_body, wrap, ControlExprKind::If)?,
            });
            return Ok(rewritten);
        }
        AstStmt::Match { value, arms } => {
            rewritten.push(AstStmt::Match {
                value: value.clone(),
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm.guard.clone(),
                            body: rewrite_control_expr_terminal_branch(
                                &arm.body,
                                wrap,
                                ControlExprKind::Match,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            });
            return Ok(rewritten);
        }
        AstStmt::Return(Some(value)) | AstStmt::Expr(value) => {
            for root_kind in [ControlExprKind::If, ControlExprKind::Match] {
                if let Some(expanded) =
                    expand_nested_control_expr_as_stmt(value, wrap, root_kind, true)?
                {
                    rewritten.extend(expanded);
                    return Ok(rewritten);
                }
            }
            rewritten.push(wrap(value.clone()));
            Ok(rewritten)
        }
        _ => Err(format!(
            "`{}` expression {} currently requires a tail expression result in each {}",
            kind.keyword(),
            kind.branch_name(),
            kind.branch_name()
        )),
    }
}

fn emit_destructure_bindings(
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

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_stmt_block_with_async(
    stmts: &[AstStmt],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirStmt>, String> {
    let mut lowered = Vec::new();
    for stmt in stmts {
        lowered.extend(lower_stmt_sequence_with_async(
            stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?);
    }
    Ok(lowered)
}

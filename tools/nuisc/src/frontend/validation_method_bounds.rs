use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstImplDef, AstMatchArm, AstModule, AstParam, AstStmt, AstStructDef,
    AstTraitDef, AstTypeAlias, AstTypeRef, AstUnaryOp,
};

use super::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type, simple_match_value_type,
};
use super::{
    infer_ast_expr_type, lower_type_ref, name_suggestions::suggest_similar_name,
    resolve_ast_type_ref_aliases, substitute_ast_type_alias_target,
};

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
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match expr {
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_) => {}
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut then_env = local_type_env.clone();
            let mut else_env = local_type_env.clone();
            validate_stmt_generic_method_bounds_block(
                then_body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut then_env,
                &format!("{context} if-then"),
            )?;
            validate_stmt_generic_method_bounds_block(
                else_body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut else_env,
                &format!("{context} if-else"),
            )?;
        }
        AstExpr::Match { value, arms } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let match_value_ty = simple_match_value_type(value, local_type_env);
            for arm in arms {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        &arm.pattern,
                        visible_type_aliases,
                        visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = &arm.guard {
                    validate_expr_generic_method_bounds(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                }
                validate_stmt_generic_method_bounds_block(
                    &arm.body,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    &mut arm_env,
                    &format!("{context} match-arm"),
                )?;
            }
        }
        AstExpr::Lambda {
            params,
            body,
            return_type: _,
        } => {
            let mut lambda_env = local_type_env.clone();
            for AstParam { name, ty } in params {
                lambda_env.insert(name.clone(), ty.clone());
            }
            validate_stmt_generic_method_bounds_block(
                body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut lambda_env,
                &format!("{context} lambda body"),
            )?;
        }
        AstExpr::Instantiate { .. } => {}
        AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Unary { op, operand } => {
            validate_expr_generic_method_bounds(
                operand,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some((operator, method, required_bound)) = unary_operator_trait_requirement(*op)
            {
                if let Some(operand_ty) = infer_ast_expr_type(
                    operand,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                ) {
                    validate_generic_receiver_operator_bound(
                        &operand_ty,
                        operator,
                        method,
                        required_bound,
                        visible_type_aliases,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        context,
                    )?;
                }
            }
        }
        AstExpr::Call { callee, args, .. } => {
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
            if let Some((trait_name, method)) = callee.rsplit_once('.') {
                if trait_methods.contains_key(trait_name) {
                    validate_explicit_trait_call_bound(
                        trait_name,
                        method,
                        args,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        local_type_env,
                        context,
                    )?;
                }
            }
        }
        AstExpr::Invoke { args, .. } => {
            for arg in args {
                validate_expr_generic_method_bounds(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
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
            if let Some(receiver_name) = super::render_field_access_path(receiver) {
                let is_shadowed_simple_local = matches!(
                    receiver.as_ref(),
                    AstExpr::Var(name) if local_type_env.contains_key(name)
                );
                if !is_shadowed_simple_local && trait_methods.contains_key(&receiver_name) {
                    for arg in args {
                        validate_expr_generic_method_bounds(
                            arg,
                            visible_type_aliases,
                            impl_lookup,
                            visible_structs,
                            function_return_types,
                            trait_methods,
                            generic_param_names,
                            generic_bounds,
                            local_type_env,
                            context,
                        )?;
                    }
                    validate_explicit_trait_call_bound(
                        &receiver_name,
                        method,
                        args,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        local_type_env,
                        context,
                    )?;
                    return Ok(());
                }
            }
            validate_expr_generic_method_bounds(
                receiver,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
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
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
            if let Some(receiver_ty) = infer_ast_expr_type(
                receiver,
                local_type_env,
                impl_lookup,
                visible_structs,
                function_return_types,
            ) {
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
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Binary { op, lhs, rhs } => {
            validate_expr_generic_method_bounds(
                lhs,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                rhs,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some((operator, method, required_bound)) = binary_operator_trait_requirement(*op)
            {
                if let Some(lhs_ty) = infer_ast_expr_type(
                    lhs,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                ) {
                    validate_generic_receiver_operator_bound(
                        &lhs_ty,
                        operator,
                        method,
                        required_bound,
                        visible_type_aliases,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        context,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn unary_operator_trait_requirement(
    op: AstUnaryOp,
) -> Option<(&'static str, &'static str, &'static str)> {
    match op {
        AstUnaryOp::Not => Some(("!", "not", "Notable")),
        AstUnaryOp::Neg => Some(("-", "neg", "Negatable")),
        AstUnaryOp::Deref => None,
    }
}

fn binary_operator_trait_requirement(
    op: AstBinaryOp,
) -> Option<(&'static str, &'static str, &'static str)> {
    match op {
        AstBinaryOp::Add => Some(("+", "add", "Addable")),
        AstBinaryOp::Sub => Some(("-", "sub", "Subtractable")),
        AstBinaryOp::Mul => Some(("*", "mul", "Multipliable")),
        AstBinaryOp::Div => Some(("/", "div", "Dividable")),
        AstBinaryOp::Rem => Some(("%", "rem", "Remainderable")),
        AstBinaryOp::Eq => Some(("==", "eq", "Equatable")),
        AstBinaryOp::Ne => Some(("!=", "eq", "Equatable")),
        AstBinaryOp::Lt => Some(("<", "lt", "Orderable")),
        AstBinaryOp::Le => Some(("<=", "le", "Orderable")),
        AstBinaryOp::Gt => Some((">", "gt", "Orderable")),
        AstBinaryOp::Ge => Some((">=", "ge", "Orderable")),
        _ => None,
    }
}

fn validate_stmt_generic_method_bounds_block(
    body: &[AstStmt],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    for stmt in body {
        validate_stmt_generic_method_bounds(
            stmt,
            visible_type_aliases,
            impl_lookup,
            visible_structs,
            function_return_types,
            trait_methods,
            generic_param_names,
            generic_bounds,
            local_type_env,
            context,
        )?;
    }
    Ok(())
}

fn validate_stmt_generic_method_bounds(
    stmt: &AstStmt,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let { name, ty, value, .. } | AstStmt::Const { name, ty, value } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(ty) = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                )
            }) {
                local_type_env.insert(name.clone(), ty);
            }
        }
        AstStmt::AssignLocal { name, value } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(ty) = local_type_env.get(name).cloned() {
                local_type_env.insert(name.clone(), ty);
            }
        }
        AstStmt::DestructureLet { value, .. } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let AstStmt::DestructureLet {
                type_ref, fields, ..
            } = stmt
            else {
                unreachable!();
            };
            let root_type = type_ref.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                )
            });
            if let Some(root_type) = root_type.as_ref() {
                bind_destructure_fields_for_type(
                    root_type,
                    fields,
                    visible_type_aliases,
                    visible_structs,
                    local_type_env,
                )?;
            }
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut then_env = local_type_env.clone();
            validate_stmt_generic_method_bounds_block(
                then_body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut then_env,
                context,
            )?;
            let mut else_env = local_type_env.clone();
            validate_stmt_generic_method_bounds_block(
                else_body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut else_env,
                context,
            )?;
        }
        AstStmt::Match { value, arms } => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let match_value_ty = simple_match_value_type(value, local_type_env);
            for AstMatchArm {
                pattern,
                guard,
                body,
            } in arms
            {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        pattern,
                        visible_type_aliases,
                        visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = guard {
                    validate_expr_generic_method_bounds(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        trait_methods,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                }
                validate_stmt_generic_method_bounds_block(
                    body,
                    visible_type_aliases,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                    trait_methods,
                    generic_param_names,
                    generic_bounds,
                    &mut arm_env,
                    context,
                )?;
            }
        }
        AstStmt::While { condition, body } => {
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut loop_env = local_type_env.clone();
            validate_stmt_generic_method_bounds_block(
                body,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                &mut loop_env,
                context,
            )?;
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstStmt::Return(Some(value)) => {
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
    }
    Ok(())
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
        if let Some(suggested_method) = trait_methods
            .get(bound)
            .and_then(|methods| suggest_trait_method_name(method, methods))
        {
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{bound}` does not define that method; did you mean `{}`?",
                suggested_method
            ));
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

#[allow(clippy::too_many_arguments)]
fn validate_generic_receiver_operator_bound(
    receiver_ty: &AstTypeRef,
    operator: &str,
    method: &str,
    required_bound: &str,
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

    if let Some(bound) = generic_bounds.get(&generic_name) {
        if bound == required_bound
            && trait_methods
                .get(bound)
                .is_some_and(|methods| methods.contains(method))
        {
            return Ok(());
        }
        return Err(format!(
            "{context} calls operator `{operator}` on generic parameter `{generic_name}` but bound `{bound}` does not satisfy required trait `{required_bound}`"
        ));
    }

    Err(format!(
        "{context} calls operator `{operator}` on generic parameter `{generic_name}` without required bound `{required_bound}`"
    ))
}

#[allow(clippy::too_many_arguments)]
fn validate_explicit_trait_call_bound(
    trait_name: &str,
    method: &str,
    args: &[AstExpr],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    let Some(receiver) = args.first() else {
        return Ok(());
    };
    if !trait_methods
        .get(trait_name)
        .is_some_and(|methods| methods.contains(method))
    {
        if let Some(suggested_method) = trait_methods
            .get(trait_name)
            .and_then(|methods| suggest_trait_method_name(method, methods))
        {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}`, but trait `{trait_name}` does not define method `{method}`; did you mean `{}.{}`?",
                trait_name, suggested_method
            ));
        }
        let variants = collect_trait_name_variants(trait_name, trait_methods);
        if variants.len() == 1
            && trait_methods
                .get(&variants[0])
                .is_some_and(|methods| methods.contains(method))
        {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}`, but trait `{trait_name}` does not define method `{method}`; did you mean `{}`?",
                format!("{}.{}", variants[0], method)
            ));
        }
        return Err(format!(
            "{context} calls trait method `{trait_name}.{method}`, but trait `{trait_name}` does not define method `{method}`"
        ));
    }
    let Some(receiver_ty) = infer_ast_expr_type(
        receiver,
        local_type_env,
        impl_lookup,
        visible_structs,
        function_return_types,
    ) else {
        return Ok(());
    };
    let receiver_rendered = lower_type_ref(&receiver_ty).render();
    let Some((generic_name, receiver_context)) = resolve_generic_receiver_context(
        &receiver_ty,
        visible_type_aliases,
        generic_param_names,
        &mut BTreeSet::new(),
    )?
    else {
        if impl_lookup.contains_key(&(trait_name.to_owned(), receiver_rendered.clone())) {
            return Ok(());
        }
        let available_impls =
            collect_receiver_trait_impl_candidates(&receiver_rendered, impl_lookup);
        if available_impls.is_empty() {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}` for `{receiver_rendered}`, but trait `{trait_name}` has no impl for `{receiver_rendered}`"
            ));
        }
        return Err(format!(
            "{context} calls trait method `{trait_name}.{method}` for `{receiver_rendered}`, but trait `{trait_name}` has no impl for `{receiver_rendered}`; available trait impls for `{receiver_rendered}`: {}",
            available_impls.join(", ")
        ));
    };
    let context = format!("{context}{receiver_context}");

    if let Some(bound) = generic_bounds.get(&generic_name) {
        if bound == trait_name {
            return Ok(());
        }
        let variants = collect_trait_name_variants(trait_name, trait_methods);
        if variants.iter().any(|candidate| candidate == bound) {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` but bound `{bound}` uses a different visible name for the same trait; use `{trait_name}` consistently"
            ));
        }
        return Err(format!(
            "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` but bound `{bound}` does not satisfy required trait `{trait_name}`"
        ));
    }

    Err(format!(
        "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` without required bound `{trait_name}`"
    ))
}

fn collect_trait_name_variants(
    trait_name: &str,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<String> {
    let short_name = trait_name.rsplit('.').next().unwrap_or(trait_name);
    trait_methods
        .keys()
        .filter(|candidate| candidate.as_str() != trait_name)
        .filter(|candidate| {
            candidate
                .rsplit('.')
                .next()
                .is_some_and(|name| name == short_name)
        })
        .cloned()
        .collect()
}

fn collect_receiver_trait_impl_candidates(
    receiver_rendered: &str,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
) -> Vec<String> {
    impl_lookup
        .keys()
        .filter(|(_, for_type)| for_type == receiver_rendered)
        .map(|(trait_name, _)| trait_name.clone())
        .collect()
}

fn suggest_trait_method_name(method: &str, methods: &BTreeSet<String>) -> Option<String> {
    suggest_similar_name(method, methods)
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

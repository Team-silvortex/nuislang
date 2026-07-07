use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstImplDef, AstStmt, AstStructDef, AstTypeRef, AstUnaryOp,
};

use super::ast_calls::infer_ast_call_type;
use super::ast_calls_views::AstCallInferenceInput;
use super::ast_patterns::{
    infer_struct_literal_ast_type_seeded, type_args_are_pattern_placeholders,
    SeededStructLiteralAstTypeInput,
};
use super::{ast_generic_named_type, ast_named_type, impl_lookup_types};
use crate::frontend::validation_binding_env::instantiate_ast_struct_field_type;
use crate::frontend::{lower_type_ref, render_field_access_path};

pub(crate) fn infer_ast_expr_type(
    expr: &AstExpr,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    infer_ast_expr_type_inner(
        expr,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &mut BTreeSet::new(),
    )
}

pub(crate) fn infer_ast_expr_type_inner(
    expr: &AstExpr,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let expr_key = expr as *const AstExpr as usize;
    if !active_exprs.insert(expr_key) {
        return None;
    }
    let inferred = match expr {
        AstExpr::Bool(_) => Some(ast_named_type("bool")),
        AstExpr::Text(_) => Some(ast_named_type("String")),
        AstExpr::Int(_) => Some(ast_named_type("i64")),
        AstExpr::Float(_) => Some(ast_named_type("f64")),
        AstExpr::Var(name) => env.get(name).cloned(),
        AstExpr::If {
            condition: _,
            then_body,
            else_body,
        } => {
            let then_ty = infer_ast_block_result_type(
                then_body,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            let else_ty = infer_ast_block_result_type(
                else_body,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if then_ty == else_ty {
                Some(then_ty)
            } else {
                None
            }
        }
        AstExpr::Match { value: _, arms } => {
            let mut arm_ty = None;
            for arm in arms {
                let current = infer_ast_block_result_type(
                    &arm.body,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                match &arm_ty {
                    Some(existing) if *existing != current => return None,
                    None => arm_ty = Some(current),
                    _ => {}
                }
            }
            arm_ty
        }
        AstExpr::Lambda { .. } => None,
        AstExpr::Invoke { .. } => None,
        AstExpr::Await(value) => infer_ast_expr_type_inner(
            value,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        )
        .and_then(|ty| {
            if ty.name == "Task" && ty.generic_args.len() == 1 {
                ty.generic_args.first().cloned()
            } else {
                Some(ty)
            }
        }),
        AstExpr::Try(value) => infer_ast_expr_type_inner(
            value,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        )
        .and_then(|ty| {
            if ty.name == "Result" && ty.generic_args.len() == 2 {
                ty.generic_args.first().cloned()
            } else {
                None
            }
        }),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => infer_ast_call_type(AstCallInferenceInput {
            callee,
            generic_args,
            args,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        }),
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args: _,
            args,
        } => {
            if let Some(trait_name) = render_field_access_path(receiver) {
                if let Some(receiver_arg) = args.first() {
                    let receiver_ty = infer_ast_expr_type_inner(
                        receiver_arg,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        active_exprs,
                    )?;
                    for rendered_receiver_ty in impl_lookup_types(&receiver_ty) {
                        if let Some(definition) =
                            impl_lookup.get(&(trait_name.clone(), rendered_receiver_ty))
                        {
                            if let Some(method_def) =
                                definition.methods.iter().find(|item| item.name == *method)
                            {
                                return method_def
                                    .return_type
                                    .clone()
                                    .or_else(|| Some(receiver_ty.clone()));
                            }
                        }
                    }
                }
            }
            let receiver_ty = infer_ast_expr_type_inner(
                receiver,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            for candidate_ty in impl_lookup_types(&receiver_ty) {
                for ((_, for_type), definition) in impl_lookup {
                    if *for_type != candidate_ty {
                        continue;
                    }
                    if let Some(method_def) =
                        definition.methods.iter().find(|item| item.name == *method)
                    {
                        return method_def
                            .return_type
                            .clone()
                            .or_else(|| Some(receiver_ty.clone()));
                    }
                }
            }
            None
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let definition = struct_table.get(type_name)?;
            let placeholder_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            if definition.generic_params.is_empty() {
                Some(ast_named_type(type_name))
            } else if type_args.len() == definition.generic_params.len()
                && !type_args_are_pattern_placeholders(type_args, &placeholder_names)
            {
                Some(ast_generic_named_type(type_name, type_args.clone()))
            } else {
                infer_struct_literal_ast_type(
                    type_name,
                    fields,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )
            }
        }
        AstExpr::FieldAccess { base, field } => {
            if let Some(base_path) = render_field_access_path(base) {
                let qualified_name = format!("{base_path}.{field}");
                if struct_table
                    .get(&qualified_name)
                    .is_some_and(|definition| definition.fields.is_empty())
                {
                    return Some(ast_named_type(&qualified_name));
                }
            }
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Node" {
                return match field.as_str() {
                    "value" => Some(ast_named_type("i64")),
                    "next" => Some(AstTypeRef {
                        name: "Node".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: true,
                    }),
                    _ => None,
                };
            }
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Buffer" {
                return match field.as_str() {
                    "len" => Some(ast_named_type("i64")),
                    _ => None,
                };
            }
            if !base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Slice" {
                return match field.as_str() {
                    "buffer" => Some(AstTypeRef {
                        name: "Buffer".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: true,
                    }),
                    "start" | "len" => Some(ast_named_type("i64")),
                    _ => None,
                };
            }
            if !base_ty.is_ref && !base_ty.is_optional && base_ty.name == "ByteSplit" {
                return match field.as_str() {
                    "before" | "after" => {
                        Some(ast_generic_named_type("Slice", vec![ast_named_type("i64")]))
                    }
                    "index" => Some(ast_named_type("i64")),
                    "found" => Some(ast_named_type("bool")),
                    _ => None,
                };
            }
            let definition = struct_table.get(&base_ty.name)?;
            definition
                .fields
                .iter()
                .find(|item| item.name == *field)
                .map(|field| instantiate_ast_struct_field_type(&base_ty, definition, &field.ty))
        }
        AstExpr::Unary { op, operand } => match op {
            AstUnaryOp::Not => Some(ast_named_type("bool")),
            AstUnaryOp::Neg => infer_ast_expr_type_inner(
                operand,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            ),
            AstUnaryOp::Deref => {
                let operand_ty = infer_ast_expr_type_inner(
                    operand,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if operand_ty.is_ref && !operand_ty.is_optional && operand_ty.name == "Node" {
                    Some(ast_named_type("i64"))
                } else {
                    None
                }
            }
        },
        AstExpr::Binary { op, lhs, rhs } => match op {
            AstBinaryOp::Eq
            | AstBinaryOp::Ne
            | AstBinaryOp::Lt
            | AstBinaryOp::Le
            | AstBinaryOp::Gt
            | AstBinaryOp::Ge => Some(ast_named_type("bool")),
            AstBinaryOp::And | AstBinaryOp::Or => {
                let lhs_ty = infer_ast_expr_type_inner(
                    lhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                let rhs_ty = infer_ast_expr_type_inner(
                    rhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if lower_type_ref(&lhs_ty).render() == lower_type_ref(&rhs_ty).render()
                    && lhs_ty.name == "bool"
                    && rhs_ty.name == "bool"
                {
                    Some(ast_named_type("bool"))
                } else {
                    None
                }
            }
            AstBinaryOp::Add
            | AstBinaryOp::Sub
            | AstBinaryOp::Mul
            | AstBinaryOp::Div
            | AstBinaryOp::Rem => {
                let lhs_ty = infer_ast_expr_type_inner(
                    lhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                let rhs_ty = infer_ast_expr_type_inner(
                    rhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if lower_type_ref(&lhs_ty).render() == lower_type_ref(&rhs_ty).render() {
                    Some(lhs_ty)
                } else {
                    None
                }
            }
        },
        AstExpr::Instantiate { .. } => None,
    };
    active_exprs.remove(&expr_key);
    inferred
}

fn infer_ast_block_result_type(
    body: &[AstStmt],
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    match body.last() {
        Some(AstStmt::Return(Some(expr))) => infer_ast_expr_type_inner(
            expr,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        ),
        _ => None,
    }
}

fn infer_struct_literal_ast_type(
    type_name: &str,
    fields: &[(String, AstExpr)],
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let definition = struct_table.get(type_name)?;
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_struct_literal_ast_type_seeded(SeededStructLiteralAstTypeInput {
        type_name,
        definition,
        fields,
        generic_names: &generic_names,
        substitutions: BTreeMap::new(),
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    })
}

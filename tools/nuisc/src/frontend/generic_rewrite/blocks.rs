use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::validation_binding_env::bind_match_pattern_for_type;
use super::super::{ast_named_type, FunctionSignature};
use super::blocks_expected::{let_binding_expected_type_from_following_use, ExpectedBlockType};
use super::blocks_hoists::{rewrite_generic_stmt_with_hoists, GenericStmtHoistRewriteInput};
use super::exprs::{rewrite_generic_calls_in_expr, GenericExprRewriteInput};
use super::GenericImplMethodTemplate;

pub(super) struct GenericBlockRewriteInput<'a> {
    pub(super) body: &'a [AstStmt],
    pub(super) context: &'a str,
    pub(super) current_return_type: Option<&'a AstTypeRef>,
    pub(super) env: &'a mut BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) generic_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) generic_impl_method_templates: &'a [GenericImplMethodTemplate],
    pub(super) higher_order_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) function_table: &'a BTreeMap<String, AstFunction>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) specialization_cache: &'a mut BTreeSet<String>,
    pub(super) specialized_functions: &'a mut Vec<AstFunction>,
    pub(super) specialized_signatures: &'a mut Vec<(String, FunctionSignature)>,
}

pub(super) struct GenericMatchArmsRewriteInput<'a> {
    pub(super) arms: &'a [AstMatchArm],
    pub(super) context: &'a str,
    pub(super) scrutinee_type: Option<&'a AstTypeRef>,
    pub(super) current_return_type: Option<&'a AstTypeRef>,
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) generic_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) generic_impl_method_templates: &'a [GenericImplMethodTemplate],
    pub(super) higher_order_templates: &'a BTreeMap<String, AstFunction>,
    pub(super) function_table: &'a BTreeMap<String, AstFunction>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) specialization_cache: &'a mut BTreeSet<String>,
    pub(super) specialized_functions: &'a mut Vec<AstFunction>,
    pub(super) specialized_signatures: &'a mut Vec<(String, FunctionSignature)>,
}

pub(super) fn rewrite_generic_calls_in_block(
    input: GenericBlockRewriteInput<'_>,
) -> Result<Vec<AstStmt>, String> {
    rewrite_generic_calls_in_block_inner(input, false)
}

pub(super) fn rewrite_generic_calls_in_result_block(
    input: GenericBlockRewriteInput<'_>,
) -> Result<Vec<AstStmt>, String> {
    rewrite_generic_calls_in_block_inner(input, true)
}

fn rewrite_generic_calls_in_block_inner(
    input: GenericBlockRewriteInput<'_>,
    result_context: bool,
) -> Result<Vec<AstStmt>, String> {
    let GenericBlockRewriteInput {
        body,
        context,
        current_return_type,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    } = input;
    let mut rewritten = Vec::new();
    for (index, stmt) in body.iter().enumerate() {
        // Expected initializer types flow only into the result tail, not into
        // arbitrary expression statements or a synthetic function return.
        if result_context && index + 1 == body.len() {
            if let AstStmt::Expr(expr) = stmt {
                if matches!(expr, nuis_semantics::model::AstExpr::Call { callee, .. } if generic_templates.contains_key(callee))
                {
                    rewritten.extend(rewrite_generic_stmt_with_hoists(
                        GenericStmtHoistRewriteInput {
                            stmt,
                            context,
                            let_fallback_expected: None,
                            current_return_type,
                            result_context: true,
                            env,
                            visible_type_aliases,
                            generic_templates,
                            generic_impl_method_templates,
                            higher_order_templates,
                            function_table,
                            signatures,
                            impl_lookup,
                            struct_table,
                            function_return_types,
                            specialization_cache,
                            specialized_functions,
                            specialized_signatures,
                        },
                    )?);
                    continue;
                }
                rewritten.push(AstStmt::Expr(rewrite_generic_calls_in_expr(
                    GenericExprRewriteInput {
                        expr,
                        context,
                        expected: current_return_type,
                        env,
                        visible_type_aliases,
                        generic_templates,
                        generic_impl_method_templates,
                        higher_order_templates,
                        function_table,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    },
                )?));
                continue;
            }
        }
        let let_fallback_expected = let_binding_expected_type_from_following_use(
            stmt,
            &body[index + 1..],
            ExpectedBlockType {
                current_return_type,
                result_context,
            },
            generic_templates,
            signatures,
            visible_type_aliases,
            struct_table,
        );
        rewritten.extend(rewrite_generic_stmt_with_hoists(
            GenericStmtHoistRewriteInput {
                stmt,
                context,
                let_fallback_expected: let_fallback_expected.as_ref(),
                current_return_type,
                result_context: false,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            },
        )?);
    }
    Ok(rewritten)
}

pub(super) fn rewrite_generic_calls_in_match_arms(
    input: GenericMatchArmsRewriteInput<'_>,
) -> Result<Vec<AstMatchArm>, String> {
    rewrite_generic_calls_in_match_arms_inner(input, false)
}

pub(super) fn rewrite_generic_calls_in_result_match_arms(
    input: GenericMatchArmsRewriteInput<'_>,
) -> Result<Vec<AstMatchArm>, String> {
    rewrite_generic_calls_in_match_arms_inner(input, true)
}

fn rewrite_generic_calls_in_match_arms_inner(
    input: GenericMatchArmsRewriteInput<'_>,
    result_context: bool,
) -> Result<Vec<AstMatchArm>, String> {
    let GenericMatchArmsRewriteInput {
        arms,
        context,
        scrutinee_type,
        current_return_type,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    } = input;
    let mut rewritten = Vec::with_capacity(arms.len());
    for arm in arms {
        let mut arm_env = env.clone();
        if let Some(scrutinee_type) = scrutinee_type {
            bind_match_pattern_for_type(
                scrutinee_type,
                &arm.pattern,
                visible_type_aliases,
                struct_table,
                &mut arm_env,
            )?;
        }
        rewritten.push(AstMatchArm {
            pattern: arm.pattern.clone(),
            guard: arm
                .guard
                .as_ref()
                .map(|guard| {
                    rewrite_generic_calls_in_expr(GenericExprRewriteInput {
                        expr: guard,
                        context,
                        expected: Some(&ast_named_type("bool")),
                        env: &arm_env,
                        visible_type_aliases,
                        generic_templates,
                        generic_impl_method_templates,
                        higher_order_templates,
                        function_table,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    })
                })
                .transpose()?,
            body: rewrite_generic_calls_in_block_inner(
                GenericBlockRewriteInput {
                    body: &arm.body,
                    context: &format!("{context} match-arm"),
                    current_return_type,
                    env: &mut arm_env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                },
                result_context,
            )?,
        });
    }
    Ok(rewritten)
}

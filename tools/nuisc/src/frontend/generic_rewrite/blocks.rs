use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstMatchArm, AstStmt, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::validation_binding_env::bind_match_pattern_for_type;
use super::super::{ast_named_type, FunctionSignature};
use super::blocks_expected::let_binding_expected_type_from_following_use;
use super::blocks_hoists::rewrite_generic_stmt_with_hoists;
use super::exprs::rewrite_generic_calls_in_expr;
use super::GenericImplMethodTemplate;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_block(
    body: &[AstStmt],
    context: &str,
    current_return_type: Option<&AstTypeRef>,
    env: &mut BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstStmt>, String> {
    let mut rewritten = Vec::new();
    for (index, stmt) in body.iter().enumerate() {
        let let_fallback_expected = let_binding_expected_type_from_following_use(
            stmt,
            &body[index + 1..],
            current_return_type,
            generic_templates,
            signatures,
            visible_type_aliases,
            struct_table,
        );
        rewritten.extend(rewrite_generic_stmt_with_hoists(
            stmt,
            context,
            let_fallback_expected.as_ref(),
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
        )?);
    }
    Ok(rewritten)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_match_arms(
    arms: &[AstMatchArm],
    context: &str,
    scrutinee_type: Option<&AstTypeRef>,
    current_return_type: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Vec<AstMatchArm>, String> {
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
                    rewrite_generic_calls_in_expr(
                        guard,
                        context,
                        Some(&ast_named_type("bool")),
                        &mut arm_env,
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
                    )
                })
                .transpose()?,
            body: rewrite_generic_calls_in_block(
                &arm.body,
                &format!("{context} match-arm"),
                current_return_type,
                &mut arm_env,
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
            )?,
        });
    }
    Ok(rewritten)
}

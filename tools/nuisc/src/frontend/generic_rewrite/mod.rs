mod blocks;
mod blocks_expected;
mod blocks_hoists;
mod blocks_stmt;
mod exprs;
mod exprs_alias_expected;
mod exprs_aliases;
mod exprs_calls;
mod exprs_expected;
mod exprs_operators;
mod exprs_specialization;
mod exprs_structs;
mod hoists;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef};

use self::blocks::rewrite_generic_calls_in_block;
use super::{function_context::render_function_body_context, FunctionSignature};

#[derive(Clone)]
pub(super) struct GenericImplMethodTemplate {
    pub(super) trait_name: String,
    pub(super) method_name: String,
    pub(super) function: AstFunction,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_function(
    function: &AstFunction,
    module_const_env: &BTreeMap<String, AstTypeRef>,
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
) -> Result<AstFunction, String> {
    let mut env = module_const_env.clone();
    for param in &function.params {
        env.insert(param.name.clone(), param.ty.clone());
    }
    let body = rewrite_generic_calls_in_block(
        &function.body,
        &render_function_body_context(&function.name),
        function.return_type.as_ref(),
        &mut env,
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
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok(rewritten)
}

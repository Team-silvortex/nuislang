mod blocks;
mod blocks_expected;
mod blocks_hoists;
mod blocks_stmt;
mod exprs;
mod exprs_alias_expected;
mod exprs_alias_inputs;
mod exprs_alias_usage;
mod exprs_aliases;
mod exprs_calls;
mod exprs_expected;
mod exprs_operators;
mod exprs_specialization;
mod exprs_structs;
mod hoists;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef};

use self::blocks::{rewrite_generic_calls_in_block, GenericBlockRewriteInput};
use super::{function_context::render_function_body_context, FunctionSignature};

#[derive(Clone)]
pub(super) struct GenericImplMethodTemplate {
    pub(super) trait_name: String,
    pub(super) method_name: String,
    pub(super) function: AstFunction,
}

pub(super) struct GenericFunctionRewriteInput<'a> {
    pub(super) function: &'a AstFunction,
    pub(super) module_const_env: &'a BTreeMap<String, AstTypeRef>,
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

pub(super) fn rewrite_generic_calls_in_function(
    input: GenericFunctionRewriteInput<'_>,
) -> Result<AstFunction, String> {
    let GenericFunctionRewriteInput {
        function,
        module_const_env,
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
    let mut env = module_const_env.clone();
    for param in &function.params {
        env.insert(param.name.clone(), param.ty.clone());
    }
    let body = rewrite_generic_calls_in_block(GenericBlockRewriteInput {
        body: &function.body,
        context: &render_function_body_context(&function.name),
        current_return_type: function.return_type.as_ref(),
        env: &mut env,
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
    })?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok(rewritten)
}

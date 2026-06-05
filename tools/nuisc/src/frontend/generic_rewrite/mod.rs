mod blocks;
mod exprs;
mod hoists;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef};

use self::blocks::rewrite_generic_calls_in_block;
use super::FunctionSignature;

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_generic_calls_in_function(
    function: &AstFunction,
    module_const_env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
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
        function.return_type.as_ref(),
        &mut env,
        visible_type_aliases,
        generic_templates,
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

mod profile;
mod runtime;

use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_shader_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    if let Some(profile_builtin) = profile::lower_shader_profile_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(profile_builtin));
    }
    runtime::lower_shader_runtime_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )
}

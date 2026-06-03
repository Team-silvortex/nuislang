mod maps;
mod profile;
mod tensors;

use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_kernel_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    if let Some(profile_builtin) = profile::lower_kernel_profile_builtin_call(
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
    if let Some(tensor_builtin) = tensors::lower_kernel_tensor_builtin_call(
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
    )? {
        return Ok(Some(tensor_builtin));
    }
    maps::lower_kernel_map_builtin_call(
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
    )
}

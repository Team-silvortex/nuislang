mod maps;
mod profile;
mod tensors;

use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{FunctionSignature, ModuleConstValue};

#[derive(Clone, Copy)]
pub(super) struct KernelBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_kernel_builtin_call(
    input: KernelBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let KernelBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        ..
    } = input;
    if let Some(profile_builtin) = profile::lower_kernel_profile_builtin_call(input)? {
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

mod profile;
mod runtime;

use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{FunctionSignature, ModuleConstValue};

#[derive(Clone, Copy)]
pub(super) struct ShaderBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_shader_builtin_call(
    input: ShaderBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    if let Some(profile_builtin) = profile::lower_shader_profile_builtin_call(input)? {
        return Ok(Some(profile_builtin));
    }
    runtime::lower_shader_runtime_builtin_call(input)
}

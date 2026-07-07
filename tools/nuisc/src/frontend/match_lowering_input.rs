use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, AstMatchArm, AstTypeAlias, AstTypeRef, NirStructDef, NirTypeRef,
};

use super::{FunctionSignature, ModuleConstValue};

pub(in crate::frontend) struct MatchStmtLoweringInput<'a> {
    pub(in crate::frontend) value: &'a AstExpr,
    pub(in crate::frontend) arms: &'a [AstMatchArm],
    pub(in crate::frontend) current_domain: &'a str,
    pub(in crate::frontend) current_function_is_async: bool,
    pub(in crate::frontend) bindings: &'a mut BTreeMap<String, NirTypeRef>,
    pub(in crate::frontend) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(in crate::frontend) return_type: Option<&'a AstTypeRef>,
    pub(in crate::frontend) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(in crate::frontend) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(in crate::frontend) struct_table: &'a BTreeMap<String, NirStructDef>,
}

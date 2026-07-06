use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef};

pub(super) struct StructConstructorAliasInput<'a> {
    pub(super) callee: &'a str,
    pub(super) generic_args: &'a [AstTypeRef],
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) args: &'a [AstExpr],
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
}

pub(super) struct StructLiteralAliasInput<'a> {
    pub(super) type_name: &'a str,
    pub(super) type_args: &'a [AstTypeRef],
    pub(super) expected: Option<&'a AstTypeRef>,
    pub(super) fields: &'a [(String, AstExpr)],
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
}

pub(super) struct MethodCallReceiverExpectedTypeInput<'a> {
    pub(super) receiver: &'a AstExpr,
    pub(super) method: &'a str,
    pub(super) generic_args: &'a [AstTypeRef],
    pub(super) args: &'a [AstExpr],
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
}

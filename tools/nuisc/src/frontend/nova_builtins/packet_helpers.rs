use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, FunctionSignature};

pub(super) fn lower_i64_arg_list(
    args: &[AstExpr],
    expected_len: usize,
    arg_error: &str,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<NirExpr>, String> {
    if args.len() != expected_len {
        return Err(arg_error.to_owned());
    }
    args.iter()
        .map(|arg| {
            lower_expr(
                arg,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )
        })
        .collect()
}

pub(super) fn build_struct_literal(
    type_name: &str,
    field_names: &[&str],
    values: Vec<NirExpr>,
) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: type_name.to_owned(),
        fields: field_names
            .iter()
            .zip(values)
            .map(|(field, value)| ((*field).to_owned(), value))
            .collect(),
    }
}

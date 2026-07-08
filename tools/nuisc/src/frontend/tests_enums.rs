use super::lower_project_ast_to_nir;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstExpr, AstStmt, NirExpr, NirStmt};

fn is_payload_value_access(expr: &NirExpr) -> bool {
    matches!(
        expr,
        NirExpr::FieldAccess { field, .. } | NirExpr::VariantFieldAccess { field, .. }
            if field == "value"
    )
}

#[path = "tests_enums/constructors_patterns.rs"]
mod constructors_patterns;
#[path = "tests_enums/generic_impl_methods.rs"]
mod generic_impl_methods;
#[path = "tests_enums/generic_impl_ops.rs"]
mod generic_impl_ops;
#[path = "tests_enums/qualified_helper_methods.rs"]
mod qualified_helper_methods;
#[path = "tests_enums/qualified_helper_ops.rs"]
mod qualified_helper_ops;

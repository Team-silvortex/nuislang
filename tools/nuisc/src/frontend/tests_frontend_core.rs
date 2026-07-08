use super::lower_project_ast_to_nir;
use super::lower_type_ref;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{
    AstBinaryOp, AstDestructureBinding, AstDestructureField, AstExpr, AstStmt, AstVisibility,
    NirBinaryOp, NirExpr, NirStmt,
};
use std::fs;
use std::path::PathBuf;

#[path = "tests_frontend_core/assignments_and_helpers.rs"]
mod assignments_and_helpers;
#[path = "tests_frontend_core/bytes_text.rs"]
mod bytes_text;
#[path = "tests_frontend_core/operator_overload.rs"]
mod operator_overload;
#[path = "tests_frontend_core/operator_precedence.rs"]
mod operator_precedence;
#[path = "tests_frontend_core/trait_qualified.rs"]
mod trait_qualified;
#[path = "tests_frontend_core/type_and_slice_basic.rs"]
mod type_and_slice_basic;
#[path = "tests_frontend_core/type_and_slice_typed.rs"]
mod type_and_slice_typed;
#[path = "tests_frontend_core/unary_and_project.rs"]
mod unary_and_project;
#[path = "tests_frontend_core/visibility_const_destructure.rs"]
mod visibility_const_destructure;

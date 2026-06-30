use std::collections::BTreeMap;

use nuis_semantics::model::{
    NirAddressClass, NirBinaryOp, NirDataFlowState, NirExpr, NirKernelFlowState,
    NirNetworkFlowState, NirResultFamily, NirResultStage, NirShaderFlowState, NirStructDef,
    NirTypeRef, NirWindowMode,
};

use super::super::render_type_name;
use super::builtin_fields::builtin_struct_field_type;
use crate::frontend::FunctionSignature;

#[path = "nir_address.rs"]
mod nir_address;
#[path = "nir_compat.rs"]
mod nir_compat;
#[path = "nir_constructors.rs"]
mod nir_constructors;
#[path = "nir_data.rs"]
mod nir_data;
#[path = "nir_expr.rs"]
mod nir_expr;
#[path = "nir_result.rs"]
mod nir_result;

pub(crate) use nir_address::infer_nir_expr_address_class;
pub(crate) use nir_compat::compatible_types;
pub(crate) use nir_constructors::{
    bool_type, f32_type, f64_type, generic_named_type, i32_type, i64_type,
    instantiate_struct_field_type, named_type, ref_type, string_type, struct_field_type, unit_type,
};
pub(crate) use nir_data::{infer_data_window_type, resolve_declared_or_inferred};
pub(crate) use nir_expr::infer_nir_expr_type;
pub(crate) use nir_result::{
    ensure_result_like, expr_type, infer_result_stage, make_result_type, result_payload_type,
};

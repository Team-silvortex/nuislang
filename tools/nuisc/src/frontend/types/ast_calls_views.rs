use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstImplDef, AstStructDef, AstTypeRef};

use super::ast_infer::infer_ast_expr_type_inner;
use super::{ast_generic_named_type, ast_named_type};

pub(super) enum AstCallInference {
    Handled(Option<AstTypeRef>),
    Unhandled,
}

pub(super) struct AstCallInferenceInput<'a> {
    pub(super) callee: &'a str,
    pub(super) generic_args: &'a [AstTypeRef],
    pub(super) args: &'a [AstExpr],
    pub(super) env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) struct_table: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) active_exprs: &'a mut BTreeSet<usize>,
}

pub(super) fn infer_view_call_type(input: AstCallInferenceInput<'_>) -> AstCallInference {
    let AstCallInferenceInput {
        callee,
        generic_args,
        args,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    } = input;
    if !is_view_call(callee) {
        return AstCallInference::Unhandled;
    }
    AstCallInference::Handled(infer_view_call_type_inner(AstCallInferenceInput {
        callee,
        generic_args,
        args,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    }))
}

fn is_view_call(callee: &str) -> bool {
    matches!(
        callee,
        "buffer_len"
            | "slice"
            | "bytes"
            | "slice_len"
            | "slice_start"
            | "slice_buffer"
            | "subslice"
            | "subbytes"
            | "fillbytes"
            | "copybytes"
            | "comparebytes"
            | "bytes_fill"
            | "bytes_copy_from"
            | "bytes_compare"
            | "bytes_eq"
            | "bytes_starts_with"
            | "bytes_ends_with"
            | "bytes_find_byte"
            | "bytes_find_text"
            | "bytes_contains_byte"
            | "bytes_contains_text"
            | "bytes_find_line_end"
            | "bytes_trim_line_end"
            | "bytes_slice_before"
            | "bytes_slice_after"
            | "bytes_split_once_byte"
            | "bytes_split_once_text"
            | "load_at"
    )
}

fn infer_view_call_type_inner(input: AstCallInferenceInput<'_>) -> Option<AstTypeRef> {
    let AstCallInferenceInput {
        callee,
        generic_args,
        args,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    } = input;
    match callee {
        "buffer_len" => Some(ast_named_type("i64")),
        "slice" => {
            let [buffer, _, _] = args else {
                return None;
            };
            let payload = match generic_args {
                [] => ast_named_type("i64"),
                [payload]
                    if *payload == ast_named_type("i64")
                        || *payload == ast_named_type("i32")
                        || *payload == ast_named_type("f32")
                        || *payload == ast_named_type("f64") =>
                {
                    payload.clone()
                }
                [payload] if *payload == ast_named_type("bool") => payload.clone(),
                [payload] => {
                    return Some(ast_generic_named_type("Slice", vec![payload.clone()]));
                }
                _ => return None,
            };
            let buffer_ty = infer_ast_expr_type_inner(
                buffer,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if buffer_ty.is_ref && buffer_ty.name == "Buffer" && !buffer_ty.is_optional {
                Some(ast_generic_named_type("Slice", vec![payload]))
            } else {
                None
            }
        }
        "bytes" => {
            let [buffer, _, _] = args else {
                return None;
            };
            if !generic_args.is_empty() {
                return None;
            }
            let buffer_ty = infer_ast_expr_type_inner(
                buffer,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if buffer_ty.is_ref && buffer_ty.name == "Buffer" && !buffer_ty.is_optional {
                Some(ast_generic_named_type("Slice", vec![ast_named_type("i64")]))
            } else {
                None
            }
        }
        "slice_len" => {
            let [base] = args else {
                return None;
            };
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.name == "Slice"
                && !base_ty.is_ref
                && !base_ty.is_optional
                && base_ty.generic_args.len() == 1
            {
                Some(ast_named_type("i64"))
            } else {
                None
            }
        }
        "slice_start" => {
            let [base] = args else {
                return None;
            };
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.name == "Slice"
                && !base_ty.is_ref
                && !base_ty.is_optional
                && base_ty.generic_args.len() == 1
            {
                Some(ast_named_type("i64"))
            } else {
                None
            }
        }
        "slice_buffer" => {
            let [base] = args else {
                return None;
            };
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.name == "Slice"
                && !base_ty.is_ref
                && !base_ty.is_optional
                && base_ty.generic_args.len() == 1
            {
                Some(AstTypeRef {
                    name: "Buffer".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: true,
                })
            } else {
                None
            }
        }
        "subslice" => {
            let [base, _, _] = args else {
                return None;
            };
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.name == "Slice"
                && !base_ty.is_ref
                && !base_ty.is_optional
                && base_ty.generic_args.len() == 1
            {
                match generic_args {
                    [] => Some(base_ty),
                    [payload] if *payload == base_ty.generic_args[0] => Some(base_ty),
                    _ => None,
                }
            } else {
                None
            }
        }
        "subbytes" => {
            let [base, _, _] = args else {
                return None;
            };
            if !generic_args.is_empty() {
                return None;
            }
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.name == "Slice"
                && !base_ty.is_ref
                && !base_ty.is_optional
                && base_ty.generic_args.len() == 1
                && base_ty.generic_args[0] == ast_named_type("i64")
            {
                Some(base_ty)
            } else {
                None
            }
        }
        "fillbytes" | "copybytes" | "comparebytes" | "bytes_fill" | "bytes_copy_from"
        | "bytes_compare" => {
            let views = match callee {
                "fillbytes" | "bytes_fill" => {
                    let [base, _] = args else {
                        return None;
                    };
                    vec![base]
                }
                "copybytes" | "comparebytes" | "bytes_copy_from" | "bytes_compare" => {
                    let [lhs, rhs] = args else {
                        return None;
                    };
                    vec![lhs, rhs]
                }
                _ => return None,
            };
            if !generic_args.is_empty() {
                return None;
            }
            for view in views {
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name != "Slice"
                    || view_ty.is_ref
                    || view_ty.is_optional
                    || view_ty.generic_args.len() != 1
                    || view_ty.generic_args[0] != ast_named_type("i64")
                {
                    return None;
                }
            }
            Some(ast_named_type("i64"))
        }
        "bytes_eq" | "bytes_starts_with" | "bytes_ends_with" => {
            let [lhs, rhs] = args else {
                return None;
            };
            if !generic_args.is_empty() {
                return None;
            }
            for view in [lhs, rhs] {
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name != "Slice"
                    || view_ty.is_ref
                    || view_ty.is_optional
                    || view_ty.generic_args.len() != 1
                    || view_ty.generic_args[0] != ast_named_type("i64")
                {
                    return None;
                }
            }
            Some(ast_named_type("bool"))
        }
        "bytes_find_byte" | "bytes_find_text" => {
            let view = match args {
                [view, _] => view,
                _ => return None,
            };
            if !generic_args.is_empty() {
                return None;
            }
            let view_ty = infer_ast_expr_type_inner(
                view,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if view_ty.name == "Slice"
                && !view_ty.is_ref
                && !view_ty.is_optional
                && view_ty.generic_args.len() == 1
                && view_ty.generic_args[0] == ast_named_type("i64")
            {
                Some(ast_named_type("i64"))
            } else {
                None
            }
        }
        "bytes_contains_byte" | "bytes_contains_text" => {
            let view = match args {
                [view, _] => view,
                _ => return None,
            };
            if !generic_args.is_empty() {
                return None;
            }
            let view_ty = infer_ast_expr_type_inner(
                view,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if view_ty.name == "Slice"
                && !view_ty.is_ref
                && !view_ty.is_optional
                && view_ty.generic_args.len() == 1
                && view_ty.generic_args[0] == ast_named_type("i64")
            {
                Some(ast_named_type("bool"))
            } else {
                None
            }
        }
        "bytes_find_line_end" | "bytes_trim_line_end" => {
            let [view] = args else {
                return None;
            };
            if !generic_args.is_empty() {
                return None;
            }
            let view_ty = infer_ast_expr_type_inner(
                view,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if view_ty.name == "Slice"
                && !view_ty.is_ref
                && !view_ty.is_optional
                && view_ty.generic_args.len() == 1
                && view_ty.generic_args[0] == ast_named_type("i64")
            {
                Some(ast_named_type("i64"))
            } else {
                None
            }
        }
        "bytes_slice_before" | "bytes_slice_after" => {
            let [view, _] = args else {
                return None;
            };
            if !generic_args.is_empty() {
                return None;
            }
            let view_ty = infer_ast_expr_type_inner(
                view,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if view_ty.name == "Slice"
                && !view_ty.is_ref
                && !view_ty.is_optional
                && view_ty.generic_args.len() == 1
                && view_ty.generic_args[0] == ast_named_type("i64")
            {
                Some(view_ty)
            } else {
                None
            }
        }
        "bytes_split_once_byte" | "bytes_split_once_text" => {
            let view = match args {
                [view, _] => view,
                _ => return None,
            };
            if !generic_args.is_empty() {
                return None;
            }
            let view_ty = infer_ast_expr_type_inner(
                view,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if view_ty.name == "Slice"
                && !view_ty.is_ref
                && !view_ty.is_optional
                && view_ty.generic_args.len() == 1
                && view_ty.generic_args[0] == ast_named_type("i64")
            {
                Some(ast_named_type("ByteSplit"))
            } else {
                None
            }
        }
        "load_at" => {
            let [target, _] = args else {
                return None;
            };
            let target_ty = infer_ast_expr_type_inner(
                target,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if target_ty.name == "Slice"
                && !target_ty.is_ref
                && !target_ty.is_optional
                && target_ty.generic_args.len() == 1
            {
                Some(target_ty.generic_args[0].clone())
            } else {
                Some(ast_named_type("i64"))
            }
        }
        _ => None,
    }
}

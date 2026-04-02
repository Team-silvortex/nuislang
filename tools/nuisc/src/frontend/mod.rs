mod lexer;
mod parser;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef, NirBinaryOp,
    NirExpr, NirFunction, NirModule, NirParam, NirStmt, NirStructDef, NirStructField, NirTypeRef,
};

pub fn frontend_name() -> &'static str {
    "nuisc-parser-minimal"
}

pub fn parse_nuis_ast(input: &str) -> Result<AstModule, String> {
    let tokens = lexer::tokenize(input)?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_module()
}

pub fn lower_ast_to_nir(module: &AstModule) -> Result<NirModule, String> {
    let struct_defs = module
        .structs
        .iter()
        .map(|definition| NirStructDef {
            name: definition.name.clone(),
            fields: definition
                .fields
                .iter()
                .map(|field| NirStructField {
                    name: field.name.clone(),
                    ty: lower_type_ref(&field.ty),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let struct_table = struct_defs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();

    let signatures = module
        .functions
        .iter()
        .map(|function| {
            (
                function.name.clone(),
                FunctionSignature {
                    params: function.params.iter().map(|param| lower_type_ref(&param.ty)).collect(),
                    return_type: function.return_type.as_ref().map(lower_type_ref),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    Ok(NirModule {
        domain: module.domain.clone(),
        unit: module.unit.clone(),
        structs: struct_defs,
        functions: module
            .functions
            .iter()
            .map(|function| lower_function(function, &signatures, &struct_table))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn parse_nuis_module(input: &str) -> Result<NirModule, String> {
    let ast = parse_nuis_ast(input)?;
    lower_ast_to_nir(&ast)
}

#[derive(Clone)]
struct FunctionSignature {
    params: Vec<NirTypeRef>,
    return_type: Option<NirTypeRef>,
}

fn lower_function(
    function: &AstFunction,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirFunction, String> {
    let mut bindings = BTreeMap::<String, NirTypeRef>::new();
    for param in &function.params {
        bindings.insert(param.name.clone(), lower_type_ref(&param.ty));
    }

    Ok(NirFunction {
        name: function.name.clone(),
        params: function.params.iter().map(lower_param).collect(),
        return_type: function.return_type.as_ref().map(lower_type_ref),
        body: function
            .body
            .iter()
            .map(|stmt| {
                lower_stmt(
                    stmt,
                    &mut bindings,
                    function.return_type.as_ref(),
                    signatures,
                    struct_table,
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn lower_param(param: &AstParam) -> NirParam {
    NirParam {
        name: param.name.clone(),
        ty: lower_type_ref(&param.ty),
    }
}

fn lower_type_ref(ty: &AstTypeRef) -> NirTypeRef {
    NirTypeRef {
        name: ty.name.clone(),
        generic_args: ty.generic_args.iter().map(lower_type_ref).collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

fn lower_stmt(
    stmt: &AstStmt,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    return_type: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    Ok(match stmt {
        AstStmt::Let { name, ty, value } => {
            let expected = ty.as_ref().map(lower_type_ref);
            let lowered = lower_expr(value, bindings, signatures, struct_table, expected.as_ref())?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
            name: name.clone(),
            ty: Some(final_type),
            value: lowered,
        }
        }
        AstStmt::Const { name, ty, value } => {
            let expected = lower_type_ref(ty);
            let lowered = lower_expr(value, bindings, signatures, struct_table, Some(&expected))?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, Some(expected), inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Const {
            name: name.clone(),
            ty: final_type,
            value: lowered,
        }
        }
        AstStmt::Print(value) => {
            NirStmt::Print(lower_expr(value, bindings, signatures, struct_table, None)?)
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => NirStmt::If {
            condition: lower_expr(
                condition,
                bindings,
                signatures,
                struct_table,
                Some(&bool_type()),
            )?,
            then_body: then_body
                .iter()
                .map(|stmt| {
                    lower_stmt(
                        stmt,
                        &mut bindings.clone(),
                        return_type,
                        signatures,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            else_body: else_body
                .iter()
                .map(|stmt| {
                    lower_stmt(
                        stmt,
                        &mut bindings.clone(),
                        return_type,
                        signatures,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstStmt::Expr(expr) => {
            NirStmt::Expr(lower_expr(expr, bindings, signatures, struct_table, None)?)
        }
        AstStmt::Return(value) => {
            let expected = return_type.map(lower_type_ref);
            NirStmt::Return(match value {
                Some(value) => Some(lower_expr(
                    value,
                    bindings,
                    signatures,
                    struct_table,
                    expected.as_ref(),
                )?),
                None => None,
            })
        }
    })
}

fn lower_expr(
    expr: &AstExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    Ok(match expr {
        AstExpr::Bool(value) => NirExpr::Bool(*value),
        AstExpr::Text(text) => NirExpr::Text(text.clone()),
        AstExpr::Int(value) => NirExpr::Int(*value),
        AstExpr::Var(name) => NirExpr::Var(name.clone()),
        AstExpr::Call { callee, args } => {
            lower_call_expr(callee, args, bindings, signatures, struct_table, expected)?
        }
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(lower_expr(receiver, bindings, signatures, struct_table, None)?),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| lower_expr(arg, bindings, signatures, struct_table, None))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral { type_name, fields } => {
            let definition = struct_table.get(type_name).ok_or_else(|| {
                format!("unknown struct type `{}`", type_name)
            })?;
            let mut seen = BTreeSet::new();
            let mut lowered_fields = Vec::new();
            for (name, value) in fields {
                let field = definition
                    .fields
                    .iter()
                    .find(|field| field.name == *name)
                    .ok_or_else(|| format!("struct `{}` has no field `{}`", type_name, name))?;
                if !seen.insert(name.clone()) {
                    return Err(format!(
                        "struct literal `{}` duplicates field `{}`",
                        type_name, name
                    ));
                }
                let lowered =
                    lower_expr(value, bindings, signatures, struct_table, Some(&field.ty))?;
                let inferred =
                    infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
                let _ = resolve_declared_or_inferred(name, Some(field.ty.clone()), inferred)?;
                lowered_fields.push((name.clone(), lowered));
            }
            if definition.fields.len() != lowered_fields.len() {
                return Err(format!(
                    "struct literal `{}` must initialize all {} field(s)",
                    type_name,
                    definition.fields.len()
                ));
            }
            NirExpr::StructLiteral {
                type_name: type_name.clone(),
                fields: lowered_fields,
            }
        }
        AstExpr::FieldAccess { base, field } => {
            let lowered_base = lower_expr(base, bindings, signatures, struct_table, None)?;
            let base_ty =
                infer_nir_expr_type(&lowered_base, bindings, signatures, struct_table)
                    .ok_or_else(|| {
                        format!("cannot infer base type for field access `.{} `", field)
                    })?;
            let definition = struct_table.get(&base_ty.name).ok_or_else(|| {
                format!("type `{}` has no known struct definition", render_type_name(&base_ty))
            })?;
            if !definition.fields.iter().any(|candidate| candidate.name == *field) {
                return Err(format!(
                    "struct `{}` has no field `{}`",
                    definition.name, field
                ));
            }
            NirExpr::FieldAccess {
                base: Box::new(lowered_base),
                field: field.clone(),
            }
        }
        AstExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: match op {
                AstBinaryOp::Add => NirBinaryOp::Add,
                AstBinaryOp::Sub => NirBinaryOp::Sub,
                AstBinaryOp::Mul => NirBinaryOp::Mul,
                AstBinaryOp::Div => NirBinaryOp::Div,
            },
            lhs: Box::new(lower_expr(lhs, bindings, signatures, struct_table, None)?),
            rhs: Box::new(lower_expr(rhs, bindings, signatures, struct_table, None)?),
        },
    })
}

fn lower_call_expr(
    callee: &str,
    args: &[AstExpr],
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    match callee {
        "null" => {
            if !args.is_empty() {
                return Err("null() expects 0 args".to_owned());
            }
            if let Some(expected) = expected {
                if !expected.is_ref {
                    return Err("null() currently requires an expected `ref` type".to_owned());
                }
            }
            Ok(NirExpr::Null)
        }
        "borrow" => {
            let [value] = args else {
                return Err("borrow(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(value, bindings, signatures, struct_table, None)?;
            ensure_ref_like("borrow", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::Borrow(Box::new(lowered)))
        }
        "move" => {
            let [value] = args else {
                return Err("move(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(value, bindings, signatures, struct_table, None)?;
            ensure_ref_like("move", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::Move(Box::new(lowered)))
        }
        "alloc_node" => {
            let [value, next] = args else {
                return Err("alloc_node(...) expects 2 args".to_owned());
            };
            let lowered_value =
                lower_expr(value, bindings, signatures, struct_table, Some(&i64_type()))?;
            let lowered_next = lower_expr(
                next,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Node")),
            )?;
            Ok(NirExpr::AllocNode {
                value: Box::new(lowered_value),
                next: Box::new(lowered_next),
            })
        }
        "alloc_buffer" => {
            let [len, fill] = args else {
                return Err("alloc_buffer(...) expects 2 args".to_owned());
            };
            Ok(NirExpr::AllocBuffer {
                len: Box::new(lower_expr(
                    len,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                fill: Box::new(lower_expr(
                    fill,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "load_value" => {
            let [ptr] = args else {
                return Err("load_value(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                ptr,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Node")),
            )?;
            Ok(NirExpr::LoadValue(Box::new(lowered)))
        }
        "load_next" => {
            let [ptr] = args else {
                return Err("load_next(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                ptr,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Node")),
            )?;
            Ok(NirExpr::LoadNext(Box::new(lowered)))
        }
        "buffer_len" => {
            let [ptr] = args else {
                return Err("buffer_len(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                ptr,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            Ok(NirExpr::BufferLen(Box::new(lowered)))
        }
        "load_at" => {
            let [buffer, index] = args else {
                return Err("load_at(...) expects 2 args".to_owned());
            };
            Ok(NirExpr::LoadAt {
                buffer: Box::new(lower_expr(
                    buffer,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Buffer")),
                )?),
                index: Box::new(lower_expr(
                    index,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "store_value" => {
            let [target, value] = args else {
                return Err("store_value(...) expects 2 args".to_owned());
            };
            Ok(NirExpr::StoreValue {
                target: Box::new(lower_expr(
                    target,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
                value: Box::new(lower_expr(
                    value,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "store_next" => {
            let [target, next] = args else {
                return Err("store_next(...) expects 2 args".to_owned());
            };
            Ok(NirExpr::StoreNext {
                target: Box::new(lower_expr(
                    target,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
                next: Box::new(lower_expr(
                    next,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
            })
        }
        "store_at" => {
            let [buffer, index, value] = args else {
                return Err("store_at(...) expects 3 args".to_owned());
            };
            Ok(NirExpr::StoreAt {
                buffer: Box::new(lower_expr(
                    buffer,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Buffer")),
                )?),
                index: Box::new(lower_expr(
                    index,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                value: Box::new(lower_expr(
                    value,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "data_bind_core" => {
            let [core] = args else {
                return Err("data_bind_core(...) expects 1 arg".to_owned());
            };
            let AstExpr::Int(core_index) = core else {
                return Err("data_bind_core(...) currently expects an integer literal".to_owned());
            };
            Ok(NirExpr::DataBindCore(*core_index))
        }
        "data_marker" => {
            let [tag] = args else {
                return Err("data_marker(...) expects 1 arg".to_owned());
            };
            let AstExpr::Text(tag) = tag else {
                return Err("data_marker(...) currently expects a string literal".to_owned());
            };
            Ok(NirExpr::DataMarker(tag.clone()))
        }
        "data_output_pipe" => {
            let [value] = args else {
                return Err("data_output_pipe(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(value, bindings, signatures, struct_table, None)?;
            Ok(NirExpr::DataOutputPipe(Box::new(lowered)))
        }
        "data_input_pipe" => {
            let [pipe] = args else {
                return Err("data_input_pipe(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(pipe, bindings, signatures, struct_table, None)?;
            Ok(NirExpr::DataInputPipe(Box::new(lowered)))
        }
        "data_copy_window" => {
            let [input, offset, len] = args else {
                return Err("data_copy_window(...) expects 3 args".to_owned());
            };
            Ok(NirExpr::DataCopyWindow {
                input: Box::new(lower_expr(input, bindings, signatures, struct_table, None)?),
                offset: Box::new(lower_expr(
                    offset,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                len: Box::new(lower_expr(
                    len,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "data_immutable_window" => {
            let [input, offset, len] = args else {
                return Err("data_immutable_window(...) expects 3 args".to_owned());
            };
            Ok(NirExpr::DataImmutableWindow {
                input: Box::new(lower_expr(input, bindings, signatures, struct_table, None)?),
                offset: Box::new(lower_expr(
                    offset,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                len: Box::new(lower_expr(
                    len,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "data_handle_table" => {
            if args.is_empty() {
                return Err("data_handle_table(...) expects at least 1 slot mapping".to_owned());
            }
            let mut entries = Vec::new();
            for arg in args {
                let AstExpr::Text(text) = arg else {
                    return Err(
                        "data_handle_table(...) currently expects string literals like \"slot=resource\""
                            .to_owned(),
                    );
                };
                let Some((slot, resource)) = text.split_once('=') else {
                    return Err(format!(
                        "data_handle_table(...) entry `{text}` must be `slot=resource`"
                    ));
                };
                entries.push((slot.trim().to_owned(), resource.trim().to_owned()));
            }
            Ok(NirExpr::DataHandleTable(entries))
        }
        "free" => {
            let [value] = args else {
                return Err("free(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(value, bindings, signatures, struct_table, None)?;
            ensure_ref_like("free", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::Free(Box::new(lowered)))
        }
        "is_null" => {
            let [value] = args else {
                return Err("is_null(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(value, bindings, signatures, struct_table, None)?;
            ensure_ref_like("is_null", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::IsNull(Box::new(lowered)))
        }
        _ => {
            let lowered_args = args
                .iter()
                .map(|arg| lower_expr(arg, bindings, signatures, struct_table, None))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(signature) = signatures.get(callee) {
                if signature.params.len() != lowered_args.len() {
                    return Err(format!(
                        "function `{callee}` expects {} args, found {}",
                        signature.params.len(),
                        lowered_args.len()
                    ));
                }
            }
            Ok(NirExpr::Call {
            callee: callee.to_owned(),
            args: lowered_args,
        })
        }
    }
}

fn ensure_ref_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.is_ref => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects a `ref` value, found `{}`",
            render_type_name(&ty)
        )),
        None => Ok(()),
    }
}

fn infer_nir_expr_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Bool(_) | NirExpr::IsNull(_) => Some(bool_type()),
        NirExpr::Text(_) => Some(named_type("String")),
        NirExpr::Int(_) => Some(i64_type()),
        NirExpr::Var(name) => bindings.get(name).cloned(),
        NirExpr::Null => None,
        NirExpr::Borrow(value) | NirExpr::Move(value) => {
            infer_nir_expr_type(value, bindings, signatures, struct_table)
        }
        NirExpr::AllocNode { .. } => Some(ref_type("Node")),
        NirExpr::AllocBuffer { .. } => Some(ref_type("Buffer")),
        NirExpr::DataBindCore(_) => Some(named_type("Unit")),
        NirExpr::DataMarker(_) => Some(named_type("Marker")),
        NirExpr::DataHandleTable(_) => Some(named_type("HandleTable")),
        NirExpr::DataOutputPipe(value) => {
            let inner = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            Some(generic_named_type("Pipe", vec![inner]))
        }
        NirExpr::DataCopyWindow { input, .. } | NirExpr::DataImmutableWindow { input, .. } => {
            let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            Some(generic_named_type("Window", vec![inner]))
        }
        NirExpr::DataInputPipe(value) => {
            let pipe_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            pipe_ty.generic_args.first().cloned()
        }
        NirExpr::LoadValue(_) | NirExpr::LoadAt { .. } | NirExpr::BufferLen(_) => Some(i64_type()),
        NirExpr::LoadNext(_) => Some(ref_type("Node")),
        NirExpr::StoreValue { .. } | NirExpr::StoreNext { .. } | NirExpr::StoreAt { .. } | NirExpr::Free(_) => {
            Some(named_type("Unit"))
        }
        NirExpr::Call { callee, .. } => {
            signatures.get(callee).and_then(|sig| sig.return_type.clone())
        }
        NirExpr::MethodCall { .. } => None,
        NirExpr::StructLiteral { type_name, .. } => Some(named_type(type_name)),
        NirExpr::FieldAccess { base, field } => {
            let base_ty = infer_nir_expr_type(base, bindings, signatures, struct_table)?;
            let definition = struct_table.get(&base_ty.name)?;
            definition
                .fields
                .iter()
                .find(|candidate| candidate.name == *field)
                .map(|field| field.ty.clone())
        }
        NirExpr::Binary { .. } => Some(i64_type()),
    }
}

fn resolve_declared_or_inferred(
    name: &str,
    declared: Option<NirTypeRef>,
    inferred: Option<NirTypeRef>,
) -> Result<NirTypeRef, String> {
    match (declared, inferred) {
        (Some(declared), Some(inferred)) => {
            if compatible_types(&declared, &inferred) {
                Ok(declared)
            } else {
                Err(format!(
                    "binding `{name}` expected type `{}`, found `{}`",
                    render_type_name(&declared),
                    render_type_name(&inferred)
                ))
            }
        }
        (Some(declared), None) => Ok(declared),
        (None, Some(inferred)) => Ok(inferred),
        (None, None) => Err(format!(
            "binding `{name}` requires an explicit type annotation in the current minimal frontend"
        )),
    }
}

fn compatible_types(expected: &NirTypeRef, actual: &NirTypeRef) -> bool {
    if expected.name == actual.name
        && expected.is_ref == actual.is_ref
        && expected.is_optional == actual.is_optional
    {
        return true;
    }
    expected.is_ref && actual.is_ref
}

fn named_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

fn generic_named_type(name: &str, generic_args: Vec<NirTypeRef>) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    }
}

fn ref_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: true,
    }
}

fn i64_type() -> NirTypeRef {
    named_type("i64")
}

fn bool_type() -> NirTypeRef {
    named_type("bool")
}

fn render_type_name(ty: &NirTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
    }
    out.push_str(&ty.name);
    if ty.is_optional {
        out.push('?');
    }
    out
}

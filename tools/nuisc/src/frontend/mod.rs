mod lexer;
mod parser;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef, NirBinaryOp,
    NirExpr, NirExternFunction, NirExternInterface, NirFunction, NirModule, NirParam, NirStmt,
    NirStructDef, NirStructField, NirTypeRef, NirUse,
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
        .externs
        .iter()
        .map(|function| {
            (
                function.name.clone(),
                FunctionSignature {
                    abi: function.abi.clone(),
                    interface: None,
                    symbol_name: function.name.clone(),
                    params: function
                        .params
                        .iter()
                        .map(|param| lower_type_ref(&param.ty))
                        .collect(),
                    return_type: Some(lower_type_ref(&function.return_type)),
                    is_extern: true,
                },
            )
        })
        .chain(module.extern_interfaces.iter().flat_map(|interface| {
            interface.methods.iter().map(move |function| {
                (
                    format!("{}.{}", interface.name, function.name),
                    FunctionSignature {
                        abi: function.abi.clone(),
                        interface: Some(interface.name.clone()),
                        symbol_name: format!("{}__{}", interface.name, function.name),
                        params: function
                            .params
                            .iter()
                            .map(|param| lower_type_ref(&param.ty))
                            .collect(),
                        return_type: Some(lower_type_ref(&function.return_type)),
                        is_extern: true,
                    },
                )
            })
        }))
        .chain(module.functions.iter().map(|function| {
            (
                function.name.clone(),
                FunctionSignature {
                    abi: "nuis".to_owned(),
                    interface: None,
                    symbol_name: function.name.clone(),
                    params: function
                        .params
                        .iter()
                        .map(|param| lower_type_ref(&param.ty))
                        .collect(),
                    return_type: function.return_type.as_ref().map(lower_type_ref),
                    is_extern: false,
                },
            )
        }))
        .collect::<BTreeMap<_, _>>();

    Ok(NirModule {
        uses: module
            .uses
            .iter()
            .map(|item| NirUse {
                domain: item.domain.clone(),
                unit: item.unit.clone(),
            })
            .collect(),
        domain: module.domain.clone(),
        unit: module.unit.clone(),
        externs: module
            .externs
            .iter()
            .map(|function| NirExternFunction {
                abi: function.abi.clone(),
                interface: None,
                name: function.name.clone(),
                params: function.params.iter().map(lower_param).collect(),
                return_type: lower_type_ref(&function.return_type),
            })
            .collect(),
        extern_interfaces: module
            .extern_interfaces
            .iter()
            .map(|interface| NirExternInterface {
                abi: interface.abi.clone(),
                name: interface.name.clone(),
                methods: interface
                    .methods
                    .iter()
                    .map(|function| NirExternFunction {
                        abi: function.abi.clone(),
                        interface: Some(interface.name.clone()),
                        name: function.name.clone(),
                        params: function.params.iter().map(lower_param).collect(),
                        return_type: lower_type_ref(&function.return_type),
                    })
                    .collect(),
            })
            .collect(),
        structs: struct_defs,
        functions: module
            .functions
            .iter()
            .map(|function| lower_function(function, &module.domain, &signatures, &struct_table))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn parse_nuis_module(input: &str) -> Result<NirModule, String> {
    let ast = parse_nuis_ast(input)?;
    lower_ast_to_nir(&ast)
}

#[derive(Clone)]
struct FunctionSignature {
    abi: String,
    interface: Option<String>,
    symbol_name: String,
    params: Vec<NirTypeRef>,
    return_type: Option<NirTypeRef>,
    is_extern: bool,
}

fn lower_function(
    function: &AstFunction,
    current_domain: &str,
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
                    current_domain,
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
    current_domain: &str,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    return_type: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    Ok(match stmt {
        AstStmt::Let { name, ty, value } => {
            let expected = ty.as_ref().map(lower_type_ref);
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                expected.as_ref(),
            )?;
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
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&expected),
            )?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, Some(expected), inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Const {
                name: name.clone(),
                ty: final_type,
                value: lowered,
            }
        }
        AstStmt::Print(value) => NirStmt::Print(lower_expr(
            value,
            current_domain,
            bindings,
            signatures,
            struct_table,
            None,
        )?),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => NirStmt::If {
            condition: lower_expr(
                condition,
                current_domain,
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
                        current_domain,
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
                        current_domain,
                        &mut bindings.clone(),
                        return_type,
                        signatures,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstStmt::Expr(expr) => NirStmt::Expr(lower_expr(
            expr,
            current_domain,
            bindings,
            signatures,
            struct_table,
            None,
        )?),
        AstStmt::Return(value) => {
            let expected = return_type.map(lower_type_ref);
            NirStmt::Return(match value {
                Some(value) => Some(lower_expr(
                    value,
                    current_domain,
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
    current_domain: &str,
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
        AstExpr::Instantiate { domain, unit } => {
            if current_domain != "cpu" {
                return Err(format!(
                    "instantiate {} {} is only allowed inside `mod cpu <unit>` in the current frontend",
                    domain, unit
                ));
            }
            NirExpr::Instantiate {
                domain: domain.clone(),
                unit: unit.clone(),
            }
        }
        AstExpr::Call { callee, args } => lower_call_expr(
            callee,
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            expected,
        )?,
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            if let AstExpr::Var(receiver_name) = receiver.as_ref() {
                let signature_key = format!("{receiver_name}.{method}");
                if let Some(signature) = signatures.get(&signature_key) {
                    let lowered_args = args
                        .iter()
                        .map(|arg| {
                            lower_expr(
                                arg,
                                current_domain,
                                bindings,
                                signatures,
                                struct_table,
                                None,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if signature.params.len() != lowered_args.len() {
                        return Err(format!(
                            "method `{signature_key}` expects {} args, found {}",
                            signature.params.len(),
                            lowered_args.len()
                        ));
                    }
                    if signature.is_extern {
                        if current_domain != "cpu" {
                            return Err(format!(
                                "extern method `{signature_key}` is currently only allowed inside `mod cpu <unit>`"
                            ));
                        }
                        return Ok(NirExpr::CpuExternCall {
                            abi: signature.abi.clone(),
                            interface: signature.interface.clone(),
                            callee: signature.symbol_name.clone(),
                            args: lowered_args,
                        });
                    }
                }
            }
            NirExpr::MethodCall {
                receiver: Box::new(lower_expr(
                    receiver,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                method: method.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        lower_expr(
                            arg,
                            current_domain,
                            bindings,
                            signatures,
                            struct_table,
                            None,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        AstExpr::StructLiteral { type_name, fields } => {
            let definition = struct_table
                .get(type_name)
                .ok_or_else(|| format!("unknown struct type `{}`", type_name))?;
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
                let lowered = lower_expr(
                    value,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&field.ty),
                )?;
                let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
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
            let lowered_base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let base_ty = infer_nir_expr_type(&lowered_base, bindings, signatures, struct_table)
                .ok_or_else(|| format!("cannot infer base type for field access `.{} `", field))?;
            let definition = struct_table.get(&base_ty.name).ok_or_else(|| {
                format!(
                    "type `{}` has no known struct definition",
                    render_type_name(&base_ty)
                )
            })?;
            if !definition
                .fields
                .iter()
                .any(|candidate| candidate.name == *field)
            {
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
            lhs: Box::new(lower_expr(
                lhs,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?),
            rhs: Box::new(lower_expr(
                rhs,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?),
        },
    })
}

fn lower_call_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
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
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("borrow", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::Borrow(Box::new(lowered)))
        }
        "move" => {
            let [value] = args else {
                return Err("move(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("move", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::Move(Box::new(lowered)))
        }
        "alloc_node" => {
            let [value, next] = args else {
                return Err("alloc_node(...) expects 2 args".to_owned());
            };
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_next = lower_expr(
                next,
                current_domain,
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
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                fill: Box::new(lower_expr(
                    fill,
                    current_domain,
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
                current_domain,
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
                current_domain,
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
                current_domain,
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
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Buffer")),
                )?),
                index: Box::new(lower_expr(
                    index,
                    current_domain,
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
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
                value: Box::new(lower_expr(
                    value,
                    current_domain,
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
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
                next: Box::new(lower_expr(
                    next,
                    current_domain,
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
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Buffer")),
                )?),
                index: Box::new(lower_expr(
                    index,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                value: Box::new(lower_expr(
                    value,
                    current_domain,
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
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(NirExpr::DataOutputPipe(Box::new(lowered)))
        }
        "data_input_pipe" => {
            let [pipe] = args else {
                return Err("data_input_pipe(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                pipe,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(NirExpr::DataInputPipe(Box::new(lowered)))
        }
        "data_copy_window" => {
            let [input, offset, len] = args else {
                return Err("data_copy_window(...) expects 3 args".to_owned());
            };
            Ok(NirExpr::DataCopyWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(lower_expr(
                    offset,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                len: Box::new(lower_expr(
                    len,
                    current_domain,
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
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(lower_expr(
                    offset,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                len: Box::new(lower_expr(
                    len,
                    current_domain,
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
        "cpu_bind_core" => {
            let [core] = args else {
                return Err("cpu_bind_core(...) expects 1 arg".to_owned());
            };
            let AstExpr::Int(core_index) = core else {
                return Err("cpu_bind_core(...) currently expects an integer literal".to_owned());
            };
            Ok(NirExpr::CpuBindCore(*core_index))
        }
        "cpu_window" => {
            let [width, height, title] = args else {
                return Err("cpu_window(...) expects 3 args".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("cpu_window(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("cpu_window(...) height must be an integer literal".to_owned());
            };
            let AstExpr::Text(title) = title else {
                return Err("cpu_window(...) title must be a string literal".to_owned());
            };
            Ok(NirExpr::CpuWindow {
                width: *width,
                height: *height,
                title: title.clone(),
            })
        }
        "cpu_input_i64" => match args {
            [channel, default] | [channel, default, ..] => {
                let AstExpr::Text(channel) = channel else {
                    return Err("cpu_input_i64(...) channel must be a string literal".to_owned());
                };
                let AstExpr::Int(default) = default else {
                    return Err("cpu_input_i64(...) default must be an integer literal".to_owned());
                };
                let (min, max, step) = match args {
                    [_, _, min, max, step] => {
                        let AstExpr::Int(min) = min else {
                            return Err(
                                "cpu_input_i64(...) min must be an integer literal".to_owned()
                            );
                        };
                        let AstExpr::Int(max) = max else {
                            return Err(
                                "cpu_input_i64(...) max must be an integer literal".to_owned()
                            );
                        };
                        let AstExpr::Int(step) = step else {
                            return Err(
                                "cpu_input_i64(...) step must be an integer literal".to_owned()
                            );
                        };
                        (Some(*min), Some(*max), Some(*step))
                    }
                    [_, _] => (None, None, None),
                    _ => return Err("cpu_input_i64(...) expects 2 args or 5 args".to_owned()),
                };
                Ok(NirExpr::CpuInputI64 {
                    channel: channel.clone(),
                    default: *default,
                    min,
                    max,
                    step,
                })
            }
            _ => Err("cpu_input_i64(...) expects 2 args or 5 args".to_owned()),
        },
        "cpu_tick_i64" => {
            let [start, step] = args else {
                return Err("cpu_tick_i64(...) expects 2 args".to_owned());
            };
            let AstExpr::Int(start) = start else {
                return Err("cpu_tick_i64(...) start must be an integer literal".to_owned());
            };
            let AstExpr::Int(step) = step else {
                return Err("cpu_tick_i64(...) step must be an integer literal".to_owned());
            };
            Ok(NirExpr::CpuTickI64 {
                start: *start,
                step: *step,
            })
        }
        "cpu_present_frame" => {
            let [frame] = args else {
                return Err("cpu_present_frame(...) expects 1 arg".to_owned());
            };
            Ok(NirExpr::CpuPresentFrame(Box::new(lower_expr(
                frame,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?)))
        }
        "shader_profile_target" => {
            let [unit] = args else {
                return Err("shader_profile_target(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_target(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_target(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfileTargetRef { unit: unit.clone() })
        }
        "shader_profile_viewport" => {
            let [unit] = args else {
                return Err("shader_profile_viewport(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_viewport(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_viewport(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfileViewportRef { unit: unit.clone() })
        }
        "shader_profile_pipeline" => {
            let [unit] = args else {
                return Err("shader_profile_pipeline(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_pipeline(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_pipeline(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePipelineRef { unit: unit.clone() })
        }
        "shader_profile_begin_pass" => {
            let [unit] = args else {
                return Err("shader_profile_begin_pass(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_begin_pass(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_begin_pass(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderBeginPass {
                target: Box::new(NirExpr::ShaderProfileTargetRef { unit: unit.clone() }),
                pipeline: Box::new(NirExpr::ShaderProfilePipelineRef { unit: unit.clone() }),
                viewport: Box::new(NirExpr::ShaderProfileViewportRef { unit: unit.clone() }),
            })
        }
        "shader_profile_vertex_count" => {
            let [unit] = args else {
                return Err("shader_profile_vertex_count(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_vertex_count(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_vertex_count(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfileVertexCountRef { unit: unit.clone() })
        }
        "shader_profile_instance_count" => {
            let [unit] = args else {
                return Err("shader_profile_instance_count(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_instance_count(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_instance_count(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfileInstanceCountRef { unit: unit.clone() })
        }
        "shader_profile_color_seed" => {
            let [unit, base, delta] = args else {
                return Err("shader_profile_color_seed(...) expects 3 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_color_seed(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_color_seed(...) expects a string literal unit name".to_owned(),
                );
            };
            let base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let delta = lower_expr(
                delta,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::ShaderProfileColorSeed {
                unit: unit.clone(),
                base: Box::new(base),
                delta: Box::new(delta),
            })
        }
        "shader_profile_speed_seed" => {
            let [unit, delta, scale, base] = args else {
                return Err("shader_profile_speed_seed(...) expects 4 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_speed_seed(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_speed_seed(...) expects a string literal unit name".to_owned(),
                );
            };
            let delta = lower_expr(
                delta,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scale = lower_expr(
                scale,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::ShaderProfileSpeedSeed {
                unit: unit.clone(),
                delta: Box::new(delta),
                scale: Box::new(scale),
                base: Box::new(base),
            })
        }
        "shader_profile_radius_seed" => {
            let [unit, base, delta] = args else {
                return Err("shader_profile_radius_seed(...) expects 3 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_radius_seed(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_radius_seed(...) expects a string literal unit name".to_owned(),
                );
            };
            let base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let delta = lower_expr(
                delta,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::ShaderProfileRadiusSeed {
                unit: unit.clone(),
                base: Box::new(base),
                delta: Box::new(delta),
            })
        }
        "shader_profile_packet" => {
            let [unit, color, speed, radius] = args else {
                return Err("shader_profile_packet(...) expects 4 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet(...) expects a string literal unit name".to_owned(),
                );
            };
            let color = lower_expr(
                color,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let speed = lower_expr(
                speed,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let radius = lower_expr(
                radius,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::ShaderProfilePacket {
                unit: unit.clone(),
                color: Box::new(color),
                speed: Box::new(speed),
                radius: Box::new(radius),
            })
        }
        "shader_profile_packet_color_slot" => {
            let [unit] = args else {
                return Err("shader_profile_packet_color_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_color_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_color_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePacketColorSlotRef { unit: unit.clone() })
        }
        "shader_profile_packet_speed_slot" => {
            let [unit] = args else {
                return Err("shader_profile_packet_speed_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_speed_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_speed_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePacketSpeedSlotRef { unit: unit.clone() })
        }
        "shader_profile_packet_radius_slot" => {
            let [unit] = args else {
                return Err("shader_profile_packet_radius_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_radius_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_radius_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePacketRadiusSlotRef { unit: unit.clone() })
        }
        "shader_profile_packet_tag" => {
            let [unit] = args else {
                return Err("shader_profile_packet_tag(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_tag(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_tag(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePacketTagRef { unit: unit.clone() })
        }
        "shader_profile_material_mode" => {
            let [unit] = args else {
                return Err("shader_profile_material_mode(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_material_mode(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_material_mode(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfileMaterialModeRef { unit: unit.clone() })
        }
        "shader_profile_pass_kind" => {
            let [unit] = args else {
                return Err("shader_profile_pass_kind(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_pass_kind(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_pass_kind(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePassKindRef { unit: unit.clone() })
        }
        "shader_profile_packet_field_count" => {
            let [unit] = args else {
                return Err("shader_profile_packet_field_count(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_field_count(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_field_count(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfilePacketFieldCountRef { unit: unit.clone() })
        }
        "data_profile_bind_core" => {
            let [unit] = args else {
                return Err("data_profile_bind_core(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_bind_core(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_bind_core(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::DataProfileBindCoreRef { unit: unit.clone() })
        }
        "data_profile_window_offset" => {
            let [unit] = args else {
                return Err("data_profile_window_offset(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_window_offset(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_window_offset(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::DataProfileWindowOffsetRef { unit: unit.clone() })
        }
        "data_profile_uplink_len" => {
            let [unit] = args else {
                return Err("data_profile_uplink_len(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_uplink_len(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_uplink_len(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::DataProfileUplinkLenRef { unit: unit.clone() })
        }
        "data_profile_downlink_len" => {
            let [unit] = args else {
                return Err("data_profile_downlink_len(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_downlink_len(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_downlink_len(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::DataProfileDownlinkLenRef { unit: unit.clone() })
        }
        "data_profile_uplink_window" => {
            let [unit, input] = args else {
                return Err("data_profile_uplink_window(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_uplink_window(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_uplink_window(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::DataImmutableWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(NirExpr::DataProfileWindowOffsetRef { unit: unit.clone() }),
                len: Box::new(NirExpr::DataProfileUplinkLenRef { unit: unit.clone() }),
            })
        }
        "data_profile_send_uplink" => {
            let [unit, input] = args else {
                return Err("data_profile_send_uplink(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_send_uplink(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_send_uplink(...) expects a string literal unit name".to_owned(),
                );
            };
            let lowered_input = lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(NirExpr::DataProfileSendUplink {
                unit: unit.clone(),
                input: Box::new(lowered_input),
            })
        }
        "data_profile_downlink_window" => {
            let [unit, input] = args else {
                return Err("data_profile_downlink_window(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_downlink_window(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_downlink_window(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::DataCopyWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(NirExpr::DataProfileWindowOffsetRef { unit: unit.clone() }),
                len: Box::new(NirExpr::DataProfileDownlinkLenRef { unit: unit.clone() }),
            })
        }
        "data_profile_send_downlink" => {
            let [unit, input] = args else {
                return Err("data_profile_send_downlink(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_send_downlink(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_send_downlink(...) expects a string literal unit name".to_owned(),
                );
            };
            let lowered_input = lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(NirExpr::DataProfileSendDownlink {
                unit: unit.clone(),
                input: Box::new(lowered_input),
            })
        }
        "data_profile_handle_table" => {
            let [unit] = args else {
                return Err("data_profile_handle_table(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_handle_table(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_handle_table(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::DataProfileHandleTableRef { unit: unit.clone() })
        }
        "data_profile_marker" => {
            let [unit, tag] = args else {
                return Err("data_profile_marker(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "data_profile_marker(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "data_profile_marker(...) expects a string literal unit name".to_owned(),
                );
            };
            let AstExpr::Text(tag) = tag else {
                return Err(
                    "data_profile_marker(...) expects a string literal marker tag".to_owned(),
                );
            };
            Ok(NirExpr::DataProfileMarkerRef {
                unit: unit.clone(),
                tag: tag.clone(),
            })
        }
        "kernel_profile_bind_core" => {
            let [unit] = args else {
                return Err("kernel_profile_bind_core(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "kernel_profile_bind_core(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "kernel_profile_bind_core(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::KernelProfileBindCoreRef { unit: unit.clone() })
        }
        "kernel_profile_queue_depth" => {
            let [unit] = args else {
                return Err("kernel_profile_queue_depth(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "kernel_profile_queue_depth(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "kernel_profile_queue_depth(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::KernelProfileQueueDepthRef { unit: unit.clone() })
        }
        "kernel_profile_batch_lanes" => {
            let [unit] = args else {
                return Err("kernel_profile_batch_lanes(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "kernel_profile_batch_lanes(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "kernel_profile_batch_lanes(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::KernelProfileBatchLanesRef { unit: unit.clone() })
        }
        "shader_target" => {
            let [format, width, height] = args else {
                return Err("shader_target(...) expects 3 args".to_owned());
            };
            let AstExpr::Text(format) = format else {
                return Err("shader_target(...) format must be a string literal".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("shader_target(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("shader_target(...) height must be an integer literal".to_owned());
            };
            Ok(NirExpr::ShaderTarget {
                format: format.clone(),
                width: *width,
                height: *height,
            })
        }
        "shader_viewport" => {
            let [width, height] = args else {
                return Err("shader_viewport(...) expects 2 args".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("shader_viewport(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("shader_viewport(...) height must be an integer literal".to_owned());
            };
            Ok(NirExpr::ShaderViewport {
                width: *width,
                height: *height,
            })
        }
        "shader_pipeline" => {
            let [name, topology] = args else {
                return Err("shader_pipeline(...) expects 2 args".to_owned());
            };
            let AstExpr::Text(name) = name else {
                return Err("shader_pipeline(...) name must be a string literal".to_owned());
            };
            let AstExpr::Text(topology) = topology else {
                return Err("shader_pipeline(...) topology must be a string literal".to_owned());
            };
            Ok(NirExpr::ShaderPipeline {
                name: name.clone(),
                topology: topology.clone(),
            })
        }
        "shader_inline_wgsl" => {
            let [entry, source] = args else {
                return Err("shader_inline_wgsl(...) expects 2 args".to_owned());
            };
            if current_domain != "shader" {
                return Err(
                    "shader_inline_wgsl(...) is currently only allowed inside `mod shader <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(entry) = entry else {
                return Err("shader_inline_wgsl(...) entry must be a string literal".to_owned());
            };
            let AstExpr::Text(source) = source else {
                return Err(
                    "shader_inline_wgsl(...) source must be a string or wgsl block".to_owned(),
                );
            };
            Ok(NirExpr::ShaderInlineWgsl {
                entry: entry.clone(),
                source: source.clone(),
            })
        }
        "shader_begin_pass" => {
            let [target, pipeline, viewport] = args else {
                return Err("shader_begin_pass(...) expects 3 args".to_owned());
            };
            Ok(NirExpr::ShaderBeginPass {
                target: Box::new(lower_expr(
                    target,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                pipeline: Box::new(lower_expr(
                    pipeline,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                viewport: Box::new(lower_expr(
                    viewport,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            })
        }
        "shader_draw_instanced" => {
            let [pass, packet, vertex_count, instance_count] = args else {
                return Err("shader_draw_instanced(...) expects 4 args".to_owned());
            };
            Ok(NirExpr::ShaderDrawInstanced {
                pass: Box::new(lower_expr(
                    pass,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                packet: Box::new(lower_expr(
                    packet,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                vertex_count: Box::new(lower_expr(
                    vertex_count,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                instance_count: Box::new(lower_expr(
                    instance_count,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            })
        }
        "shader_profile_draw_instanced" => {
            let [unit, pass, packet] = args else {
                return Err("shader_profile_draw_instanced(...) expects 3 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_draw_instanced(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_draw_instanced(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            Ok(NirExpr::ShaderDrawInstanced {
                pass: Box::new(lower_expr(
                    pass,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&named_type("Pass")),
                )?),
                packet: Box::new(lower_expr(
                    packet,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                vertex_count: Box::new(NirExpr::ShaderProfileVertexCountRef { unit: unit.clone() }),
                instance_count: Box::new(NirExpr::ShaderProfileInstanceCountRef {
                    unit: unit.clone(),
                }),
            })
        }
        "shader_profile_render" => {
            let [unit, packet] = args else {
                return Err("shader_profile_render(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_render(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_render(...) expects a string literal unit name".to_owned(),
                );
            };
            Ok(NirExpr::ShaderProfileRender {
                unit: unit.clone(),
                packet: Box::new(lower_expr(
                    packet,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            })
        }
        "free" => {
            let [value] = args else {
                return Err("free(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("free", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::Free(Box::new(lowered)))
        }
        "is_null" => {
            let [value] = args else {
                return Err("is_null(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("is_null", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::IsNull(Box::new(lowered)))
        }
        _ => {
            let lowered_args = args
                .iter()
                .map(|arg| {
                    lower_expr(
                        arg,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        None,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(signature) = signatures.get(callee) {
                if signature.params.len() != lowered_args.len() {
                    return Err(format!(
                        "function `{callee}` expects {} args, found {}",
                        signature.params.len(),
                        lowered_args.len()
                    ));
                }
                if signature.is_extern {
                    if current_domain != "cpu" {
                        return Err(format!(
                            "extern call `{callee}` is currently only allowed inside `mod cpu <unit>`"
                        ));
                    }
                    return Ok(NirExpr::CpuExternCall {
                        abi: signature.abi.clone(),
                        interface: None,
                        callee: signature.symbol_name.clone(),
                        args: lowered_args,
                    });
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
        NirExpr::Instantiate { unit, .. } => {
            Some(generic_named_type("Instance", vec![named_type(unit)]))
        }
        NirExpr::Null => None,
        NirExpr::Borrow(value) | NirExpr::Move(value) => {
            infer_nir_expr_type(value, bindings, signatures, struct_table)
        }
        NirExpr::AllocNode { .. } => Some(ref_type("Node")),
        NirExpr::AllocBuffer { .. } => Some(ref_type("Buffer")),
        NirExpr::DataBindCore(_) | NirExpr::CpuBindCore(_) => Some(named_type("Unit")),
        NirExpr::CpuWindow { .. } => Some(named_type("Window")),
        NirExpr::CpuInputI64 { .. } | NirExpr::CpuTickI64 { .. } => Some(i64_type()),
        NirExpr::CpuPresentFrame(_) => Some(named_type("Unit")),
        NirExpr::ShaderProfileTargetRef { .. } => Some(named_type("Target")),
        NirExpr::ShaderProfileViewportRef { .. } => Some(named_type("Viewport")),
        NirExpr::ShaderProfilePipelineRef { .. } => Some(named_type("Pipeline")),
        NirExpr::ShaderProfileVertexCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileInstanceCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketColorSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketTagRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileMaterialModeRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePassKindRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketFieldCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileColorSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfileSpeedSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfileRadiusSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacket { unit, .. } => Some(named_type(&format!("{unit}Packet"))),
        NirExpr::DataProfileBindCoreRef { .. } => Some(named_type("Unit")),
        NirExpr::DataProfileWindowOffsetRef { .. } => Some(i64_type()),
        NirExpr::DataProfileUplinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileDownlinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileHandleTableRef { .. } => Some(named_type("HandleTable")),
        NirExpr::DataProfileMarkerRef { .. } => Some(named_type("Marker")),
        NirExpr::KernelProfileBindCoreRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileQueueDepthRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileBatchLanesRef { .. } => Some(i64_type()),
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            let window_inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            Some(generic_named_type("Window", vec![window_inner]))
        }
        NirExpr::CpuExternCall { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
        NirExpr::DataMarker(_) => Some(named_type("Marker")),
        NirExpr::DataHandleTable(_) => Some(named_type("HandleTable")),
        NirExpr::ShaderTarget { .. } => Some(named_type("Target")),
        NirExpr::ShaderViewport { .. } => Some(named_type("Viewport")),
        NirExpr::ShaderPipeline { .. } => Some(named_type("Pipeline")),
        NirExpr::ShaderInlineWgsl { .. } => Some(named_type("ShaderModule")),
        NirExpr::ShaderBeginPass { .. } => Some(named_type("Pass")),
        NirExpr::ShaderDrawInstanced { .. } => Some(named_type("Frame")),
        NirExpr::ShaderProfileRender { .. } => Some(named_type("Frame")),
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
        NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::Free(_) => Some(named_type("Unit")),
        NirExpr::Call { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
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

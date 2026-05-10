mod lexer;
mod parser;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef, NirBinaryOp,
    NirDataFlowState, NirExpr, NirExternFunction, NirExternInterface, NirFunction,
    NirKernelFlowState, NirModule, NirParam, NirResultFamily, NirResultStage, NirShaderFlowState,
    NirStmt, NirStructDef, NirStructField, NirTypeRef, NirUse, NirWindowMode,
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
                    is_async: false,
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
                        is_async: false,
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
                    is_async: function.is_async,
                },
            )
        }))
        .collect::<BTreeMap<_, _>>();

    let nir = NirModule {
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
    };
    validate_declared_nir_types(&nir)?;
    Ok(nir)
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
    is_async: bool,
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
        is_async: function.is_async,
        params: function.params.iter().map(lower_param).collect(),
        return_type: function.return_type.as_ref().map(lower_type_ref),
        body: function
            .body
            .iter()
            .map(|stmt| {
                lower_stmt_with_async(
                    stmt,
                    current_domain,
                    function.is_async,
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

#[allow(dead_code)]
fn lower_stmt(
    stmt: &AstStmt,
    current_domain: &str,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    return_type: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    lower_stmt_with_async(
        stmt,
        current_domain,
        false,
        bindings,
        return_type,
        signatures,
        struct_table,
    )
}

fn lower_stmt_with_async(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    return_type: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    Ok(match stmt {
        AstStmt::Let { name, ty, value } => {
            let expected = ty.as_ref().map(lower_type_ref);
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            let lowered = lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                expected.as_ref(),
                false,
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
            validate_type_ref(&expected)?;
            let lowered = lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                Some(&expected),
                false,
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
        AstStmt::Print(value) => NirStmt::Print(lower_expr_with_async(
            value,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            None,
            false,
        )?),
        AstStmt::Await(value) => {
            if !current_function_is_async {
                return Err("`await` is only allowed inside `async fn`".to_owned());
            }
            NirStmt::Await(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                None,
                true,
            )?)
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => NirStmt::If {
            condition: lower_expr_with_async(
                condition,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                Some(&bool_type()),
                false,
            )?,
            then_body: then_body
                .iter()
                .map(|stmt| {
                    lower_stmt_with_async(
                        stmt,
                        current_domain,
                        current_function_is_async,
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
                    lower_stmt_with_async(
                        stmt,
                        current_domain,
                        current_function_is_async,
                        &mut bindings.clone(),
                        return_type,
                        signatures,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstStmt::Expr(expr) => NirStmt::Expr(lower_expr_with_async(
            expr,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            None,
            false,
        )?),
        AstStmt::Return(value) => {
            let expected = return_type.map(lower_type_ref);
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            NirStmt::Return(match value {
                Some(value) => Some(lower_expr_with_async(
                    value,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    signatures,
                    struct_table,
                    expected.as_ref(),
                    false,
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
    lower_expr_with_async(
        expr,
        current_domain,
        false,
        bindings,
        signatures,
        struct_table,
        expected,
        false,
    )
}

fn lower_nested_expr_with_async(
    expr: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_expr_with_async(
        expr,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
        expected,
        false,
    )
}

fn lower_expr_with_async(
    expr: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    Ok(match expr {
        AstExpr::Bool(value) => NirExpr::Bool(*value),
        AstExpr::Text(text) => NirExpr::Text(text.clone()),
        AstExpr::Int(value) => NirExpr::Int(*value),
        AstExpr::Var(name) => NirExpr::Var(name.clone()),
        AstExpr::Await(value) => {
            if !current_function_is_async {
                return Err("`await` is only allowed inside `async fn`".to_owned());
            }
            NirExpr::Await(Box::new(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                expected,
                true,
            )?))
        }
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
        AstExpr::Call { callee, args } => lower_call_expr_with_async(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            expected,
            allow_async_calls,
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
                            lower_nested_expr_with_async(
                                arg,
                                current_domain,
                                current_function_is_async,
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
                receiver: Box::new(lower_nested_expr_with_async(
                    receiver,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                method: method.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        lower_nested_expr_with_async(
                            arg,
                            current_domain,
                            current_function_is_async,
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
                let lowered = lower_nested_expr_with_async(
                    value,
                    current_domain,
                    current_function_is_async,
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
            let lowered_base = lower_nested_expr_with_async(
                base,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let base_ty = infer_nir_expr_type(&lowered_base, bindings, signatures, struct_table)
                .ok_or_else(|| format!("cannot infer base type for field access `.{} `", field))?;
            if struct_field_type(&base_ty, field, struct_table).is_none() {
                return Err(format!(
                    "type `{}` has no field `{}`",
                    render_type_name(&base_ty),
                    field
                ));
            }
            NirExpr::FieldAccess {
                base: Box::new(lowered_base),
                field: field.clone(),
            }
        }
        AstExpr::Binary { op, lhs, rhs } => lower_binary_expr_with_async(
            op,
            lhs,
            rhs,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
        )?,
    })
}

#[allow(dead_code)]
fn lower_binary_expr(
    op: &AstBinaryOp,
    lhs: &AstExpr,
    rhs: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    lower_binary_expr_with_async(
        op,
        lhs,
        rhs,
        current_domain,
        false,
        bindings,
        signatures,
        struct_table,
    )
}

fn lower_binary_expr_with_async(
    op: &AstBinaryOp,
    lhs: &AstExpr,
    rhs: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let lowered_lhs = lower_nested_expr_with_async(
        lhs,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
        None,
    )?;
    let lowered_rhs = lower_nested_expr_with_async(
        rhs,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
        None,
    )?;
    let lhs_ty = infer_nir_expr_type(&lowered_lhs, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer binary lhs type".to_owned())?;
    let rhs_ty = infer_nir_expr_type(&lowered_rhs, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer binary rhs type".to_owned())?;
    let result_ty = binary_result_type(*op, &lhs_ty, &rhs_ty)?;
    if !compatible_types(&lhs_ty, &result_ty) || !compatible_types(&rhs_ty, &result_ty) {
        return Err(format!(
            "binary operands must agree on type, found `{}` and `{}`",
            lhs_ty.render(),
            rhs_ty.render()
        ));
    }
    Ok(NirExpr::Binary {
        op: match op {
            AstBinaryOp::Add => NirBinaryOp::Add,
            AstBinaryOp::Sub => NirBinaryOp::Sub,
            AstBinaryOp::Mul => NirBinaryOp::Mul,
            AstBinaryOp::Div => NirBinaryOp::Div,
        },
        lhs: Box::new(lowered_lhs),
        rhs: Box::new(lowered_rhs),
    })
}

fn binary_result_type(
    op: AstBinaryOp,
    lhs: &NirTypeRef,
    rhs: &NirTypeRef,
) -> Result<NirTypeRef, String> {
    if !compatible_types(lhs, rhs) {
        return Err(format!(
            "binary `{}` expects matching operand types, found `{}` and `{}`",
            render_binary_op(op),
            lhs.render(),
            rhs.render()
        ));
    }
    if !lhs.is_numeric_scalar() || !rhs.is_numeric_scalar() {
        return Err(format!(
            "binary `{}` currently expects numeric scalar operands, found `{}` and `{}`",
            render_binary_op(op),
            lhs.render(),
            rhs.render()
        ));
    }
    Ok(lhs.clone())
}

fn render_binary_op(op: AstBinaryOp) -> &'static str {
    match op {
        AstBinaryOp::Add => "+",
        AstBinaryOp::Sub => "-",
        AstBinaryOp::Mul => "*",
        AstBinaryOp::Div => "/",
    }
}

#[allow(dead_code)]
fn lower_call_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_call_expr_with_async(
        callee,
        args,
        current_domain,
        false,
        bindings,
        signatures,
        struct_table,
        expected,
        false,
    )
}

fn lower_call_expr_with_async(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    match callee {
        "spawn" => {
            if current_domain != "cpu" {
                return Err(
                    "spawn(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [call] = args else {
                return Err("spawn(...) expects exactly one async function call".to_owned());
            };
            let AstExpr::Call {
                callee: spawned_callee,
                args: spawned_args,
            } = call
            else {
                return Err(
                    "spawn(...) expects an async function call like `spawn(task())`".to_owned(),
                );
            };
            let signature = signatures.get(spawned_callee).ok_or_else(|| {
                format!("spawn(...) references unknown function `{spawned_callee}`")
            })?;
            if !signature.is_async {
                return Err(format!(
                    "spawn(...) expects async function call, found sync function `{spawned_callee}`"
                ));
            }
            if signature.params.len() != spawned_args.len() {
                return Err(format!(
                    "function `{spawned_callee}` expects {} args, found {}",
                    signature.params.len(),
                    spawned_args.len()
                ));
            }
            Ok(NirExpr::CpuSpawn {
                callee: spawned_callee.clone(),
                args: spawned_args
                    .iter()
                    .map(|arg| {
                        lower_nested_expr_with_async(
                            arg,
                            current_domain,
                            current_function_is_async,
                            bindings,
                            signatures,
                            struct_table,
                            None,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        "join" => {
            if current_domain != "cpu" {
                return Err(
                    "join(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task] = args else {
                return Err("join(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("join", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::CpuJoin(Box::new(lowered)))
        }
        "cancel" => {
            if current_domain != "cpu" {
                return Err(
                    "cancel(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task] = args else {
                return Err("cancel(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("cancel", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::CpuCancel(Box::new(lowered)))
        }
        "join_result" => {
            if current_domain != "cpu" {
                return Err(
                    "join_result(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task] = args else {
                return Err("join_result(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("join_result", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::CpuJoinResult(Box::new(lowered)))
        }
        "task_completed" => lower_result_observer_call(
            "task_completed",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskCompleted(Box::new(expr)),
        ),
        "task_timed_out" => lower_result_observer_call(
            "task_timed_out",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskTimedOut(Box::new(expr)),
        ),
        "task_cancelled" => lower_result_observer_call(
            "task_cancelled",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskCancelled(Box::new(expr)),
        ),
        "task_value" => lower_result_observer_call(
            "task_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskValue(Box::new(expr)),
        ),
        "timeout" => {
            if current_domain != "cpu" {
                return Err(
                    "timeout(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task, limit] = args else {
                return Err("timeout(...) expects exactly two arguments: task and limit".to_owned());
            };
            let lowered_task = lower_nested_expr_with_async(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("timeout", &lowered_task, bindings, signatures, struct_table)?;
            let lowered_limit = lower_nested_expr_with_async(
                limit,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let limit_ty = infer_nir_expr_type(&lowered_limit, bindings, signatures, struct_table)
                .ok_or_else(|| "timeout(...) limit requires an explicit integer type".to_owned())?;
            if !limit_ty.is_integer_scalar() {
                return Err(format!(
                    "timeout(...) expects integer limit, found `{}`",
                    limit_ty.render()
                ));
            }
            Ok(NirExpr::CpuTimeout {
                task: Box::new(lowered_task),
                limit: Box::new(lowered_limit),
            })
        }
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
        "borrow_end" => {
            let [value] = args else {
                return Err("borrow_end(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("borrow_end", &lowered, bindings, signatures, struct_table)?;
            Ok(NirExpr::BorrowEnd(Box::new(lowered)))
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
            let marker_type = select_expected_semantic_token_type(expected, "Marker");
            validate_type_ref(&marker_type)?;
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
        "data_result" => lower_result_wrapper_call(
            "data_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Data,
            infer_result_stage,
            |value, stage| match stage {
                NirResultStage::Data(state) => Ok(NirExpr::DataResult { value, state }),
                other => Err(format!(
                    "expected data result stage, found `{}`",
                    other.render()
                )),
            },
            "expects a direct data operation like pipe/window/profile send",
        ),
        "data_ready" => lower_result_observer_call(
            "data_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Data,
            |expr| NirExpr::DataReady(Box::new(expr)),
        ),
        "data_moved" => lower_result_observer_call(
            "data_moved",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Data,
            |expr| NirExpr::DataMoved(Box::new(expr)),
        ),
        "data_windowed" => lower_result_observer_call(
            "data_windowed",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Data,
            |expr| NirExpr::DataWindowed(Box::new(expr)),
        ),
        "data_value" => lower_result_observer_call(
            "data_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Data,
            |expr| NirExpr::DataValue(Box::new(expr)),
        ),
        "shader_result" => lower_result_wrapper_call(
            "shader_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            infer_result_stage,
            |value, stage| match stage {
                NirResultStage::Shader(state) => Ok(NirExpr::ShaderResult { value, state }),
                other => Err(format!(
                    "expected shader result stage, found `{}`",
                    other.render()
                )),
            },
            "expects a direct shader operation like begin_pass/render",
        ),
        "shader_pass_ready" => lower_result_observer_call(
            "shader_pass_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |expr| NirExpr::ShaderPassReady(Box::new(expr)),
        ),
        "shader_frame_ready" => lower_result_observer_call(
            "shader_frame_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |expr| NirExpr::ShaderFrameReady(Box::new(expr)),
        ),
        "shader_value" => lower_result_observer_call(
            "shader_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |expr| NirExpr::ShaderValue(Box::new(expr)),
        ),
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
        "data_read_window" => {
            let [window, index] = args else {
                return Err("data_read_window(...) expects 2 args".to_owned());
            };
            let window_expr = lower_expr(
                window,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let index_expr = lower_expr(
                index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let Some(window_ty) = expr_type(&window_expr, bindings, signatures, struct_table)
            else {
                return Err("data_read_window(...) could not infer window type".to_owned());
            };
            if window_ty.window_mode().is_none() {
                return Err(format!(
                    "data_read_window(...) expects Window<T> or WindowMut<T>, got `{}`",
                    window_ty.render()
                ));
            }
            Ok(NirExpr::DataReadWindow {
                window: Box::new(window_expr),
                index: Box::new(index_expr),
            })
        }
        "data_write_window" => {
            let [window, index, value] = args else {
                return Err("data_write_window(...) expects 3 args".to_owned());
            };
            let window_expr = lower_expr(
                window,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let index_expr = lower_expr(
                index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let value_expr = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let Some(window_ty) = expr_type(&window_expr, bindings, signatures, struct_table)
            else {
                return Err("data_write_window(...) could not infer window type".to_owned());
            };
            if window_ty.window_mode() != Some(NirWindowMode::Mutable) {
                return Err(format!(
                    "data_write_window(...) expects WindowMut<T>, got `{}`",
                    window_ty.render()
                ));
            }
            let payload_ty = window_ty
                .container_payload()
                .cloned()
                .ok_or_else(|| "data_write_window(...) expects window payload type".to_owned())?;
            let Some(value_ty) = expr_type(&value_expr, bindings, signatures, struct_table) else {
                return Err("data_write_window(...) could not infer value type".to_owned());
            };
            if !compatible_types(&payload_ty, &value_ty) {
                return Err(format!(
                    "data_write_window(...) expects payload `{}`, got `{}`",
                    payload_ty.render(),
                    value_ty.render()
                ));
            }
            Ok(NirExpr::DataWriteWindow {
                window: Box::new(window_expr),
                index: Box::new(index_expr),
                value: Box::new(value_expr),
            })
        }
        "data_freeze_window" => {
            let [input] = args else {
                return Err("data_freeze_window(...) expects 1 arg".to_owned());
            };
            Ok(NirExpr::DataFreezeWindow(Box::new(lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?)))
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
            let handle_table_type = select_expected_semantic_token_type(expected, "HandleTable");
            validate_type_ref(&handle_table_type)?;
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
        "shader_profile_packet" | "shader_profile_panel_packet" | "nova_panel_packet" => {
            if current_domain != "cpu" {
                return Err(
                    if callee == "shader_profile_panel_packet" {
                        "shader_profile_panel_packet(...) is currently only allowed inside `mod cpu <unit>`"
                    } else if callee == "nova_panel_packet" {
                        "nova_panel_packet(...) is currently only allowed inside `mod cpu <unit>`"
                    } else {
                        "shader_profile_packet(...) is currently only allowed inside `mod cpu <unit>`"
                    }
                        .to_owned(),
                );
            }
            let (unit_name, color, speed, radius, accent, toggle_state, focus_index) = if callee
                == "nova_panel_packet"
            {
                match args {
                    [color, speed, radius, accent, toggle_state, focus_index] => (
                        "__nova__".to_owned(),
                        color,
                        speed,
                        radius,
                        Some(accent),
                        Some(toggle_state),
                        Some(focus_index),
                    ),
                    _ => return Err("nova_panel_packet(...) expects 6 args".to_owned()),
                }
            } else {
                let (unit, color, speed, radius, accent, toggle_state, focus_index) = match args {
                    [unit, color, speed, radius] => (unit, color, speed, radius, None, None, None),
                    [unit, color, speed, radius, accent, toggle_state, focus_index] => (
                        unit,
                        color,
                        speed,
                        radius,
                        Some(accent),
                        Some(toggle_state),
                        Some(focus_index),
                    ),
                    _ => {
                        return Err(if callee == "shader_profile_panel_packet" {
                            "shader_profile_panel_packet(...) expects 7 args".to_owned()
                        } else {
                            "shader_profile_packet(...) expects 4 or 7 args".to_owned()
                        })
                    }
                };
                if callee == "shader_profile_panel_packet"
                    && (accent.is_none() || toggle_state.is_none() || focus_index.is_none())
                {
                    return Err("shader_profile_panel_packet(...) expects 7 args".to_owned());
                }
                let AstExpr::Text(unit_name) = unit else {
                    return Err(if callee == "shader_profile_panel_packet" {
                        "shader_profile_panel_packet(...) expects a string literal unit name"
                            .to_owned()
                    } else {
                        "shader_profile_packet(...) expects a string literal unit name".to_owned()
                    });
                };
                (
                    unit_name.clone(),
                    color,
                    speed,
                    radius,
                    accent,
                    toggle_state,
                    focus_index,
                )
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
            let accent = accent
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                    .map(Box::new)
                })
                .transpose()?;
            let toggle_state = toggle_state
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                    .map(Box::new)
                })
                .transpose()?;
            let focus_index = focus_index
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                    .map(Box::new)
                })
                .transpose()?;
            Ok(NirExpr::ShaderProfilePacket {
                unit: unit_name,
                packet_type_name: if callee == "shader_profile_panel_packet"
                    || callee == "nova_panel_packet"
                {
                    Some("NovaPanelPacket".to_owned())
                } else {
                    None
                },
                color: Box::new(color),
                speed: Box::new(speed),
                radius: Box::new(radius),
                accent,
                toggle_state,
                focus_index,
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
        "nova_header_packet" => {
            let (accent, title_mode) = match args {
                [accent] => (accent, None),
                [accent, title_mode] => (accent, Some(title_mode)),
                _ => return Err("nova_header_packet(...) expects 1 or 2 args".to_owned()),
            };
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let title_mode = title_mode
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| accent.clone());
            Ok(NirExpr::StructLiteral {
                type_name: "NovaHeaderPacket".to_owned(),
                fields: vec![
                    ("accent".to_owned(), accent),
                    ("title_mode".to_owned(), title_mode),
                ],
            })
        }
        "nova_theme_packet" => {
            let (accent, surface, panel_mode, contrast) = match args {
                [accent, surface, panel_mode, contrast] => (accent, surface, panel_mode, contrast),
                _ => return Err("nova_theme_packet(...) expects 4 args".to_owned()),
            };
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let surface = lower_expr(
                surface,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let panel_mode = lower_expr(
                panel_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let contrast = lower_expr(
                contrast,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaThemePacket".to_owned(),
                fields: vec![
                    ("accent".to_owned(), accent),
                    ("surface".to_owned(), surface),
                    ("panel_mode".to_owned(), panel_mode),
                    ("contrast".to_owned(), contrast),
                ],
            })
        }
        "nova_surface_packet" => {
            let (density, elevation, grid, sheen) = match args {
                [density, elevation, grid, sheen] => (density, elevation, grid, sheen),
                _ => return Err("nova_surface_packet(...) expects 4 args".to_owned()),
            };
            let density = lower_expr(
                density,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let elevation = lower_expr(
                elevation,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let grid = lower_expr(
                grid,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let sheen = lower_expr(
                sheen,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSurfacePacket".to_owned(),
                fields: vec![
                    ("density".to_owned(), density),
                    ("elevation".to_owned(), elevation),
                    ("grid".to_owned(), grid),
                    ("sheen".to_owned(), sheen),
                ],
            })
        }
        "nova_viewport_packet" => {
            let (origin_x, origin_y, width, height) = match args {
                [origin_x, origin_y, width, height] => (origin_x, origin_y, width, height),
                _ => return Err("nova_viewport_packet(...) expects 4 args".to_owned()),
            };
            let origin_x = lower_expr(
                origin_x,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let origin_y = lower_expr(
                origin_y,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let width = lower_expr(
                width,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let height = lower_expr(
                height,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaViewportPacket".to_owned(),
                fields: vec![
                    ("origin_x".to_owned(), origin_x),
                    ("origin_y".to_owned(), origin_y),
                    ("width".to_owned(), width),
                    ("height".to_owned(), height),
                ],
            })
        }
        "nova_layer_packet" => {
            let (order, blend, visibility, clip) = match args {
                [order, blend, visibility, clip] => (order, blend, visibility, clip),
                _ => return Err("nova_layer_packet(...) expects 4 args".to_owned()),
            };
            let order = lower_expr(
                order,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let blend = lower_expr(
                blend,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let visibility = lower_expr(
                visibility,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let clip = lower_expr(
                clip,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaLayerPacket".to_owned(),
                fields: vec![
                    ("order".to_owned(), order),
                    ("blend".to_owned(), blend),
                    ("visibility".to_owned(), visibility),
                    ("clip".to_owned(), clip),
                ],
            })
        }
        "nova_slider_packet" => {
            let (value, min_value, max_value, step_value, disabled) = match args {
                [value] => (value, None, None, None, None),
                [value, min_value, max_value, step_value] => (
                    value,
                    Some(min_value),
                    Some(max_value),
                    Some(step_value),
                    None,
                ),
                [value, min_value, max_value, step_value, disabled] => (
                    value,
                    Some(min_value),
                    Some(max_value),
                    Some(step_value),
                    Some(disabled),
                ),
                _ => return Err("nova_slider_packet(...) expects 1, 4 or 5 args".to_owned()),
            };
            let value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let min_expr = min_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            let max_expr = max_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(127));
            let step_expr = step_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(1));
            let disabled_expr = disabled
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSliderPacket".to_owned(),
                fields: vec![
                    ("value".to_owned(), value),
                    ("min".to_owned(), min_expr),
                    ("max".to_owned(), max_expr),
                    ("step".to_owned(), step_expr),
                    ("disabled".to_owned(), disabled_expr),
                ],
            })
        }
        "nova_progress_packet" | "nova_meter_packet" => {
            let (value, max_value) = match args {
                [value] => (value, None),
                [value, max_value] => (value, Some(max_value)),
                _ => return Err(format!("{callee}(...) expects 1 or 2 args")),
            };
            let value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let max_expr = max_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(127));
            let type_name = match callee {
                "nova_progress_packet" => "NovaProgressPacket",
                _ => "NovaMeterPacket",
            };
            Ok(NirExpr::StructLiteral {
                type_name: type_name.to_owned(),
                fields: vec![("value".to_owned(), value), ("max".to_owned(), max_expr)],
            })
        }
        "nova_toggle_packet" => {
            let (live, disabled) = match args {
                [live] => (live, None),
                [live, disabled] => (live, Some(disabled)),
                _ => return Err("nova_toggle_packet(...) expects 1 or 2 args".to_owned()),
            };
            let live = lower_expr(
                live,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let disabled = disabled
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTogglePacket".to_owned(),
                fields: vec![("live".to_owned(), live), ("disabled".to_owned(), disabled)],
            })
        }
        "nova_button_packet" => {
            let (active, accent, intent) = match args {
                [active, accent] => (active, accent, None),
                [active, accent, intent] => (active, accent, Some(intent)),
                _ => return Err("nova_button_packet(...) expects 2 or 3 args".to_owned()),
            };
            let active = lower_expr(
                active,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let intent = intent
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| active.clone());
            Ok(NirExpr::StructLiteral {
                type_name: "NovaButtonPacket".to_owned(),
                fields: vec![
                    ("active".to_owned(), active),
                    ("accent".to_owned(), accent),
                    ("intent".to_owned(), intent),
                ],
            })
        }
        "nova_text_input_packet" => {
            let (echo, caret, placeholder, read_only, dirty) = match args {
                [echo, caret] => (echo, caret, None, None, None),
                [echo, caret, placeholder] => (echo, caret, Some(placeholder), None, None),
                [echo, caret, placeholder, read_only] => {
                    (echo, caret, Some(placeholder), Some(read_only), None)
                }
                [echo, caret, placeholder, read_only, dirty] => {
                    (echo, caret, Some(placeholder), Some(read_only), Some(dirty))
                }
                _ => {
                    return Err("nova_text_input_packet(...) expects 2, 3, 4 or 5 args".to_owned());
                }
            };
            let echo = lower_expr(
                echo,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let caret = lower_expr(
                caret,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let placeholder = placeholder
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| echo.clone());
            let read_only = read_only
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            let dirty = dirty
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTextInputPacket".to_owned(),
                fields: vec![
                    ("echo".to_owned(), echo),
                    ("caret".to_owned(), caret),
                    ("placeholder".to_owned(), placeholder),
                    ("read_only".to_owned(), read_only),
                    ("dirty".to_owned(), dirty),
                ],
            })
        }
        "nova_select_packet" => {
            let (selected, accent, options, multiple, committed) = match args {
                [selected, accent] => (selected, accent, None, None, None),
                [selected, accent, options] => (selected, accent, Some(options), None, None),
                [selected, accent, options, multiple] => {
                    (selected, accent, Some(options), Some(multiple), None)
                }
                [selected, accent, options, multiple, committed] => (
                    selected,
                    accent,
                    Some(options),
                    Some(multiple),
                    Some(committed),
                ),
                _ => return Err("nova_select_packet(...) expects 2, 3, 4 or 5 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let options = options
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(3));
            let multiple = multiple
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            let committed = committed
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(1));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSelectPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("accent".to_owned(), accent),
                    ("options".to_owned(), options),
                    ("multiple".to_owned(), multiple),
                    ("committed".to_owned(), committed),
                ],
            })
        }
        "nova_checkbox_packet" => {
            let (checked, accent, disabled) = match args {
                [checked, accent] => (checked, accent, None),
                [checked, accent, disabled] => (checked, accent, Some(disabled)),
                _ => return Err("nova_checkbox_packet(...) expects 2 or 3 args".to_owned()),
            };
            let checked = lower_expr(
                checked,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let disabled = disabled
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaCheckboxPacket".to_owned(),
                fields: vec![
                    ("checked".to_owned(), checked),
                    ("accent".to_owned(), accent),
                    ("disabled".to_owned(), disabled),
                ],
            })
        }
        "nova_radio_packet" => {
            let (selected, options, accent, disabled) = match args {
                [selected, options, accent] => (selected, options, accent, None),
                [selected, options, accent, disabled] => {
                    (selected, options, accent, Some(disabled))
                }
                _ => return Err("nova_radio_packet(...) expects 3 or 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let options = lower_expr(
                options,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let disabled = disabled
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaRadioPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("options".to_owned(), options),
                    ("accent".to_owned(), accent),
                    ("disabled".to_owned(), disabled),
                ],
            })
        }
        "nova_textarea_packet" => {
            let (lines, scroll, placeholder, read_only, dirty) = match args {
                [lines, scroll] => (lines, scroll, None, None, None),
                [lines, scroll, placeholder] => (lines, scroll, Some(placeholder), None, None),
                [lines, scroll, placeholder, read_only] => {
                    (lines, scroll, Some(placeholder), Some(read_only), None)
                }
                [lines, scroll, placeholder, read_only, dirty] => (
                    lines,
                    scroll,
                    Some(placeholder),
                    Some(read_only),
                    Some(dirty),
                ),
                _ => {
                    return Err("nova_textarea_packet(...) expects 2, 3, 4 or 5 args".to_owned());
                }
            };
            let lines = lower_expr(
                lines,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scroll = lower_expr(
                scroll,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let placeholder = placeholder
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| lines.clone());
            let read_only = read_only
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            let dirty = dirty
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTextAreaPacket".to_owned(),
                fields: vec![
                    ("lines".to_owned(), lines),
                    ("scroll".to_owned(), scroll),
                    ("placeholder".to_owned(), placeholder),
                    ("read_only".to_owned(), read_only),
                    ("dirty".to_owned(), dirty),
                ],
            })
        }
        "nova_tabs_packet" => {
            let (active, count, accent, compact) = match args {
                [active, count, accent] => (active, count, accent, None),
                [active, count, accent, compact] => (active, count, accent, Some(compact)),
                _ => return Err("nova_tabs_packet(...) expects 3 or 4 args".to_owned()),
            };
            let active = lower_expr(
                active,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let count = lower_expr(
                count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let compact = compact
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTabsPacket".to_owned(),
                fields: vec![
                    ("active".to_owned(), active),
                    ("count".to_owned(), count),
                    ("accent".to_owned(), accent),
                    ("compact".to_owned(), compact),
                ],
            })
        }
        "nova_list_packet" => {
            let (selected, items, accent, dense) = match args {
                [selected, items, accent] => (selected, items, accent, None),
                [selected, items, accent, dense] => (selected, items, accent, Some(dense)),
                _ => return Err("nova_list_packet(...) expects 3 or 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let items = lower_expr(
                items,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let dense = dense
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(0));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaListPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("items".to_owned(), items),
                    ("accent".to_owned(), accent),
                    ("dense".to_owned(), dense),
                ],
            })
        }
        "nova_table_packet" => {
            let (rows, cols, selected_row, zebra) = match args {
                [rows, cols, selected_row] => (rows, cols, selected_row, None),
                [rows, cols, selected_row, zebra] => (rows, cols, selected_row, Some(zebra)),
                _ => return Err("nova_table_packet(...) expects 3 or 4 args".to_owned()),
            };
            let rows = lower_expr(
                rows,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let cols = lower_expr(
                cols,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let selected_row = lower_expr(
                selected_row,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let zebra = zebra
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| NirExpr::Int(1));
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTablePacket".to_owned(),
                fields: vec![
                    ("rows".to_owned(), rows),
                    ("cols".to_owned(), cols),
                    ("selected_row".to_owned(), selected_row),
                    ("zebra".to_owned(), zebra),
                ],
            })
        }
        "nova_tree_packet" => {
            let (selected, nodes, expanded, accent) = match args {
                [selected, nodes, expanded, accent] => (selected, nodes, expanded, accent),
                _ => return Err("nova_tree_packet(...) expects 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let nodes = lower_expr(
                nodes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let expanded = lower_expr(
                expanded,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTreePacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("nodes".to_owned(), nodes),
                    ("expanded".to_owned(), expanded),
                    ("accent".to_owned(), accent),
                ],
            })
        }
        "nova_inspector_packet" => {
            let (selected, fields, pinned, accent) = match args {
                [selected, fields, pinned, accent] => (selected, fields, pinned, accent),
                _ => return Err("nova_inspector_packet(...) expects 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let fields = lower_expr(
                fields,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let pinned = lower_expr(
                pinned,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaInspectorPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("fields".to_owned(), fields),
                    ("pinned".to_owned(), pinned),
                    ("accent".to_owned(), accent),
                ],
            })
        }
        "nova_outline_packet" => {
            let (selected, items, collapsed, accent) = match args {
                [selected, items, collapsed, accent] => (selected, items, collapsed, accent),
                _ => return Err("nova_outline_packet(...) expects 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let items = lower_expr(
                items,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let collapsed = lower_expr(
                collapsed,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaOutlinePacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("items".to_owned(), items),
                    ("collapsed".to_owned(), collapsed),
                    ("accent".to_owned(), accent),
                ],
            })
        }
        "nova_selection_packet" => {
            let (selected, span, mode, origin) = match args {
                [selected, span, mode, origin] => (selected, span, mode, origin),
                _ => return Err("nova_selection_packet(...) expects 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let span = lower_expr(
                span,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let mode = lower_expr(
                mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let origin = lower_expr(
                origin,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSelectionPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("span".to_owned(), span),
                    ("mode".to_owned(), mode),
                    ("origin".to_owned(), origin),
                ],
            })
        }
        "nova_focus_packet" => {
            let [slot] = args else {
                return Err("nova_focus_packet(...) expects 1 arg".to_owned());
            };
            let slot = lower_expr(
                slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaFocusPacket".to_owned(),
                fields: vec![("slot".to_owned(), slot)],
            })
        }
        "nova_slider_group_packet" => {
            let [color, speed, radius] = args else {
                return Err("nova_slider_group_packet(...) expects 3 args".to_owned());
            };
            let color = lower_expr(
                color,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            let speed = lower_expr(
                speed,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            let radius = lower_expr(
                radius,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSliderGroupPacket".to_owned(),
                fields: vec![
                    ("color".to_owned(), color),
                    ("speed".to_owned(), speed),
                    ("radius".to_owned(), radius),
                ],
            })
        }
        "nova_panel_from_parts" => {
            let [header, sliders, toggle, progress, meter, button, text_input, select, checkbox, radio, textarea, tabs, list, table, tree, inspector, outline, theme, surface, viewport, layer, focus] =
                args
            else {
                return Err("nova_panel_from_parts(...) expects 22 args".to_owned());
            };
            let header = lower_expr(
                header,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaHeaderPacket")),
            )?;
            let sliders = lower_expr(
                sliders,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderGroupPacket")),
            )?;
            let toggle = lower_expr(
                toggle,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTogglePacket")),
            )?;
            let progress = lower_expr(
                progress,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaProgressPacket")),
            )?;
            let meter = lower_expr(
                meter,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaMeterPacket")),
            )?;
            let button = lower_expr(
                button,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaButtonPacket")),
            )?;
            let text_input = lower_expr(
                text_input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextInputPacket")),
            )?;
            let select = lower_expr(
                select,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSelectPacket")),
            )?;
            let checkbox = lower_expr(
                checkbox,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCheckboxPacket")),
            )?;
            let radio = lower_expr(
                radio,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaRadioPacket")),
            )?;
            let textarea = lower_expr(
                textarea,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextAreaPacket")),
            )?;
            let tabs = lower_expr(
                tabs,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTabsPacket")),
            )?;
            let list = lower_expr(
                list,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaListPacket")),
            )?;
            let table = lower_expr(
                table,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTablePacket")),
            )?;
            let tree = lower_expr(
                tree,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTreePacket")),
            )?;
            let inspector = lower_expr(
                inspector,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaInspectorPacket")),
            )?;
            let outline = lower_expr(
                outline,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaOutlinePacket")),
            )?;
            let theme = lower_expr(
                theme,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaThemePacket")),
            )?;
            let surface = lower_expr(
                surface,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSurfacePacket")),
            )?;
            let viewport = lower_expr(
                viewport,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaViewportPacket")),
            )?;
            let layer = lower_expr(
                layer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaLayerPacket")),
            )?;
            let focus = lower_expr(
                focus,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFocusPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPanelPacket".to_owned(),
                fields: vec![
                    ("header".to_owned(), header),
                    ("sliders".to_owned(), sliders),
                    ("toggle".to_owned(), toggle),
                    ("progress".to_owned(), progress),
                    ("meter".to_owned(), meter),
                    ("button".to_owned(), button),
                    ("text_input".to_owned(), text_input),
                    ("select".to_owned(), select),
                    ("checkbox".to_owned(), checkbox),
                    ("radio".to_owned(), radio),
                    ("textarea".to_owned(), textarea),
                    ("tabs".to_owned(), tabs),
                    ("list".to_owned(), list),
                    ("table".to_owned(), table),
                    ("tree".to_owned(), tree),
                    ("inspector".to_owned(), inspector),
                    ("outline".to_owned(), outline),
                    ("theme".to_owned(), theme),
                    ("surface".to_owned(), surface),
                    ("viewport".to_owned(), viewport),
                    ("layer".to_owned(), layer),
                    ("focus".to_owned(), focus),
                ],
            })
        }
        "nova_slider_disabled"
        | "nova_toggle_disabled"
        | "nova_text_input_dirty"
        | "nova_text_input_read_only"
        | "nova_select_committed"
        | "nova_select_multiple"
        | "nova_checkbox_checked"
        | "nova_checkbox_disabled"
        | "nova_radio_disabled"
        | "nova_textarea_dirty"
        | "nova_textarea_read_only"
        | "nova_tabs_compact"
        | "nova_list_dense"
        | "nova_table_zebra"
        | "nova_tree_expanded"
        | "nova_inspector_pinned"
        | "nova_outline_collapsed"
        | "nova_selection_selected"
        | "nova_selection_mode" => {
            let [packet] = args else {
                return Err(format!("{callee}(...) expects 1 arg"));
            };
            let (expected_type, field_name) = match callee {
                "nova_slider_disabled" => ("NovaSliderPacket", "disabled"),
                "nova_toggle_disabled" => ("NovaTogglePacket", "disabled"),
                "nova_text_input_dirty" => ("NovaTextInputPacket", "dirty"),
                "nova_text_input_read_only" => ("NovaTextInputPacket", "read_only"),
                "nova_select_committed" => ("NovaSelectPacket", "committed"),
                "nova_select_multiple" => ("NovaSelectPacket", "multiple"),
                "nova_checkbox_checked" => ("NovaCheckboxPacket", "checked"),
                "nova_checkbox_disabled" => ("NovaCheckboxPacket", "disabled"),
                "nova_radio_disabled" => ("NovaRadioPacket", "disabled"),
                "nova_textarea_dirty" => ("NovaTextAreaPacket", "dirty"),
                "nova_textarea_read_only" => ("NovaTextAreaPacket", "read_only"),
                "nova_tabs_compact" => ("NovaTabsPacket", "compact"),
                "nova_list_dense" => ("NovaListPacket", "dense"),
                "nova_table_zebra" => ("NovaTablePacket", "zebra"),
                "nova_tree_expanded" => ("NovaTreePacket", "expanded"),
                "nova_inspector_pinned" => ("NovaInspectorPacket", "pinned"),
                "nova_outline_collapsed" => ("NovaOutlinePacket", "collapsed"),
                "nova_selection_selected" => ("NovaSelectionPacket", "selected"),
                _ => ("NovaSelectionPacket", "mode"),
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type(expected_type)),
            )?;
            Ok(NirExpr::FieldAccess {
                base: Box::new(packet),
                field: field_name.to_owned(),
            })
        }
        "nova_slider_state" => {
            let [packet] = args else {
                return Err("nova_slider_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSliderState".to_owned(),
                fields: vec![
                    (
                        "value".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "value".to_owned(),
                        },
                    ),
                    (
                        "min".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "min".to_owned(),
                        },
                    ),
                    (
                        "max".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "max".to_owned(),
                        },
                    ),
                    (
                        "step".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "step".to_owned(),
                        },
                    ),
                    (
                        "disabled".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "disabled".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_toggle_state" => {
            let [packet] = args else {
                return Err("nova_toggle_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTogglePacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaToggleState".to_owned(),
                fields: vec![
                    (
                        "live".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "live".to_owned(),
                        },
                    ),
                    (
                        "disabled".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "disabled".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_text_input_state" => {
            let [packet] = args else {
                return Err("nova_text_input_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextInputPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTextInputState".to_owned(),
                fields: vec![
                    (
                        "dirty".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "dirty".to_owned(),
                        },
                    ),
                    (
                        "read_only".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "read_only".to_owned(),
                        },
                    ),
                    (
                        "caret".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "caret".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_select_state" => {
            let [packet] = args else {
                return Err("nova_select_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSelectPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSelectState".to_owned(),
                fields: vec![
                    (
                        "committed".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "committed".to_owned(),
                        },
                    ),
                    (
                        "multiple".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "multiple".to_owned(),
                        },
                    ),
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "selected".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_checkbox_state" => {
            let [packet] = args else {
                return Err("nova_checkbox_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCheckboxPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaCheckboxState".to_owned(),
                fields: vec![
                    (
                        "checked".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "checked".to_owned(),
                        },
                    ),
                    (
                        "disabled".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "disabled".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_radio_state" => {
            let [packet] = args else {
                return Err("nova_radio_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaRadioPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaRadioState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected".to_owned(),
                        },
                    ),
                    (
                        "options".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "options".to_owned(),
                        },
                    ),
                    (
                        "disabled".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "disabled".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_textarea_state" => {
            let [packet] = args else {
                return Err("nova_textarea_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextAreaPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTextAreaState".to_owned(),
                fields: vec![
                    (
                        "lines".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "lines".to_owned(),
                        },
                    ),
                    (
                        "scroll".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "scroll".to_owned(),
                        },
                    ),
                    (
                        "read_only".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "read_only".to_owned(),
                        },
                    ),
                    (
                        "dirty".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "dirty".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_tabs_state" => {
            let [packet] = args else {
                return Err("nova_tabs_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTabsPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTabsState".to_owned(),
                fields: vec![
                    (
                        "active".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "active".to_owned(),
                        },
                    ),
                    (
                        "count".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "count".to_owned(),
                        },
                    ),
                    (
                        "compact".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "compact".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_list_state" => {
            let [packet] = args else {
                return Err("nova_list_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaListPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaListState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected".to_owned(),
                        },
                    ),
                    (
                        "items".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "items".to_owned(),
                        },
                    ),
                    (
                        "dense".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "dense".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_table_state" => {
            let [packet] = args else {
                return Err("nova_table_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTablePacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTableState".to_owned(),
                fields: vec![
                    (
                        "rows".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "rows".to_owned(),
                        },
                    ),
                    (
                        "cols".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "cols".to_owned(),
                        },
                    ),
                    (
                        "selected_row".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected_row".to_owned(),
                        },
                    ),
                    (
                        "zebra".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "zebra".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_tree_state" => {
            let [packet] = args else {
                return Err("nova_tree_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTreePacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTreeState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected".to_owned(),
                        },
                    ),
                    (
                        "nodes".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "nodes".to_owned(),
                        },
                    ),
                    (
                        "expanded".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "expanded".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_inspector_state" => {
            let [packet] = args else {
                return Err("nova_inspector_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaInspectorPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaInspectorState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected".to_owned(),
                        },
                    ),
                    (
                        "fields".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "fields".to_owned(),
                        },
                    ),
                    (
                        "pinned".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "pinned".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_outline_state" => {
            let [packet] = args else {
                return Err("nova_outline_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaOutlinePacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaOutlineState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected".to_owned(),
                        },
                    ),
                    (
                        "items".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "items".to_owned(),
                        },
                    ),
                    (
                        "collapsed".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "collapsed".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_theme_state" => {
            let [packet] = args else {
                return Err("nova_theme_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaThemePacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaThemeState".to_owned(),
                fields: vec![
                    (
                        "accent".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "accent".to_owned(),
                        },
                    ),
                    (
                        "surface".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "surface".to_owned(),
                        },
                    ),
                    (
                        "panel_mode".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "panel_mode".to_owned(),
                        },
                    ),
                    (
                        "contrast".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "contrast".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_surface_state" => {
            let [packet] = args else {
                return Err("nova_surface_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSurfacePacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSurfaceState".to_owned(),
                fields: vec![
                    (
                        "density".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "density".to_owned(),
                        },
                    ),
                    (
                        "elevation".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "elevation".to_owned(),
                        },
                    ),
                    (
                        "grid".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "grid".to_owned(),
                        },
                    ),
                    (
                        "sheen".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "sheen".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_viewport_state" => {
            let [packet] = args else {
                return Err("nova_viewport_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaViewportPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaViewportState".to_owned(),
                fields: vec![
                    (
                        "origin_x".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "origin_x".to_owned(),
                        },
                    ),
                    (
                        "origin_y".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "origin_y".to_owned(),
                        },
                    ),
                    (
                        "width".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "width".to_owned(),
                        },
                    ),
                    (
                        "height".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "height".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_layer_state" => {
            let [packet] = args else {
                return Err("nova_layer_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaLayerPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaLayerState".to_owned(),
                fields: vec![
                    (
                        "order".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "order".to_owned(),
                        },
                    ),
                    (
                        "blend".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "blend".to_owned(),
                        },
                    ),
                    (
                        "visibility".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "visibility".to_owned(),
                        },
                    ),
                    (
                        "clip".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "clip".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_selection_state" => {
            let [packet] = args else {
                return Err("nova_selection_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSelectionPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSelectionState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "selected".to_owned(),
                        },
                    ),
                    (
                        "span".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "span".to_owned(),
                        },
                    ),
                    (
                        "mode".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: "mode".to_owned(),
                        },
                    ),
                    (
                        "origin".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: "origin".to_owned(),
                        },
                    ),
                ],
            })
        }
        "nova_list_selection"
        | "nova_table_selection"
        | "nova_tree_selection"
        | "nova_inspector_selection"
        | "nova_outline_selection" => {
            let [packet] = args else {
                return Err(format!("{callee}(...) expects 1 arg"));
            };
            let (expected_type, selected_field, span_field, mode_field, origin) = match callee {
                "nova_list_selection" => ("NovaListPacket", "selected", "items", "dense", 0),
                "nova_table_selection" => ("NovaTablePacket", "selected_row", "rows", "zebra", 1),
                "nova_tree_selection" => ("NovaTreePacket", "selected", "nodes", "expanded", 2),
                "nova_inspector_selection" => {
                    ("NovaInspectorPacket", "selected", "fields", "pinned", 3)
                }
                _ => ("NovaOutlinePacket", "selected", "items", "collapsed", 4),
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type(expected_type)),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSelectionState".to_owned(),
                fields: vec![
                    (
                        "selected".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: selected_field.to_owned(),
                        },
                    ),
                    (
                        "span".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet.clone()),
                            field: span_field.to_owned(),
                        },
                    ),
                    (
                        "mode".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(packet),
                            field: mode_field.to_owned(),
                        },
                    ),
                    ("origin".to_owned(), NirExpr::Int(origin)),
                ],
            })
        }
        "nova_slider_state_disabled"
        | "nova_toggle_state_disabled"
        | "nova_text_input_state_dirty"
        | "nova_text_input_state_read_only"
        | "nova_select_state_committed"
        | "nova_select_state_multiple"
        | "nova_checkbox_state_checked"
        | "nova_checkbox_state_disabled"
        | "nova_radio_state_selected"
        | "nova_radio_state_disabled"
        | "nova_textarea_state_dirty"
        | "nova_textarea_state_read_only"
        | "nova_tabs_state_active"
        | "nova_tabs_state_compact"
        | "nova_list_state_dense"
        | "nova_list_state_selected"
        | "nova_table_state_zebra"
        | "nova_table_state_selected_row"
        | "nova_tree_state_expanded"
        | "nova_tree_state_selected"
        | "nova_inspector_state_pinned"
        | "nova_inspector_state_selected"
        | "nova_outline_state_collapsed"
        | "nova_outline_state_selected"
        | "nova_theme_state_accent"
        | "nova_theme_state_surface"
        | "nova_theme_state_panel_mode"
        | "nova_theme_state_contrast"
        | "nova_surface_state_density"
        | "nova_surface_state_elevation"
        | "nova_surface_state_grid"
        | "nova_surface_state_sheen"
        | "nova_viewport_state_origin_x"
        | "nova_viewport_state_origin_y"
        | "nova_viewport_state_width"
        | "nova_viewport_state_height"
        | "nova_layer_state_order"
        | "nova_layer_state_blend"
        | "nova_layer_state_visibility"
        | "nova_layer_state_clip"
        | "nova_selection_state_selected"
        | "nova_selection_state_span"
        | "nova_selection_state_mode"
        | "nova_selection_state_origin" => {
            let [state] = args else {
                return Err(format!("{callee}(...) expects 1 arg"));
            };
            let (expected_type, field_name) = match callee {
                "nova_slider_state_disabled" => ("NovaSliderState", "disabled"),
                "nova_toggle_state_disabled" => ("NovaToggleState", "disabled"),
                "nova_text_input_state_dirty" => ("NovaTextInputState", "dirty"),
                "nova_text_input_state_read_only" => ("NovaTextInputState", "read_only"),
                "nova_select_state_committed" => ("NovaSelectState", "committed"),
                "nova_select_state_multiple" => ("NovaSelectState", "multiple"),
                "nova_checkbox_state_checked" => ("NovaCheckboxState", "checked"),
                "nova_checkbox_state_disabled" => ("NovaCheckboxState", "disabled"),
                "nova_radio_state_selected" => ("NovaRadioState", "selected"),
                "nova_radio_state_disabled" => ("NovaRadioState", "disabled"),
                "nova_textarea_state_dirty" => ("NovaTextAreaState", "dirty"),
                "nova_textarea_state_read_only" => ("NovaTextAreaState", "read_only"),
                "nova_tabs_state_active" => ("NovaTabsState", "active"),
                "nova_tabs_state_compact" => ("NovaTabsState", "compact"),
                "nova_list_state_dense" => ("NovaListState", "dense"),
                "nova_list_state_selected" => ("NovaListState", "selected"),
                "nova_table_state_zebra" => ("NovaTableState", "zebra"),
                "nova_table_state_selected_row" => ("NovaTableState", "selected_row"),
                "nova_tree_state_expanded" => ("NovaTreeState", "expanded"),
                "nova_tree_state_selected" => ("NovaTreeState", "selected"),
                "nova_inspector_state_pinned" => ("NovaInspectorState", "pinned"),
                "nova_inspector_state_selected" => ("NovaInspectorState", "selected"),
                "nova_outline_state_collapsed" => ("NovaOutlineState", "collapsed"),
                "nova_outline_state_selected" => ("NovaOutlineState", "selected"),
                "nova_theme_state_accent" => ("NovaThemeState", "accent"),
                "nova_theme_state_surface" => ("NovaThemeState", "surface"),
                "nova_theme_state_panel_mode" => ("NovaThemeState", "panel_mode"),
                "nova_theme_state_contrast" => ("NovaThemeState", "contrast"),
                "nova_surface_state_density" => ("NovaSurfaceState", "density"),
                "nova_surface_state_elevation" => ("NovaSurfaceState", "elevation"),
                "nova_surface_state_grid" => ("NovaSurfaceState", "grid"),
                "nova_surface_state_sheen" => ("NovaSurfaceState", "sheen"),
                "nova_viewport_state_origin_x" => ("NovaViewportState", "origin_x"),
                "nova_viewport_state_origin_y" => ("NovaViewportState", "origin_y"),
                "nova_viewport_state_width" => ("NovaViewportState", "width"),
                "nova_viewport_state_height" => ("NovaViewportState", "height"),
                "nova_layer_state_order" => ("NovaLayerState", "order"),
                "nova_layer_state_blend" => ("NovaLayerState", "blend"),
                "nova_layer_state_visibility" => ("NovaLayerState", "visibility"),
                "nova_layer_state_clip" => ("NovaLayerState", "clip"),
                "nova_selection_state_selected" => ("NovaSelectionState", "selected"),
                "nova_selection_state_span" => ("NovaSelectionState", "span"),
                "nova_selection_state_mode" => ("NovaSelectionState", "mode"),
                _ => ("NovaSelectionState", "origin"),
            };
            let state = lower_expr(
                state,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type(expected_type)),
            )?;
            Ok(NirExpr::FieldAccess {
                base: Box::new(state),
                field: field_name.to_owned(),
            })
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
            let handle_table_type = select_expected_semantic_token_type(expected, "HandleTable");
            validate_type_ref(&handle_table_type)?;
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
            let marker_type = select_expected_semantic_token_type(expected, "Marker");
            validate_type_ref(&marker_type)?;
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
        "kernel_result" => lower_result_wrapper_call(
            "kernel_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Kernel,
            infer_result_stage,
            |value, stage| match stage {
                NirResultStage::Kernel(state) => Ok(NirExpr::KernelResult { value, state }),
                other => Err(format!(
                    "expected kernel result stage, found `{}`",
                    other.render()
                )),
            },
            "expects a direct kernel profile/config expression",
        ),
        "kernel_config_ready" => lower_result_observer_call(
            "kernel_config_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Kernel,
            |expr| NirExpr::KernelConfigReady(Box::new(expr)),
        ),
        "kernel_value" => lower_result_observer_call(
            "kernel_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            NirResultFamily::Kernel,
            |expr| NirExpr::KernelValue(Box::new(expr)),
        ),
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
                    lower_nested_expr_with_async(
                        arg,
                        current_domain,
                        current_function_is_async,
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
                if signature.is_async {
                    if !current_function_is_async {
                        return Err(format!(
                            "async function `{callee}` can only be called inside `async fn`"
                        ));
                    }
                    if !allow_async_calls {
                        return Err(format!(
                            "async function `{callee}` must be used under `await`"
                        ));
                    }
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

fn ensure_task_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.container_kind() == Some(nuis_semantics::model::NirContainerKind::Task) => {
            Ok(())
        }
        Some(ty) => Err(format!(
            "{name}(...) expects `Task<...>`, found `{}`",
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed task handle in the current frontend"
        )),
    }
}

fn lower_single_nested_expr(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [value] = args else {
        return Err(format!("{name}(...) expects exactly one argument"));
    };
    lower_nested_expr_with_async(
        value,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
        None,
    )
}

fn lower_result_wrapper_call(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    family: NirResultFamily,
    infer_stage: fn(&NirExpr) -> Option<NirResultStage>,
    build: fn(Box<NirExpr>, NirResultStage) -> Result<NirExpr, String>,
    usage_hint: &str,
) -> Result<NirExpr, String> {
    let lowered = lower_single_nested_expr(
        name,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
    )?;
    let Some(stage) = infer_stage(&lowered) else {
        return Err(format!("{name}(...) {usage_hint}"));
    };
    if !family.supports_stage(stage) {
        return Err(format!(
            "{name}(...) inferred incompatible `{}` stage `{}`",
            family.type_name(),
            stage.render()
        ));
    }
    let payload = expr_type(&lowered, bindings, signatures, struct_table)
        .ok_or_else(|| format!("{name}(...) could not infer payload type for result wrapper"))?;
    validate_result_stage_payload(stage, &payload)
        .map_err(|error| format!("{name}(...): {error}"))?;
    build(Box::new(lowered), stage).map_err(|error| format!("{name}(...): {error}"))
}

fn validate_result_stage_payload(
    stage: NirResultStage,
    payload: &NirTypeRef,
) -> Result<(), String> {
    stage.validate_payload(payload)
}

fn lower_result_observer_call(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    family: NirResultFamily,
    build: fn(NirExpr) -> NirExpr,
) -> Result<NirExpr, String> {
    let lowered = lower_single_nested_expr(
        name,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
    )?;
    ensure_result_like(name, &lowered, family, bindings, signatures, struct_table)?;
    Ok(build(lowered))
}

fn infer_result_stage(expr: &NirExpr) -> Option<NirResultStage> {
    match expr {
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::DataInputPipe(_) => Some(NirDataFlowState::Ready.into()),
        NirExpr::DataOutputPipe(_) => Some(NirDataFlowState::Moved.into()),
        NirExpr::DataCopyWindow { .. }
        | NirExpr::DataWriteWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. } => Some(NirDataFlowState::Windowed.into()),
        NirExpr::ShaderBeginPass { .. } => Some(NirShaderFlowState::PassReady.into()),
        NirExpr::ShaderDrawInstanced { .. } | NirExpr::ShaderProfileRender { .. } => {
            Some(NirShaderFlowState::FrameReady.into())
        }
        NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. } => {
            Some(NirKernelFlowState::ConfigReady.into())
        }
        _ => None,
    }
}

fn ensure_result_like(
    name: &str,
    expr: &NirExpr,
    family: NirResultFamily,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.result_family() == Some(family) => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects `{}<...>`, found `{}`",
            family.type_name(),
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed {} in the current frontend",
            family.type_name().to_ascii_lowercase()
        )),
    }
}

fn make_result_type(family: NirResultFamily, payload: NirTypeRef) -> NirTypeRef {
    generic_named_type(family.type_name(), vec![payload])
}

fn expr_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    infer_nir_expr_type(expr, bindings, signatures, struct_table)
}

fn result_payload_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    expr_type(expr, bindings, signatures, struct_table).and_then(|ty| {
        ty.result_payload()
            .cloned()
            .or_else(|| ty.container_payload().cloned())
    })
}

fn infer_nir_expr_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Bool(_) | NirExpr::IsNull(_) => Some(bool_type()),
        NirExpr::Text(_) => Some(string_type()),
        NirExpr::Int(_) => Some(i64_type()),
        NirExpr::Var(name) => bindings.get(name).cloned(),
        NirExpr::Await(value) => infer_nir_expr_type(value, bindings, signatures, struct_table),
        NirExpr::Instantiate { unit, .. } => {
            Some(generic_named_type("Instance", vec![named_type(unit)]))
        }
        NirExpr::Null => None,
        NirExpr::Borrow(value) | NirExpr::Move(value) => {
            infer_nir_expr_type(value, bindings, signatures, struct_table)
        }
        NirExpr::BorrowEnd(_) => Some(unit_type()),
        NirExpr::AllocNode { .. } => Some(ref_type("Node")),
        NirExpr::AllocBuffer { .. } => Some(ref_type("Buffer")),
        NirExpr::DataBindCore(_) | NirExpr::CpuBindCore(_) => Some(unit_type()),
        NirExpr::CpuWindow { .. } => Some(named_type("Window")),
        NirExpr::CpuInputI64 { .. } | NirExpr::CpuTickI64 { .. } => Some(i64_type()),
        NirExpr::CpuSpawn { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone())
            .map(|ty| generic_named_type("Task", vec![ty])),
        NirExpr::CpuJoin(task) => result_payload_type(task, bindings, signatures, struct_table),
        NirExpr::CpuCancel(task) => infer_nir_expr_type(task, bindings, signatures, struct_table),
        NirExpr::CpuJoinResult(task) => {
            result_payload_type(task, bindings, signatures, struct_table)
                .map(|ty| make_result_type(NirResultFamily::Task, ty))
        }
        NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_) => Some(bool_type()),
        NirExpr::CpuTaskValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::CpuTimeout { task, .. } => {
            infer_nir_expr_type(task, bindings, signatures, struct_table)
        }
        NirExpr::CpuPresentFrame(_) => Some(unit_type()),
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
        NirExpr::ShaderProfilePacket {
            unit,
            packet_type_name,
            ..
        } => {
            let packet_name = packet_type_name
                .clone()
                .unwrap_or_else(|| format!("{unit}Packet"));
            Some(named_type(&packet_name))
        }
        NirExpr::DataProfileBindCoreRef { .. } => Some(named_type("Unit")),
        NirExpr::DataProfileWindowOffsetRef { .. } => Some(i64_type()),
        NirExpr::DataProfileUplinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileDownlinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileHandleTableRef { .. } => Some(named_type("HandleTable")),
        NirExpr::DataProfileMarkerRef { .. } => Some(named_type("Marker")),
        NirExpr::KernelProfileBindCoreRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileQueueDepthRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileBatchLanesRef { .. } => Some(i64_type()),
        NirExpr::KernelResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Kernel, inner)),
        NirExpr::KernelConfigReady(_) => Some(bool_type()),
        NirExpr::KernelValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            let window_inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            Some(generic_named_type("Window", vec![window_inner]))
        }
        NirExpr::DataResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Data, inner)),
        NirExpr::DataReady(_) | NirExpr::DataMoved(_) | NirExpr::DataWindowed(_) => {
            Some(bool_type())
        }
        NirExpr::DataValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::DataFreezeWindow(input) => {
            let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            let payload = match inner.window_mode() {
                Some(NirWindowMode::Mutable | NirWindowMode::Immutable) => {
                    inner.container_payload()?.clone()
                }
                None => return None,
            };
            Some(generic_named_type("Window", vec![payload]))
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
        NirExpr::ShaderResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Shader, inner)),
        NirExpr::ShaderPassReady(_) | NirExpr::ShaderFrameReady(_) => Some(bool_type()),
        NirExpr::ShaderValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::ShaderBeginPass { .. } => Some(named_type("Pass")),
        NirExpr::ShaderDrawInstanced { .. } => Some(named_type("Frame")),
        NirExpr::ShaderProfileRender { .. } => Some(named_type("Frame")),
        NirExpr::DataOutputPipe(value) => {
            let inner = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            Some(generic_named_type("Pipe", vec![inner]))
        }
        NirExpr::DataCopyWindow { input, .. } => infer_data_window_type(
            input,
            bindings,
            signatures,
            struct_table,
            NirWindowMode::Mutable,
        ),
        NirExpr::DataReadWindow { window, .. } => {
            let window_ty = infer_nir_expr_type(window, bindings, signatures, struct_table)?;
            window_ty.container_payload().cloned()
        }
        NirExpr::DataWriteWindow { window, value, .. } => {
            let window_ty = infer_nir_expr_type(window, bindings, signatures, struct_table)?;
            if window_ty.window_mode() != Some(NirWindowMode::Mutable) {
                return None;
            }
            let payload = window_ty.container_payload()?.clone();
            let value_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            if compatible_types(&payload, &value_ty) {
                Some(window_ty)
            } else {
                None
            }
        }
        NirExpr::DataImmutableWindow { input, .. } => infer_data_window_type(
            input,
            bindings,
            signatures,
            struct_table,
            NirWindowMode::Immutable,
        ),
        NirExpr::DataInputPipe(value) => {
            let pipe_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            pipe_ty.generic_args.first().cloned()
        }
        NirExpr::LoadValue(_) | NirExpr::LoadAt { .. } | NirExpr::BufferLen(_) => Some(i64_type()),
        NirExpr::LoadNext(_) => Some(ref_type("Node")),
        NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::Free(_) => Some(unit_type()),
        NirExpr::Call { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
        NirExpr::MethodCall { .. } => None,
        NirExpr::StructLiteral { type_name, .. } => Some(named_type(type_name)),
        NirExpr::FieldAccess { base, field } => {
            let base_ty = infer_nir_expr_type(base, bindings, signatures, struct_table)?;
            struct_field_type(&base_ty, field, struct_table)
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            let lhs_ty = infer_nir_expr_type(lhs, bindings, signatures, struct_table)?;
            let rhs_ty = infer_nir_expr_type(rhs, bindings, signatures, struct_table)?;
            if compatible_types(&lhs_ty, &rhs_ty) && lhs_ty.is_numeric_scalar() {
                Some(lhs_ty)
            } else {
                None
            }
        }
    }
}

fn infer_data_window_type(
    input: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    mode: NirWindowMode,
) -> Option<NirTypeRef> {
    let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
    let payload = if inner.is_ref && inner.name == "Buffer" {
        i64_type()
    } else {
        inner
    };
    Some(match mode {
        NirWindowMode::Mutable => generic_named_type("WindowMut", vec![payload]),
        NirWindowMode::Immutable => generic_named_type("Window", vec![payload]),
    })
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
    if expected.window_mode() == Some(NirWindowMode::Immutable)
        && actual.window_mode() == Some(NirWindowMode::Mutable)
        && expected.is_optional == actual.is_optional
        && expected.is_ref == actual.is_ref
        && expected.generic_args.len() == actual.generic_args.len()
    {
        return expected
            .generic_args
            .iter()
            .zip(&actual.generic_args)
            .all(|(lhs, rhs)| compatible_types(lhs, rhs));
    }
    if expected.name == actual.name
        && !expected.is_ref
        && !actual.is_ref
        && !expected.is_optional
        && !actual.is_optional
        && matches!(expected.name.as_str(), "Marker" | "HandleTable")
    {
        return expected.generic_args.is_empty()
            || actual.generic_args.is_empty()
            || (expected.generic_args.len() == actual.generic_args.len()
                && expected
                    .generic_args
                    .iter()
                    .zip(&actual.generic_args)
                    .all(|(lhs, rhs)| compatible_types(lhs, rhs)));
    }
    if expected.name != actual.name
        || expected.is_ref != actual.is_ref
        || expected.is_optional != actual.is_optional
        || expected.generic_args.len() != actual.generic_args.len()
    {
        return expected.is_ref && actual.is_ref && expected.generic_args.is_empty();
    }
    expected
        .generic_args
        .iter()
        .zip(&actual.generic_args)
        .all(|(lhs, rhs)| compatible_types(lhs, rhs))
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

fn string_type() -> NirTypeRef {
    named_type("String")
}

fn unit_type() -> NirTypeRef {
    named_type("Unit")
}

fn struct_field_type(
    base_ty: &NirTypeRef,
    field: &str,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    if let Some(builtin) = builtin_struct_field_type(&base_ty.name, field) {
        return Some(builtin);
    }
    struct_table
        .get(&base_ty.name)?
        .field(field)
        .map(|field| field.ty.clone())
}

fn builtin_struct_field_type(type_name: &str, field: &str) -> Option<NirTypeRef> {
    let i64 = || i64_type();
    let named = |name: &str| named_type(name);
    match type_name {
        "NovaHeaderPacket" => match field {
            "accent" | "title_mode" => Some(i64()),
            _ => None,
        },
        "NovaThemePacket" => match field {
            "accent" | "surface" | "panel_mode" | "contrast" => Some(i64()),
            _ => None,
        },
        "NovaSurfacePacket" => match field {
            "density" | "elevation" | "grid" | "sheen" => Some(i64()),
            _ => None,
        },
        "NovaViewportPacket" => match field {
            "origin_x" | "origin_y" | "width" | "height" => Some(i64()),
            _ => None,
        },
        "NovaLayerPacket" => match field {
            "order" | "blend" | "visibility" | "clip" => Some(i64()),
            _ => None,
        },
        "NovaSliderPacket" => match field {
            "value" | "min" | "max" | "step" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaSliderGroupPacket" => match field {
            "color" | "speed" | "radius" => Some(named("NovaSliderPacket")),
            _ => None,
        },
        "NovaTogglePacket" => match field {
            "live" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaProgressPacket" | "NovaMeterPacket" => match field {
            "value" | "max" => Some(i64()),
            _ => None,
        },
        "NovaButtonPacket" => match field {
            "active" | "accent" | "intent" => Some(i64()),
            _ => None,
        },
        "NovaTextInputPacket" => match field {
            "echo" | "caret" | "placeholder" | "read_only" | "dirty" => Some(i64()),
            _ => None,
        },
        "NovaSelectPacket" => match field {
            "selected" | "accent" | "options" | "multiple" | "committed" => Some(i64()),
            _ => None,
        },
        "NovaCheckboxPacket" => match field {
            "checked" | "accent" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaRadioPacket" => match field {
            "selected" | "options" | "accent" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaTextAreaPacket" => match field {
            "lines" | "scroll" | "placeholder" | "read_only" | "dirty" => Some(i64()),
            _ => None,
        },
        "NovaTabsPacket" => match field {
            "active" | "count" | "accent" | "compact" => Some(i64()),
            _ => None,
        },
        "NovaListPacket" => match field {
            "selected" | "items" | "accent" | "dense" => Some(i64()),
            _ => None,
        },
        "NovaTablePacket" => match field {
            "rows" | "cols" | "selected_row" | "zebra" => Some(i64()),
            _ => None,
        },
        "NovaTreePacket" => match field {
            "selected" | "nodes" | "expanded" | "accent" => Some(i64()),
            _ => None,
        },
        "NovaInspectorPacket" => match field {
            "selected" | "fields" | "pinned" | "accent" => Some(i64()),
            _ => None,
        },
        "NovaOutlinePacket" => match field {
            "selected" | "items" | "collapsed" | "accent" => Some(i64()),
            _ => None,
        },
        "NovaSelectionPacket" => match field {
            "selected" | "span" | "mode" | "origin" => Some(i64()),
            _ => None,
        },
        "NovaFocusPacket" => match field {
            "slot" => Some(i64()),
            _ => None,
        },
        "NovaPanelPacket" => match field {
            "header" => Some(named("NovaHeaderPacket")),
            "sliders" => Some(named("NovaSliderGroupPacket")),
            "toggle" => Some(named("NovaTogglePacket")),
            "progress" => Some(named("NovaProgressPacket")),
            "meter" => Some(named("NovaMeterPacket")),
            "button" => Some(named("NovaButtonPacket")),
            "text_input" => Some(named("NovaTextInputPacket")),
            "select" => Some(named("NovaSelectPacket")),
            "checkbox" => Some(named("NovaCheckboxPacket")),
            "radio" => Some(named("NovaRadioPacket")),
            "textarea" => Some(named("NovaTextAreaPacket")),
            "tabs" => Some(named("NovaTabsPacket")),
            "list" => Some(named("NovaListPacket")),
            "table" => Some(named("NovaTablePacket")),
            "tree" => Some(named("NovaTreePacket")),
            "inspector" => Some(named("NovaInspectorPacket")),
            "outline" => Some(named("NovaOutlinePacket")),
            "theme" => Some(named("NovaThemePacket")),
            "surface" => Some(named("NovaSurfacePacket")),
            "viewport" => Some(named("NovaViewportPacket")),
            "layer" => Some(named("NovaLayerPacket")),
            "focus" => Some(named("NovaFocusPacket")),
            _ => None,
        },
        "NovaSliderState" => match field {
            "value" | "min" | "max" | "step" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaToggleState" => match field {
            "live" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaTextInputState" => match field {
            "dirty" | "read_only" | "caret" => Some(i64()),
            _ => None,
        },
        "NovaSelectState" => match field {
            "committed" | "multiple" | "selected" => Some(i64()),
            _ => None,
        },
        "NovaCheckboxState" => match field {
            "checked" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaRadioState" => match field {
            "selected" | "options" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaTextAreaState" => match field {
            "lines" | "scroll" | "read_only" | "dirty" => Some(i64()),
            _ => None,
        },
        "NovaTabsState" => match field {
            "active" | "count" | "compact" => Some(i64()),
            _ => None,
        },
        "NovaListState" => match field {
            "selected" | "items" | "dense" => Some(i64()),
            _ => None,
        },
        "NovaTableState" => match field {
            "rows" | "cols" | "selected_row" | "zebra" => Some(i64()),
            _ => None,
        },
        "NovaTreeState" => match field {
            "selected" | "nodes" | "expanded" => Some(i64()),
            _ => None,
        },
        "NovaInspectorState" => match field {
            "selected" | "fields" | "pinned" => Some(i64()),
            _ => None,
        },
        "NovaOutlineState" => match field {
            "selected" | "items" | "collapsed" => Some(i64()),
            _ => None,
        },
        "NovaThemeState" => match field {
            "accent" | "surface" | "panel_mode" | "contrast" => Some(i64()),
            _ => None,
        },
        "NovaSurfaceState" => match field {
            "density" | "elevation" | "grid" | "sheen" => Some(i64()),
            _ => None,
        },
        "NovaViewportState" => match field {
            "origin_x" | "origin_y" | "width" | "height" => Some(i64()),
            _ => None,
        },
        "NovaLayerState" => match field {
            "order" | "blend" | "visibility" | "clip" => Some(i64()),
            _ => None,
        },
        "NovaSelectionState" => match field {
            "selected" | "span" | "mode" | "origin" => Some(i64()),
            _ => None,
        },
        _ => None,
    }
}

fn validate_declared_nir_types(module: &NirModule) -> Result<(), String> {
    for function in &module.externs {
        for param in &function.params {
            validate_type_ref(&param.ty)?;
        }
        validate_type_ref(&function.return_type)?;
    }
    for interface in &module.extern_interfaces {
        for method in &interface.methods {
            for param in &method.params {
                validate_type_ref(&param.ty)?;
            }
            validate_type_ref(&method.return_type)?;
        }
    }
    for definition in &module.structs {
        for field in &definition.fields {
            validate_type_ref(&field.ty)?;
        }
    }
    for function in &module.functions {
        if function.is_async && module.domain != "cpu" {
            return Err(format!(
                "mod {} {} cannot declare `async fn {}` yet; async entry is currently only supported in `mod cpu` while {} logic must stay AOT/synchronous and interact through explicit profile/data contracts",
                module.domain,
                module.unit,
                function.name,
                module.domain
            ));
        }
        if function.is_async
            && module.domain == "cpu"
            && function.name == "main"
            && !function.params.is_empty()
        {
            return Err(format!(
                "async entry `mod cpu {}::main` cannot take parameters in the current scheduler; pass data through explicit data/profile contracts or call async helpers from `main` instead",
                module.unit
            ));
        }
        for param in &function.params {
            validate_type_ref(&param.ty)?;
            if function.is_async && !param.ty.is_async_boundary_safe() {
                return Err(format!(
                    "async function `{}` parameter `{}` cannot cross async boundary with type `{}`; async parameters currently forbid `ref`, `?`, and `Instance<...>`",
                    function.name,
                    param.name,
                    param.ty.render()
                ));
            }
        }
        if let Some(return_type) = &function.return_type {
            validate_type_ref(return_type)?;
            if function.is_async && !return_type.is_async_boundary_safe() {
                return Err(format!(
                    "async function `{}` cannot return `{}` across async boundary; async returns currently forbid `ref`, `?`, and `Instance<...>`",
                    function.name,
                    return_type.render()
                ));
            }
        }
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { ty, .. } => {
                    if let Some(ty) = ty {
                        validate_type_ref(ty)?;
                    }
                }
                NirStmt::Const { ty, .. } => validate_type_ref(ty)?,
                NirStmt::Print(_)
                | NirStmt::Await(_)
                | NirStmt::Expr(_)
                | NirStmt::Return(_)
                | NirStmt::If { .. } => {}
            }
        }
    }
    Ok(())
}

fn validate_type_ref(ty: &NirTypeRef) -> Result<(), String> {
    ty.validate_container_contract()
        .map_err(|error| format!("invalid type `{}`: {error}", ty.render()))
}

fn select_expected_semantic_token_type(
    expected: Option<&NirTypeRef>,
    token_name: &str,
) -> NirTypeRef {
    match expected {
        Some(expected)
            if expected.name == token_name
                && !expected.is_ref
                && !expected.is_optional
                && expected.generic_args.len() <= 1 =>
        {
            expected.clone()
        }
        _ => named_type(token_name),
    }
}

fn render_type_name(ty: &NirTypeRef) -> String {
    ty.render()
}

#[cfg(test)]
mod tests {
    use super::parse_nuis_module;
    use nuis_semantics::model::{
        NirDataFlowState, NirExpr, NirKernelFlowState, NirShaderFlowState, NirStmt,
    };

    #[test]
    fn infers_struct_field_type_from_shared_type_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              struct Packet {
                count: i32,
                label: String,
              }

              fn pick(packet: Packet) -> i32 {
                return packet.count;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "pick")
            .unwrap();
        let return_type = function.return_type.as_ref().unwrap();
        assert_eq!(return_type.render(), "i32");
    }

    #[test]
    fn infers_binary_result_from_operand_scalar_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn add(lhs: i32, rhs: i32) -> i32 {
                let sum: i32 = lhs + rhs;
                return sum;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "add")
            .unwrap();
        let sum_stmt = function
            .body
            .iter()
            .find_map(|stmt| match stmt {
                NirStmt::Let { name, ty, .. } if name == "sum" => ty.as_ref(),
                _ => None,
            })
            .unwrap();
        assert_eq!(sum_stmt.render(), "i32");
    }

    #[test]
    fn rejects_non_numeric_binary_operands() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn join(lhs: String, rhs: String) -> String {
                let out: String = lhs + rhs;
                return out;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("numeric scalar operands"));
    }

    #[test]
    fn rejects_bare_window_type_without_payload() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let packet: Window = data_profile_send_uplink("FabricPlane", 7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("Window"));
        assert!(error.contains("payload type argument"));
    }

    #[test]
    fn rejects_nested_pipe_payload_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let pipe: Pipe<Pipe<i64>> = data_output_pipe(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("Pipe<Pipe"));
    }

    #[test]
    fn accepts_window_mut_type_annotation() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn keeps_window_annotation_compatible_with_copy_window_for_now() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: Window<i64> = data_copy_window(7, 0, 1);
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn infers_frozen_window_as_immutable_window_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let frozen: Window<i64> = data_freeze_window(data_copy_window(7, 0, 1));
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[0] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "Window<i64>");
    }

    #[test]
    fn infers_written_window_as_mutable_window_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
                let updated: WindowMut<i64> = data_write_window(copy, 0, 9);
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "WindowMut<i64>");
    }

    #[test]
    fn infers_buffer_backed_window_payload_as_i64() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let backing: ref Buffer = alloc_buffer(4, 0);
                let copy: WindowMut<i64> = data_copy_window(backing, 1, 2);
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "WindowMut<i64>");
    }

    #[test]
    fn infers_read_window_payload_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
                let value: i64 = data_read_window(copy, 0);
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "i64");
    }

    #[test]
    fn rejects_instance_of_scalar_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let wrong: Instance<i64> = instantiate shader SurfaceShader;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("nominal unit type"));
    }

    #[test]
    fn accepts_typed_marker_and_handle_table_annotations() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let handles: HandleTable<FabricBindings> =
                  data_profile_handle_table("FabricPlane");
                let ready: Marker<CpuToShader> =
                  data_profile_marker("FabricPlane", "cpu_to_shader");
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        let declared_types = function
            .body
            .iter()
            .filter_map(|stmt| match stmt {
                NirStmt::Let { ty: Some(ty), .. } => Some(ty.render()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(declared_types.contains(&"HandleTable<FabricBindings>".to_owned()));
        assert!(declared_types.contains(&"Marker<CpuToShader>".to_owned()));
    }

    #[test]
    fn rejects_marker_with_non_nominal_tag_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let ready: Marker<i64> = data_marker("cpu_to_shader");
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("nominal tag type"));
    }

    #[test]
    fn lowers_async_fn_and_await_stmt_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() {
                await ping();
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.is_async);
        assert!(matches!(function.body.first(), Some(NirStmt::Await(_))));
    }

    #[test]
    fn lowers_await_expression_in_let_and_return() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() -> i64 {
                let value: i64 = await ping();
                return await ping();
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                value: NirExpr::Await(_),
                ..
            })
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Return(Some(NirExpr::Await(_))))
        ));
    }

    #[test]
    fn lowers_await_expression_inside_call_args_and_binary_expr() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn add_one(value: i64) -> i64 {
                return value + 1;
              }

              async fn main() -> i64 {
                let value: i64 = add_one(await ping());
                return await ping() + value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                value: NirExpr::Call { args, .. },
                ..
            }) if matches!(args.first(), Some(NirExpr::Await(_)))
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Return(Some(NirExpr::Binary { lhs, .. })))
                if matches!(lhs.as_ref(), NirExpr::Await(_))
        ));
    }

    #[test]
    fn lowers_explicit_spawn_join_and_cancel() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = spawn(ping());
                cancel(task);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuSpawn { .. },
                ..
            }) if ty.render() == "Task<i64>"
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Expr(NirExpr::CpuCancel(_)))
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Return(Some(NirExpr::CpuJoin(_))))
        ));
    }

    #[test]
    fn rejects_spawn_of_sync_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() {
                let task: Task<i64> = spawn(ping());
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("spawn(...) expects async function call"));
    }

    #[test]
    fn rejects_join_of_non_task_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return join(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("expects `Task<...>`"));
    }

    #[test]
    fn lowers_explicit_data_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod data FabricPlane {
              fn main() -> i64 {
                let pipe_result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
                let moved: bool = data_moved(pipe_result);
                let intake: DataResult<i64> = data_result(data_input_pipe(data_output_pipe(9)));
                let ready: bool = data_ready(intake);
                let value: i64 = data_value(intake);
                return value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataResult { state, .. },
                ..
            }) if ty.render() == "DataResult<Pipe<i64>>"
                && matches!(state, NirDataFlowState::Moved)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataMoved(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataResult { state, .. },
                ..
            }) if ty.render() == "DataResult<i64>"
                && matches!(state, NirDataFlowState::Ready)
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn rejects_data_result_of_non_data_operation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let result: DataResult<i64> = data_result(7);
                return data_value(result);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("data_result(...) expects a direct data operation"));
    }

    #[test]
    fn lowers_explicit_shader_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass_result: ShaderResult<Pass> = shader_result(shader_begin_pass(
                  shader_target("rgba8", 16, 16),
                  shader_pipeline("flat", "triangle"),
                  shader_viewport(16, 16)
                ));
                let frame_result: ShaderResult<Frame> = shader_result(shader_profile_render(
                  "SurfaceShader",
                  shader_profile_packet("SurfaceShader", 1, 2, 3)
                ));
                let ready: bool = shader_frame_ready(frame_result);
                let frame: Frame = shader_value(frame_result);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderResult { state, .. },
                ..
            }) if ty.render() == "ShaderResult<Pass>"
                && matches!(state, NirShaderFlowState::PassReady)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderResult { state, .. },
                ..
            }) if ty.render() == "ShaderResult<Frame>"
                && matches!(state, NirShaderFlowState::FrameReady)
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderFrameReady(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderValue(_),
                ..
            }) if ty.render() == "Frame"
        ));
    }

    #[test]
    fn lowers_nova_panel_packet_without_shader_unit_literal() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value:
                    NirExpr::ShaderProfilePacket {
                        unit,
                        packet_type_name,
                        accent: Some(_),
                        toggle_state: Some(_),
                        focus_index: Some(_),
                        ..
                    },
                ..
            }) if ty.render() == "NovaPanelPacket"
                && unit == "__nova__"
                && packet_type_name.as_deref() == Some("NovaPanelPacket")
        ));
    }

    #[test]
    fn lowers_nova_control_packet_builders() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
                let progress: NovaProgressPacket = nova_progress_packet(4, 10);
                let toggle: NovaTogglePacket = nova_toggle_packet(1, 1);
                let button: NovaButtonPacket = nova_button_packet(1, 9, 2);
                let text_input: NovaTextInputPacket =
                  nova_text_input_packet(8, 1, 4, 1, 1);
                let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 0);
                let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 1);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 0, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 0);
                let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
                let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
                let selection: NovaSelectionPacket = nova_selection_packet(1, 6, 1, 4);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSliderPacket" && type_name == "NovaSliderPacket"
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaProgressPacket" && type_name == "NovaProgressPacket"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTogglePacket" && type_name == "NovaTogglePacket"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaButtonPacket" && type_name == "NovaButtonPacket"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextInputPacket" && type_name == "NovaTextInputPacket"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectPacket" && type_name == "NovaSelectPacket"
        ));
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaCheckboxPacket" && type_name == "NovaCheckboxPacket"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaRadioPacket" && type_name == "NovaRadioPacket"
        ));
        assert!(matches!(
            function.body.get(8),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextAreaPacket" && type_name == "NovaTextAreaPacket"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTabsPacket" && type_name == "NovaTabsPacket"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaListPacket" && type_name == "NovaListPacket"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTablePacket" && type_name == "NovaTablePacket"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTreePacket" && type_name == "NovaTreePacket"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaInspectorPacket" && type_name == "NovaInspectorPacket"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaOutlinePacket" && type_name == "NovaOutlinePacket"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaThemePacket" && type_name == "NovaThemePacket"
        ));
        assert!(matches!(
            function.body.get(16),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionPacket" && type_name == "NovaSelectionPacket"
        ));
    }

    #[test]
    fn lowers_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
                let text_input: NovaTextInputPacket =
                  nova_text_input_packet(8, 1, 4, 1, 1);
                let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
                let slider_disabled: i64 = nova_slider_disabled(slider);
                let dirty: i64 = nova_text_input_dirty(text_input);
                let committed: i64 = nova_select_committed(select);
                return committed;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "disabled"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dirty"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "committed"
        ));
    }

    #[test]
    fn lowers_extended_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 1);
                let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 0);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 1, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 1);
                let checkbox_state: NovaCheckboxState = nova_checkbox_state(checkbox);
                let radio_state: NovaRadioState = nova_radio_state(radio);
                let textarea_state: NovaTextAreaState = nova_textarea_state(textarea);
                let tabs_state: NovaTabsState = nova_tabs_state(tabs);
                let checked: i64 = nova_checkbox_state_checked(checkbox_state);
                let radio_disabled: i64 = nova_radio_state_disabled(radio_state);
                let dirty: i64 = nova_textarea_state_dirty(textarea_state);
                let compact: i64 = nova_tabs_state_compact(tabs_state);
                return checked + radio_disabled + dirty + compact;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaCheckboxState" && type_name == "NovaCheckboxState"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaRadioState" && type_name == "NovaRadioState"
        ));
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextAreaState" && type_name == "NovaTextAreaState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTabsState" && type_name == "NovaTabsState"
        ));
        assert!(matches!(
            function.body.get(8),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "checked"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "disabled"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dirty"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "compact"
        ));
    }

    #[test]
    fn lowers_complex_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
                let list_state: NovaListState = nova_list_state(list);
                let table_state: NovaTableState = nova_table_state(table);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
                let tree_state: NovaTreeState = nova_tree_state(tree);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
                let inspector_state: NovaInspectorState = nova_inspector_state(inspector);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
                let outline_state: NovaOutlineState = nova_outline_state(outline);
                let dense: i64 = nova_list_state_dense(list_state);
                let selected: i64 = nova_list_state_selected(list_state);
                let zebra: i64 = nova_table_state_zebra(table_state);
                let selected_row: i64 = nova_table_state_selected_row(table_state);
                let expanded: i64 = nova_tree_state_expanded(tree_state);
                let tree_selected: i64 = nova_tree_state_selected(tree_state);
                let pinned: i64 = nova_inspector_state_pinned(inspector_state);
                let inspected: i64 = nova_inspector_state_selected(inspector_state);
                let collapsed: i64 = nova_outline_state_collapsed(outline_state);
                let outlined: i64 = nova_outline_state_selected(outline_state);
                return dense + selected + zebra + selected_row + expanded + tree_selected + pinned + inspected + collapsed + outlined;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaListState" && type_name == "NovaListState"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTableState" && type_name == "NovaTableState"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTreeState" && type_name == "NovaTreeState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaInspectorState" && type_name == "NovaInspectorState"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaOutlineState" && type_name == "NovaOutlineState"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dense"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "zebra"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected_row"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "expanded"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(16),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "pinned"
        ));
        assert!(matches!(
            function.body.get(17),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(18),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "collapsed"
        ));
        assert!(matches!(
            function.body.get(19),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
    }

    #[test]
    fn lowers_shared_nova_selection_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let selection: NovaSelectionPacket = nova_selection_packet(2, 6, 1, 4);
                let list: NovaListPacket = nova_list_packet(2, 6, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 2, 1);
                let tree: NovaTreePacket = nova_tree_packet(2, 6, 1, 7);
                let inspector: NovaInspectorPacket = nova_inspector_packet(2, 4, 1, 7);
                let outline: NovaOutlinePacket = nova_outline_packet(2, 6, 1, 7);
                let state: NovaSelectionState = nova_selection_state(selection);
                let list_selection: NovaSelectionState = nova_list_selection(list);
                let table_selection: NovaSelectionState = nova_table_selection(table);
                let tree_selection: NovaSelectionState = nova_tree_selection(tree);
                let inspector_selection: NovaSelectionState = nova_inspector_selection(inspector);
                let outline_selection: NovaSelectionState = nova_outline_selection(outline);
                let selected: i64 = nova_selection_state_selected(state);
                let span: i64 = nova_selection_state_span(list_selection);
                let mode: i64 = nova_selection_state_mode(table_selection);
                let origin: i64 = nova_selection_state_origin(tree_selection);
                let inspector_origin: i64 = nova_selection_state_origin(inspector_selection);
                let outline_origin: i64 = nova_selection_state_origin(outline_selection);
                return selected + span + mode + origin + inspector_origin + outline_origin;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "span"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "mode"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "origin"
        ));
    }

    #[test]
    fn lowers_nova_theme_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
                let state: NovaThemeState = nova_theme_state(theme);
                let accent: i64 = nova_theme_state_accent(state);
                let surface: i64 = nova_theme_state_surface(state);
                let panel_mode: i64 = nova_theme_state_panel_mode(state);
                let contrast: i64 = nova_theme_state_contrast(state);
                return accent + surface + panel_mode + contrast;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaThemeState" && type_name == "NovaThemeState"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "accent"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "contrast"
        ));
    }

    #[test]
    fn lowers_nova_render_state_contracts() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let surface_state: NovaSurfaceState = nova_surface_state(surface);
                let viewport_state: NovaViewportState = nova_viewport_state(viewport);
                let layer_state: NovaLayerState = nova_layer_state(layer);
                let density: i64 = nova_surface_state_density(surface_state);
                let width: i64 = nova_viewport_state_width(viewport_state);
                let visibility: i64 = nova_layer_state_visibility(layer_state);
                return density + width + visibility;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSurfaceState" && type_name == "NovaSurfaceState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaViewportState" && type_name == "NovaViewportState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLayerState" && type_name == "NovaLayerState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_panel_from_parts_builder() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let header: NovaHeaderPacket = nova_header_packet(8);
                let slider_color: NovaSliderPacket = nova_slider_packet(1);
                let slider_speed: NovaSliderPacket = nova_slider_packet(2);
                let slider_radius: NovaSliderPacket = nova_slider_packet(3);
                let sliders: NovaSliderGroupPacket =
                  nova_slider_group_packet(slider_color, slider_speed, slider_radius);
                let toggle: NovaTogglePacket = nova_toggle_packet(1);
                let progress: NovaProgressPacket = nova_progress_packet(2);
                let meter: NovaMeterPacket = nova_meter_packet(3);
                let button: NovaButtonPacket = nova_button_packet(1, 8);
                let text_input: NovaTextInputPacket = nova_text_input_packet(4, 1);
                let select: NovaSelectPacket = nova_select_packet(0, 8);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 8);
                let radio: NovaRadioPacket = nova_radio_packet(1, 4, 8);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(0, 4, 8);
                let list: NovaListPacket = nova_list_packet(1, 5, 8);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 8);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 8);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 8);
                let theme: NovaThemePacket = nova_theme_packet(8, 3, 1, 2);
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let focus: NovaFocusPacket = nova_focus_packet(2);
                let panel: NovaPanelPacket = nova_panel_from_parts(
                  header,
                  sliders,
                  toggle,
                  progress,
                  meter,
                  button,
                  text_input,
                  select,
                  checkbox,
                  radio,
                  textarea,
                  tabs,
                  list,
                  table,
                  tree,
                  inspector,
                  outline,
                  theme,
                  surface,
                  viewport,
                  layer,
                  focus
                );
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPanelPacket" && type_name == "NovaPanelPacket",
            _ => false,
        }));
    }

    #[test]
    fn lowers_explicit_kernel_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let lanes: KernelResult<i64> = kernel_result(kernel_profile_batch_lanes("KernelUnit"));
                let ready: bool = kernel_config_ready(lanes);
                let value: i64 = kernel_value(lanes);
                return value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelResult { state, .. },
                ..
            }) if ty.render() == "KernelResult<i64>"
                && matches!(state, NirKernelFlowState::ConfigReady)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelConfigReady(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn lowers_explicit_timeout_on_task_handle() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuTimeout { .. },
                ..
            }) if ty.render() == "Task<i64>"
        ));
    }

    #[test]
    fn lowers_explicit_join_result_and_task_state_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                let result: TaskResult<i64> = join_result(task);
                if task_completed(result) {
                  return task_value(result);
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
                ..
            }) if ty.render() == "TaskResult<i64>"
        ));
    }

    #[test]
    fn rejects_timeout_with_non_integer_limit() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), "slow");
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("expects integer limit"));
    }

    #[test]
    fn rejects_await_inside_sync_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() {
                await ping();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("`await`"));
        assert!(error.contains("async fn"));
    }

    #[test]
    fn rejects_async_function_returning_ref_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn head() -> ref Node {
                return null();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return"));
        assert!(error.contains("ref Node"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_returning_result_family() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main() -> DataResult<i64> {
                return data_result(data_input_pipe(data_output_pipe(7)));
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("DataResult<i64>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_taking_instance_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn render(shader: Instance<SurfaceShader>) {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `shader`"));
        assert!(error.contains("Instance<SurfaceShader>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_taking_result_family_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(result: ShaderResult<Frame>) -> i64 {
                if shader_frame_ready(result) {
                  return 1;
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `result`"));
        assert!(error.contains("ShaderResult<Frame>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_shader_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod shader SurfaceShader {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod shader SurfaceShader"));
        assert!(error.contains("async fn profile"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_data_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod data FabricPlane {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod data FabricPlane"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_kernel_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod kernel KernelUnit {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod kernel KernelUnit"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_main_with_parameters() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main(seed: i64) {
                print(seed);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("async entry"));
        assert!(error.contains("Main::main"));
        assert!(error.contains("cannot take parameters"));
    }

    #[test]
    fn rejects_async_call_without_await() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() -> i64 {
                return ping();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("must be used under `await`"));
    }
}

mod lexer;
mod parser;

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef, NirBinaryOp,
    NirExpr, NirFunction, NirModule, NirParam, NirStmt, NirTypeRef,
};

pub fn frontend_name() -> &'static str {
    "nuisc-parser-minimal"
}

pub fn parse_nuis_ast(input: &str) -> Result<AstModule, String> {
    let tokens = lexer::tokenize(input)?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_module()
}

pub fn lower_ast_to_nir(module: &AstModule) -> NirModule {
    NirModule {
        domain: module.domain.clone(),
        name: module.name.clone(),
        functions: module.functions.iter().map(lower_function).collect(),
    }
}

pub fn parse_nuis_module(input: &str) -> Result<NirModule, String> {
    let ast = parse_nuis_ast(input)?;
    Ok(lower_ast_to_nir(&ast))
}

fn lower_function(function: &AstFunction) -> NirFunction {
    NirFunction {
        name: function.name.clone(),
        params: function.params.iter().map(lower_param).collect(),
        return_type: function.return_type.as_ref().map(lower_type_ref),
        body: function.body.iter().map(lower_stmt).collect(),
    }
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

fn lower_stmt(stmt: &AstStmt) -> NirStmt {
    match stmt {
        AstStmt::Let { name, ty, value } => NirStmt::Let {
            name: name.clone(),
            ty: ty.as_ref().map(lower_type_ref),
            value: lower_expr(value),
        },
        AstStmt::Const { name, ty, value } => NirStmt::Const {
            name: name.clone(),
            ty: lower_type_ref(ty),
            value: lower_expr(value),
        },
        AstStmt::Print(value) => NirStmt::Print(lower_expr(value)),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => NirStmt::If {
            condition: lower_expr(condition),
            then_body: then_body.iter().map(lower_stmt).collect(),
            else_body: else_body.iter().map(lower_stmt).collect(),
        },
        AstStmt::Expr(expr) => NirStmt::Expr(lower_expr(expr)),
        AstStmt::Return(value) => NirStmt::Return(value.as_ref().map(lower_expr)),
    }
}

fn lower_expr(expr: &AstExpr) -> NirExpr {
    match expr {
        AstExpr::Bool(value) => NirExpr::Bool(*value),
        AstExpr::Text(text) => NirExpr::Text(text.clone()),
        AstExpr::Int(value) => NirExpr::Int(*value),
        AstExpr::Var(name) => NirExpr::Var(name.clone()),
        AstExpr::Call { callee, args } => lower_call_expr(callee, args),
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(lower_expr(receiver)),
            method: method.clone(),
            args: args.iter().map(lower_expr).collect(),
        },
        AstExpr::StructLiteral { type_name, fields } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), lower_expr(value)))
                .collect(),
        },
        AstExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(lower_expr(base)),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: match op {
                AstBinaryOp::Add => NirBinaryOp::Add,
                AstBinaryOp::Sub => NirBinaryOp::Sub,
                AstBinaryOp::Mul => NirBinaryOp::Mul,
                AstBinaryOp::Div => NirBinaryOp::Div,
            },
            lhs: Box::new(lower_expr(lhs)),
            rhs: Box::new(lower_expr(rhs)),
        },
    }
}

fn lower_call_expr(callee: &str, args: &[AstExpr]) -> NirExpr {
    let lowered_args = args.iter().map(lower_expr).collect::<Vec<_>>();
    match (callee, lowered_args.as_slice()) {
        ("null", []) => NirExpr::Null,
        ("borrow", [value]) => NirExpr::Borrow(Box::new(value.clone())),
        ("move", [value]) => NirExpr::Move(Box::new(value.clone())),
        ("alloc_node", [value, next]) => NirExpr::AllocNode {
            value: Box::new(value.clone()),
            next: Box::new(next.clone()),
        },
        ("alloc_buffer", [len, fill]) => NirExpr::AllocBuffer {
            len: Box::new(len.clone()),
            fill: Box::new(fill.clone()),
        },
        ("load_value", [ptr]) => NirExpr::LoadValue(Box::new(ptr.clone())),
        ("load_next", [ptr]) => NirExpr::LoadNext(Box::new(ptr.clone())),
        ("buffer_len", [ptr]) => NirExpr::BufferLen(Box::new(ptr.clone())),
        ("load_at", [buffer, index]) => NirExpr::LoadAt {
            buffer: Box::new(buffer.clone()),
            index: Box::new(index.clone()),
        },
        ("store_value", [target, value]) => NirExpr::StoreValue {
            target: Box::new(target.clone()),
            value: Box::new(value.clone()),
        },
        ("store_next", [target, next]) => NirExpr::StoreNext {
            target: Box::new(target.clone()),
            next: Box::new(next.clone()),
        },
        ("store_at", [buffer, index, value]) => NirExpr::StoreAt {
            buffer: Box::new(buffer.clone()),
            index: Box::new(index.clone()),
            value: Box::new(value.clone()),
        },
        ("free", [value]) => NirExpr::Free(Box::new(value.clone())),
        ("is_null", [value]) => NirExpr::IsNull(Box::new(value.clone())),
        _ => NirExpr::Call {
            callee: callee.to_owned(),
            args: lowered_args,
        },
    }
}

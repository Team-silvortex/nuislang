use std::collections::BTreeSet;

use crate::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstMatchArm, AstMatchPattern, AstStmt,
    AstTypeRef, AstUnaryOp,
};

use super::{
    Validator, CODE_AWAIT, CODE_DEREF, CODE_FLOAT, CODE_HOST_EFFECT, CODE_INSTANTIATE,
    CODE_INTRINSIC, CODE_LAMBDA,
};

impl Validator<'_> {
    pub(super) fn validate_body(
        &mut self,
        body: &[AstStmt],
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        for (index, statement) in body.iter().enumerate() {
            self.validate_stmt(statement, generics, &format!("{path} statement[{index}]"));
        }
    }

    fn validate_stmt(&mut self, statement: &AstStmt, generics: &BTreeSet<String>, path: &str) {
        self.checked_nodes += 1;
        match statement {
            AstStmt::Let {
                ty, value, name, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_type(ty, generics, &format!("{path} let {name} type"));
                }
                self.validate_expr(value, generics, &format!("{path} let {name} value"));
            }
            AstStmt::AssignLocal { name, value } => {
                self.validate_expr(value, generics, &format!("{path} assign {name}"));
            }
            AstStmt::DestructureLet {
                type_ref,
                fields,
                value,
            } => {
                if let Some(ty) = type_ref {
                    self.validate_type(ty, generics, &format!("{path} destructure type"));
                }
                self.validate_destructure_fields(fields, generics, path);
                self.validate_expr(value, generics, &format!("{path} destructure value"));
            }
            AstStmt::Const { name, ty, value } => {
                if let Some(ty) = ty {
                    self.validate_type(ty, generics, &format!("{path} const {name} type"));
                }
                self.validate_expr(value, generics, &format!("{path} const {name} value"));
            }
            AstStmt::Print(value) => {
                self.reject(
                    CODE_HOST_EFFECT,
                    path,
                    "printing is a host effect; bootstrap components must return diagnostics as data",
                );
                self.validate_expr(value, generics, &format!("{path} print value"));
            }
            AstStmt::Await(value) => {
                self.reject(
                    CODE_AWAIT,
                    path,
                    "await statements are outside bootstrap subset v1",
                );
                self.validate_expr(value, generics, &format!("{path} await value"));
            }
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.validate_expr(condition, generics, &format!("{path} if condition"));
                self.validate_body(then_body, generics, &format!("{path} then"));
                self.validate_body(else_body, generics, &format!("{path} else"));
            }
            AstStmt::Match { value, arms } => {
                self.validate_expr(value, generics, &format!("{path} match value"));
                self.validate_match_arms(arms, generics, path);
            }
            AstStmt::While { condition, body } => {
                self.validate_expr(condition, generics, &format!("{path} while condition"));
                self.validate_body(body, generics, &format!("{path} while body"));
            }
            AstStmt::Break | AstStmt::Continue => {}
            AstStmt::Expr(value) => self.validate_expr(value, generics, path),
            AstStmt::Return(value) => {
                if let Some(value) = value {
                    self.validate_expr(value, generics, &format!("{path} return value"));
                }
            }
        }
    }

    pub(super) fn validate_expr(
        &mut self,
        expression: &AstExpr,
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        self.checked_nodes += 1;
        match expression {
            AstExpr::Bool(_) | AstExpr::Text(_) | AstExpr::Int(_) | AstExpr::Var(_) => {}
            AstExpr::Float(_) => self.reject(
                CODE_FLOAT,
                path,
                "floating literals are outside bootstrap subset v1",
            ),
            AstExpr::If {
                condition,
                then_body,
                else_body,
            } => {
                self.validate_expr(condition, generics, &format!("{path} if condition"));
                self.validate_body(then_body, generics, &format!("{path} then"));
                self.validate_body(else_body, generics, &format!("{path} else"));
            }
            AstExpr::Match { value, arms } => {
                self.validate_expr(value, generics, &format!("{path} match value"));
                self.validate_match_arms(arms, generics, path);
            }
            AstExpr::Lambda {
                params,
                return_type,
                body,
            } => {
                self.reject(
                    CODE_LAMBDA,
                    path,
                    "lambdas and dynamic invocation are deferred beyond bootstrap subset v1",
                );
                for param in params {
                    self.validate_type(
                        &param.ty,
                        generics,
                        &format!("{path} lambda parameter {}", param.name),
                    );
                }
                if let Some(ty) = return_type {
                    self.validate_type(ty, generics, &format!("{path} lambda return"));
                }
                self.validate_body(body, generics, &format!("{path} lambda body"));
            }
            AstExpr::Await(value) => {
                self.reject(
                    CODE_AWAIT,
                    path,
                    "await expressions are outside bootstrap subset v1",
                );
                self.validate_expr(value, generics, &format!("{path} await value"));
            }
            AstExpr::Try(value) => {
                self.validate_expr(value, generics, &format!("{path} try value"));
            }
            AstExpr::Instantiate { domain, unit } => self.reject(
                CODE_INSTANTIATE,
                path,
                format!(
                    "heterogeneous unit instantiation `{domain} {unit}` is outside bootstrap subset v1"
                ),
            ),
            AstExpr::Call {
                callee,
                generic_args,
                args,
            } => {
                if is_reserved_intrinsic(callee) {
                    self.reject(
                        CODE_INTRINSIC,
                        path,
                        format!(
                            "intrinsic `{callee}` crosses the bootstrap subset's owned, deterministic boundary"
                        ),
                    );
                }
                self.validate_type_args(generic_args, generics, path);
                self.validate_exprs(args, generics, path);
            }
            AstExpr::Invoke { callee, args } => {
                self.reject(
                    CODE_LAMBDA,
                    path,
                    "dynamic function invocation is deferred beyond bootstrap subset v1",
                );
                self.validate_expr(callee, generics, &format!("{path} callee"));
                self.validate_exprs(args, generics, path);
            }
            AstExpr::MethodCall {
                receiver,
                generic_args,
                args,
                ..
            } => {
                self.validate_expr(receiver, generics, &format!("{path} receiver"));
                self.validate_type_args(generic_args, generics, path);
                self.validate_exprs(args, generics, path);
            }
            AstExpr::StructLiteral {
                type_name,
                type_args,
                fields,
            } => {
                let ty = AstTypeRef {
                    name: type_name.clone(),
                    generic_args: type_args.clone(),
                    is_optional: false,
                    is_ref: false,
                };
                self.validate_type(&ty, generics, &format!("{path} literal type"));
                for (field, value) in fields {
                    self.validate_expr(value, generics, &format!("{path} field {field}"));
                }
            }
            AstExpr::FieldAccess { base, .. } => {
                self.validate_expr(base, generics, &format!("{path} field base"));
            }
            AstExpr::Unary { op, operand } => {
                if *op == AstUnaryOp::Deref {
                    self.reject(
                        CODE_DEREF,
                        path,
                        "raw address dereference is outside bootstrap subset v1",
                    );
                }
                self.validate_expr(operand, generics, &format!("{path} unary operand"));
            }
            AstExpr::Binary { lhs, rhs, .. } => {
                self.validate_expr(lhs, generics, &format!("{path} binary lhs"));
                self.validate_expr(rhs, generics, &format!("{path} binary rhs"));
            }
        }
    }

    fn validate_match_arms(
        &mut self,
        arms: &[AstMatchArm],
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        for (index, arm) in arms.iter().enumerate() {
            let arm_path = format!("{path} arm[{index}]");
            self.validate_pattern(&arm.pattern, generics, &arm_path);
            if let Some(guard) = &arm.guard {
                self.validate_expr(guard, generics, &format!("{arm_path} guard"));
            }
            self.validate_body(&arm.body, generics, &arm_path);
        }
    }

    fn validate_pattern(
        &mut self,
        pattern: &AstMatchPattern,
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        self.checked_nodes += 1;
        match pattern {
            AstMatchPattern::Wildcard
            | AstMatchPattern::Bind(_)
            | AstMatchPattern::Bool(_)
            | AstMatchPattern::Int(_)
            | AstMatchPattern::IntRangeInclusive(_, _) => {}
            AstMatchPattern::Or(patterns) | AstMatchPattern::Tuple(patterns) => {
                for (index, pattern) in patterns.iter().enumerate() {
                    self.validate_pattern(pattern, generics, &format!("{path}[{index}]"));
                }
            }
            AstMatchPattern::PayloadStruct { type_ref, payload } => {
                self.validate_type(type_ref, generics, &format!("{path} payload type"));
                self.validate_pattern(payload, generics, &format!("{path} payload"));
            }
            AstMatchPattern::StructFields { type_ref, fields } => {
                if let Some(ty) = type_ref {
                    self.validate_type(ty, generics, &format!("{path} struct type"));
                }
                for (field, pattern) in fields {
                    self.validate_pattern(pattern, generics, &format!("{path}.{field}"));
                }
            }
        }
    }

    fn validate_destructure_fields(
        &mut self,
        fields: &[AstDestructureField],
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        for field in fields {
            self.validate_destructure_binding(
                &field.binding,
                generics,
                &format!("{path}.{}", field.field),
            );
        }
    }

    fn validate_destructure_binding(
        &mut self,
        binding: &AstDestructureBinding,
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        self.checked_nodes += 1;
        if let AstDestructureBinding::Nested { type_ref, fields } = binding {
            if let Some(ty) = type_ref {
                self.validate_type(ty, generics, &format!("{path} type"));
            }
            self.validate_destructure_fields(fields, generics, path);
        }
    }

    fn validate_type_args(
        &mut self,
        types: &[AstTypeRef],
        generics: &BTreeSet<String>,
        path: &str,
    ) {
        for (index, ty) in types.iter().enumerate() {
            self.validate_type(ty, generics, &format!("{path} type-argument[{index}]"));
        }
    }

    fn validate_exprs(&mut self, expressions: &[AstExpr], generics: &BTreeSet<String>, path: &str) {
        for (index, expression) in expressions.iter().enumerate() {
            self.validate_expr(expression, generics, &format!("{path} argument[{index}]"));
        }
    }
}

fn is_reserved_intrinsic(callee: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "buffer_",
        "bytes_",
        "cpu_",
        "data_",
        "deserialize_",
        "host_",
        "kernel_",
        "mutex_",
        "network_",
        "owned_utf8_",
        "serialize_",
        "shader_",
        "slice_",
        "thread_",
    ];
    const EXACT: &[&str] = &[
        "alloc_buffer",
        "alloc_node",
        "borrow",
        "borrow_end",
        "host_buffer_handle",
        "i32_from_i64",
        "join",
        "join_result",
        "load_at",
        "load_next",
        "load_value",
        "move",
        "null",
        "select_owned_ptr",
        "spawn",
        "store_at",
        "store_next",
        "store_value",
    ];
    PREFIXES.iter().any(|prefix| callee.starts_with(prefix)) || EXACT.contains(&callee)
}

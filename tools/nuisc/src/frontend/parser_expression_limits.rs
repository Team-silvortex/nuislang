use super::*;
use crate::frontend::{lexer::tokenize, parse_nuis_ast};

#[test]
fn expression_reentry_limit_rejects_deep_calls_groups_and_mixed_arguments() {
    for (open, close) in [("f(", ")"), ("(", ")"), ("f(1, ", ")")] {
        for (depth, admitted) in [(31, true), (32, false), (64, false), (4096, false)] {
            let expr = format!("{}1{}", open.repeat(depth), close.repeat(depth));
            let mut parser = Parser::new(tokenize(&expr).unwrap());
            let result = parser.parse_expr();
            assert_eq!(result.is_ok(), admitted, "depth={depth}, form={open}");
            if let Err(error) = result {
                assert!(
                    error.contains("expression nesting exceeds parser limit"),
                    "{error}"
                );
            }
            assert_eq!(parser.expression_depth, 0);
        }
    }
}

#[test]
fn expression_reentry_budget_is_released_between_siblings_and_after_errors() {
    let args = (0..512).map(|_| "f(f(1))").collect::<Vec<_>>().join(",");
    let mut parser = Parser::new(tokenize(&format!("f({args})")).unwrap());
    assert!(parser.parse_expr().is_ok());
    assert_eq!(parser.expression_depth, 0);
    let mut parser = Parser::new(tokenize("f(1 + )").unwrap());
    assert!(parser.parse_expr().is_err());
    assert_eq!(parser.expression_depth, 0);
    let source = format!(
        "mod cpu Main {{ fn main() -> i64 {{ return {}1{}; }} }}",
        "f(".repeat(64),
        ")".repeat(64)
    );
    assert!(parse_nuis_ast(&source)
        .unwrap_err()
        .contains("expression nesting exceeds parser limit"));
}

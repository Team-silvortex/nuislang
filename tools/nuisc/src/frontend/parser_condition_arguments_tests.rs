use super::*;
use crate::frontend::{lexer::tokenize, parse_nuis_ast, parse_nuis_module};

#[test]
fn condition_call_arguments_admit_nested_literals_without_consuming_outer_blocks() {
    for body in [
        "if accept(Packet { value: 1 }) { return 11; } return 19;",
        "while accept(Packet { value: 1 }) { break; } return 19;",
        "return match accept(Packet { value: 1 }) { true => { return 11; }, false => { return 19; } };",
        "if accept(wrap(Packet { value: 1 })) { return 11; } return 19;",
        "if accept(Packet<i64> { value: 1 }) { return 11; } return 19;",
    ] {
        let definition = if body.contains("Packet<i64>") {
            "struct Packet<T> { value: T }"
        } else {
            "struct Packet { value: i64 }"
        };
        let kind = if body.contains("Packet<i64>") {
            "Packet<i64>"
        } else {
            "Packet"
        };
        let source = format!(
            "mod cpu Main {{ {definition}
            fn accept(value: {kind}) -> bool {{ return value.value > 0; }}
            fn wrap(value: {kind}) -> {kind} {{ return value; }}
            fn main() -> i64 {{ {body} }} }}"
        );
        parse_nuis_ast(&source).unwrap();
        let nir = parse_nuis_module(&source).unwrap();
        crate::nir_verify::verify_nir_module(&nir).unwrap();
    }
    let ast = parse_nuis_ast(
        "mod cpu Main { fn entry(flag: bool) -> i64 {
        if flag { return 11; } return 19;
    } }",
    )
    .unwrap();
    assert!(
        matches!(&ast.functions[0].body[0], AstStmt::If { condition: AstExpr::Var(name), .. } if name == "flag")
    );
    let mut parser = Parser::new(tokenize("flag { return 11; }").unwrap());
    parser.allow_struct_literals = false;
    assert!(matches!(parser.parse_expr().unwrap(), AstExpr::Var(name) if name == "flag"));
    assert!(parser.peek_symbol('{'));
    assert!(!parser.allow_struct_literals);
}

#[test]
fn condition_call_arguments_restore_literal_mode_on_success_errors_and_nested_conditions() {
    for mode in [false, true] {
        for (args, valid) in [
            ("Packet { value: 1 })", true),
            ("accept(Packet { value: 1 }), Packet { value: 2 })", true),
            (
                "if flag { Packet { value: 1 } } else { Packet { value: 2 } })",
                true,
            ),
            ("Packet { value: })", false),
            ("accept(Packet { value: })", false),
        ] {
            let mut parser = Parser::new(tokenize(args).unwrap());
            parser.allow_struct_literals = mode;
            assert_eq!(parser.parse_argument_list(')').is_ok(), valid, "{args}");
            assert_eq!(parser.allow_struct_literals, mode, "{args}");
            assert_eq!(parser.expression_depth, 0);
        }
    }
}

#[test]
fn condition_call_arguments_retain_expression_depth_limits_and_error_restoration() {
    for (depth, admitted) in [(8, true), (64, false)] {
        let text = format!(
            "{}Packet {{ value: 1 }}{} )",
            "accept(".repeat(depth),
            ")".repeat(depth)
        );
        let mut parser = Parser::new(tokenize(&text).unwrap());
        parser.allow_struct_literals = false;
        let result = parser.parse_argument_list(')');
        assert_eq!(result.is_ok(), admitted);
        if let Err(error) = result {
            assert!(
                error.contains("expression nesting exceeds parser limit"),
                "{error}"
            );
        }
        assert!(!parser.allow_struct_literals);
        assert_eq!(parser.expression_depth, 0);
    }
}

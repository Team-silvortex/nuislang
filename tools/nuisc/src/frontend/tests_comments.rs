use super::{parse_nuis_ast, parse_nuis_module};
use nuis_semantics::model::AstAttributeValue;

#[test]
fn parse_ast_ignores_module_and_inline_comments() {
    let ast = parse_nuis_ast(
        r#"
// top-level comment
mod cpu main {
    /// answer doc
    /* function comment */
    fn answer() -> i32 {
        let value = 42; // end-of-line comment
        value
    }
}
"#,
    )
    .expect("ast parses with comments");

    assert_eq!(ast.domain, "cpu");
    assert_eq!(ast.unit, "main");
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].name, "answer");
    assert_eq!(ast.functions[0].attributes.len(), 1);
    assert_eq!(ast.functions[0].attributes[0].name, "doc");
    assert!(matches!(
        ast.functions[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "answer doc"
    ));
}

#[test]
fn parse_module_ignores_nested_block_comments() {
    let module = parse_nuis_module(
        r#"
mod cpu math {
    fn add(a: i32, b: i32) -> i32 {
        /* outer
           /* nested */
           still outer */
        let sum = a + b;
        sum
    }
}
"#,
    )
    .expect("module lowers with nested comments");

    assert_eq!(module.domain, "cpu");
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "add");
}

#[test]
fn parse_ast_collects_doc_comments_for_top_level_items() {
    let ast = parse_nuis_ast(
        r#"
mod cpu docs {
    /// const doc
    const ANSWER: i32 = 42;

    /// alias doc
    type Word = Text;

    /// trait doc
    trait Displayable {
        fn render(self: Self) -> Text;
    }
}
"#,
    )
    .expect("doc-commented items parse");

    assert_eq!(ast.consts[0].attributes[0].name, "doc");
    assert!(matches!(
        ast.consts[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "const doc"
    ));
    assert_eq!(ast.type_aliases[0].attributes[0].name, "doc");
    assert!(matches!(
        ast.type_aliases[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "alias doc"
    ));
    assert_eq!(ast.traits[0].attributes[0].name, "doc");
    assert!(matches!(
        ast.traits[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "trait doc"
    ));
}

#[test]
fn parse_ast_collects_doc_comments_for_variants_and_trait_methods() {
    let ast = parse_nuis_ast(
        r#"
mod cpu docs {
    enum Maybe<T> {
        /// empty doc
        None,
        /// some doc
        Some(T),
    }

    trait Displayable {
        /// render doc
        fn render(self: Self) -> Text;
    }
}
"#,
    )
    .expect("nested doc-commented items parse");

    assert_eq!(ast.enums[0].variants[0].attributes[0].name, "doc");
    assert!(matches!(
        ast.enums[0].variants[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "empty doc"
    ));
    assert_eq!(ast.enums[0].variants[1].attributes[0].name, "doc");
    assert!(matches!(
        ast.enums[0].variants[1].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "some doc"
    ));
    assert_eq!(ast.traits[0].methods[0].attributes[0].name, "doc");
    assert!(matches!(
        ast.traits[0].methods[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "render doc"
    ));
}

#[test]
fn parse_ast_collects_multiple_doc_comment_lines() {
    let ast = parse_nuis_ast(
        r#"
mod cpu docs {
    /// first line
    /// second line
    fn answer() -> i32 {
        42
    }
}
"#,
    )
    .expect("multi-line doc comments parse");

    assert_eq!(ast.functions[0].attributes.len(), 2);
    assert!(matches!(
        ast.functions[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "first line"
    ));
    assert!(matches!(
        ast.functions[0].attributes[1].args[0].value,
        AstAttributeValue::String(ref value) if value == "second line"
    ));
}

#[test]
fn parse_ast_collects_doc_comments_for_public_functions() {
    let ast = parse_nuis_ast(
        r#"
mod cpu docs {
    /// public answer doc
    pub fn answer() -> i32 {
        42
    }
}
"#,
    )
    .expect("public doc-commented function parses");

    assert_eq!(ast.functions[0].attributes.len(), 1);
    assert!(matches!(
        ast.functions[0].attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "public answer doc"
    ));
}

#[test]
fn parse_ast_accepts_file_level_doc_comments_before_module() {
    let ast = parse_nuis_ast(
        r#"
/// file docs
/// more file docs
mod cpu docs {
    fn answer() -> i32 {
        42
    }
}
"#,
    )
    .expect("file-level doc comments before module parse");

    assert_eq!(ast.domain, "cpu");
    assert_eq!(ast.unit, "docs");
    assert_eq!(ast.attributes.len(), 2);
    assert!(matches!(
        ast.attributes[0].args[0].value,
        AstAttributeValue::String(ref value) if value == "file docs"
    ));
    assert!(matches!(
        ast.attributes[1].args[0].value,
        AstAttributeValue::String(ref value) if value == "more file docs"
    ));
    assert_eq!(ast.functions.len(), 1);
}

#[test]
fn parse_ast_reports_unterminated_block_comment() {
    let error = parse_nuis_ast(
        "mod cpu broken {\n    fn demo() -> i32 {\n        /* missing close\n        1\n    }\n}\n",
    )
    .expect_err("unterminated comment should fail");

    assert!(error.contains("unterminated block comment"));
}

use super::{extract_ast_doc_index, parse_nuis_ast};

#[test]
fn extracts_flat_doc_index_for_documented_items() {
    let ast = parse_nuis_ast(
        r#"
mod cpu docs {
    /// const docs
    const ANSWER: i32 = 42;

    /// alias docs
    type Word = Text;

    /// struct docs
    struct User {
        /// name docs
        name: Text,
    }

    /// enum docs
    enum Maybe<T> {
        /// empty docs
        None,
        /// some docs
        Some(T),
    }

    /// trait docs
    trait Displayable {
        /// render docs
        fn render(self: Self) -> Text;
    }

    /// first line
    /// second line
    async fn answer(name: Text) -> i32 {
        42
    }
}
"#,
    )
    .expect("ast parses");

    let index = extract_ast_doc_index(&ast);
    assert_eq!(index.module_path, "cpu.docs");

    let paths = index
        .items
        .iter()
        .map(|item| item.path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"cpu.docs::ANSWER"));
    assert!(paths.contains(&"cpu.docs::Word"));
    assert!(paths.contains(&"cpu.docs::User"));
    assert!(paths.contains(&"cpu.docs::User::name"));
    assert!(paths.contains(&"cpu.docs::Maybe"));
    assert!(paths.contains(&"cpu.docs::Maybe::None"));
    assert!(paths.contains(&"cpu.docs::Maybe::Some"));
    assert!(paths.contains(&"cpu.docs::Displayable"));
    assert!(paths.contains(&"cpu.docs::Displayable::render"));
    assert!(paths.contains(&"cpu.docs::answer"));

    let function = index
        .items
        .iter()
        .find(|item| item.path == "cpu.docs::answer")
        .expect("function docs indexed");
    assert_eq!(function.kind, "function");
    assert_eq!(function.docs, vec!["first line".to_owned(), "second line".to_owned()]);
    assert_eq!(
        function.signature.as_deref(),
        Some("async fn answer(name: Text) -> i32")
    );

    let variant = index
        .items
        .iter()
        .find(|item| item.path == "cpu.docs::Maybe::Some")
        .expect("variant docs indexed");
    assert_eq!(variant.kind, "enum_variant");
    assert_eq!(variant.docs, vec!["some docs".to_owned()]);
    assert_eq!(variant.signature.as_deref(), Some("variant Some(T)"));

    let trait_method = index
        .items
        .iter()
        .find(|item| item.path == "cpu.docs::Displayable::render")
        .expect("trait method docs indexed");
    assert_eq!(trait_method.kind, "trait_method");
    assert_eq!(trait_method.docs, vec!["render docs".to_owned()]);
    assert_eq!(
        trait_method.signature.as_deref(),
        Some("fn render(self: Self) -> Text")
    );
}

#[test]
fn skips_undocumented_items_in_doc_index() {
    let ast = parse_nuis_ast(
        r#"
mod cpu docs {
    const ANSWER: i32 = 42;

    fn answer() -> i32 {
        42
    }
}
"#,
    )
    .expect("ast parses");

    let index = extract_ast_doc_index(&ast);
    assert!(index.items.is_empty());
}

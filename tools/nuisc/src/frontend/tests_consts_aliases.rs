use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstVisibility, NirExpr, NirStmt};

#[test]
fn helper_pub_consts_can_cross_module_but_private_ones_cannot() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Limits;

        mod cpu Main {
          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu Limits {
          pub const LIMIT: i64 = 9;
          const SECRET: i64 = 5;
        }
        "#,
    )
    .unwrap();
    let module = super::lower_project_ast_to_nir(&entry, std::slice::from_ref(&helper)).unwrap();
    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Int(9))))
    ));

    let hidden_entry = parse_nuis_ast(
        r#"
        use cpu Limits;

        mod cpu Main {
          fn main() -> i64 {
            return SECRET;
          }
        }
        "#,
    )
    .unwrap();
    let error = super::lower_project_ast_to_nir(&hidden_entry, &[helper]).unwrap_err();
    assert!(
        error.contains("unknown value `SECRET`"),
        "unexpected error: {error}"
    );
}

#[test]
fn parses_pub_type_alias_items_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          pub type Count = i64;

          fn main() -> Count {
            return 7;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(ast.type_aliases.len(), 1);
    assert!(matches!(
        ast.type_aliases[0].visibility,
        AstVisibility::Public
    ));
    assert_eq!(ast.type_aliases[0].name, "Count");
    assert_eq!(ast.type_aliases[0].target.name, "i64");
}

#[test]
fn lowers_type_aliases_into_nir_and_resolves_declared_types() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Count = i64;

          fn main() -> Count {
            let value: Count = 7;
            return value;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(module.type_aliases.len(), 1);
    assert_eq!(module.type_aliases[0].target.name, "i64");
    assert_eq!(
        module.functions[0]
            .return_type
            .as_ref()
            .map(|ty| ty.render()),
        Some("i64".to_owned())
    );
    assert!(matches!(
        module.functions[0].body.first(),
        Some(NirStmt::Let { ty: Some(ty), .. }) if ty.render() == "i64"
    ));
}

#[test]
fn helper_pub_type_aliases_can_cross_module() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Types;

        mod cpu Main {
          fn main() -> i64 {
            let value: Count = 7;
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu Types {
          pub type Count = i64;
        }
        "#,
    )
    .unwrap();
    let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Let { ty: Some(ty), .. }) if ty.render() == "i64"
    ));
}

#[test]
fn rejects_cyclic_type_aliases() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type A = B;
          type B = A;

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();
    assert!(
        error.contains("type alias `A` is cyclic") || error.contains("type alias `B` is cyclic"),
        "unexpected error: {error}"
    );
}

#[test]
fn lowers_generic_type_aliases_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          pub type PipeOf<T> = Pipe<T>;

          fn use_pipe(pipe: PipeOf<i64>) -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(module.type_aliases.len(), 1);
    assert_eq!(module.type_aliases[0].generic_params.len(), 1);
    assert_eq!(module.type_aliases[0].target.render(), "Pipe<T>");
    let use_pipe = module
        .functions
        .iter()
        .find(|function| function.name == "use_pipe")
        .unwrap();
    assert_eq!(use_pipe.params[0].ty.render(), "Pipe<i64>");
}

#[test]
fn helper_pub_generic_type_aliases_can_cross_module() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Types;

        mod cpu Main {
          fn use_pipe(pipe: PipeOf<i64>) -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu Types {
          pub type PipeOf<T> = Pipe<T>;
        }
        "#,
    )
    .unwrap();
    let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
    let use_pipe = module
        .functions
        .iter()
        .find(|function| function.name == "use_pipe")
        .unwrap();
    assert_eq!(use_pipe.params[0].ty.render(), "Pipe<i64>");
}

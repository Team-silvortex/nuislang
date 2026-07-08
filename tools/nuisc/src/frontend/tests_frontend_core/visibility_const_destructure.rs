use super::*;

#[test]
fn suggests_similar_visible_field_name_for_field_access() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Config {
            count: i64,
            label: String,
          }

          fn main() -> i64 {
            let cfg: Config = Config { count: 7, label: "ok" };
            return cfg.cout;
          }
        }
        "#,
    );

    let error = module.unwrap_err();
    assert!(
        error.contains("type `Config` has no field `cout`"),
        "{error}"
    );
    assert!(error.contains("did you mean `count`?"), "{error}");
}

#[test]
fn rejects_struct_literals_for_imported_structs_with_hidden_private_fields() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Shapes;

        mod cpu Main {
          fn main() -> i64 {
            let cfg: Config = Config {
              visible: 1
            };
            return cfg.visible;
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu Shapes {
          pub struct Config {
            pub visible: i64,
            secret: i64
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("struct literal `Config` cannot be constructed outside its defining module because it hides 1 private field"),
        "unexpected error: {error}"
    );
}

#[test]
fn parses_pub_const_items_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          pub const LIMIT: i64 = 7;

          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(ast.consts.len(), 1);
    assert!(matches!(ast.consts[0].visibility, AstVisibility::Public));
    assert_eq!(ast.consts[0].name, "LIMIT");
    assert_eq!(
        ast.consts[0]
            .ty
            .as_ref()
            .map(|ty| lower_type_ref(ty).render())
            .as_deref(),
        Some("i64")
    );
}

#[test]
fn parses_top_level_const_items_without_explicit_type() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          const LIMIT = 7;
        }
        "#,
    )
    .unwrap();
    assert_eq!(ast.consts.len(), 1);
    assert!(ast.consts[0].ty.is_none());
}

#[test]
fn lowers_top_level_const_reads_by_inlining_values() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          const LIMIT: i64 = 7;

          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(module.consts.len(), 1);
    assert!(matches!(
        module.functions[0].body.first(),
        Some(NirStmt::Return(Some(NirExpr::Int(7))))
    ));
}

#[test]
fn infers_top_level_const_item_types_from_values() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          const LIMIT = 7;

          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(module.consts.len(), 1);
    assert_eq!(module.consts[0].ty.render(), "i64");
    assert!(matches!(
        module.functions[0].body.first(),
        Some(NirStmt::Return(Some(NirExpr::Int(7))))
    ));
}

#[test]
fn parses_local_const_without_explicit_type() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            const LIMIT = 7;
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    match &ast.functions[0].body[0] {
        AstStmt::Const { ty, .. } => assert!(ty.is_none()),
        other => panic!("expected local const statement, found {other:?}"),
    }
}

#[test]
fn infers_local_const_item_types_inside_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            if true {
              const LIMIT = 7;
              return LIMIT;
            } else {
              match 1 {
                1 => {
                  const LIMIT = 8;
                  return LIMIT;
                }
                _ => {
                  return 9;
                }
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    match &module.functions[0].body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            match &then_body[0] {
                NirStmt::Const { ty, .. } => assert_eq!(ty.render(), "i64"),
                other => panic!("expected inferred const in then branch, found {other:?}"),
            }
            match &else_body[0] {
                NirStmt::If { then_body, .. } => match &then_body[0] {
                    NirStmt::Const { ty, .. } => assert_eq!(ty.render(), "i64"),
                    other => {
                        panic!("expected inferred const in match arm branch, found {other:?}")
                    }
                },
                other => panic!("expected lowered match branch if, found {other:?}"),
            }
        }
        other => panic!("expected if statement, found {other:?}"),
    }
}

#[test]
fn parses_struct_destructuring_let_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind, ready } = packet;
            if ready {
              return kind;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => {
            assert_eq!(type_ref.as_ref().unwrap().name, "Packet");
            assert_eq!(
                fields,
                &vec![
                    AstDestructureField {
                        field: "kind".to_owned(),
                        binding: AstDestructureBinding::Bind("kind".to_owned())
                    },
                    AstDestructureField {
                        field: "ready".to_owned(),
                        binding: AstDestructureBinding::Bind("ready".to_owned())
                    }
                ]
            );
            assert!(matches!(value, nuis_semantics::model::AstExpr::Var(name) if name == "packet"));
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

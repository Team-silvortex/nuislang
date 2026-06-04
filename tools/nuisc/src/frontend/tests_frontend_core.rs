use super::lower_type_ref;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstStmt, AstVisibility, NirExpr, NirStmt};

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
fn lowers_project_local_cpu_helper_calls_with_qualified_callees() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_completed(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          pub fn encode_completed(value: i64) -> i64 {
            return value + 1;
          }

          pub fn task_policy_completed(value: i64) -> i64 {
            return encode_completed(value);
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
    let helper_function = module
        .functions
        .iter()
        .find(|function| function.name == "TaskHelpers.task_policy_completed")
        .unwrap();
    assert!(matches!(
        helper_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "TaskHelpers.encode_completed"
    ));

    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "TaskHelpers.task_policy_completed"
    ));
}

#[test]
fn rejects_private_local_cpu_helper_calls_across_modules() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_completed(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("unknown function `task_policy_completed`"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_private_helper_field_access_across_modules() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Shapes;

        mod cpu Main {
          fn main() -> i64 {
            let cfg: Config = Shapes.make();
            return cfg.secret;
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

          pub fn make() -> Config {
            return Config {
              visible: 1,
              secret: 2
            };
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("type `Config` has no field `secret`"),
        "unexpected error: {error}"
    );
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

use super::lower_type_ref;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstStmt, AstVisibility, NirExpr, NirStmt,
};
use std::fs;
use std::path::PathBuf;

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
fn lowers_float_literals_with_expected_scalar_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add32() -> f32 {
            let sum: f32 = 1.5 + 2.25;
            return sum;
          }

          fn add64() -> f64 {
            return 1.5 + 2.25;
          }
        }
        "#,
    )
    .unwrap();

    let add32 = module
        .functions
        .iter()
        .find(|function| function.name == "add32")
        .unwrap();
    let sum_ty = add32
        .body
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::Let { name, ty, value } if name == "sum" => {
                assert!(matches!(
                    value,
                    NirExpr::Binary {
                        lhs,
                        rhs,
                        ..
                    } if matches!(lhs.as_ref(), NirExpr::F32(value) if value == "1.5")
                        && matches!(rhs.as_ref(), NirExpr::F32(value) if value == "2.25")
                ));
                ty.as_ref()
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(sum_ty.render(), "f32");

    let add64 = module
        .functions
        .iter()
        .find(|function| function.name == "add64")
        .unwrap();
    assert!(matches!(
        add64.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. })))
            if matches!(lhs.as_ref(), NirExpr::F64(value) if value == "1.5")
                && matches!(rhs.as_ref(), NirExpr::F64(value) if value == "2.25")
    ));
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
fn lowers_payload_style_single_field_struct_constructor_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just {
            value: i64,
          }

          fn main() -> i64 {
            let payload: Just = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "payload");
            assert!(matches!(
                value,
                NirExpr::StructLiteral { type_name, fields, .. }
                    if type_name == "Just"
                        && matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Int(7))] if field == "value"
                        )
            ));
        }
        other => panic!("expected lowered payload constructor let, found {other:?}"),
    }
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
fn lowers_project_local_cpu_helper_calls_with_shader_and_data_modules_present() {
    let entry = parse_nuis_ast(
        r#"
        use cpu ShaderTaskAsyncShapes;
        use data FabricPlane;
        use shader SurfaceShader;

        mod cpu Main {
          fn main(primary_result: TaskResult<i64>, secondary_result: TaskResult<i64>) -> i64 {
            return ShaderTaskAsyncShapes.async_policy_summary_completed(
              primary_result,
              secondary_result
            );
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu ShaderTaskAsyncShapes {
          pub fn encode_completed(result: TaskResult<i64>) -> i64 {
            if task_completed(result) {
              return 1;
            }
            return 0;
          }

          pub fn async_policy_summary_completed(
            primary_result: TaskResult<i64>,
            secondary_result: TaskResult<i64>
          ) -> i64 {
            return encode_completed(primary_result) + encode_completed(secondary_result);
          }
        }
        "#,
    )
    .unwrap();
    let data_module = parse_nuis_ast(
        r#"
        mod data FabricPlane {
          struct SurfaceShaderPacket {
            color: i64,
          }
        }
        "#,
    )
    .unwrap();
    let shader_module = parse_nuis_ast(
        r#"
        mod shader SurfaceShader {
          struct SurfaceShaderPacket {
            color: i64,
          }
        }
        "#,
    )
    .unwrap();

    let module =
        super::lower_project_ast_to_nir(&entry, &[helper, data_module, shader_module]).unwrap();
    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "ShaderTaskAsyncShapes.async_policy_summary_completed"
    ));
}

#[test]
fn lowers_real_shader_project_helper_calls_from_disk() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/shader_async_policy_profile_demo");
    let shared_root = root.join("../shared");
    let entry = parse_nuis_ast(&fs::read_to_string(root.join("main.ns")).unwrap()).unwrap();
    let shader_module =
        parse_nuis_ast(&fs::read_to_string(root.join("surface_shader.ns")).unwrap()).unwrap();
    let data_module =
        parse_nuis_ast(&fs::read_to_string(root.join("fabric_plane.ns")).unwrap()).unwrap();
    let helper = parse_nuis_ast(
        &fs::read_to_string(shared_root.join("shader_task_async_shapes.ns")).unwrap(),
    )
    .unwrap();

    let module =
        super::lower_project_ast_to_nir(&entry, &[shader_module, data_module, helper]).unwrap();
    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(main_function.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::Call { callee, .. },
                ..
            } if callee == "ShaderTaskAsyncShapes.async_policy_summary_completed"
        )
    }));
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

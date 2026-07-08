use super::*;

#[test]
fn lowers_non_numeric_binary_comparisons_via_trait_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for Pair {
            fn eq(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value == rhs.value;
            }
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn main() -> i64 {
            let same: bool = Pair { value: 1 } == Pair { value: 1 };
            let different: bool = Pair { value: 1 } != Pair { value: 2 };
            let less: bool = Pair { value: 1 } < Pair { value: 2 };
            let less_eq: bool = Pair { value: 1 } <= Pair { value: 2 };
            let greater: bool = Pair { value: 3 } > Pair { value: 2 };
            let greater_eq: bool = Pair { value: 3 } >= Pair { value: 2 };
            if same && different && less && less_eq && greater && greater_eq {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "same" && callee == "impl.Equatable.for.Pair.eq"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs,
                rhs,
            },
            ..
        }) if name == "different"
            && matches!(lhs.as_ref(), NirExpr::Call { callee, .. } if callee == "impl.Equatable.for.Pair.eq")
            && matches!(rhs.as_ref(), NirExpr::Bool(false))
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "less" && callee == "impl.Orderable.for.Pair.lt"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "less_eq" && callee == "impl.Orderable.for.Pair.le"
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "greater" && callee == "impl.Orderable.for.Pair.gt"
    ));
    assert!(matches!(
        main.body.get(5),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "greater_eq" && callee == "impl.Orderable.for.Pair.ge"
    ));
}

#[test]
fn lowers_builtin_unary_not_and_neg() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let toggled: bool = !false;
            let negated: i64 = -7;
            if toggled {
              return negated;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs,
                rhs,
            },
            ..
        }) if name == "toggled"
            && matches!(lhs.as_ref(), NirExpr::Bool(false))
            && matches!(rhs.as_ref(), NirExpr::Bool(false))
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary {
                op: NirBinaryOp::Sub,
                lhs,
                rhs,
            },
            ..
        }) if name == "negated"
            && matches!(lhs.as_ref(), NirExpr::Int(0))
            && matches!(rhs.as_ref(), NirExpr::Int(7))
    ));
}

#[test]
fn lowers_non_builtin_unary_not_and_neg_via_trait_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Notable {
            fn not(value: Self) -> bool;
          }

          trait Negatable {
            fn neg(value: Self) -> Self;
          }

          impl Notable for Pair {
            fn not(value: Pair) -> bool {
              return value.value == 0;
            }
          }

          impl Negatable for Pair {
            fn neg(value: Pair) -> Pair {
              return Pair { value: 0 - value.value };
            }
          }

          fn main() -> i64 {
            let empty: bool = !Pair { value: 0 };
            let flipped: Pair = -Pair { value: 7 };
            if empty {
              return flipped.value;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "empty" && callee == "impl.Notable.for.Pair.not"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "flipped" && callee == "impl.Negatable.for.Pair.neg"
    ));
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

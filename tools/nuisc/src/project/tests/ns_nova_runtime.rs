use super::*;
use nuis_semantics::model::{NirExpr, NirStmt};

fn showcase_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/ns_nova_showcase")
}

#[test]
fn compiles_ns_nova_lifecycle_with_pixelmagic_rendering() {
    let project = load_project(showcase_root().as_path()).unwrap();
    let resolved_names = project
        .resolved_galaxies
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(resolved_names, vec!["core", "ns-nova", "pixelmagic", "std"]);
    assert!(project.modules.iter().any(|module| {
        module.ast.unit == "NovaAppRuntime"
            && module.origin.source_kind() == "galaxy-explicit-import"
    }));
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "PixelMagicRenderSurface"));

    let resolution = resolve_project_abi(&project).unwrap();
    assert!(!resolution.explicit);
    assert_eq!(
        resolution
            .requirements
            .iter()
            .map(|item| item.domain.as_str())
            .collect::<Vec<_>>(),
        vec!["cpu", "data", "shader"]
    );

    let artifacts = crate::pipeline::compile_project(showcase_root().as_path()).unwrap();
    for function in [
        "NovaAppRuntime.open",
        "NovaAppRuntime.begin_frame",
        "NovaAppRuntime.capture_frame_result",
        "NovaAppRuntime.frame_result_ready",
        "NovaAppRuntime.submit_frame",
        "NovaAppRuntime.commit_frame",
        "NovaAppRuntime.close",
        "NovaAppRuntime.summary",
    ] {
        assert!(
            artifacts
                .nir
                .functions
                .iter()
                .any(|item| item.name == function),
            "missing {function}"
        );
    }

    let render_frame = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "render_showcase_frame")
        .unwrap();
    for name in [
        "opened",
        "frame",
        "gpu_packet",
        "render_result",
        "completion",
        "submitted",
        "committed",
    ] {
        assert!(render_frame.body.iter().any(|statement| {
            matches!(
                statement,
                NirStmt::Let { name: statement_name, .. } if statement_name == name
            )
        }));
    }
    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(main
        .body
        .iter()
        .any(|statement| matches!(statement, NirStmt::Expr(NirExpr::CpuWindow { .. }))));
    assert!(main.body.iter().any(|statement| {
        matches!(
            statement,
            NirStmt::While {
                condition: NirExpr::Binary { .. },
                ..
            }
        )
    }));
    let update_loop = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_i64_effect"
                && node
                    .op
                    .args
                    .iter()
                    .any(|arg| arg == "render_showcase_frame")
        })
        .expect("showcase should lower its bounded update loop through a scoped frame helper");
    assert_eq!(
        &update_loop.op.args[3..7],
        ["lt", "add", "cpu", "scoped_call"]
    );
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "shader" && node.op.instruction == "inline_wgsl" }));
    let present = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "branch_effect")
        .expect("showcase should lower conditional presentation through branch-effect");
    assert!(present.op.args.iter().any(|arg| arg == "present_frame"));
    let present_condition = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.name == present.op.args[0])
        .expect("conditional presentation should have a lowered predicate");
    assert_eq!(present_condition.op.full_name(), "cpu.eq");
    let present_request = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.name == present_condition.op.args[0])
        .expect("presentation predicate should read the submitted frame");
    assert_eq!(present_request.op.full_name(), "cpu.field");
    assert_eq!(present_request.op.args[1], "present_requested");
    assert!(artifacts.yir.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "call_owned_struct"
            && node.op.args.first().map(String::as_str)
                == Some("NovaAppRuntime.capture_frame_result")
    }));
    assert!(artifacts.llvm_ir.contains("projected shader.observe"));
    assert!(artifacts
        .llvm_ir
        .contains("projected shader result state `frame_ready` through shader.is_frame_ready"));
    assert!(artifacts
        .llvm_ir
        .contains("payload remains shader-provider-owned"));
    assert!(!artifacts
        .llvm_ir
        .contains("deferred lowering for shader.is_frame_ready"));
    assert!(!artifacts
        .llvm_ir
        .contains("deferred lowering for cpu.call_owned_struct"));
    assert!(!artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "present_frame"));
}

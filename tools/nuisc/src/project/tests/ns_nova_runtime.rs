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

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    for name in [
        "opened",
        "frame",
        "gpu_packet",
        "render_result",
        "submitted",
        "committed",
        "closed",
    ] {
        assert!(main.body.iter().any(|statement| {
            matches!(
                statement,
                NirStmt::Let { name: statement_name, .. } if statement_name == name
            )
        }));
    }
    assert!(main
        .body
        .iter()
        .any(|statement| matches!(statement, NirStmt::Expr(NirExpr::CpuWindow { .. }))));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "shader" && node.op.instruction == "inline_wgsl" }));
}

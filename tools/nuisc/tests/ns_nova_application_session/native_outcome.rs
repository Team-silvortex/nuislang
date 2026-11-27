use super::*;
use yir_lower_llvm::native_session::emit_registered;

#[test]
fn native_parent_checks_enum_identity_before_emitting_discriminants() {
    let module = image_module();
    let bridge = emit_registered(module, "parent").unwrap();
    assert!(!bridge.llvm_ir.contains("deferred lowering"));
    assert!(bridge
        .llvm_ir
        .contains("call [1 x i64] @nuis_fn_NovaAppRuntime.failure_kind"));
    let node = module
        .nodes
        .iter()
        .position(|n| {
            n.op.instruction == "variant_is"
                && n.op.args[1] == "NovaFailureKind.None"
                && module
                    .node_lanes
                    .get(&n.name)
                    .is_some_and(|l| l == "fn:NovaAppRuntime.terminal_outcome")
        })
        .unwrap();
    for wrong in ["NovaCloseReason.Requested", "NovaFailureKind.NotDeclared"] {
        let mut drift = module.clone();
        drift.nodes[node].op.args[1] = wrong.to_owned();
        let err = emit_registered(&drift, "parent")
            .err()
            .expect("bad predicate must not emit");
        assert!(err.contains("matching unit variant"), "{err}");
    }
}

#[cfg(target_os = "macos")]
#[test]
fn packaged_native_showcase_parent_matches_reference_at_every_delivery() {
    use std::{fs, process::Command};
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!("nuis-native-outcome-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let out = Scratch(path);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let built = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .current_dir(&root)
        .args([
            "compile",
            "examples/projects/domains/ns_nova_image_showcase",
        ])
        .arg(&out.0)
        .args(["--packaging-mode", "native-session-aot-bundle:parent"])
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let manifest =
        nuisc::aot::verify_build_manifest(&out.0.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(manifest.packaging_mode, "native-session-aot-bundle:parent");
    let mut native = Command::new(out.0.join(manifest.artifact_binary_name));
    native.current_dir(&out.0).env("PATH", "").args([
        "--application-session",
        "parent",
        "--open-args",
        "",
    ]);
    let registry = yir_verify::default_registry();
    let (mut reference, _) =
        ApplicationSession::open_registered(image_module(), &registry, "parent", vec![]).unwrap();
    let mut expected = vec![format!("open:{}", reference.state())];
    let events = [
        [1, 1, 0],
        [2, 0, 1],
        [2, 1, 6],
        [2, 1, 11],
        [2, 1, 99],
        [2, 1, -1],
    ];
    for [status, cleanup, failure] in events {
        native.args(["--event-args", &format!("{status},{cleanup},{failure}")]);
        reference
            .event(vec![
                Value::Int(status),
                Value::Int(cleanup),
                Value::Int(failure),
            ])
            .unwrap();
        expected.push(format!("event:{}", reference.state()));
    }
    reference.close(vec![]).unwrap().unwrap();
    reference.completion_status().unwrap();
    expected.push(format!("close:{}", reference.state()));
    let run = native.args(["--close-args", ""]).output().unwrap();
    let log = String::from_utf8(run.stderr).unwrap();
    assert!(run.status.success(), "{log}");
    assert_eq!(
        log.lines()
            .filter_map(|l| l.strip_prefix("nuis: native_session_state="))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        log.lines()
            .filter(|l| *l == "nuis: native_session_completed=1")
            .count(),
        1
    );
}

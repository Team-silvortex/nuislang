use super::*;
use crate::aot_native_session::{
    emit_for_packaging_mode, verify_checkpoint_for_packaging_mode, POLICY_PACKAGING_PREFIX,
};
use yir_lower_llvm::native_session::LiteralPrintBuildPolicy;
#[allow(dead_code)]
#[path = "lowering/buffer_loop_outline/conditional_returns_effect_typed_return_fixtures.rs"]
mod fixtures;

fn profile(sites: Vec<String>, loops: u64, entries: u64) -> String {
    format!(
        "{POLICY_PACKAGING_PREFIX}{}:counter",
        LiteralPrintBuildPolicy::new(sites, loops, entries)
            .unwrap()
            .token()
    )
}

fn read(fixture: &Fixture) -> (PathBuf, String, Vec<ArtifactHashRow>) {
    let path = fixture.0.join("nuis.build.manifest.toml");
    let source = fs::read_to_string(&path).unwrap();
    let rows = parse_artifact_hash_blocks(&source, &path).unwrap();
    (path, source, rows)
}

fn replace_input(
    source: &str,
    path: &Path,
    rows: &[ArtifactHashRow],
    kind: &str,
    key: &str,
    replacement: &str,
) -> (String, Vec<ArtifactHashRow>) {
    let encoded = unique_value(source, key, path).unwrap();
    let changed = source.replace(&encoded, &hex_encode_bytes(replacement.as_bytes()));
    let mut rows = rows.to_vec();
    let row = rows.iter_mut().find(|row| row.kind == kind).unwrap();
    row.bytes = replacement.len();
    row.fnv1a64 = fnv1a64_hex(replacement.as_bytes());
    (changed, rows)
}

#[test]
fn native_policy_inputs_bind_exact_limits_schema_and_checkpoint() {
    let mode = profile(vec![], 19, 23);
    let fixture = fixture_for_mode(&mode);
    let (path, source, rows) = read(&fixture);
    verify_sources(&source, &path, &rows).unwrap();
    assert!(source.contains("nuis-native-session-build-inputs-v2"));
    assert!(source.contains("native-scalar-literal-print-llvm-v1"));
    let llvm = fs::read_to_string(fixture.0.join("demo.ll")).unwrap();
    assert_eq!(llvm.matches("store i64 19, ptr %nuis_loop_work").count(), 3);
    assert_eq!(
        llvm.matches("store i64 23, ptr %nuis_helper_entries")
            .count(),
        3
    );
    for changed in [
        source.replace(&mode, &profile(vec![], 20, 23)),
        source.replace(&mode, &profile(vec![], 19, 24)),
        source.replace(&mode, "native-session-aot-bundle:counter"),
        source.replace(
            "nuis-native-session-build-inputs-v2",
            "nuis-native-session-build-inputs-v1",
        ),
        source.replace(
            "native-scalar-literal-print-llvm-v1",
            "native-scalar-llvm-v1",
        ),
        source.replace(":counter", ":other"),
    ] {
        assert!(verify_sources(&changed, &path, &rows).is_err());
    }
    let replacement = llvm.replace(
        "store i64 23, ptr %nuis_helper_entries",
        "store i64 24, ptr %nuis_helper_entries",
    );
    let (changed, rows) = replace_input(
        &source,
        &path,
        &rows,
        "llvm_ir",
        "native_session_llvm_hex",
        &replacement,
    );
    assert!(verify_sources(&changed, &path, &rows)
        .unwrap_err()
        .contains("LLVM checkpoint"));
}

#[test]
fn native_policy_inputs_reject_rehashed_missing_duplicate_changed_and_extra_claims() {
    let mode = profile(vec![], 19, 23);
    let fixture = fixture_for_mode(&mode);
    let (path, source, rows) = read(&fixture);
    let bundle = fs::read_to_string(fixture.0.join("bundle.txt")).unwrap();
    let policy = crate::aot_native_session::build_policy(&mode)
        .unwrap()
        .unwrap();
    for (key, value) in policy.bundle_claims() {
        let line = format!("{key}={value}\n");
        for replacement in [
            bundle.replace(&line, ""),
            format!("{bundle}{line}"),
            bundle.replace(&line, &format!("{key}=drift\n")),
        ] {
            let (changed, rows) = replace_input(
                &source,
                &path,
                &rows,
                "application_bundle",
                "native_session_bundle_hex",
                &replacement,
            );
            let error = verify_sources(&changed, &path, &rows).unwrap_err();
            assert!(error.contains(key), "{error}");
        }
    }
    let replacement = format!("{bundle}native_session_policy_override=allow-all\n");
    let (changed, rows) = replace_input(
        &source,
        &path,
        &rows,
        "application_bundle",
        "native_session_bundle_hex",
        &replacement,
    );
    assert!(verify_sources(&changed, &path, &rows)
        .unwrap_err()
        .contains("unexpected policy claim"));
}

#[test]
fn native_pure_profile_rejects_policy_claims_without_implicit_grants() {
    let mode = "native-session-aot-bundle:counter";
    let fixture = fixture_for_mode(mode);
    let (path, source, rows) = read(&fixture);
    let bundle = fs::read_to_string(fixture.0.join("bundle.txt")).unwrap();
    for (key, value) in LiteralPrintBuildPolicy::new(vec!["missing"], 19, 23)
        .unwrap()
        .bundle_claims()
    {
        let replacement = format!("{bundle}{key}={value}\n");
        let (changed, rows) = replace_input(
            &source,
            &path,
            &rows,
            "application_bundle",
            "native_session_bundle_hex",
            &replacement,
        );
        assert!(verify_sources(&changed, &path, &rows)
            .unwrap_err()
            .contains("unexpected policy claim"));
    }
}

#[test]
fn native_policy_artifact_rechecks_exact_sites_literals_and_session_closure() {
    let source = fixtures::source("i32", "complete", false);
    let seed = fixture_for_mode("native-session-aot-bundle:counter");
    fs::write(seed.0.join("main.ns"), &source).unwrap();
    let module = crate::pipeline::compile_project(&seed.0).unwrap().yir;
    let sites = module
        .nodes
        .iter()
        .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
        .map(|n| n.name.clone())
        .collect::<Vec<_>>();
    let mode = profile(sites.clone(), 0, 100);
    let project_manifest = seed.0.join("nuis.toml");
    let text = fs::read_to_string(&project_manifest).unwrap();
    fs::write(
        &project_manifest,
        format!("{text}packaging_mode = \"{mode}\"\n"),
    )
    .unwrap();
    let resolved = crate::pipeline::resolve_compile_input(&seed.0).unwrap();
    let inspected = resolved.compile_for_inspection().unwrap();
    assert_eq!(
        inspected.view().llvm_ir.unwrap(),
        emit_for_packaging_mode(&module, &mode).unwrap().llvm_ir
    );
    for site in &sites {
        let node = module.nodes.iter().find(|n| &n.name == site).unwrap();
        let operand = module
            .nodes
            .iter()
            .find(|n| Some(&n.name) == node.op.args.last())
            .unwrap();
        assert!(
            matches!(operand.op.instruction.as_str(), "const" | "const_i64"),
            "site {node:?}, operand {operand:?}"
        );
    }
    let fixture = fixture_with_source(&mode, Some(&source));
    let (path, manifest, rows) = read(&fixture);
    verify_sources(&manifest, &path, &rows).unwrap();
    crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact")).unwrap();
    let llvm = fs::read_to_string(fixture.0.join("demo.ll")).unwrap();
    let bundle = fs::read_to_string(fixture.0.join("bundle.txt")).unwrap();
    verify_checkpoint_for_packaging_mode(&module, &mode, &llvm, &bundle).unwrap();
    assert!(
        crate::aot_native_session::verify_checkpoint(&module, "counter", &llvm, &bundle).is_err()
    );
    for invalid in [
        profile(sites[1..].to_vec(), 0, 100),
        profile(vec!["missing".to_owned()], 0, 100),
        profile(
            [
                sites.clone(),
                vec![module.functions[0].result.as_ref().unwrap().node.clone()],
            ]
            .concat(),
            0,
            100,
        ),
    ] {
        assert!(emit_for_packaging_mode(&module, &invalid).is_err());
        assert!(verify_sources(&manifest.replace(&mode, &invalid), &path, &rows).is_err());
    }
    let mut computed = module.clone();
    let site = computed
        .nodes
        .iter_mut()
        .find(|n| n.name == sites[0])
        .unwrap();
    site.op.args.pop();
    site.op.args.push(
        module
            .nodes
            .iter()
            .find(|n| n.op.instruction == "param_i64")
            .unwrap()
            .name
            .clone(),
    );
    assert!(emit_for_packaging_mode(&computed, &mode).is_err());
}

#[test]
fn native_policy_build_does_not_admit_compound_selected_or_unordered_helper_prints() {
    let seed = fixture_for_mode("native-session-aot-bundle:counter");
    for (body, expected) in [
        (
            "if gate { print(70); return value + 1; } print(71); return value + 1;",
            "unsupported effect",
        ),
        (
            "if gate { print(70); } else { print(71); } return value + 1;",
            "direct i64 constant",
        ),
        (
            "let selected: i64 = if gate { yes(value) } else { no(value) }; return selected;",
            "lacks dependency order",
        ),
    ] {
        let source = format!("mod cpu Main {{ struct State {{ value: i64, gate: bool }}
            @noinline fn yes(value: i64) -> i64 {{ print(70); return value + 1; }}
            @noinline fn no(value: i64) -> i64 {{ print(71); return value + 1; }}
            fn observe(value: i64, gate: bool) -> i64 {{ print(99); {body} }}
            fn start(value: i64, gate: bool) -> State {{ return State {{ value: observe(value, gate), gate: gate }}; }}
            fn step(state: State, gate: bool) -> State {{ return State {{ value: observe(state.value, gate), gate: gate }}; }}
            fn stop(state: State) -> State {{ return state; }} fn main() -> i64 {{ return 0; }} }}");
        fs::write(seed.0.join("main.ns"), source).unwrap();
        let mut module = match crate::pipeline::compile_project(&seed.0) {
            Ok(compiled) => compiled.yir,
            Err(error) => {
                assert_eq!(expected, "unsupported effect");
                assert!(error.contains("guard_print_return"), "{error}");
                continue;
            }
        };
        let sites = module
            .nodes
            .iter()
            .filter(|n| n.op.instruction.contains("print"))
            .map(|n| n.name.clone())
            .collect::<Vec<_>>();
        if expected == "lacks dependency order" {
            // Valid source now gets guarded calls and ordered effects. A graph
            // with those order edges removed must still fail closed.
            emit_for_packaging_mode(&module, &profile(sites.clone(), 0, 100)).unwrap();
            module
                .edges
                .retain(|edge| !matches!(edge.kind, yir_core::EdgeKind::Effect));
        }
        let error = emit_for_packaging_mode(&module, &profile(sites, 0, 100)).unwrap_err();
        assert!(error.contains(expected), "{expected}: {error}");
    }
}

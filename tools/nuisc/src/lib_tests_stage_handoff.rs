use std::path::Path;

pub(super) fn assert_compiler_stage_handoff(
    output_dir: &Path,
    build_manifest_path: &Path,
) -> nuis_artifact::CompilerStageHandoff {
    let build_manifest = nuis_artifact::parse_build_manifest(build_manifest_path).unwrap();
    for kind in [
        "compiler_source",
        "compiler_tokens",
        "compiler_stage_handoff",
    ] {
        assert!(
            build_manifest
                .artifact_hashes
                .iter()
                .any(|artifact| artifact.kind == kind),
            "expected build manifest to hash `{kind}`"
        );
    }

    let (handoff, payloads) = nuis_artifact::read_compiler_stage_handoff(
        &output_dir.join("nuis.compiler-stage-handoff.toml"),
    )
    .unwrap();
    assert_eq!(handoff.records.len(), 5);
    assert_eq!(payloads.len(), 5);
    handoff
}

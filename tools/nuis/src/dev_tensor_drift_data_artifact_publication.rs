use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-fresh-identity",
        path: "crates/nuis-artifact/src/file_publication.rs",
        required_patterns: &[
            "pub fn atomic_copy_artifact_file",
            "pub fn atomic_write_artifact_file",
            "options.write(true).create_new(true)",
            "let temporary = TemporaryFile(temporary_path)",
            "file.sync_all()",
            "fs::rename(&temporary.0, target)",
            "fs::symlink_metadata(target)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-cache-integration",
        path: "tools/nuisc/src/cache_fs.rs",
        required_patterns: &["nuis_artifact::atomic_copy_artifact_file(&from, &to)"],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-materialization-admission",
        path: "tools/nuis/src/artifact_materialization.rs",
        required_patterns: &[
            "nuis_artifact::atomic_write_artifact_file(&binary_path, &artifact.binary_blob)",
            "verify_build_manifest(&output_dir.join(\"nuis.build.manifest.toml\"))",
            "Enable execution only after the restored artifact identity is verified",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-native-cache-regression",
        path: "tools/nuisc/src/cache_restore_native_tests.rs",
        required_patterns: &[
            "cache_restore_replaces_files_without_mutating_existing_file_identity",
            "cache_restore_alternating_native_images_executes_each_exact_image",
            "for iteration in 0..6",
            "run.status.code()",
            "no retry",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-source-free-replacement",
        path: "tools/nuis/tests/native_session_workflow/publication.rs",
        required_patterns: &[
            "native_cache_and_repeated_materialization_publish_fresh_file_identities",
            "for cycle in 0..4",
            "fs::remove_file(project.0.join(\"main.ns\"))",
            "source-free publication cycle {cycle}",
            "non-regular destination",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-failure-atomicity-tests",
        path: "crates/nuis-artifact/src/file_publication_tests.rs",
        required_patterns: &[
            "publication_writer_failure_preserves_old_file_and_cleans_staging",
            "publication_readers_observe_complete_old_or_complete_new_file",
            "publication_refuses_symlinks_without_modifying_their_referent",
            "publication_write_does_not_inherit_execute_permissions",
            "publication_does_not_extend_near_limit_destination_names",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-file-publication-documented-boundaries",
        path: "docs/reference/nuis-artifact-file-publication-v1.md",
        required_patterns: &[
            "## Reproduced Failure",
            "The exact phase/PID of the earlier",
            "not protection against",
            "multi-file bundle transaction",
            "not fresh Linux/Windows certification",
        ],
    },
];

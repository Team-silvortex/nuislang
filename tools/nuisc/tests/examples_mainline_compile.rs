use std::{fs, path::Path};

fn should_skip_dir_name(name: &str) -> bool {
    matches!(name, ".git" | "target" | ".github" | ".idea" | ".vscode")
}

const SHADER_KERNEL_PROFILE_MAINLINE_EXAMPLES: &[&str] = &[
    "../../examples/projects/domains/shader_profile_demo",
    "../../examples/projects/domains/shader_render_profile_demo",
    "../../examples/projects/domains/shader_result_enum_demo",
    "../../examples/projects/domains/kernel_profile_demo",
    "../../examples/projects/domains/kernel_result_profile_demo",
    "../../examples/projects/domains/kernel_tensor_profile_demo",
    "../../examples/projects/domains/kernel_tensor_axis_pipeline_demo",
];

fn compile_project(path: &str) {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("example project `{path}` should compile: {error}"));
}

fn collect_ns_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("read {}: {error}", root.display()))
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if should_skip_dir_name(
                path.file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(""),
            ) {
                continue;
            }
            collect_ns_files(&path, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some("ns") {
            files.push(path);
        }
    }
}

fn collect_named_files(root: &Path, file_name: &str, files: &mut Vec<std::path::PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("read {}: {error}", root.display()))
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if should_skip_dir_name(
                path.file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(""),
            ) {
                continue;
            }
            collect_named_files(&path, file_name, files);
        } else if path.file_name().and_then(|value| value.to_str()) == Some(file_name) {
            files.push(path);
        }
    }
}

fn collect_files_with_extension(root: &Path, extension: &str, files: &mut Vec<std::path::PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("read {}: {error}", root.display()))
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if should_skip_dir_name(
                path.file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(""),
            ) {
                continue;
            }
            collect_files_with_extension(&path, extension, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

#[test]
fn compiles_filesystem_mainline_examples() {
    for path in [
        "../../examples/projects/filesystem/path_runtime_demo",
        "../../examples/projects/filesystem/path_copy_demo",
        "../../examples/projects/filesystem/path_rename_demo",
        "../../examples/projects/filesystem/path_remove_demo",
        "../../examples/projects/filesystem/directory_runtime_demo",
        "../../examples/projects/filesystem/directory_create_demo",
        "../../examples/projects/filesystem/directory_remove_demo",
        "../../examples/projects/filesystem/directory_stat_demo",
        "../../examples/projects/filesystem/filesystem_report_demo",
        "../../examples/projects/filesystem/fs_metadata_runtime_demo",
    ] {
        compile_project(path);
    }
}

#[test]
fn compiles_official_galaxy_mainline_examples() {
    for path in [
        "../../examples/projects/domains/pixelmagic_profile_demo",
        "../../examples/projects/domains/pixelmagic_analysis_demo",
        "../../examples/projects/domains/witsage_kernel_demo",
        "../../examples/projects/domains/witsage_classifier_demo",
    ] {
        compile_project(path);
    }
}

#[test]
fn compiles_shader_kernel_profile_mainline_examples() {
    for path in SHADER_KERNEL_PROFILE_MAINLINE_EXAMPLES {
        compile_project(path);
    }
}

#[test]
fn checked_in_examples_use_abi_derived_kernel_target_config() {
    let mut files = Vec::new();
    collect_ns_files(Path::new("../../examples/projects"), &mut files);

    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        assert!(
            !source.contains("kernel_target_config"),
            "checked-in example `{}` should rely on ABI-derived kernel target config",
            path.display()
        );
    }
}

#[test]
fn checked_in_example_manifests_use_current_abi_field() {
    let mut manifests = Vec::new();
    collect_named_files(
        Path::new("../../examples/projects"),
        "nuis.toml",
        &mut manifests,
    );

    for path in manifests {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for required in ["name =", "version =", "entry =", "modules ="] {
            assert!(
                source.contains(required),
                "checked-in example manifest `{}` should explicitly declare `{required}`",
                path.display()
            );
        }
        for legacy in [
            "[project]",
            "[abi]",
            "abis = [",
            "target = \"host-native\"",
            "kind = \"staticlib\"",
        ] {
            assert!(
                !source.contains(legacy),
                "checked-in example manifest `{}` should use current `abi = [\"domain=abi\"]` declarations instead of legacy `{legacy}`",
                path.display()
            );
        }
    }
}

#[test]
fn checked_in_docs_do_not_embed_host_absolute_paths() {
    let mut docs = Vec::new();
    collect_files_with_extension(Path::new("../../"), "md", &mut docs);

    for path in docs {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for forbidden in [
            "/Users/",
            "/private/",
            "/var/folders/",
            "/tmp/",
            "/var/tmp/",
            "/Library/",
            "/Applications/",
            "/.../",
            "file://",
        ] {
            assert!(
                !source.contains(forbidden),
                "checked-in doc `{}` should avoid host absolute path `{forbidden}`",
                path.display()
            );
        }
    }
}

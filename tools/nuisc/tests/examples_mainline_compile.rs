use std::path::Path;

fn compile_project(path: &str) {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("example project `{path}` should compile: {error}"));
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
    for path in [
        "../../examples/projects/domains/shader_profile_demo",
        "../../examples/projects/domains/shader_render_profile_demo",
        "../../examples/projects/domains/kernel_profile_demo",
        "../../examples/projects/domains/kernel_tensor_profile_demo",
        "../../examples/projects/domains/kernel_tensor_axis_pipeline_demo",
    ] {
        compile_project(path);
    }
}

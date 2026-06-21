use std::path::Path;

fn compiled_source(path: &str) {
    nuisc::pipeline::compile_source_path(Path::new(path))
        .unwrap_or_else(|error| panic!("ffi source `{path}` should compile: {error}"));
}

#[test]
fn compiles_host_bridge_ffi_frontdoor_sources() {
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns");
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_c_ffi.ns");
}

#[test]
fn compiles_task_runtime_ffi_frontdoor_sources() {
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_runtime_facades.ns",
    );
    compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_task_cli_facades.ns");
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns",
    );
}

#[test]
fn compiles_path_runtime_ffi_frontdoor_sources() {
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns",
    );
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_runtime_facades.ns",
    );
    compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_runtime_facades.ns",
    );
}

#[test]
fn compiles_representative_ffi_companion_sources() {
    for path in [
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_argv_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_benchmark_report_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_benchmark_report_count_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_benchmark_report_file_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_process_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_command_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_subprocess_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_host_text_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_io_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_temp_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_fs_metadata_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stat_runtime_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_parent_facades.ns",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_depth_facades.ns",
    ] {
        compiled_source(path);
    }
}

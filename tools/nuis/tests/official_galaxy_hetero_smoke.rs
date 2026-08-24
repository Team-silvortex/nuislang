use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct TestOutputDir(PathBuf);

impl std::ops::Deref for TestOutputDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for TestOutputDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestOutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(label: &str) -> TestOutputDir {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_official_hetero_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    TestOutputDir(dir)
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {:?}: {error}", args))
}

fn run_nsld(args: &[&str]) -> std::process::Output {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_nsld").map(PathBuf::from) {
        return Command::new(path)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsld {:?}: {error}", args));
    }
    Command::new("cargo")
        .args(["run", "-q", "-p", "nsld", "--"])
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsld through cargo {:?}: {error}", args))
}

fn run_nsdb(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "-q", "-p", "nsdb", "--"])
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsdb through cargo {:?}: {error}", args))
}

fn json_string_values(source: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\":\"");
    source
        .split(&needle)
        .skip(1)
        .filter_map(|tail| tail.split('"').next())
        .map(str::to_owned)
        .collect()
}

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_file_contains(path: &Path, needle: &str, context: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        source.contains(needle),
        "expected {context} file {} to contain `{needle}`\n{source}",
        path.display()
    );
}

fn provider_family_artifact_component(provider_family: &str) -> String {
    provider_family
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

struct OfficialGalaxyHeteroBuildCase<'a> {
    label: &'a str,
    project: &'a str,
    domain: &'a str,
    backend_family: &'a str,
    target_device: &'a str,
    trace_record_count: usize,
    expected_trace_id: &'a str,
    yir_needles: &'a [&'a str],
    sidecar_needles: &'a [&'a str],
    payload_needles: &'a [&'a str],
}

#[path = "official_galaxy_hetero_smoke/build.rs"]
mod build;
#[path = "official_galaxy_hetero_smoke/final_image.rs"]
mod final_image;
#[cfg(target_os = "linux")]
#[path = "official_galaxy_hetero_smoke/linux_cuda.rs"]
mod linux_cuda;
#[cfg(target_os = "linux")]
#[path = "official_galaxy_hetero_smoke/linux_vulkan.rs"]
mod linux_vulkan;
#[path = "official_galaxy_hetero_smoke/provider_execution_evidence.rs"]
mod provider_execution_evidence;
#[path = "official_galaxy_hetero_smoke/replay.rs"]
mod replay;
#[cfg(target_os = "macos")]
#[path = "official_galaxy_hetero_smoke/shader_metal.rs"]
mod shader_metal;

use build::assert_official_galaxy_hetero_build;
use final_image::finalize_official_hetero;
use provider_execution_evidence::{
    assert_pixelmagic_execution, assert_pixelmagic_trace_evidence,
    assert_provider_bundle_audit_evidence, assert_provider_execution_evidence,
};

#[test]
fn official_galaxy_hetero_projects_emit_shader_and_kernel_artifacts() {
    assert_official_galaxy_hetero_build(OfficialGalaxyHeteroBuildCase {
        label: "pixelmagic_threshold_provider_demo",
        project: "../../examples/projects/domains/pixelmagic_threshold_provider_demo",
        domain: "shader",
        backend_family: "metal",
        target_device: "apple-silicon-gpu",
        trace_record_count: 2,
        expected_trace_id: "hetero-trace:shader:metal:apple-silicon-gpu",
        yir_needles: &[
            "shader.begin_pass",
            "shader.draw_instanced",
            "PixelMagicContracts.filter_packet_total",
            "PixelMagicContracts.threshold_op_kind",
        ],
        sidecar_needles: &[
            "shader_stage_model = \"metal-render-pipeline\"",
            "lowering_capabilities",
            "pipeline_lowering = \"metal-render-pipeline-state\"",
            "execution_route = \"unified-render-graph\"",
        ],
        payload_needles: &[
            "backend_family = \"metal\"",
            "target_device = \"apple-silicon-gpu\"",
            "shader.profile.render",
        ],
    });

    assert_official_galaxy_hetero_build(OfficialGalaxyHeteroBuildCase {
        label: "pixelmagic_pipeline_demo",
        project: "../../examples/projects/domains/pixelmagic_pipeline_demo",
        domain: "shader",
        backend_family: "metal",
        target_device: "apple-silicon-gpu",
        trace_record_count: 3,
        expected_trace_id: "hetero-trace:shader:metal:apple-silicon-gpu",
        yir_needles: &[
            "shader.begin_pass",
            "shader.draw_instanced",
            "shader.inline_wgsl",
            "PixelMagicContracts.shader_pipeline_total",
        ],
        sidecar_needles: &[
            "shader_stage_model = \"metal-render-pipeline\"",
            "lowering_capabilities",
            "pipeline_lowering = \"metal-render-pipeline-state\"",
            "execution_route = \"unified-render-graph\"",
        ],
        payload_needles: &[
            "backend_family = \"metal\"",
            "target_device = \"apple-silicon-gpu\"",
            "shader.inline_wgsl",
        ],
    });

    assert_official_galaxy_hetero_build(OfficialGalaxyHeteroBuildCase {
        label: "witsage_kernel_demo",
        project: "../../examples/projects/domains/witsage_kernel_demo",
        domain: "kernel",
        backend_family: "coreml",
        target_device: "apple-ane",
        trace_record_count: 1,
        expected_trace_id: "hetero-trace:kernel:coreml:apple-ane",
        yir_needles: &[
            "kernel.tensor",
            "kernel.reduce_mean_axis",
            "kernel.topk_axis",
            "WitSageContracts.kernel_pipeline_total",
        ],
        sidecar_needles: &[
            "kernel_ir = \"coreml-program\"",
            "kernel_entry_model = \"mlmodelc-function\"",
            "tensor_lowering = \"ranked-tensor-graph\"",
        ],
        payload_needles: &[
            "backend_family = \"coreml\"",
            "target_device = \"apple-ane\"",
            "kernel.reduce_mean_axis",
        ],
    });
}

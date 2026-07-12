use super::*;

impl ShaderStageContract {
    pub fn backend_variants(&self) -> Vec<ShaderBackendVariant> {
        if self.lowering != ShaderLoweringMode::BackendEligible {
            return Vec::new();
        }

        let stage_id = self.node.clone();
        vec![
            shader_backend_variant(
                "metal",
                "gpu",
                "macos",
                "apple-gpu",
                "msl",
                "metal-render-pipeline",
                "msl-source",
                10,
                "active",
                stage_id.clone(),
                format!("metal/{stage_id}.metal"),
                "Apple GPU backend source artifact".to_owned(),
            ),
            shader_backend_variant(
                "vulkan",
                "gpu",
                "cross-platform",
                "vulkan-device",
                "glsl450",
                "vulkan-graphics-pipeline",
                "glsl450-source",
                20,
                "active",
                stage_id.clone(),
                format!("vulkan/{stage_id}.vk.glsl"),
                "Portable Vulkan GLSL source artifact".to_owned(),
            ),
            shader_backend_variant(
                "directx",
                "gpu",
                "windows",
                "d3d12-device",
                "hlsl",
                "d3d12-graphics-pipeline",
                "hlsl-source",
                30,
                "active",
                stage_id.clone(),
                format!("directx/{stage_id}.hlsl"),
                "Windows DirectX backend source artifact".to_owned(),
            ),
            shader_backend_variant(
                "webgpu",
                "gpu",
                "cross-platform",
                "webgpu-device",
                "wgsl",
                "webgpu-render-pipeline",
                "wgsl-source",
                40,
                "planned",
                stage_id.clone(),
                format!("webgpu/{stage_id}.wgsl"),
                "WebGPU/WGSL portable backend artifact".to_owned(),
            ),
            shader_backend_variant(
                "opengl",
                "gpu",
                "cross-platform",
                "opengl-device",
                "glsl460",
                "opengl-graphics-pipeline",
                "glsl460-source",
                80,
                "active",
                stage_id,
                format!("opengl/{}.glsl", self.node),
                "OpenGL GLSL 460 source artifact".to_owned(),
            ),
        ]
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn shader_backend_variant(
    backend: &'static str,
    backend_family: &'static str,
    target_os: &'static str,
    target_device: &'static str,
    ir_format: &'static str,
    dispatch_abi: &'static str,
    kind: &'static str,
    priority: usize,
    status: &'static str,
    entry: String,
    artifact: String,
    notes: String,
) -> ShaderBackendVariant {
    ShaderBackendVariant {
        backend,
        backend_family,
        target_os,
        target_device,
        ir_format,
        dispatch_abi,
        kind,
        priority,
        status,
        verification: "contract-only",
        entry,
        artifact,
        notes,
    }
}

pub(super) fn render_shader_variant(table: &str, variant: &ShaderBackendVariant) -> String {
    render_backend_variant(
        table,
        variant.backend,
        variant.backend_family,
        variant.target_os,
        variant.target_device,
        variant.ir_format,
        variant.dispatch_abi,
        variant.kind,
        variant.priority,
        variant.status,
        variant.verification,
        &variant.entry,
        &variant.artifact,
        &variant.notes,
    )
}

#[allow(clippy::too_many_arguments)]
fn kernel_backend_variant(
    backend: &'static str,
    backend_family: &'static str,
    target_os: &'static str,
    target_device: &'static str,
    ir_format: &'static str,
    dispatch_abi: &'static str,
    kind: &'static str,
    priority: usize,
    status: &'static str,
    entry: String,
    artifact: String,
    notes: String,
) -> KernelBackendVariant {
    KernelBackendVariant {
        backend,
        backend_family,
        target_os,
        target_device,
        ir_format,
        dispatch_abi,
        kind,
        priority,
        status,
        verification: "contract-only",
        entry,
        artifact,
        notes,
    }
}

pub(super) fn render_kernel_variant(table: &str, variant: &KernelBackendVariant) -> String {
    render_backend_variant(
        table,
        variant.backend,
        variant.backend_family,
        variant.target_os,
        variant.target_device,
        variant.ir_format,
        variant.dispatch_abi,
        variant.kind,
        variant.priority,
        variant.status,
        variant.verification,
        &variant.entry,
        &variant.artifact,
        &variant.notes,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_backend_variant(
    table: &str,
    backend: &str,
    backend_family: &str,
    target_os: &str,
    target_device: &str,
    ir_format: &str,
    dispatch_abi: &str,
    kind: &str,
    priority: usize,
    status: &str,
    verification: &str,
    entry: &str,
    artifact: &str,
    notes: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n[[{table}]]\n"));
    out.push_str(&format!("backend = \"{}\"\n", escape_toml(backend)));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml(backend_family)
    ));
    out.push_str(&format!("target_os = \"{}\"\n", escape_toml(target_os)));
    out.push_str(&format!(
        "target_device = \"{}\"\n",
        escape_toml(target_device)
    ));
    out.push_str(&format!("ir_format = \"{}\"\n", escape_toml(ir_format)));
    out.push_str(&format!(
        "dispatch_abi = \"{}\"\n",
        escape_toml(dispatch_abi)
    ));
    out.push_str(&format!("kind = \"{}\"\n", escape_toml(kind)));
    out.push_str(&format!("priority = {}\n", priority));
    out.push_str(&format!("status = \"{}\"\n", escape_toml(status)));
    out.push_str(&format!(
        "verification = \"{}\"\n",
        escape_toml(verification)
    ));
    out.push_str(&format!("entry = \"{}\"\n", escape_toml(entry)));
    out.push_str(&format!("artifact = \"{}\"\n", escape_toml(artifact)));
    out.push_str(&format!("notes = \"{}\"\n", escape_toml(notes)));
    out
}

impl KernelStageContract {
    pub fn backend_variants(&self) -> Vec<KernelBackendVariant> {
        let stage_id = self.node.clone();
        match self.lowering {
            KernelLoweringMode::BackendEligible => {
                let preferred_backend = self.target_runtime.as_deref();
                let mut variants = Vec::new();
                if matches!(preferred_backend, Some("coreml")) {
                    variants.push(kernel_backend_variant(
                        "coreml",
                        "npu",
                        "macos",
                        "apple-ane",
                        "mlmodel",
                        "coreml-predict",
                        "mlmodel",
                        10,
                        "planned",
                        stage_id.clone(),
                        format!("coreml/{stage_id}.mlmodel"),
                        "Apple ANE / CoreML compute artifact".to_owned(),
                    ));
                    variants.push(kernel_backend_variant(
                        "mps-graph",
                        "gpu",
                        "macos",
                        "apple-gpu",
                        "mps-graph-json",
                        "mps-graph-dispatch",
                        "graph",
                        20,
                        "planned",
                        stage_id.clone(),
                        format!("mps-graph/{stage_id}.json"),
                        "Apple GPU graph fallback artifact".to_owned(),
                    ));
                }
                if matches!(preferred_backend, Some("vulkan")) {
                    variants.push(kernel_backend_variant(
                        "vulkan",
                        "gpu",
                        "cross-platform",
                        "vulkan-device",
                        "spirv",
                        "vulkan-compute-pipeline",
                        "spirv",
                        30,
                        "planned",
                        stage_id.clone(),
                        format!("vulkan/{stage_id}.spv"),
                        "Portable Vulkan compute artifact".to_owned(),
                    ));
                }
                variants.push(kernel_backend_variant(
                    "cpu-fallback",
                    "cpu",
                    "cross-platform",
                    "host-cpu",
                    "llvm-bitcode",
                    "nuis-host-call",
                    "native",
                    900,
                    "planned",
                    stage_id,
                    format!("cpu-fallback/{}.bc", self.node),
                    "Host CPU fallback artifact".to_owned(),
                ));
                variants
            }
            KernelLoweringMode::CpuFallbackOnly => vec![kernel_backend_variant(
                "cpu-fallback",
                "cpu",
                "cross-platform",
                "host-cpu",
                "llvm-bitcode",
                "nuis-host-call",
                "native",
                900,
                "active",
                stage_id,
                format!("cpu-fallback/{}.bc", self.node),
                "Requires host CPU fallback because the op is outside the current backend portability subset".to_owned(),
            )],
        }
    }
}

impl KernelComputeGraphContract {
    pub fn backend_variants(&self) -> Vec<KernelBackendVariant> {
        let entry = self.id.clone();
        match self.lowering {
            KernelLoweringMode::BackendEligible => {
                let preferred_backend = self.target_runtime.as_deref();
                let mut variants = Vec::new();
                if matches!(preferred_backend, Some("coreml")) {
                    variants.push(kernel_backend_variant(
                        "coreml",
                        "npu",
                        "macos",
                        "apple-ane",
                        "mlpackage",
                        "coreml-predict",
                        "mlpackage",
                        10,
                        "planned",
                        entry.clone(),
                        format!("coreml/{}.mlpackage", self.id),
                        "Fused kernel compute graph for Apple ANE / CoreML".to_owned(),
                    ));
                    variants.push(kernel_backend_variant(
                        "mps-graph",
                        "gpu",
                        "macos",
                        "apple-gpu",
                        "mps-graph-json",
                        "mps-graph-dispatch",
                        "graph",
                        20,
                        "planned",
                        entry.clone(),
                        format!("mps-graph/{}.json", self.id),
                        "Fused kernel compute graph for Apple GPU fallback".to_owned(),
                    ));
                }
                if matches!(preferred_backend, Some("vulkan")) {
                    variants.push(kernel_backend_variant(
                        "vulkan",
                        "gpu",
                        "cross-platform",
                        "vulkan-device",
                        "spirv",
                        "vulkan-compute-pipeline",
                        "spirv",
                        30,
                        "planned",
                        entry.clone(),
                        format!("vulkan/{}.spv", self.id),
                        "Fused Vulkan compute graph artifact".to_owned(),
                    ));
                }
                variants.push(kernel_backend_variant(
                    "cpu-fallback",
                    "cpu",
                    "cross-platform",
                    "host-cpu",
                    "llvm-bitcode",
                    "nuis-host-call",
                    "native",
                    900,
                    "planned",
                    entry,
                    format!("cpu-fallback/{}.bc", self.id),
                    "Fused host CPU fallback graph".to_owned(),
                ));
                variants
            }
            KernelLoweringMode::CpuFallbackOnly => vec![kernel_backend_variant(
                "cpu-fallback",
                "cpu",
                "cross-platform",
                "host-cpu",
                "llvm-bitcode",
                "nuis-host-call",
                "native",
                900,
                "active",
                entry,
                format!("cpu-fallback/{}.bc", self.id),
                "Graph requires host CPU fallback because one or more stages are outside the current backend portability subset".to_owned(),
            )],
        }
    }
}
